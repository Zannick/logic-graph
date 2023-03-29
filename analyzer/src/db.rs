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
use rocksdb::{
    BlockBasedOptions, Cache, CuckooTableOptions, IteratorMode, MemtableFactory, MergeOperands,
    Options, ReadOptions, WriteBatchWithTransaction, WriteOptions, DB,
};
use serde::Serialize;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicUsize, Ordering};

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

    max_time: AtomicI32,
    scale_factor: i32,

    seq: AtomicU64,
    size: AtomicUsize,
    seen: AtomicUsize,
    iskips: AtomicUsize,
    pskips: AtomicUsize,
    dup_iskips: AtomicUsize,
    dup_pskips: AtomicUsize,

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

impl From<rmp_serde::decode::Error> for Error {
    fn from(value: rmp_serde::decode::Error) -> Self {
        Error {
            message: format!("{:?}", value),
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
    pub fn open<P>(p: P, scale_factor: i32) -> Result<HeapDB<T>, String>
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
            max_time: i32::MAX.into(),
            scale_factor,
            seq: 0.into(),
            size: 0.into(),
            seen: 0.into(),
            iskips: 0.into(),
            pskips: 0.into(),
            dup_iskips: 0.into(),
            dup_pskips: 0.into(),
            phantom: PhantomData,
        })
    }

    /// Returns the number of elements in the heap (tracked separately from the db).
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Acquire)
    }

    /// Returns the number of unique states we've seen so far (tracked separately from the db).
    pub fn seen(&self) -> usize {
        self.seen.load(Ordering::Acquire)
    }

    /// Returns details about the number of states we've skipped (tracked separately from the db).
    /// Specifically:
    ///   1) states not added (on push) to the db due to exceeding max time,
    ///   2) states not returned (on pop) from the db due to exceeding max time,
    ///   3) states not added (on push) to the db due to being duplicates with worse times,
    ///   4) states not returned (on pop) from the db due to being duplicates with worse times.
    pub fn skip_stats(&self) -> (usize, usize, usize, usize) {
        (
            self.iskips.load(Ordering::Acquire),
            self.pskips.load(Ordering::Acquire),
            self.dup_iskips.load(Ordering::Acquire),
            self.dup_pskips.load(Ordering::Acquire),
        )
    }

    pub fn max_time(&self) -> i32 {
        self.max_time.load(Ordering::Acquire)
    }

    pub fn set_max_time(&mut self, max_time: i32) {
        self.max_time.fetch_min(max_time, Ordering::Release);
    }

    pub fn set_lenient_max_time(&mut self, max_time: i32) {
        self.set_max_time(max_time + (max_time / 128))
    }

    pub fn scale_factor(&self) -> i32 {
        self.scale_factor
    }

    /// The key for a ContextWrapper<T> in the heap is:
    /// the score (4 bytes),
    /// elapsed time (4 bytes),
    /// a sequence number (8 bytes)
    fn get_heap_key(&self, el: &ContextWrapper<T>) -> [u8; 16] {
        let mut key: [u8; 16] = [0; 16];
        key[0..4].copy_from_slice(&el.score(self.scale_factor).to_be_bytes());
        key[4..8].copy_from_slice(&el.elapsed().to_be_bytes());
        key[8..16].copy_from_slice(&self.seq.fetch_add(1, Ordering::AcqRel).to_be_bytes());
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

    fn from_heap_value(&self, buf: &[u8]) -> Result<ContextWrapper<T>, Error> {
        Ok(rmp_serde::from_slice::<ContextWrapper<T>>(buf)?)
    }

    fn get_seen_value(&self, seen_key: &[u8]) -> Result<Option<i32>, Error> {
        match self.seendb.get_pinned(&seen_key)? {
            Some(slice) => {
                if slice.len() != 4 {
                    return Err(Error {
                        message: format!("Invalid seen elapsed time length: {}", slice.len()),
                    });
                }
                Ok(Some(i32::from_be_bytes(slice.as_ref().try_into().unwrap())))
            }
            None => Ok(None),
        }
    }

    /// Pushes an element into the heap.
    /// If the element's elapsed time is greater than the allowed maximum,
    /// or, the state has been previously seen with an equal or lower elapsed time, does nothing.
    pub fn push(&self, el: ContextWrapper<T>) -> Result<(), Error> {
        if el.elapsed() > self.max_time.load(Ordering::Acquire) {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        }
        let seen_key = self.get_seen_key(el.get());

        let should_write = match self.get_seen_value(&seen_key)? {
            Some(stored) => {
                if stored < el.elapsed() {
                    self.dup_iskips.fetch_add(1, Ordering::Release);
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
            self.seen.fetch_add(1, Ordering::Release);
        }
        let key = self.get_heap_key(&el);
        let val = self.get_heap_value(&el);
        self.db.put_opt(key, val, &self.write_opts)?;
        self.size.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self) -> Result<Option<ContextWrapper<T>>, Error> {
        let mut tail_opts = ReadOptions::default();
        tail_opts.set_tailing(true);
        let mut iter = self.db.iterator_opt(IteratorMode::Start, tail_opts);
        for item in iter {
            let (key, value) = item?;

            // Ignore error
            let _ = self.db.delete_opt(key, &self.write_opts);
            self.size.fetch_sub(1, Ordering::Release);

            let el = self.from_heap_value(&value)?;
            if el.elapsed() > self.max_time() {
                self.pskips.fetch_add(1, Ordering::Release);
                continue;
            }

            let seen_key = self.get_seen_key(el.get());
            if let Some(stored) = self.get_seen_value(&seen_key)? {
                if el.elapsed() > stored {
                    self.dup_pskips.fetch_add(1, Ordering::Release);
                    continue;
                }
            }
            return Ok(Some(el));
        }

        Ok(None)
    }

    pub fn extend<I>(&self, iter: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = ContextWrapper<T>>,
    {
        let mut batch = WriteBatchWithTransaction::<false>::default();
        let mut seen_batch = WriteBatchWithTransaction::<false>::default();
        let max_time = self.max_time();
        let mut skips = 0;
        let mut dups = 0;

        for el in iter {
            if el.elapsed() > max_time {
                skips += 1;
                continue;
            }
            let seen_key = self.get_seen_key(el.get());

            // TODO: Use multi_get instead of get in a loop.
            let should_write = match self.get_seen_value(&seen_key)? {
                Some(stored) => {
                    if stored < el.elapsed() {
                        dups += 1;
                        continue;
                    }
                    stored > el.elapsed()
                }
                None => true,
            };
            // If the value seen is also what we have, we still want to put it into the heap,
            // but we don't have to write the value again as it's a maximum.
            if should_write {
                seen_batch.merge(&seen_key, el.elapsed().to_be_bytes());
            }
            let key = self.get_heap_key(&el);
            let val = self.get_heap_value(&el);
            batch.put(key, val);
        }
        let new = batch.len();
        let new_seen = seen_batch.len();
        self.db.write_opt(batch, &self.write_opts)?;
        self.seendb.write_opt(seen_batch, &self.write_opts)?;

        self.iskips.fetch_add(skips, Ordering::Release);
        self.dup_iskips.fetch_add(dups, Ordering::Release);
        self.size.fetch_add(new, Ordering::Release);
        self.seen.fetch_add(new_seen, Ordering::Release);

        Ok(())
    }

    // TODO: data dashboard
}
