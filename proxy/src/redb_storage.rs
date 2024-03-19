use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use parking_lot::RwLock;
use pingora_cache::key::{CacheHashKey, CompactCacheKey};
use pingora_cache::storage::{HandleHit, HandleMiss, Storage};
use pingora_cache::trace::SpanHandle;
use pingora_cache::{CacheKey, CacheMeta, HitHandler, MissHandler};
use pingora_error::{Error, ErrorType, Result};
use redb::{Database, ReadableTable, TableDefinition};
use tokio::sync::watch;
use tracing::{info, warn};

pub type CacheObject = (Vec<u8>, Vec<u8>, Vec<u8>);

/// ReDb based in cache storage
pub struct ReDbCache {
    pub db: Arc<Database>,
    pub table_name: String,
}

impl ReDbCache {
    pub fn new(dbfilename: String) -> Self {
        let db = match Database::create(dbfilename) {
            Ok(db) => db,
            Err(_) => panic!("Failed to open cache file."),
        };
        ReDbCache {
            db: Arc::new(db),
            table_name: "cache".into(),
        }
    }

    pub fn table(&self) -> TableDefinition<&str, CacheObject> {
        TableDefinition::new(self.table_name.as_str())
    }
}

pub struct ReDbHitHandler {
    body: Arc<Vec<u8>>,
    done: bool,
    range_start: usize,
    range_end: usize,
}

#[derive(Copy, Clone)]
enum PartialState {
    Partial(usize),
    Complete(usize),
}

impl ReDbHitHandler {
    fn get(&mut self) -> Option<Bytes> {
        if self.done {
            None
        } else {
            self.done = true;
            Some(Bytes::copy_from_slice(
                &self.body.as_slice()[self.range_start..self.range_end],
            ))
        }
    }

    fn seek(&mut self, start: usize, end: Option<usize>) -> Result<()> {
        if start >= self.body.len() {
            return pingora_error::Error::e_explain(
                pingora_error::ErrorType::InternalError,
                format!("seek start out of range {start} >= {}", self.body.len()),
            );
        }
        self.range_start = start;
        if let Some(end) = end {
            // end over the actual last byte is allowed, we just need to return the actual bytes
            self.range_end = std::cmp::min(self.body.len(), end);
        }
        // seek resets read so that one handler can be used for multiple ranges
        self.done = false;
        Ok(())
    }
}

#[async_trait]
impl HandleHit for ReDbHitHandler {
    async fn read_body(&mut self) -> Result<Option<Bytes>> {
        Ok(self.get())
    }
    async fn finish(
        self: Box<Self>, // because self is always used as a trait object
        _storage: &'static (dyn Storage + Sync),
        _key: &CacheKey,
        _trace: &SpanHandle,
    ) -> Result<()> {
        Ok(())
    }

    fn can_seek(&self) -> bool {
        true
    }

    fn seek(&mut self, start: usize, end: Option<usize>) -> Result<()> {
        self.seek(start, end)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

pub struct ReDbMissHandler {
    meta: (Vec<u8>, Vec<u8>),
    body: Arc<RwLock<Vec<u8>>>,
    bytes_written: Arc<watch::Sender<PartialState>>,
    // these are used only in finish() to to data from temp to cache
    key: String,
    db: Arc<Database>,
    table_name: String,
}

#[async_trait]
impl HandleMiss for ReDbMissHandler {
    async fn write_body(&mut self, data: bytes::Bytes, eof: bool) -> Result<()> {
        let current_bytes = match *self.bytes_written.borrow() {
            PartialState::Partial(p) => p,
            PartialState::Complete(_) => panic!("already EOF"),
        };
        self.body.write().extend_from_slice(&data);
        let written = current_bytes + data.len();
        let new_state = if eof {
            PartialState::Complete(written)
        } else {
            PartialState::Partial(written)
        };
        self.bytes_written.send_replace(new_state);
        Ok(())
    }

    async fn finish(self: Box<Self>) -> Result<usize> {
        let write_txn = match self.db.begin_write() {
            Ok(result) => result,
            Err(_) => {
                return Err(Error::new(ErrorType::Custom(
                    "Error opening write transaction",
                )))
            }
        };
        let table_def: TableDefinition<&str, CacheObject> =
            TableDefinition::new(self.table_name.as_str());
        let meta = self.meta.clone();
        let size = self.body.read().len();
        let body = self.body.read().clone();
        {
            let mut table = match write_txn.open_table(table_def) {
                Ok(table) => table,
                Err(_) => {
                    return Err(Error::new(ErrorType::Custom(
                        "Error opening table transaction.",
                    )))
                }
            };
            match table.insert(self.key.as_str(), &(meta.0, meta.1, body)) {
                Ok(_) => info!("Succesfully wrote {key} to cache.", key = self.key.as_str()),
                Err(_) => return Err(Error::new(ErrorType::Custom("Error inserting into ReDb"))),
            };
        }
        match write_txn.commit() {
            Ok(_) => (),
            Err(_) => {
                return Err(Error::new(ErrorType::Custom(
                    "Error committing transaction",
                )))
            }
        };
        Ok(size)
    }
}

#[async_trait]
impl Storage for ReDbCache {
    async fn lookup(
        &'static self,
        key: &CacheKey,
        _trace: &SpanHandle,
    ) -> Result<Option<(CacheMeta, HitHandler)>> {
        let hash = key.combined();

        let read_txn = match self.db.begin_read() {
            Ok(transaction) => transaction,
            Err(err) => {
                warn!(
                    "Error when opening write transaction for cache lookup: {}",
                    err
                );
                return Ok(None);
            }
        };
        let table = match read_txn.open_table(self.table()) {
            Ok(tbl) => tbl,
            Err(err) => {
                warn!(
                    "Error when opening write transaction for cache lookup: {}",
                    err
                );
                return Ok(None);
            }
        };

        let value = match table.get(hash.as_str()) {
            Ok(obj) => obj.map(|guard| guard.value()),
            Err(err) => {
                info!("Error when retrieving from cache: {}", err);
                None
            }
        };

        if let Some(data) = value {
            let meta = CacheMeta::deserialize(&data.0, &data.1)?;
            let hit_handler = ReDbHitHandler {
                body: Arc::new(data.2.clone()),
                done: false,
                range_start: 0,
                range_end: data.2.len(),
            };
            Ok(Some((meta, Box::new(hit_handler))))
        } else {
            Ok(None)
        }
    }

    async fn get_miss_handler(
        &'static self,
        key: &CacheKey,
        meta: &CacheMeta,
        _trace: &SpanHandle,
    ) -> Result<MissHandler> {
        let hash = key.combined();
        let miss_handler = ReDbMissHandler {
            meta: meta.serialize()?,
            body: Arc::new(RwLock::new(Vec::new())),
            bytes_written: Arc::new(watch::Sender::new(PartialState::Partial(0))),
            key: hash.clone(),
            db: self.db.clone(),
            table_name: self.table_name.clone(),
        };
        Ok(Box::new(miss_handler))
    }

    async fn purge(&'static self, key: &CompactCacheKey, _trace: &SpanHandle) -> Result<bool> {
        let hash = key.combined();
        let table = self.table();
        let write_txn = match self.db.begin_write() {
            Ok(txn) => txn,
            Err(_) => return Err(Error::new(ErrorType::Custom("Error opening table"))),
        };
        {
            let mut table = match write_txn.open_table(table) {
                Ok(txn) => txn,
                Err(_) => return Err(Error::new(ErrorType::Custom("Error opening table"))),
            };
            match table.remove(hash.as_str()) {
                Ok(_) => (),
                Err(_) => return Err(Error::new(ErrorType::Custom("Error removing cache entry"))),
            };
        }
        match write_txn.commit() {
            Ok(_) => Ok(true),
            Err(_) => Err(Error::new(ErrorType::Custom("Error commiting cache entry"))),
        }
    }

    async fn update_meta(
        &'static self,
        key: &CacheKey,
        meta: &CacheMeta,
        _trace: &SpanHandle,
    ) -> Result<bool> {
        let hash = key.combined();
        let table = self.table();
        let write_txn = match self.db.begin_write() {
            Ok(txn) => txn,
            Err(_) => return Err(Error::new(ErrorType::Custom("Error opening table"))),
        };
        {
            let mut table = match write_txn.open_table(table) {
                Ok(txn) => txn,
                Err(_) => return Err(Error::new(ErrorType::Custom("Error opening table"))),
            };
            let data = match table.get(hash.as_str()) {
                Ok(obj) => {
                    if let Some(guard) = obj {
                        guard.value()
                    } else {
                        return Err(Error::new(ErrorType::Custom("Empty value for cache key")));
                    }
                }
                Err(err) => {
                    warn!("Error when retrieving from cache: {}", err);
                    return Err(Error::new(ErrorType::Custom(
                        "Error when retrieving from cache",
                    )));
                }
            };

            let des_new_meta = meta.serialize()?;
            let new_data = (des_new_meta.0, des_new_meta.1, data.2);

            match table.insert(hash.as_str(), new_data) {
                Ok(_) => (),
                Err(_) => return Err(Error::new(ErrorType::Custom("Error updating cache entry"))),
            };
        }
        match write_txn.commit() {
            Ok(_) => Ok(true),
            Err(_) => Err(Error::new(ErrorType::Custom(
                "Error commiting cache entry update",
            ))),
        }
    }

    fn support_streaming_partial_write(&self) -> bool {
        false
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use once_cell::sync::Lazy;
    use pingora::http::ResponseHeader;
    use pingora_cache::CacheMeta;
    use rustracing::span::Span;
    use std::time::{Duration, SystemTime};

    fn gen_meta() -> CacheMeta {
        let mut header = ResponseHeader::build(200, None).unwrap();
        header.append_header("foo1", "bar1").unwrap();
        header.append_header("foo2", "bar2").unwrap();
        header.append_header("foo3", "bar3").unwrap();
        header.append_header("Server", "Pingora").unwrap();
        CacheMeta::new(
            SystemTime::now()
                .checked_add(Duration::new(3600, 0))
                .unwrap(),
            SystemTime::now(),
            10,
            10,
            header,
        )
    }

    #[tokio::test]
    async fn test_write_then_read() {
        static CACHE: Lazy<ReDbCache> = Lazy::new(|| {
            let file = tempfile::NamedTempFile::new().unwrap();
            let filepath = file.path().to_str().unwrap().to_owned();
            ReDbCache::new(filepath)
        });
        let span = &Span::inactive().handle();

        let key1 = CacheKey::new("", "a", "1");
        let res = CACHE.lookup(&key1, span).await.unwrap();
        assert!(res.is_none());

        let cache_meta = gen_meta();

        let mut miss_handler = CACHE
            .get_miss_handler(&key1, &cache_meta, span)
            .await
            .unwrap();
        miss_handler
            .write_body(b"test1"[..].into(), false)
            .await
            .unwrap();
        miss_handler
            .write_body(b"test2"[..].into(), false)
            .await
            .unwrap();
        miss_handler.finish().await.unwrap();

        let (_, mut hit_handler) = CACHE.lookup(&key1, span).await.unwrap().unwrap();
        let data = hit_handler.read_body().await.unwrap().unwrap();
        assert_eq!("test1test2", data);
        let data = hit_handler.read_body().await.unwrap();
        assert!(data.is_none());
    }

    #[tokio::test]
    async fn test_read_range() {
        static CACHE: Lazy<ReDbCache> = Lazy::new(|| {
            let file = tempfile::NamedTempFile::new().unwrap();
            let filepath = file.path().to_str().unwrap().to_owned();
            ReDbCache::new(filepath)
        });
        let span = &Span::inactive().handle();

        let key1 = CacheKey::new("", "b", "1");
        let res = CACHE.lookup(&key1, span).await.unwrap();
        assert!(res.is_none());

        let cache_meta = gen_meta();

        let mut miss_handler = CACHE
            .get_miss_handler(&key1, &cache_meta, span)
            .await
            .unwrap();
        miss_handler
            .write_body(b"test1test2"[..].into(), false)
            .await
            .unwrap();
        miss_handler.finish().await.unwrap();

        let (_, mut hit_handler) = CACHE.lookup(&key1, span).await.unwrap().unwrap();
        // out of range
        assert!(hit_handler.seek(10000, None).is_err());
        assert!(hit_handler.seek(5, None).is_ok());
        let data = hit_handler.read_body().await.unwrap().unwrap();
        assert_eq!("test2", data);
        let data = hit_handler.read_body().await.unwrap();
        assert!(data.is_none());

        assert!(hit_handler.seek(4, Some(5)).is_ok());
        let data = hit_handler.read_body().await.unwrap().unwrap();
        assert_eq!("1", data);
        let data = hit_handler.read_body().await.unwrap();
        assert!(data.is_none());
    }
}
