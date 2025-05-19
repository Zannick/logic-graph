use crate::context::Ctx;
use crate::db::RouteDb;
use rocksdb::{ColumnFamily, DB};
use tempfile::TempDir;
use serde::{Deserialize, Serialize};

pub struct TestRouteDb<T>
where
    T: Ctx,
    T::PropertyObservation: Serialize + for<'a> Deserialize<'a>,
{
    pub rdb: RouteDb<T>,
    _temp_dir: TempDir,
}

impl<T> Default for TestRouteDb<T>
where
    T: Ctx,
    T::PropertyObservation: Serialize + for<'a> Deserialize<'a>,
{
    fn default() -> Self {
        let (opts, cache) = RouteDb::<T>::test_options();
        let temp_dir = TempDir::new().unwrap();
        Self {
            rdb: RouteDb::open(&temp_dir, opts, cache, false).unwrap(),
            _temp_dir: temp_dir,
        }
    }
}

pub fn all_keys(db: &DB) -> impl Iterator<Item = Box<[u8]>> + use<'_> {
    db.iterator(rocksdb::IteratorMode::Start).map(|el| el.unwrap().0)
}

pub fn all_keys_cf<'a>(db: &'a DB, cf: &'a ColumnFamily) -> impl Iterator<Item = Box<[u8]>> + use<'a> {
    db.iterator_cf(cf, rocksdb::IteratorMode::Start).map(|el| el.unwrap().0)
}
