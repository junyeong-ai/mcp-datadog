use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CacheEntry<T> {
    data: T,
    created_at: Instant,
    last_accessed: Instant,
}

impl<T: Clone> CacheEntry<T> {
    fn new(data: T) -> Self {
        let now = Instant::now();
        Self {
            data,
            created_at: now,
            last_accessed: now,
        }
    }

    fn access(&mut self) -> T {
        self.last_accessed = Instant::now();
        self.data.clone()
    }

    fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

pub struct GenericCache<T: Clone> {
    entries: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    ttl: Duration,
    max_entries: usize,
}

impl<T: Clone + Serialize> GenericCache<T> {
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            max_entries,
        }
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        let mut cache = self.entries.write().await;

        if let Some(entry) = cache.get_mut(key) {
            if entry.age() < self.ttl {
                return Some(entry.access());
            }
            cache.remove(key);
            log::debug!("Cache expired: {}", key);
        }
        None
    }

    pub async fn set(&self, key: String, data: T) {
        let mut cache = self.entries.write().await;

        if cache.len() >= self.max_entries && !cache.contains_key(&key) {
            self.evict_lru(&mut cache);
        }

        cache.insert(key, CacheEntry::new(data));
    }

    pub async fn get_or_fetch<F, Fut>(&self, key: &str, fetch_fn: F) -> crate::error::Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = crate::error::Result<T>>,
    {
        if let Some(cached) = self.get(key).await {
            log::debug!("Cache hit: {}", key);
            return Ok(cached);
        }

        log::debug!("Cache miss: {}", key);
        let data = fetch_fn().await?;
        self.set(key.to_string(), data.clone()).await;

        Ok(data)
    }

    fn evict_lru(&self, cache: &mut HashMap<String, CacheEntry<T>>) {
        if let Some(lru_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&lru_key);
            log::debug!("Evicted LRU cache entry: {}", lru_key);
        }
    }

    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.entries.write().await;
        let initial_count = cache.len();

        cache.retain(|key, entry| {
            let keep = entry.age() < self.ttl;
            if !keep {
                log::debug!("Expired cache entry: {}", key);
            }
            keep
        });

        initial_count - cache.len()
    }
}

use crate::datadog::models::*;

pub struct DataCache {
    dashboards: GenericCache<Vec<DashboardSummary>>,
    monitors: GenericCache<Vec<Monitor>>,
    events: GenericCache<Vec<Event>>,
}

impl DataCache {
    pub fn new(ttl_seconds: u64) -> Self {
        let ttl = Duration::from_secs(ttl_seconds);
        let max_entries = 100;

        Self {
            dashboards: GenericCache::new(ttl, max_entries),
            monitors: GenericCache::new(ttl, max_entries),
            events: GenericCache::new(ttl, max_entries),
        }
    }

    pub async fn set_dashboards(&self, key: String, data: Vec<DashboardSummary>) {
        self.dashboards.set(key, data).await
    }

    pub async fn get_or_fetch_dashboards<F, Fut>(
        &self,
        key: &str,
        fetch: F,
    ) -> crate::error::Result<Vec<DashboardSummary>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = crate::error::Result<Vec<DashboardSummary>>>,
    {
        self.dashboards.get_or_fetch(key, fetch).await
    }

    pub async fn set_monitors(&self, key: String, data: Vec<Monitor>) {
        self.monitors.set(key, data).await
    }

    pub async fn get_or_fetch_monitors<F, Fut>(
        &self,
        key: &str,
        fetch: F,
    ) -> crate::error::Result<Vec<Monitor>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = crate::error::Result<Vec<Monitor>>>,
    {
        self.monitors.get_or_fetch(key, fetch).await
    }

    pub async fn set_events(&self, key: String, data: Vec<Event>) {
        self.events.set(key, data).await
    }

    pub async fn get_or_fetch_events<F, Fut>(
        &self,
        key: &str,
        fetch: F,
    ) -> crate::error::Result<Vec<Event>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = crate::error::Result<Vec<Event>>>,
    {
        self.events.get_or_fetch(key, fetch).await
    }

    pub async fn cleanup_all_expired(&self) -> usize {
        let mut total = 0;
        total += self.dashboards.cleanup_expired().await;
        total += self.monitors.cleanup_expired().await;
        total += self.events.cleanup_expired().await;
        total
    }
}

pub fn create_cache_key<T: Serialize>(endpoint: &str, params: &T) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let params_json = serde_json::to_string(params).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    params_json.hash(&mut hasher);
    let hash = hasher.finish();

    format!("{}:{:x}", endpoint, hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let cache: GenericCache<String> = GenericCache::new(Duration::from_secs(60), 100);

        cache.set("key1".to_string(), "value1".to_string()).await;

        let result = cache.get("key1").await;
        assert_eq!(result, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache: GenericCache<String> = GenericCache::new(Duration::from_secs(60), 100);

        let result = cache.get("nonexistent").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_ttl_expiration() {
        let cache: GenericCache<String> = GenericCache::new(Duration::from_millis(100), 100);

        cache.set("key1".to_string(), "value1".to_string()).await;

        // Should exist immediately
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired now
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_get_or_fetch_hit() {
        let cache: GenericCache<i32> = GenericCache::new(Duration::from_secs(60), 100);

        // Pre-populate cache
        cache.set("key1".to_string(), 42).await;

        // Fetch should return cached value without calling fetch function
        let result = cache.get_or_fetch("key1", || async { Ok(100) }).await;
        assert_eq!(result.unwrap(), 42); // Should be cached value, not 100
    }

    #[tokio::test]
    async fn test_cache_get_or_fetch_miss() {
        let cache: GenericCache<i32> = GenericCache::new(Duration::from_secs(60), 100);

        // Fetch should call the function and cache the result
        let result = cache.get_or_fetch("key1", || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);

        // Second fetch should return cached value
        let result2 = cache.get("key1").await;
        assert_eq!(result2, Some(42));
    }

    #[tokio::test]
    async fn test_cache_cleanup_expired() {
        let cache: GenericCache<String> = GenericCache::new(Duration::from_millis(50), 100);

        // Add some entries
        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.set("key2".to_string(), "value2".to_string()).await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Cleanup should remove expired entries
        let removed = cache.cleanup_expired().await;
        assert_eq!(removed, 2);
    }

    #[test]
    fn test_create_cache_key() {
        let key1 = create_cache_key("/api/metrics", &json!({"query": "cpu"}));
        let key2 = create_cache_key("/api/metrics", &json!({"query": "cpu"}));
        let key3 = create_cache_key("/api/metrics", &json!({"query": "mem"}));

        // Same params should create same key
        assert_eq!(key1, key2);

        // Different params should create different key
        assert_ne!(key1, key3);

        // Keys should start with endpoint
        assert!(key1.starts_with("/api/metrics:"));
    }

    #[tokio::test]
    async fn test_concurrent_cache_access() {
        let cache: Arc<GenericCache<i32>> =
            Arc::new(GenericCache::new(Duration::from_secs(60), 100));
        let mut handles = vec![];

        // Spawn multiple concurrent writes
        for i in 0..10 {
            let cache_clone = cache.clone();
            handles.push(tokio::spawn(async move {
                cache_clone.set(format!("key{}", i), i).await;
            }));
        }

        // Wait for all writes
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all writes succeeded
        for i in 0..10 {
            let result = cache.get(&format!("key{}", i)).await;
            assert_eq!(result, Some(i));
        }
    }
}
