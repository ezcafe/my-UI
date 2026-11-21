use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MinifluxCache {
    store: Arc<RwLock<HashMap<String, (Instant, Duration, serde_json::Value)>>>,
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
        if let Some((inserted_at, ttl, val)) = store.get(key) {
            if inserted_at.elapsed() < *ttl {
                return Some(val.clone());
            }
        }
        None
    }

    pub async fn insert(&self, key: String, val: serde_json::Value) {
        self.insert_with_ttl(key, val, self.default_ttl).await;
    }

    pub async fn insert_with_ttl(&self, key: String, val: serde_json::Value, ttl: Duration) {
        let mut store = self.store.write().await;
        store.insert(key, (Instant::now(), ttl, val));
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
}
