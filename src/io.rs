use crate::model::{ComposerJson, Lock};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use serde_json;

// Composer JSON support
pub fn read_composer_json(path: &Path) -> Result<ComposerJson> {
    let s = fs::read_to_string(path).with_context(|| format!("read {path:?}"))?;
    let json: ComposerJson = serde_json::from_str(&s).context("parse composer.json")?;
    Ok(json)
}

pub fn write_composer_json(path: &Path, composer: &ComposerJson) -> Result<()> {
    let s = serde_json::to_string_pretty(composer)?;
    let mut f = fs::File::create(path)?;
    f.write_all(s.as_bytes())?;
    Ok(())
}

// Read project configuration
pub fn read_project_config(working_dir: &Path) -> Result<ComposerJson> {
    let composer_path = working_dir.join("composer.json");
    if composer_path.exists() {
        read_composer_json(&composer_path)
    } else {
        Err(anyhow::anyhow!("No composer.json found"))
    }
}

// Lock file operations (JSON format for Composer compatibility)
pub fn read_lock(path: &Path) -> Result<Lock> {
    let s = fs::read_to_string(path).with_context(|| format!("read {path:?}"))?;
    let lock: Lock = serde_json::from_str(&s).context("parse lock file")?;
    Ok(lock)
}

pub fn write_lock(path: &Path, lock: &Lock) -> Result<()> {
    let s = serde_json::to_string_pretty(lock)?;
    let mut f = fs::File::create(path)?;
    f.write_all(s.as_bytes())?;
    Ok(())
}

/// Write cache data to a file
pub fn write_cache(path: &Path, cache: &HashMap<String, String>) -> Result<()> {
    let cache_dir = Path::new(".lectern_cache");
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)?;
    }
    let cache_path = cache_dir.join(path);
    let s = serde_json::to_string_pretty(cache)?;
    let mut f = fs::File::create(&cache_path)?;
    f.write_all(s.as_bytes())?;
    Ok(())
}

/// Read cache data from a file
pub fn read_cache(path: &Path) -> Result<HashMap<String, String>> {
    let cache_dir = Path::new(".lectern_cache");
    let cache_path = cache_dir.join(path);
    if cache_path.exists() {
        let s = fs::read_to_string(&cache_path).with_context(|| format!("read {cache_path:?}"))?;
        let cache: HashMap<String, String> = serde_json::from_str(&s).context("parse cache file")?;
        Ok(cache)
    } else {
        Ok(HashMap::new())
    }
}

pub async fn clean(dir: &Path) -> Result<()> {
    let vendor = dir.join("vendor");
    if vendor.exists() {
        tokio::fs::remove_dir_all(&vendor).await.ok();
    }
    // Clean lock file
    let composer_lock = dir.join("composer.lock");
    if composer_lock.exists() {
        tokio::fs::remove_file(&composer_lock).await.ok();
    }
    Ok(())
}
