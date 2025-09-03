use crate::model::{ComposerJson, LockedPackage};
use crate::resolver::packagist::{
    P2Version, fetch_packagist_versions_cached, is_platform_dependency,
};
use crate::resolver::version::parse_constraint;
use crate::utils::{print_error, print_info, print_step, print_success, print_warning};
use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use semver::{Version, VersionReq};
use std::collections::{BTreeSet, VecDeque};
use std::path::Path;

/// Main dependency resolution function
pub async fn solve(composer: &ComposerJson) -> Result<crate::model::Lock> {
    print_step("🔍 Resolving dependencies...");

    let client = Client::new();
    let mut locked_packages = Vec::new();
    let mut processed = BTreeSet::new();
    let mut queue = VecDeque::new();

    // Add all direct dependencies to the queue
    for (name, constraint) in &composer.require {
        // Skip platform dependencies
        if is_platform_dependency(name) {
            print_info(&format!("⏭️  Skipping platform dependency: {name}"));
            continue;
        }
        queue.push_back((name.clone(), constraint.clone(), false));
    }

    for (name, constraint) in &composer.require_dev {
        // Skip platform dependencies
        if is_platform_dependency(name) {
            print_info(&format!("⏭️  Skipping platform dependency: {name}"));
            continue;
        }
        queue.push_back((name.clone(), constraint.clone(), true));
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
                dist_url: None,
                dist_type: None,
                dist_shasum: None,
                source_url: Some(pkg_name.clone()),
                source_reference: None,
                source_path: Some(pkg_name.clone()),
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
            dist_url: best_version.dist.as_ref().and_then(|d| d.url.clone()),
            dist_type: best_version.dist.as_ref().and_then(|d| d.dtype.clone()),
            dist_shasum: best_version.dist.as_ref().and_then(|d| d.shasum.clone()),
            source_url: best_version.source.as_ref().and_then(|s| s.url.clone()),
            source_reference: best_version
                .source
                .as_ref()
                .and_then(|s| s.reference.clone()),
            source_path: None,
        };

        // Add dependencies to the queue
        if let Some(deps) = &best_version.require {
            for (dep_name, dep_constraint) in deps {
                // Skip platform dependencies
                if is_platform_dependency(dep_name) {
                    continue;
                }
                if !processed.contains(dep_name) {
                    queue.push_back((dep_name.clone(), dep_constraint.clone(), is_dev));
                }
            }
        }

        locked_packages.push(locked);
    }

    // Sort packages by name for consistent output
    locked_packages.sort_by(|a, b| a.name.cmp(&b.name));

    print_success(&format!("✅ Resolved {} packages", locked_packages.len()));

    Ok(crate::model::Lock {
        packages: locked_packages,
        packages_dev: Vec::new(),
    })
}

/// Find the best version that satisfies the constraint
fn find_best_version<'a>(
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
