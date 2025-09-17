use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub data: JsonValue,
    pub timestamp: u64,
    pub ttl: u64,
}

impl CacheEntry {
    pub fn new(data: JsonValue, ttl: Duration) -> Self {
        Self {
            data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: ttl.as_secs(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.timestamp > self.ttl
    }
}

pub fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn get_cache_dir() -> PathBuf {
    // Prefer XDG_CACHE_HOME if set, otherwise fall back to ~/.cache/lectern
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("lectern");
    }

    if let Some(home) = dirs::home_dir() {
        return home.join(".cache").join("lectern");
    }

    // Fallback to current dir if we couldn't determine a home directory
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".lectern_cache")
}

// Note: The cache is now global per user. It lives under `$XDG_CACHE_HOME/lectern` when
// available, otherwise `~/.cache/lectern`. This keeps cache data shared across projects
// and avoids creating per-project `.lectern_cache` directories.

pub fn get_cache_file_path(cache_type: &str, key: &str) -> PathBuf {
    let cache_dir = get_cache_dir().join(cache_type);
    let hashed_key = hash_key(key);
    cache_dir.join(format!("{hashed_key}.json"))
}
