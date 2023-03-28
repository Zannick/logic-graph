//! Wrapper around rocksdb with logic-graph specific features.
#![allow(unused)]
extern crate rocksdb;

use crate::context::*;
use crate::CommonHasher;
use lru::LruCache;
use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins, Plot};
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use rand::{thread_rng, Rng};
use rmp_serde::Serializer;
use rocksdb::MergeOperands;
use rocksdb::WriteOptions;
use rocksdb::{BlockBasedOptions, Cache, CuckooTableOptions, MemtableFactory, Options, DB};
use serde::Serialize;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;

// We need the following in this wrapper impl:
// 1. The contextwrapper db is mainly iterated over, via either
//    getting the minimum-score element (i.e. iterating from start)
//    or running over the whole db (e.g. for statistics). BlockDB is best for this.
// 2. We'll add two LRU cache layers that must outlive the BlockDB,
//    one for uncompressed blocks and the other for compressed blocks.
// 3. The best-seen version of a context is mainly point lookups,
//    which is better served by CuckooTable. This will be a Ctx -> u32
//    map, which fulfills the fixed-length key and value limitations.

struct HeapDBOptions {
    opts: Options,
    path: PathBuf,
}

pub struct HeapDB<T> {
    db: DB,
    seendb: DB,
    cache_uncompressed: Cache,
    cache_cmprsd: Cache,
    row_cache: Cache,
    opts: HeapDBOptions,
    seen_opts: HeapDBOptions,
    write_opts: WriteOptions,

    max_time: i32,
    scale_factor: i32,

    size: usize,
    seen: usize,
    iskips: usize,
    pskips: usize,
    dup_iskips: usize,
    dup_pskips: usize,

    phantom: PhantomData<T>,
}

// Final cleanup, done in a separate struct here to ensure it's done
// after the db is dropped.
impl Drop for HeapDBOptions {
    fn drop(&mut self) {
        let _ = DB::destroy(&self.opts, &self.path);
    }
}

pub struct Error {
    pub message: String,
}

impl From<rocksdb::Error> for Error {
    fn from(value: rocksdb::Error) -> Self {
        Error {
            message: value.into(),
        }
    }
}

impl From<Error> for String {
    fn from(value: Error) -> Self {
        value.message
    }
}

fn min_merge(
    new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
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
    } else if let Some(v) = existing_val {
        Some(v.into())
    } else {
        None
    }
}

impl<T> HeapDB<T>
where
    T: Ctx,
{
    pub fn open<P>(p: P) -> Result<HeapDB<T>, String>
    where
        P: AsRef<Path>,
    {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        // For now, the db should be deleted.
        opts.set_error_if_exists(true);
        // change compression options?
        // 4 write buffers at 256 MiB = 1 GiB
        opts.set_write_buffer_size(256 * 1024 * 1024);
        opts.set_max_write_buffer_number(4);
        opts.increase_parallelism(2);

        let mut opts2 = opts.clone();

        let mut block_opts = BlockBasedOptions::default();
        // blockdb caches = 3 GiB
        let cache = Cache::new_lru_cache(2 * 1024 * 1024 * 1024)?;
        let cache2 = Cache::new_lru_cache(1 * 1024 * 1024 * 1024)?;
        block_opts.set_block_cache(&cache);
        block_opts.set_block_cache_compressed(&cache2);
        opts.set_block_based_table_factory(&block_opts);

        let mut path = p.as_ref().to_owned();
        let mut path2 = path.clone();
        path.push("states");
        path2.push("seen");

        // 1 + 3 = 4 GiB roughly for this db
        let db = DB::open(&opts, &path)?;

        let cache3 = Cache::new_lru_cache(2 * 1024 * 1024 * 1024)?;
        opts2.set_row_cache(&cache3);
        let mut cuckoo_opts = CuckooTableOptions::default();
        cuckoo_opts.set_hash_ratio(0.75);
        cuckoo_opts.set_use_module_hash(false);
        opts2.set_cuckoo_table_factory(&cuckoo_opts);
        opts2.set_merge_operator_associative("min", min_merge);

        // 1 GiB blocks + 2 GiB row cache = 3 GiB for this one?
        let seendb = DB::open(&opts2, &path2)?;

        let mut write_opts = WriteOptions::default();
        write_opts.disable_wal(true);

        Ok(HeapDB {
            db,
            seendb,
            cache_uncompressed: cache,
            cache_cmprsd: cache2,
            row_cache: cache3,
            opts: HeapDBOptions { opts, path },
            seen_opts: HeapDBOptions {
                opts: opts2,
                path: path2,
            },
            write_opts,
            max_time: i32::MAX,
            scale_factor: 50,
            size: 0,
            seen: 0,
            iskips: 0,
            pskips: 0,
            dup_iskips: 0,
            dup_pskips: 0,
            phantom: PhantomData,
        })
    }

    /// Returns the number of elements in the heap (tracked separately from the db).
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns the number of unique states we've seen so far (tracked separately from the db).
    pub fn seen(&self) -> usize {
        self.seen
    }

    pub fn max_time(&self) -> i32 {
        self.max_time
    }

    pub fn set_max_time(&mut self, max_time: i32) {
        self.max_time = max_time;
    }

    pub fn set_lenient_max_time(&mut self, max_time: i32) {
        self.set_max_time(max_time + (max_time / 128))
    }

    pub fn scale_factor(&self) -> i32 {
        self.scale_factor
    }

    pub fn set_scale_factor(&mut self, factor: i32) {
        self.scale_factor = factor;
    }

    /// The key for a ContextWrapper<T> in the heap is:
    /// the score (4 bytes),
    /// elapsed time (4 bytes),
    /// random data (8 bytes, 1 in 2**64 chance of collision)
    fn get_heap_key(&self, el: &ContextWrapper<T>) -> [u8; 16] {
        let mut key: [u8; 16] = [0; 16];
        key[0..4].copy_from_slice(&el.score(self.scale_factor).to_be_bytes());
        key[4..8].copy_from_slice(&el.elapsed().to_be_bytes());
        thread_rng().fill(&mut key[8..]);
        key
    }

    /// The key for a T (Ctx) in the seendb is... itself!
    fn get_seen_key(&self, el: &T) -> Vec<u8> {
        let mut key = Vec::with_capacity(std::mem::size_of::<T>());
        el.serialize(&mut Serializer::new(&mut key)).unwrap();
        key
    }

    /// The value for a ContextWrapper<T> in the heap is... itself!
    /// Unfortunately this is serializing the T (Ctx) a second time if we already got the seen key.
    fn get_heap_value(&self, el: &ContextWrapper<T>) -> Vec<u8> {
        let mut val = Vec::new();
        el.serialize(&mut Serializer::new(&mut val)).unwrap();
        val
    }

    /// Pushes an element into the heap.
    /// If the element's elapsed time is greater than the allowed maximum,
    /// or, the state has been previously seen with an equal or lower elapsed time, does nothing.
    pub fn push(&mut self, el: ContextWrapper<T>) -> Result<(), Error> {
        if el.elapsed() > self.max_time {
            self.iskips += 1;
            return Ok(());
        }
        let seen_key = self.get_seen_key(el.get());

        let should_write = match self.seendb.get_pinned(&seen_key)? {
            Some(slice) => {
                if slice.len() != 4 {
                    return Err(Error {
                        message: format!("Invalid seen elapsed time length: {}", slice.len()),
                    });
                }
                let stored = i32::from_be_bytes(slice.as_ref().try_into().unwrap());
                if stored < el.elapsed() {
                    self.dup_iskips += 1;
                    return Ok(());
                }
                stored > el.elapsed()
            }
            None => true,
        };
        // If the value seen is also what we have, we still want to put it into the heap,
        // but we don't have to write the value again as it's a maximum.
        if should_write {
            self.seendb
                .merge_opt(&seen_key, el.elapsed().to_be_bytes(), &self.write_opts);
        }
        let key = self.get_heap_key(&el);
        let val = self.get_heap_value(&el);
        self.db.put_opt(key, val, &self.write_opts)?;
        Ok(())
    }
}
