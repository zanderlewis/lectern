use crate::cache;
use anyhow::{Context, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct P2Envelope {
    pub packages: BTreeMap<String, Vec<P2Version>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct P2Version {
    pub version: String,
    #[serde(default)]
    pub version_normalized: String,
    #[serde(default)]
    pub dist: Option<P2Dist>,
    #[serde(default)]
    pub source: Option<P2Source>,
    #[serde(default)]
    pub require: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub extra: Option<serde_json::Value>,
    // Everything else as raw JSON to avoid parsing issues
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct P2Dist {
    #[serde(rename = "type")]
    pub dtype: Option<String>,
    pub url: Option<String>,
    pub reference: Option<String>,
    pub shasum: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct P2Source {
    #[serde(rename = "type")]
    pub stype: Option<String>,
    pub url: Option<String>,
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub repository: Option<String>,
    pub downloads: Option<u32>,
    pub favers: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageInfo {
    pub package: PackageDetails,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageDetails {
    pub name: String,
    pub description: Option<String>,
    pub time: Option<String>,
    pub maintainers: Option<Vec<Maintainer>>,
    pub versions: Option<BTreeMap<String, VersionDetails>>,
    pub repository: Option<String>,
    #[serde(rename = "type")]
    pub package_type: Option<String>,
    pub downloads: Option<DownloadStats>,
    pub favers: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Maintainer {
    pub name: String,
    pub email: Option<String>,
    pub homepage: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionDetails {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub license: Option<Vec<String>>,
    pub authors: Option<Vec<Author>>,
    pub require: Option<BTreeMap<String, String>>,
    #[serde(rename = "require-dev")]
    pub require_dev: Option<BTreeMap<String, String>>,
    pub time: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
    pub homepage: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadStats {
    pub total: Option<u32>,
    pub monthly: Option<u32>,
    pub daily: Option<u32>,
}

/// Recursively remove fields with "__unset" values from JSON
fn clean_unset_values(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            // Remove keys with "__unset" values
            map.retain(|_, v| {
                if let serde_json::Value::String(s) = v {
                    s != "__unset"
                } else {
                    true
                }
            });
            // Recursively clean remaining values
            for v in map.values_mut() {
                clean_unset_values(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                clean_unset_values(v);
            }
        }
        _ => {}
    }
}

/// Fetch packagist p2 JSON using client, with in-memory cache
pub async fn fetch_packagist_versions_cached(client: &Client, pkg: &str) -> Result<Vec<P2Version>> {
    if let Some(cached) = cache::cache_get_meta(&format!("p2:{pkg}")).await {
        let list: Vec<P2Version> = serde_json::from_value(cached)?;
        return Ok(list);
    }
    let url = format!("https://repo.packagist.org/p2/{pkg}.json");
    let resp = client
        .get(&url)
        .send()
        .await
        .context("packagist request")?
        .error_for_status()?;

    // Get the raw JSON text
    let json_text = resp.text().await.context("get response text")?;

    // Try to parse as raw JSON first
    let mut json_value: serde_json::Value =
        serde_json::from_str(&json_text).context("parse raw json")?;

    // Clean up "__unset" values that Packagist uses
    clean_unset_values(&mut json_value);

    // Try to extract the envelope
    let env: P2Envelope = serde_json::from_value(json_value)
        .with_context(|| format!("parse packagist p2 json for package: {pkg}"))?;

    let list = env.packages.get(pkg).cloned().unwrap_or_default();
    cache::cache_set_meta(&format!("p2:{pkg}"), serde_json::to_value(&list)?).await;
    Ok(list)
}

/// Check if a package name represents a platform dependency
pub fn is_platform_dependency(package_name: &str) -> bool {
    package_name == "php"
        || package_name.starts_with("ext-")
        || package_name.starts_with("lib-")
        || package_name == "hhvm"
        || package_name == "composer-runtime-api"
        || package_name == "composer-plugin-api"
}

/// Search packages on Packagist
pub async fn search_packagist(terms: &[String]) -> Result<Vec<SearchResult>> {
    let query = terms.join(" ");

    // Check cache first
    let cache_key = format!("search:{query}");
    if let Some(cached) = cache::cache_get_search(&cache_key).await {
        return Ok(serde_json::from_value(cached)?);
    }

    let client = Client::new();
    let url = format!(
        "https://packagist.org/search.json?q={}&per_page=15",
        urlencoding::encode(&query)
    );

    let resp = client
        .get(&url)
        .header("User-Agent", "Lectern/0.1.0")
        .send()
        .await
        .context("packagist search request")?
        .error_for_status()?;

    #[derive(Deserialize)]
    struct SearchResponse {
        results: Vec<SearchResult>,
    }

    let search_resp: SearchResponse = resp.json().await.context("parse search response")?;

    // Cache the results
    cache::cache_set_search(&cache_key, serde_json::to_value(&search_resp.results)?).await;

    Ok(search_resp.results)
}

/// Fetch detailed package information
pub async fn fetch_package_info(package_name: &str) -> Result<PackageInfo> {
    // Check cache first
    let cache_key = format!("package_info:{package_name}");
    if let Some(cached) = cache::cache_get_package_info(&cache_key).await {
        return Ok(serde_json::from_value(cached)?);
    }

    let client = Client::new();
    let url = format!("https://packagist.org/packages/{package_name}.json");

    let resp = client
        .get(&url)
        .header("User-Agent", "Lectern/0.1.0")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .context("packagist package info request")?
        .error_for_status()?;

    let package_info: PackageInfo = resp.json().await.context("parse package info response")?;

    // Cache the results
    cache::cache_set_package_info(&cache_key, serde_json::to_value(&package_info)?).await;

    Ok(package_info)
}

/// Fetch multiple package info concurrently with caching
pub async fn fetch_multiple_package_info(
    package_names: &[String],
) -> Result<Vec<(String, Option<PackageInfo>)>> {
    // Try to get from bulk cache first
    let cached_results = cache::cache_get_multiple_package_info(package_names).await;

    // Convert cached results to expected format
    let mut final_results = Vec::new();
    let mut missing_packages = Vec::new();

    for package_name in package_names {
        if let Some(cached_value) = cached_results.get(package_name) {
            match serde_json::from_value::<PackageInfo>(cached_value.clone()) {
                Ok(package_info) => final_results.push((package_name.clone(), Some(package_info))),
                Err(_) => missing_packages.push(package_name.clone()),
            }
        } else {
            missing_packages.push(package_name.clone());
        }
    }

    // If we have all results cached, return them
    if missing_packages.is_empty() {
        return Ok(final_results);
    }

    let mut futures = FuturesUnordered::new();

    for package_name in &missing_packages {
        let name = package_name.clone();
        futures.push(async move {
            match fetch_package_info(&name).await {
                Ok(info) => (name, Some(info)),
                Err(_) => (name, None),
            }
        });
    }

    while let Some(result) = futures.next().await {
        final_results.push(result);
    }

    // Cache the new results
    let mut cache_data = std::collections::HashMap::new();
    for (name, info_opt) in &final_results {
        if let Some(info) = info_opt {
            if let Ok(json_value) = serde_json::to_value(info) {
                cache_data.insert(name.clone(), json_value);
            }
        }
    }

    if !cache_data.is_empty() {
        cache::cache_set_multiple_package_info(cache_data).await;
    }

    Ok(final_results)
}
