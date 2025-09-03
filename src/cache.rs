use anyhow::Result;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sha2::{Digest, Sha256};
use tokio::fs;

const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour TTL
const PACKAGE_INFO_TTL: Duration = Duration::from_secs(1800); // 30 minutes for package info
const SEARCH_TTL: Duration = Duration::from_secs(900); // 15 minutes for search results

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

async fn get_cache_file_path(cache_type: &str, key: &str) -> PathBuf {
    let cache_dir = get_cache_dir().join(cache_type);
    let hashed_key = hash_key(key);
    cache_dir.join(format!("{}.json", hashed_key))
}

async fn load_from_cache(cache_type: &str, key: &str) -> Option<JsonValue> {
    let file_path = get_cache_file_path(cache_type, key).await;
    
    match fs::read_to_string(&file_path).await {
        Ok(content) => {
            match serde_json::from_str::<CacheEntry>(&content) {
                Ok(entry) => {
                    if entry.is_expired() {
                        // Remove expired cache file
                        fs::remove_file(&file_path).await.ok();
                        None
                    } else {
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
    let cache_dir = get_cache_dir().join(cache_type);
    fs::create_dir_all(&cache_dir).await?;

    let file_path = get_cache_file_path(cache_type, key).await;
    let entry = CacheEntry::new(value.clone(), ttl);
    let content = serde_json::to_string(&entry)?;
    
    fs::write(&file_path, content).await?;
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

    for key in keys {
        if let Some(value) = load_from_cache("package_info", key).await {
            results.insert(key.clone(), value);
        }
    }

    results
}

pub async fn cache_set_multiple_package_info(data: HashMap<String, JsonValue>) {
    for (key, value) in data {
        save_to_cache("package_info", &key, &value, PACKAGE_INFO_TTL).await.ok();
    }
}

// Clear all caches
pub async fn clear_cache() -> Result<()> {
    let cache_dir = get_cache_dir();
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).await?;
    }
    Ok(())
}

// Clear specific cache type
pub async fn clear_cache_type(cache_type: &str) -> Result<()> {
    let cache_dir = get_cache_dir().join(cache_type);
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).await?;
    }
    Ok(())
}

// Get cache statistics
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
                    stats.insert(cache_type.to_string(), count);
                }
                Err(_) => {
                    stats.insert(cache_type.to_string(), 0);
                }
            }
        } else {
            stats.insert(cache_type.to_string(), 0);
        }
    }
    
    Ok(stats)
}
