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
use std::sync::{Arc, Mutex};

const SEPARATOR: u8 = ':' as u8;
const MB: usize = 1 << 20;
const GB: usize = 1 << 30;

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
where V: for <'de> Deserialize<'de> {
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
    fn lookup(&self, similar: &StructType) -> Vec<ValueType>;
    fn insert(&self, observations: Vec<StructType::PropertyObservation>, value: ValueType);
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
    _cache: Cache,
    phantom: PhantomData<(StructType, ValueType)>,
}

impl<StructType, ValueType> MatcherRocksDb<StructType, ValueType>
where
    StructType: Observable,
    StructType::PropertyObservation: Serialize + for<'a> Deserialize<'a>,
    ValueType: Ord + PartialOrd + Serialize + for<'a> Deserialize<'a> + 'static,
{
    pub fn open<P>(p: P, delete_first: bool) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        // change compression options?
        // 4 write buffers at 256 MiB = 1 GiB
        opts.set_write_buffer_size(256 * MB);
        opts.set_max_write_buffer_number(4);
        opts.set_target_file_size_base(128 * 1024 * 1024);
        // use half the logical cores, clamp between 2 and 32
        opts.increase_parallelism(std::cmp::max(
            2,
            std::cmp::min(num_cpus::get() / 2, 32).try_into().unwrap(),
        ));
        opts.set_max_background_jobs(4);

        let mut env = Env::new().unwrap();
        env.set_low_priority_background_threads(3);
        opts.set_env(&env);
        opts.set_max_open_files(512);

        let mut block_opts = BlockBasedOptions::default();
        // blockdb caches = 2 GiB
        let cache = Cache::new_lru_cache(2 * GB);
        block_opts.set_block_cache(&cache);
        block_opts.set_block_size(16 * 1024);
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts.set_ribbon_filter(9.9);
        opts.set_block_based_table_factory(&block_opts);

        opts.set_merge_operator_associative("min", min_merge::<ValueType>);
        
        let mut path = p.as_ref().to_owned();
        path.push("routes");
        if delete_first {
            let _ = DB::destroy(&opts, &path);
        }
        
        Ok(MatcherRocksDb {
            db: DB::open(&opts, &path)?,
            _cache: cache,
            phantom: PhantomData::default(),
        })
    }
}

impl<StructType, ValueType> MatcherTrieDb<StructType, ValueType>
    for MatcherRocksDb<StructType, ValueType>
where
    StructType: Observable,
    StructType::PropertyObservation: Serialize + for<'a> Deserialize<'a>,
    ValueType: Serialize + for<'a> Deserialize<'a>,
{
    fn lookup(&self, similar: &StructType) -> Vec<ValueType> {
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
        let vec = serialize_data(root);

        let mut iter_opts = ReadOptions::default();
        iter_opts.set_tailing(true);
        iter_opts.fill_cache(false);
        // Takes care of the partitionable root observation and ending when we don't match anymore.
        iter_opts.set_iterate_range(PrefixRange(vec.clone()));
        let mut iter = self.db.raw_iterator_opt(iter_opts);

        'db_iter: while iter.valid() {
            let (key, value) = iter.item().unwrap();
            let splits: Vec<_> = key.split(|&n| n == SEPARATOR).collect();

            for (i, ser_obs) in splits[1..].iter().enumerate() {
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
    ) {
        let ser = observations
            .iter()
            .map(serialize_data)
            .collect::<Vec<_>>()
            .join(&SEPARATOR);
        self.db
            .merge(ser, serialize_data(value))
            .expect("Error updating matcher table");
    }

    fn size(&self) -> usize {
        todo!()
    }

    fn max_depth(&self) -> usize {
        todo!()
    }

    fn num_values(&self) -> usize {
        todo!()
    }
}
