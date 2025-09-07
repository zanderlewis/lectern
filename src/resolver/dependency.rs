use crate::model::{ComposerJson, LockedPackage, SourceInfo, DistInfo};
use crate::resolver::packagist::{
    P2Version, fetch_packagist_versions_cached, fetch_packagist_versions_bulk, is_platform_dependency,
};
use crate::resolver::version::parse_constraint;
use crate::utils::{print_error, print_info, print_step, print_success, print_warning};
use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use semver::{Version, VersionReq};
use std::collections::{BTreeSet, VecDeque, BTreeMap};
use std::path::Path;
use sha2::{Sha256, Digest};

/// Main dependency resolution function with batch processing optimization
pub async fn solve(composer: &ComposerJson) -> Result<crate::model::Lock> {
    print_step("🔍 Resolving dependencies...");

    let client = Client::new();
    let mut locked_packages = Vec::new();
    let mut processed = BTreeSet::new();
    let mut queue = VecDeque::new();
    let mut dev_package_names = BTreeSet::new();

    // Collect all dependencies first for batch processing
    let mut all_deps = Vec::new();
    
    // Add all direct dependencies to the queue
    for (name, constraint) in &composer.require {
        // Skip platform dependencies
        if is_platform_dependency(name) {
            print_info(&format!("⏭️  Skipping platform dependency: {name}"));
            continue;
        }
        queue.push_back((name.clone(), constraint.clone(), false));
        all_deps.push(name.clone());
    }

    for (name, constraint) in &composer.require_dev {
        // Skip platform dependencies
        if is_platform_dependency(name) {
            print_info(&format!("⏭️  Skipping platform dependency: {name}"));
            continue;
        }
        dev_package_names.insert(name.clone());
        queue.push_back((name.clone(), constraint.clone(), true));
        all_deps.push(name.clone());
    }

    // Pre-fetch all direct dependencies in bulk for better performance
    if !all_deps.is_empty() {
        print_info(&format!("📥 Pre-fetching {} dependencies in batch...", all_deps.len()));
        let _bulk_versions = fetch_packagist_versions_bulk(&client, &all_deps).await.unwrap_or_default();
        print_success("✅ Batch pre-fetch completed");
    }

    while let Some((pkg_name, constraint_str, is_dev)) = queue.pop_front() {
        if processed.contains(&pkg_name) {
            continue;
        }
        processed.insert(pkg_name.clone());

        print_info(&format!("📦 Processing: {pkg_name} ({constraint_str})"));

        // Handle repository paths
        if let Some(path_pkg) = read_package_from_path(Path::new(&pkg_name))? {
            let locked = LockedPackage {
                name: path_pkg.0,
                version: path_pkg.1.unwrap_or_else(|| "dev-main".to_string()),
                source: Some(SourceInfo {
                    source_type: "path".to_string(),
                    url: pkg_name.clone(),
                    reference: "HEAD".to_string(),
                }),
                dist: None,
                require: None,
                require_dev: None,
                conflict: None,
                replace: None,
                provide: None,
                suggest: None,
                package_type: Some("library".to_string()),
                extra: None,
                autoload: None,
                autoload_dev: None,
                notification_url: None,
                license: None,
                authors: None,
                description: None,
                homepage: None,
                keywords: None,
                support: None,
                funding: None,
                time: None,
                bin: None,
                include_path: None,
            };
            locked_packages.push(locked);
            continue;
        }

        // Fetch available versions from Packagist
        let versions = match fetch_packagist_versions_cached(&client, &pkg_name).await {
            Ok(v) => v,
            Err(e) => {
                print_warning(&format!("⚠️  Could not fetch versions for {pkg_name}: {e}"));
                continue;
            }
        };

        if versions.is_empty() {
            print_warning(&format!("⚠️  No versions found for package: {pkg_name}"));
            continue;
        }

        // Parse the constraint
        let constraint = match parse_constraint(&constraint_str) {
            Ok(c) => c,
            Err(e) => {
                print_error(&format!(
                    "❌ Invalid constraint '{constraint_str}' for package {pkg_name}: {e}"
                ));
                continue;
            }
        };

        // Find the best matching version
        let best_version = match find_best_version(&versions, &constraint) {
            Ok(v) => v,
            Err(e) => {
                print_error(&format!(
                    "❌ No version satisfies constraint '{constraint_str}' for package {pkg_name}: {e}"
                ));
                print_info(&format!("Available versions for {pkg_name}: {}", 
                    versions.iter()
                        .take(5)
                        .map(|v| v.version.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                return Err(anyhow!("No version satisfies constraint '{constraint_str}' for package {pkg_name}"));
            }
        };

        let locked = LockedPackage {
            name: pkg_name.clone(),
            version: best_version.version.clone(),
            source: best_version.source.as_ref().map(|s| SourceInfo {
                source_type: s.stype.clone().unwrap_or_else(|| "git".to_string()),
                url: s.url.clone().unwrap_or_default(),
                reference: s.reference.clone().unwrap_or_default(),
            }),
            dist: best_version.dist.as_ref().map(|d| DistInfo {
                dist_type: d.dtype.clone().unwrap_or_else(|| "zip".to_string()),
                url: d.url.clone().unwrap_or_default(),
                reference: d.reference.clone().unwrap_or_default(),
                shasum: d.shasum.clone().unwrap_or_default(),
            }),
            require: best_version.require.clone(),
            require_dev: best_version.other.get("require-dev")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            conflict: best_version.other.get("conflict")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            replace: best_version.other.get("replace")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            provide: best_version.other.get("provide")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            suggest: best_version.other.get("suggest")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            package_type: best_version.other.get("type")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .or_else(|| Some("library".to_string())),
            extra: best_version.extra.clone(),
            autoload: best_version.other.get("autoload")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            autoload_dev: best_version.other.get("autoload-dev")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            notification_url: Some("https://packagist.org/downloads/".to_string()),
            license: best_version.other.get("license")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            authors: best_version.other.get("authors")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            description: best_version.other.get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            homepage: best_version.other.get("homepage")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            keywords: best_version.other.get("keywords")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            support: best_version.other.get("support")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            funding: best_version.other.get("funding")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            time: best_version.other.get("time")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            bin: best_version.other.get("bin")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            include_path: best_version.other.get("include-path")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
        };

        // Add dependencies to the queue
        if let Some(deps) = &best_version.require {
            for (dep_name, dep_constraint) in deps {
                // Skip platform dependencies
                if is_platform_dependency(dep_name) {
                    continue;
                }
                if !processed.contains(dep_name) {
                    // Mark transitive dependencies of dev packages as dev too
                    if is_dev {
                        dev_package_names.insert(dep_name.clone());
                    }
                    queue.push_back((dep_name.clone(), dep_constraint.clone(), is_dev));
                }
            }
        }

        locked_packages.push(locked);
    }

    // Sort packages by name for consistent output
    locked_packages.sort_by(|a, b| a.name.cmp(&b.name));
    
    // Separate dev and regular packages
    let (dev_packages, regular_packages): (Vec<_>, Vec<_>) = locked_packages
        .into_iter()
        .partition(|pkg| dev_package_names.contains(&pkg.name));

    print_success(&format!("✅ Resolved {} packages", regular_packages.len() + dev_packages.len()));

    // Generate content hash for the lock file
    let content_hash = generate_content_hash_from_composer(composer);
    
    Ok(crate::model::Lock {
        _readme: vec![
            "This file locks the dependencies of your project to a known state".to_string(),
            "Read more about it at https://getcomposer.org/doc/01-basic-usage.md#installing-dependencies".to_string(),
            "This file is @generated automatically".to_string(),
        ],
        content_hash,
        packages: regular_packages,
        packages_dev: dev_packages,
        aliases: vec![],
        minimum_stability: composer.minimum_stability.clone().unwrap_or_else(|| "stable".to_string()),
        stability_flags: BTreeMap::new(),
        prefer_stable: composer.prefer_stable.unwrap_or(false),
        prefer_lowest: false,
        platform: BTreeMap::new(),
        platform_dev: BTreeMap::new(),
        plugin_api_version: Some("2.6.0".to_string()),
    })
}

/// Generate content hash from composer.json content
pub fn generate_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Generate content hash from ComposerJson structure
fn generate_content_hash_from_composer(composer: &ComposerJson) -> String {
    let mut hasher = Sha256::new();
    
    // Create a normalized representation for hashing
    let mut content = String::new();
    content.push_str(&serde_json::to_string(&composer.require).unwrap_or_default());
    content.push_str(&serde_json::to_string(&composer.require_dev).unwrap_or_default());
    
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Find the best version that satisfies the constraint
pub fn find_best_version<'a>(
    versions: &'a [P2Version],
    constraint: &VersionReq,
) -> Result<&'a P2Version> {
    let mut candidates = Vec::new();

    for version in versions {
        // Try to parse the version string
        let version_string = if !version.version_normalized.is_empty() {
            &version.version_normalized
        } else {
            &version.version
        };

        // Handle development versions more broadly
        if version_string.contains("dev") || version_string.starts_with("dev-") 
           || version_string.ends_with("-dev") {
            // For dev versions, we'll be more lenient
            if constraint == &VersionReq::STAR {
                candidates.push((version, Version::parse("999.0.0-dev").unwrap()));
                continue;
            }
            // Try to match dev versions with appropriate constraints
            if format!("{constraint}").contains("dev") {
                candidates.push((version, Version::parse("999.0.0-dev").unwrap()));
                continue;
            }
        }

        // Try to normalize and parse the version
        let normalized_version = match normalize_version_string(version_string) {
            Ok(v) => v,
            Err(_) => {
                // Try some alternative normalization strategies
                if let Ok(alt_version) = try_alternative_normalization(version_string) {
                    alt_version
                } else {
                    continue; // Skip unparseable versions
                }
            }
        };

        if let Ok(semver_version) = Version::parse(&normalized_version) {
            if constraint.matches(&semver_version) {
                candidates.push((version, semver_version));
            }
        }
    }

    if candidates.is_empty() {
        return Err(anyhow!(
            "No version satisfies constraint. Constraint: {}, Available versions: [{}]",
            constraint,
            versions.iter()
                .take(10)
                .map(|v| v.version.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // Sort by version (highest first) and return the best one
    candidates.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(candidates[0].0)
}

/// Try alternative normalization strategies for version strings
fn try_alternative_normalization(version: &str) -> Result<String> {
    let version = version.trim();
    
    // Handle versions with only major.minor (add .0)
    if version.matches('.').count() == 1 && !version.contains('-') {
        return Ok(format!("{version}.0"));
    }
    
    // Handle versions with only major (add .0.0)
    if !version.contains('.') && !version.contains('-') && version.chars().all(|c| c.is_ascii_digit()) {
        return Ok(format!("{version}.0.0"));
    }
    
    // Handle versions like "1.0.x" or "1.x"
    if version.contains('x') {
        let normalized = version.replace('x', "0");
        if let Ok(parts) = normalize_basic_version(&normalized) {
            return Ok(parts);
        }
    }
    
    Err(anyhow!("Could not normalize version: {}", version))
}

/// Normalize a version string for semver parsing
fn normalize_version_string(version: &str) -> Result<String> {
    let version = version.trim();

    // Remove common prefixes
    let version = version.strip_prefix('v').unwrap_or(version);

    // Handle development versions
    if version.starts_with("dev-") || version == "dev-master" || version == "dev-main" {
        return Ok("999.0.0-dev".to_string());
    }

    // Handle version with stability suffix
    if let Some(pos) = version.find('-') {
        let (version_part, suffix) = version.split_at(pos);
        if let Ok(normalized) = normalize_basic_version(version_part) {
            return Ok(format!("{normalized}{suffix}"));
        }
    }

    // Handle basic version
    normalize_basic_version(version)
}

/// Normalize a basic version string (major.minor.patch)
fn normalize_basic_version(version: &str) -> Result<String> {
    let parts: Vec<&str> = version.split('.').collect();

    if parts.is_empty() {
        return Err(anyhow!("Empty version string"));
    }

    let major = parts.first().unwrap_or(&"0");
    let minor = parts.get(1).unwrap_or(&"0");
    let patch = parts.get(2).unwrap_or(&"0");

    // Clean each part
    let clean_part = |part: &str| -> Result<String> {
        if part.chars().all(|c| c.is_ascii_digit()) {
            Ok(part.to_string())
        } else {
            Err(anyhow!("Non-numeric version part: {}", part))
        }
    };

    Ok(format!(
        "{}.{}.{}",
        clean_part(major)?,
        clean_part(minor)?,
        clean_part(patch)?
    ))
}

/// Read package information from a local path
fn read_package_from_path(path: &Path) -> Result<Option<(String, Option<String>)>> {
    let composer_json_path = path.join("composer.json");

    if !composer_json_path.exists() {
        return Ok(None);
    }

    let composer_content =
        std::fs::read_to_string(&composer_json_path).context("Failed to read composer.json")?;

    let composer: ComposerJson =
        serde_json::from_str(&composer_content).context("Failed to parse composer.json")?;

    let name = composer
        .name
        .ok_or_else(|| anyhow!("Package name not found in composer.json"))?;
    let version = None; // composer.version

    Ok(Some((name, version)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_version_string() {
        assert_eq!(normalize_version_string("1.2.3").unwrap(), "1.2.3");
        assert_eq!(normalize_version_string("v1.2.3").unwrap(), "1.2.3");
        assert_eq!(
            normalize_version_string("dev-master").unwrap(),
            "999.0.0-dev"
        );
        assert_eq!(
            normalize_version_string("1.2.3-alpha").unwrap(),
            "1.2.3-alpha"
        );
    }

    #[test]
    fn test_normalize_basic_version() {
        assert_eq!(normalize_basic_version("1.2.3").unwrap(), "1.2.3");
        assert_eq!(normalize_basic_version("1.2").unwrap(), "1.2.0");
        assert_eq!(normalize_basic_version("1").unwrap(), "1.0.0");
    }
}
