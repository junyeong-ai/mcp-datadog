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
            } else {
                cache.remove(key);
                log::debug!("Cache expired: {}", key);
            }
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
