use crate::models::model::{ComposerJson, DistInfo, LockedPackage, SourceInfo};
use crate::resolver::dependency_utils as utils_dep;
use crate::resolver::dependency_utils::read_package_from_path;
pub use crate::resolver::dependency_utils::{find_best_version, generate_content_hash};
use crate::resolver::packagist::{
    fetch_packagist_versions_bulk, fetch_packagist_versions_cached, is_platform_dependency,
};
use crate::resolver::version::parse_constraint;
use crate::utils::{print_error, print_info, print_step, print_success, print_warning};
use anyhow::{Result, anyhow};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

/// Main dependency resolution function with batch processing optimization
pub async fn solve(composer: &ComposerJson) -> Result<crate::models::model::Lock> {
    print_step("🔍 Resolving dependencies...");

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
        print_info(&format!(
            "📥 Pre-fetching {} dependencies in batch...",
            all_deps.len()
        ));
        let _bulk_versions = fetch_packagist_versions_bulk(&all_deps)
            .await
            .unwrap_or_default();
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
        let versions = match fetch_packagist_versions_cached(&pkg_name).await {
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
                print_info(&format!(
                    "Available versions for {pkg_name}: {}",
                    versions
                        .iter()
                        .take(5)
                        .map(|v| v.version.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                return Err(anyhow!(
                    "No version satisfies constraint '{constraint_str}' for package {pkg_name}"
                ));
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
            require_dev: best_version
                .other
                .get("require-dev")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            conflict: best_version
                .other
                .get("conflict")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            replace: best_version
                .other
                .get("replace")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            provide: best_version
                .other
                .get("provide")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            suggest: best_version
                .other
                .get("suggest")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            package_type: best_version
                .other
                .get("type")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .or_else(|| Some("library".to_string())),
            extra: best_version.extra.clone(),
            autoload: best_version
                .other
                .get("autoload")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            autoload_dev: best_version
                .other
                .get("autoload-dev")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            notification_url: Some("https://packagist.org/downloads/".to_string()),
            license: best_version
                .other
                .get("license")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            authors: best_version
                .other
                .get("authors")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            description: best_version
                .other
                .get("description")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            homepage: best_version
                .other
                .get("homepage")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            keywords: best_version
                .other
                .get("keywords")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            support: best_version
                .other
                .get("support")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            funding: best_version
                .other
                .get("funding")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            time: best_version
                .other
                .get("time")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            bin: best_version
                .other
                .get("bin")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            include_path: best_version
                .other
                .get("include-path")
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

    print_success(&format!(
        "✅ Resolved {} packages",
        regular_packages.len() + dev_packages.len()
    ));

    // Generate content hash for the lock file
    let content_hash = utils_dep::generate_content_hash_from_composer(composer);

    Ok(crate::models::model::Lock {
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

// Helper functions are in `dependency_utils.rs` and imported above

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_version_string() {
        assert_eq!(
            utils_dep::normalize_version_string("1.2.3").unwrap(),
            "1.2.3"
        );
        assert_eq!(
            utils_dep::normalize_version_string("v1.2.3").unwrap(),
            "1.2.3"
        );
        assert_eq!(
            utils_dep::normalize_version_string("dev-master").unwrap(),
            "999.0.0-dev"
        );
        assert_eq!(
            utils_dep::normalize_version_string("1.2.3-alpha").unwrap(),
            "1.2.3-alpha"
        );
    }

    #[test]
    fn test_normalize_basic_version() {
        assert_eq!(
            utils_dep::normalize_basic_version("1.2.3").unwrap(),
            "1.2.3"
        );
        assert_eq!(utils_dep::normalize_basic_version("1.2").unwrap(), "1.2.0");
        assert_eq!(utils_dep::normalize_basic_version("1").unwrap(), "1.0.0");
    }
}
