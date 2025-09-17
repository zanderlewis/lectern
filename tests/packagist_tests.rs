use lectern::resolver::packagist::{
    P2Dist, P2Source, P2Version, fetch_packagist_versions_cached, search_packagist,
};
use std::collections::BTreeMap;

#[test]
fn test_p2_version_structure() {
    let mut other = serde_json::Map::new();
    other.insert(
        "description".to_string(),
        serde_json::Value::String("Test package".to_string()),
    );
    other.insert(
        "license".to_string(),
        serde_json::Value::Array(vec![serde_json::Value::String("MIT".to_string())]),
    );

    let p2_version = P2Version {
        version: "1.2.3".to_string(),
        version_normalized: "1.2.3.0".to_string(),
        dist: Some(P2Dist {
            dtype: Some("zip".to_string()),
            url: Some("https://example.com/package.zip".to_string()),
            reference: Some("abc123".to_string()),
            shasum: Some("sha256:def456".to_string()),
        }),
        source: Some(P2Source {
            stype: Some("git".to_string()),
            url: Some("https://github.com/test/repo.git".to_string()),
            reference: Some("abc123".to_string()),
        }),
        require: Some({
            let mut req = BTreeMap::new();
            req.insert("php".to_string(), ">=8.0".to_string());
            req
        }),
        extra: None,
        other,
    };

    assert_eq!(p2_version.version, "1.2.3");
    assert_eq!(p2_version.version_normalized, "1.2.3.0");

    let dist = p2_version.dist.as_ref().unwrap();
    assert_eq!(dist.dtype, Some("zip".to_string()));
    assert_eq!(
        dist.url,
        Some("https://example.com/package.zip".to_string())
    );

    let source = p2_version.source.as_ref().unwrap();
    assert_eq!(source.stype, Some("git".to_string()));
    assert_eq!(
        source.url,
        Some("https://github.com/test/repo.git".to_string())
    );

    assert_eq!(
        p2_version
            .other
            .get("description")
            .unwrap()
            .as_str()
            .unwrap(),
        "Test package"
    );
    assert!(p2_version.other.get("license").unwrap().is_array());
}

#[test]
fn test_p2_version_serialization() {
    let p2_version = P2Version {
        version: "2.0.0".to_string(),
        version_normalized: "2.0.0.0".to_string(),
        dist: None,
        source: None,
        require: None,
        extra: Some(serde_json::json!({"branch-alias": {"dev-main": "2.x-dev"}})),
        other: serde_json::Map::new(),
    };

    let json = serde_json::to_string(&p2_version).unwrap();
    assert!(json.contains("2.0.0"));

    let deserialized: P2Version = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.version, "2.0.0");
    assert!(deserialized.extra.is_some());
}

#[tokio::test]
async fn test_search_packages_integration() {
    // This is an integration test that requires network access
    // Skip if network is not available
    let result = search_packagist(&["guzzle".to_string()]).await;

    match result {
        Ok(packages) => {
            assert!(!packages.is_empty());
            // Guzzle should be in the results
            let has_guzzle = packages.iter().any(|p| p.name.contains("guzzle"));
            assert!(has_guzzle, "Expected to find guzzle in search results");
        }
        Err(_) => {
            // Network might not be available, skip this test
            println!("Skipping network-dependent test");
        }
    }
}

#[tokio::test]
async fn test_get_package_versions_integration() {
    // This is an integration test that requires network access
    let client = reqwest::Client::new();
    let result = fetch_packagist_versions_cached(&client, "monolog/monolog").await;

    match result {
        Ok(versions) => {
            assert!(!versions.is_empty());
            // Check that versions are properly formatted
            let has_valid_version = versions
                .iter()
                .any(|v| v.version.chars().next().unwrap_or('x').is_ascii_digit());
            assert!(
                has_valid_version,
                "Expected to find versions starting with digits"
            );

            // Check that some version has required metadata
            let has_metadata = versions
                .iter()
                .any(|v| v.dist.is_some() || v.source.is_some());
            assert!(
                has_metadata,
                "Expected versions to have dist or source metadata"
            );
        }
        Err(_) => {
            // Network might not be available, skip this test
            println!("Skipping network-dependent test");
        }
    }
}

#[test]
fn test_p2_dist_structure() {
    let dist = P2Dist {
        dtype: Some("zip".to_string()),
        url: Some("https://api.github.com/repos/test/package/zipball/v1.0.0".to_string()),
        reference: Some("v1.0.0".to_string()),
        shasum: Some("abc123def456".to_string()),
    };

    assert_eq!(dist.dtype, Some("zip".to_string()));
    assert!(dist.url.as_ref().unwrap().contains("api.github.com"));
    assert_eq!(dist.reference, Some("v1.0.0".to_string()));
    assert_eq!(dist.shasum, Some("abc123def456".to_string()));
}

#[test]
fn test_p2_source_structure() {
    let source = P2Source {
        stype: Some("git".to_string()),
        url: Some("https://github.com/test/package.git".to_string()),
        reference: Some("main".to_string()),
    };

    assert_eq!(source.stype, Some("git".to_string()));
    assert!(source.url.as_ref().unwrap().ends_with(".git"));
    assert_eq!(source.reference, Some("main".to_string()));
}

#[test]
fn test_p2_version_with_empty_fields() {
    let p2_version = P2Version {
        version: "1.0.0".to_string(),
        version_normalized: "".to_string(),
        dist: None,
        source: None,
        require: None,
        extra: None,
        other: serde_json::Map::new(),
    };

    assert_eq!(p2_version.version, "1.0.0");
    assert_eq!(p2_version.version_normalized, "");
    assert!(p2_version.dist.is_none());
    assert!(p2_version.source.is_none());
    assert!(p2_version.require.is_none());
    assert!(p2_version.extra.is_none());
    assert!(p2_version.other.is_empty());
}

#[test]
fn test_p2_version_metadata_extraction() {
    let mut other = serde_json::Map::new();
    other.insert(
        "description".to_string(),
        serde_json::Value::String("A test description".to_string()),
    );
    other.insert(
        "keywords".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("test".to_string()),
            serde_json::Value::String("package".to_string()),
        ]),
    );
    other.insert(
        "homepage".to_string(),
        serde_json::Value::String("https://example.com".to_string()),
    );
    other.insert(
        "license".to_string(),
        serde_json::Value::Array(vec![serde_json::Value::String("MIT".to_string())]),
    );

    let p2_version = P2Version {
        version: "1.0.0".to_string(),
        version_normalized: "1.0.0.0".to_string(),
        dist: None,
        source: None,
        require: None,
        extra: None,
        other,
    };

    // Test that we can extract metadata from the other field
    assert_eq!(
        p2_version.other.get("description").and_then(|v| v.as_str()),
        Some("A test description")
    );

    let keywords = p2_version.other.get("keywords").and_then(|v| v.as_array());
    assert!(keywords.is_some());
    assert_eq!(keywords.unwrap().len(), 2);

    assert_eq!(
        p2_version.other.get("homepage").and_then(|v| v.as_str()),
        Some("https://example.com")
    );

    let license = p2_version.other.get("license").and_then(|v| v.as_array());
    assert!(license.is_some());
    assert_eq!(license.unwrap()[0].as_str(), Some("MIT"));
}
