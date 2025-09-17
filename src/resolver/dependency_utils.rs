use crate::models::model::ComposerJson;
use crate::resolver::packagist::P2Version;
use anyhow::{Context, Result, anyhow};
use semver::Version;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Generate content hash from composer.json content
pub fn generate_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Generate content hash from ComposerJson structure
pub fn generate_content_hash_from_composer(composer: &ComposerJson) -> String {
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
    constraint: &semver::VersionReq,
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
        if version_string.contains("dev")
            || version_string.starts_with("dev-")
            || version_string.ends_with("-dev")
        {
            // For dev versions, we'll be more lenient
            if constraint == &semver::VersionReq::STAR {
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
            versions
                .iter()
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
pub fn try_alternative_normalization(version: &str) -> Result<String> {
    let version = version.trim();

    // Handle versions with only major.minor (add .0)
    if version.matches('.').count() == 1 && !version.contains('-') {
        return Ok(format!("{version}.0"));
    }

    // Handle versions with only major (add .0.0)
    if !version.contains('.')
        && !version.contains('-')
        && version.chars().all(|c| c.is_ascii_digit())
    {
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
pub fn normalize_version_string(version: &str) -> Result<String> {
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
pub fn normalize_basic_version(version: &str) -> Result<String> {
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
pub fn read_package_from_path(path: &Path) -> Result<Option<(String, Option<String>)>> {
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
