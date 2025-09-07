use anyhow::Result;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::sync::RwLock;
use std::sync::{Arc, LazyLock};
use lru::LruCache;
use std::num::NonZeroUsize;

const CACHE_TTL: Duration = Duration::from_secs(86400); // 24 hours TTL
const PACKAGE_INFO_TTL: Duration = Duration::from_secs(43200); // 12 hours for package info
const SEARCH_TTL: Duration = Duration::from_secs(7200); // 2 hours for search results
const DEPENDENCY_RESOLVE_TTL: Duration = Duration::from_secs(604800); // 7 days for dependency resolution

// Type alias for complex cache type
type MemoryCacheType = LazyLock<Arc<RwLock<LruCache<String, (JsonValue, u64)>>>>;

// Memory cache for ultra-fast access
static MEMORY_CACHE: MemoryCacheType = 
    LazyLock::new(|| Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(10000).unwrap()))));

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    data: JsonValue,
    timestamp: u64,
    ttl: u64,
}

impl CacheEntry {
    fn new(data: JsonValue, ttl: Duration) -> Self {
        Self {
            data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: ttl.as_secs(),
        }
    }

    fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.timestamp > self.ttl
    }
}

fn get_cache_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".lectern_cache")
}

fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_cache_file_path(cache_type: &str, key: &str) -> PathBuf {
    let cache_dir = get_cache_dir().join(cache_type);
    let hashed_key = hash_key(key);
    cache_dir.join(format!("{hashed_key}.json"))
}

async fn load_from_cache(cache_type: &str, key: &str) -> Option<JsonValue> {
    let cache_key = format!("{cache_type}:{key}");
    
    // First check memory cache for ultra-fast access
    {
        let cache = MEMORY_CACHE.read().await;
        if let Some((value, timestamp)) = cache.peek(&cache_key) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let ttl = match cache_type {
                "package_info" => PACKAGE_INFO_TTL.as_secs(),
                "search" => SEARCH_TTL.as_secs(),
                "dependency_resolution" => DEPENDENCY_RESOLVE_TTL.as_secs(),
                _ => CACHE_TTL.as_secs(),
            };
            
            if now - timestamp <= ttl {
                return Some(value.clone());
            }
        }
    }
    
    // Fallback to disk cache
    let file_path = get_cache_file_path(cache_type, key);
    
    match fs::read_to_string(&file_path).await {
        Ok(content) => {
            match serde_json::from_str::<CacheEntry>(&content) {
                Ok(entry) => {
                    if entry.is_expired() {
                        // Remove expired cache file asynchronously
                        tokio::spawn(async move {
                            fs::remove_file(&file_path).await.ok();
                        });
                        None
                    } else {
                        // Store in memory cache for next time
                        {
                            let mut cache = MEMORY_CACHE.write().await;
                            cache.put(cache_key, (entry.data.clone(), entry.timestamp));
                        }
                        Some(entry.data)
                    }
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

async fn save_to_cache(cache_type: &str, key: &str, value: &JsonValue, ttl: Duration) -> Result<()> {
    let cache_key = format!("{cache_type}:{key}");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Store in memory cache first for immediate access
    {
        let mut cache = MEMORY_CACHE.write().await;
        cache.put(cache_key, (value.clone(), timestamp));
    }
    
    // Asynchronously save to disk cache
    let cache_dir = get_cache_dir().join(cache_type);
    let file_path = get_cache_file_path(cache_type, key);
    let entry = CacheEntry::new(value.clone(), ttl);
    
    tokio::spawn(async move {
        if let Err(e) = fs::create_dir_all(&cache_dir).await {
            eprintln!("Failed to create cache dir: {e}");
            return;
        }
        
        if let Ok(content) = serde_json::to_string(&entry) {
            if let Err(e) = fs::write(&file_path, content).await {
                eprintln!("Failed to write cache file: {e}");
            }
        }
    });
    
    Ok(())
}

// Generic cache functions
pub async fn cache_get_meta(key: &str) -> Option<JsonValue> {
    load_from_cache("meta", key).await
}

pub async fn cache_set_meta(key: &str, val: JsonValue) {
    save_to_cache("meta", key, &val, CACHE_TTL).await.ok();
}

// Package info specific cache
pub async fn cache_get_package_info(key: &str) -> Option<JsonValue> {
    load_from_cache("package_info", key).await
}

pub async fn cache_set_package_info(key: &str, val: JsonValue) {
    save_to_cache("package_info", key, &val, PACKAGE_INFO_TTL).await.ok();
}

// Search cache
pub async fn cache_get_search(key: &str) -> Option<JsonValue> {
    load_from_cache("search", key).await
}

pub async fn cache_set_search(key: &str, val: JsonValue) {
    save_to_cache("search", key, &val, SEARCH_TTL).await.ok();
}

// Bulk cache operations for better performance
pub async fn cache_get_multiple_package_info(keys: &[String]) -> HashMap<String, JsonValue> {
    let mut results = HashMap::new();
    
    // Use concurrent loading for better performance
    let futures: Vec<_> = keys.iter().map(|key| {
        let key_clone = key.clone();
        async move {
            load_from_cache("package_info", &key_clone).await.map(|value| (key_clone, value))
        }
    }).collect();
    
    let cache_results = futures::future::join_all(futures).await;
    
    for (key, value) in cache_results.into_iter().flatten() {
        results.insert(key, value);
    }

    results
}

pub async fn cache_set_multiple_package_info<S: ::std::hash::BuildHasher>(data: HashMap<String, JsonValue, S>) {
    // Use concurrent saving for better performance
    let futures: Vec<_> = data.into_iter().map(|(key, value)| {
        async move {
            save_to_cache("package_info", &key, &value, PACKAGE_INFO_TTL).await.ok();
        }
    }).collect();
    
    futures::future::join_all(futures).await;
}

// Enhanced dependency resolution cache
pub async fn cache_get_dependency_resolution(key: &str) -> Option<JsonValue> {
    load_from_cache("dependency_resolution", key).await
}

pub async fn cache_set_dependency_resolution(key: &str, val: JsonValue) {
    save_to_cache("dependency_resolution", key, &val, DEPENDENCY_RESOLVE_TTL).await.ok();
}

// Clear all caches
/// # Errors
/// Returns an error if the cache directory cannot be removed
pub async fn clear_cache() -> Result<()> {
    let cache_dir = get_cache_dir();
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).await?;
    }
    Ok(())
}

// Clear specific cache type
/// # Errors
/// Returns an error if the cache directory cannot be removed
pub async fn clear_cache_type(cache_type: &str) -> Result<()> {
    let cache_dir = get_cache_dir().join(cache_type);
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).await?;
    }
    Ok(())
}

// Get cache statistics
/// # Errors
/// Returns an error if the cache directory cannot be read
pub async fn get_cache_stats() -> Result<HashMap<String, usize>> {
    let mut stats = HashMap::new();
    let cache_dir = get_cache_dir();
    
    if !cache_dir.exists() {
        return Ok(stats);
    }

    let cache_types = ["meta", "package_info", "search"];
    
    for cache_type in &cache_types {
        let type_dir = cache_dir.join(cache_type);
        if type_dir.exists() {
            match fs::read_dir(&type_dir).await {
                Ok(mut entries) => {
                    let mut count = 0;
                    while let Ok(Some(_)) = entries.next_entry().await {
                        count += 1;
                    }
                    stats.insert((*cache_type).to_string(), count);
                }
                Err(_) => {
                    stats.insert((*cache_type).to_string(), 0);
                }
            }
        } else {
            stats.insert((*cache_type).to_string(), 0);
        }
    }
    
    Ok(stats)
}
