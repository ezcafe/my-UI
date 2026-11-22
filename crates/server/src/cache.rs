use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub const MAX_CACHE_CAPACITY: usize = 1000;

#[derive(Clone)]
pub struct CacheEntry {
    pub inserted_at: Instant,
    pub ttl: Duration,
    pub val: serde_json::Value,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() >= self.ttl
    }
}

#[derive(Clone)]
pub struct MinifluxCache {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Duration,
}

impl MinifluxCache {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let store = self.store.read().await;
        if let Some(entry) = store.get(key) {
            if !entry.is_expired() {
                return Some(entry.val.clone());
            }
        }
        None
    }

    pub async fn insert(&self, key: String, val: serde_json::Value) {
        self.insert_with_ttl(key, val, self.default_ttl).await;
    }

    pub async fn insert_with_ttl(&self, key: String, val: serde_json::Value, ttl: Duration) {
        let mut store = self.store.write().await;
        if store.len() >= MAX_CACHE_CAPACITY {
            store.retain(|_, entry| !entry.is_expired());
            if store.len() >= MAX_CACHE_CAPACITY {
                if let Some(oldest_key) = store
                    .iter()
                    .min_by_key(|(_, entry)| entry.inserted_at)
                    .map(|(k, _)| k.clone())
                {
                    store.remove(&oldest_key);
                }
            }
        }
        store.insert(
            key,
            CacheEntry {
                inserted_at: Instant::now(),
                ttl,
                val,
            },
        );
    }

    pub async fn len(&self) -> usize {
        let store = self.store.read().await;
        store.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    pub async fn invalidate(&self, key_prefix: &str) {
        let mut store = self.store.write().await;
        store.retain(|k, _| !k.starts_with(key_prefix));
    }

    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        store.clear();
    }
}

impl Default for MinifluxCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_miniflux_cache_get_and_insert_with_ttl() {
        let cache = MinifluxCache::new(Duration::from_millis(50));
        let val = json!({"foo": "bar"});

        cache.insert_with_ttl("test_key".to_string(), val.clone(), Duration::from_millis(50)).await;

        let cached = cache.get("test_key").await;
        assert_eq!(cached, Some(val));

        tokio::time::sleep(Duration::from_millis(60)).await;
        let expired = cache.get("test_key").await;
        assert_eq!(expired, None);
    }

    #[tokio::test]
    async fn test_miniflux_cache_bounded_capacity() {
        let cache = MinifluxCache::new(Duration::from_secs(60));
        for i in 0..1005 {
            cache
                .insert_with_ttl(format!("key_{}", i), json!(i), Duration::from_secs(60))
                .await;
        }

        assert_eq!(cache.len().await, 1000);
        // The first 5 entries should have been evicted as oldest
        for i in 0..5 {
            assert_eq!(cache.get(&format!("key_{}", i)).await, None);
        }
        // Newer entries should still exist
        assert_eq!(cache.get("key_1004").await, Some(json!(1004)));
    }

    #[tokio::test]
    async fn test_miniflux_cache_inline_eviction_of_expired() {
        let cache = MinifluxCache::new(Duration::from_secs(60));
        // Fill cache with 1000 items, first 10 have very short TTL
        for i in 0..10 {
            cache
                .insert_with_ttl(format!("expired_{}", i), json!(i), Duration::from_millis(10))
                .await;
        }
        for i in 10..1000 {
            cache
                .insert_with_ttl(format!("alive_{}", i), json!(i), Duration::from_secs(60))
                .await;
        }

        tokio::time::sleep(Duration::from_millis(25)).await;

        // Inserting one more item should evict expired items rather than oldest alive items
        cache
            .insert_with_ttl("new_item".to_string(), json!("new"), Duration::from_secs(60))
            .await;

        // Expired items should be gone
        assert_eq!(cache.get("expired_0").await, None);
        // An alive item (e.g., alive_10) should still be present
        assert_eq!(cache.get("alive_10").await, Some(json!(10)));
        assert_eq!(cache.get("new_item").await, Some(json!("new")));
        assert!(cache.len().await <= 1000);
    }
}
