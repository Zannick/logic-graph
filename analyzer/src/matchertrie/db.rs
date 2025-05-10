#![allow(unused)]

use super::matcher::{MatcherDispatch, Observable};
use rmp_serde::Serializer;
use rocksdb::{
    perf, BlockBasedOptions, Cache, ColumnFamily, ColumnFamilyDescriptor, Env, IteratorMode,
    MergeOperands, Options, PrefixRange, ReadOptions, WriteBatchWithTransaction, WriteOptions, DB,
};
use serde::{Deserialize, Serialize};

use std::collections::VecDeque;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const SEPARATOR: u8 = ':' as u8;
const MB: usize = 1 << 20;
const GB: usize = 1 << 30;

const ROUTE: &str = "route";
const TRIE: &str = "trie";

fn serialize_data<V>(v: V) -> Vec<u8>
where
    V: Serialize,
{
    let mut val = Vec::with_capacity(std::mem::size_of::<V>());
    v.serialize(&mut Serializer::new(&mut val)).unwrap();
    val
}

fn get_obj_from_data<V>(buf: &[u8]) -> Result<V, anyhow::Error>
where
    V: for<'de> Deserialize<'de>,
{
    Ok(rmp_serde::from_slice::<V>(buf)?)
}

fn min_merge<V>(
    _new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>>
where
    V: for<'de> Deserialize<'de>,
{
    if let Some(res) = operands.iter().min() {
        if let Some(v) = existing_val {
            if res < v {
                Some(res.into())
            } else {
                Some(v.into())
            }
        } else {
            Some(res.into())
        }
    } else {
        existing_val.map(|v| v.into())
    }
}

pub trait MatcherTrieDb<StructType, ValueType>
where
    StructType: Observable,
{
    /// Performs a lookup for all states similar to this one.
    fn lookup(&self, similar: &StructType, prefix: &Vec<u8>) -> Vec<ValueType>;
    fn insert(
        &self,
        observations: Vec<StructType::PropertyObservation>,
        value: ValueType,
        prefix: &Vec<u8>,
    );
    fn insert_batch<const TRANSACTION: bool>(
        &self,
        batch: &mut WriteBatchWithTransaction<TRANSACTION>,
        observations: Vec<StructType::PropertyObservation>,
        value: ValueType,
        prefix: &Vec<u8>,
    );
    fn size(&self) -> usize;
    fn max_depth(&self) -> usize;
    fn num_values(&self) -> usize;
}

pub struct MatcherRocksDb<StructType, ValueType>
where
    StructType: Observable,
    StructType::PropertyObservation: Serialize + for<'a> Deserialize<'a>,
    ValueType: Serialize + for<'a> Deserialize<'a>,
{
    db: DB,
    col: &'static str,
    phantom: PhantomData<(StructType, ValueType)>,
}

impl<StructType, ValueType> MatcherRocksDb<StructType, ValueType>
where
    StructType: Observable,
    StructType::PropertyObservation: Serialize + for<'a> Deserialize<'a>,
    ValueType: Serialize + for<'a> Deserialize<'a>,
{
    pub fn from_db_cf(db: DB, col: &'static str) -> MatcherRocksDb<StructType, ValueType> {
        MatcherRocksDb {
            db,
            col,
            phantom: PhantomData::default(),
        }
    }

    fn cf(&self) -> &ColumnFamily {
        self.db.cf_handle(self.col).unwrap()
    }

    pub fn db(&self) -> &DB {
        &self.db
    }
}

impl<StructType, ValueType> MatcherTrieDb<StructType, ValueType>
    for MatcherRocksDb<StructType, ValueType>
where
    StructType: Observable,
    StructType::PropertyObservation: Serialize + for<'a> Deserialize<'a>,
    ValueType: Serialize + for<'a> Deserialize<'a>,
{
    fn lookup(&self, similar: &StructType, prefix: &Vec<u8>) -> Vec<ValueType> {
        // Process:
        // 1. Get the partitionable root observation from similar.
        // 2. Create a tailing iterator in the db at the root observation and scan forward.
        // 3. Loop for each key/value pair in the db read:
        //    a. split the key by the joining element (eg. ":")
        //    b. if the first part doesn't match the root observation, break
        //    c. deserialize all parts and check whether they match similar. If so, add the value to the vec.
        //       if not, skip forward to a key greater than the current key at the current part.
        // OR... when we read a part (an observation), we can see whether it's a partitionable observation
        // and thus skip to exactly the right one.
        let root = similar.root_observation();
        let mut results = Vec::new();
        let mut vec = prefix.clone();
        vec.push(SEPARATOR);
        vec.extend(serialize_data(root));

        let mut iter_opts = ReadOptions::default();
        iter_opts.set_tailing(true);
        iter_opts.fill_cache(false);
        // Takes care of the partitionable root observation and ending when we don't match anymore.
        iter_opts.set_iterate_range(PrefixRange(vec.clone()));
        let mut iter = self.db.raw_iterator_cf_opt(self.cf(), iter_opts);

        'db_iter: while iter.valid() {
            let (key, value) = iter.item().unwrap();
            let splits: Vec<_> = key.split(|&n| n == SEPARATOR).collect();

            for (i, ser_obs) in splits[2..].iter().enumerate() {
                let obs: StructType::PropertyObservation = get_obj_from_data(ser_obs).unwrap();
                if !similar.matches(&obs) {
                    let mut new_key: Vec<_> = splits[0..=i + 1].join(&SEPARATOR);
                    new_key.push(SEPARATOR + 1);
                    // Implicitly dropping the item
                    iter.seek(new_key);
                    continue 'db_iter;
                }
            }
            results.push(get_obj_from_data::<ValueType>(value).unwrap());
            iter.next();
        }
        iter.status().expect("Error reading matcher table");
        results
    }

    fn insert(
        &self,
        observations: Vec<<StructType as Observable>::PropertyObservation>,
        value: ValueType,
        prefix: &Vec<u8>,
    ) {
        let mut sections = Vec::with_capacity(observations.len() + 1);
        sections.push(prefix.clone());
        for obs in observations {
            sections.push(serialize_data(obs));
        }
        let key = sections.join(&SEPARATOR);
        self.db
            .merge_cf(self.cf(), key, serialize_data(value))
            .expect("Error updating matcher table");
    }

    fn insert_batch<const TRANSACTION: bool>(
        &self,
        batch: &mut WriteBatchWithTransaction<TRANSACTION>,
        observations: Vec<<StructType as Observable>::PropertyObservation>,
        value: ValueType,
        prefix: &Vec<u8>,
    ) {
        let mut sections = Vec::with_capacity(observations.len() + 1);
        sections.push(prefix.clone());
        for obs in observations {
            sections.push(serialize_data(obs));
        }
        let key = sections.join(&SEPARATOR);
        batch.merge_cf(self.cf(), key, serialize_data(value));
    }

    fn size(&self) -> usize {
        self.db
            .property_int_value_cf(self.cf(), "rocksdb.estimate-num-keys")
            .unwrap()
            .unwrap_or(0) as usize
    }

    fn max_depth(&self) -> usize {
        todo!()
    }

    fn num_values(&self) -> usize {
        self.db
            .property_int_value_cf(self.cf(), "rocksdb.estimate-num-keys")
            .unwrap()
            .unwrap_or(0) as usize
    }
}
