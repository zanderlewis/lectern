use lectern::model::{ComposerJson, DistInfo, Lock, LockedPackage, SourceInfo};
use std::collections::BTreeMap;

#[test]
fn test_composer_json_serialization() {
    let mut require = BTreeMap::new();
    require.insert("guzzlehttp/guzzle".to_string(), "^7.0".to_string());

    let composer = ComposerJson {
        name: Some("test/package".to_string()),
        description: Some("A test package".to_string()),
        version: Some("1.0.0".to_string()),
        package_type: Some("library".to_string()),
        keywords: Some(vec!["test".to_string(), "package".to_string()]),
        homepage: Some("https://example.com".to_string()),
        readme: None,
        time: None,
        license: Some(vec!["MIT".to_string()]),
        authors: None,
        support: None,
        require,
        require_dev: BTreeMap::new(),
        conflict: None,
        replace: None,
        provide: None,
        suggest: None,
        autoload: None,
        autoload_dev: None,
        include_path: None,
        target_dir: None,
        repositories: None,
        config: None,
        scripts: None,
        extra: None,
        minimum_stability: Some("stable".to_string()),
        prefer_stable: Some(true),
        bin: None,
    };

    let json = serde_json::to_string(&composer).unwrap();
    assert!(json.contains("test/package"));
    assert!(json.contains("guzzlehttp/guzzle"));
    assert!(json.contains("^7.0"));

    let deserialized: ComposerJson = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, Some("test/package".to_string()));
    assert_eq!(
        deserialized.require.get("guzzlehttp/guzzle"),
        Some(&"^7.0".to_string())
    );
}

#[test]
fn test_composer_json_with_dev_dependencies() {
    let mut require_dev = BTreeMap::new();
    require_dev.insert("phpunit/phpunit".to_string(), "^10.0".to_string());

    let composer = ComposerJson {
        name: Some("test/package".to_string()),
        description: None,
        version: None,
        package_type: None,
        keywords: None,
        homepage: None,
        readme: None,
        time: None,
        license: None,
        authors: None,
        support: None,
        require: BTreeMap::new(),
        require_dev,
        conflict: None,
        replace: None,
        provide: None,
        suggest: None,
        autoload: None,
        autoload_dev: None,
        include_path: None,
        target_dir: None,
        repositories: None,
        config: None,
        scripts: None,
        extra: None,
        minimum_stability: None,
        prefer_stable: None,
        bin: None,
    };

    let json = serde_json::to_string(&composer).unwrap();
    // The require-dev field should be serialized with hyphen due to serde rename
    assert!(json.contains("require-dev"));
    assert!(json.contains("phpunit/phpunit"));
}

#[test]
fn test_locked_package_creation() {
    let mut require = BTreeMap::new();
    require.insert("php".to_string(), ">=8.0".to_string());

    let source_info = SourceInfo {
        source_type: "git".to_string(),
        url: "https://github.com/example/package.git".to_string(),
        reference: "abc123".to_string(),
    };

    let dist_info = DistInfo {
        dist_type: "zip".to_string(),
        url: "https://api.github.com/repos/example/package/zipball/abc123".to_string(),
        reference: "abc123".to_string(),
        shasum: "".to_string(),
    };

    let locked_package = LockedPackage {
        name: "example/package".to_string(),
        version: "1.2.3".to_string(),
        source: Some(source_info),
        dist: Some(dist_info),
        require: Some(require),
        require_dev: None,
        conflict: None,
        replace: None,
        provide: None,
        suggest: None,
        package_type: Some("library".to_string()),
        extra: None,
        autoload: None,
        autoload_dev: None,
        notification_url: Some("https://packagist.org/downloads/".to_string()),
        license: Some(vec!["MIT".to_string()]),
        authors: None,
        description: Some("Example package".to_string()),
        homepage: None,
        keywords: None,
        support: None,
        funding: None,
        time: None,
        bin: None,
        include_path: None,
    };

    assert_eq!(locked_package.name, "example/package");
    assert_eq!(locked_package.version, "1.2.3");
    assert!(locked_package.source.is_some());
    assert!(locked_package.dist.is_some());
    assert_eq!(locked_package.package_type, Some("library".to_string()));
}

#[test]
fn test_lock_file_structure() {
    let packages = vec![LockedPackage {
        name: "test/package".to_string(),
        version: "1.0.0".to_string(),
        source: None,
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
    }];

    let lock = Lock {
        _readme: vec![
            "This file locks the dependencies of your project to a known state".to_string(),
            "Read more about it at https://getcomposer.org/doc/01-basic-usage.md#installing-dependencies".to_string(),
            "This file is @generated automatically".to_string(),
        ],
        content_hash: "test-hash".to_string(),
        packages,
        packages_dev: vec![],
        aliases: vec![],
        minimum_stability: "stable".to_string(),
        stability_flags: BTreeMap::new(),
        prefer_stable: true,
        prefer_lowest: false,
        platform: BTreeMap::new(),
        platform_dev: BTreeMap::new(),
        plugin_api_version: Some("2.3.0".to_string()),
    };

    let json = serde_json::to_string_pretty(&lock).unwrap();
    assert!(json.contains("_readme"));
    assert!(json.contains("content-hash"));
    assert!(json.contains("test/package"));
    assert!(json.contains("This file locks the dependencies"));
}

#[test]
fn test_source_and_dist_info() {
    let source = SourceInfo {
        source_type: "git".to_string(),
        url: "https://github.com/test/repo.git".to_string(),
        reference: "main".to_string(),
    };

    let dist = DistInfo {
        dist_type: "zip".to_string(),
        url: "https://github.com/test/repo/archive/main.zip".to_string(),
        reference: "abc123def456".to_string(),
        shasum: "sha256:abcdef123456".to_string(),
    };

    assert_eq!(source.source_type, "git");
    assert_eq!(source.url, "https://github.com/test/repo.git");
    assert_eq!(source.reference, "main");

    assert_eq!(dist.dist_type, "zip");
    assert_eq!(dist.url, "https://github.com/test/repo/archive/main.zip");
    assert_eq!(dist.reference, "abc123def456");
    assert_eq!(dist.shasum, "sha256:abcdef123456");
}

#[test]
fn test_composer_json_deserialization_minimal() {
    let json = r#"{"name": "test/package", "require": {}}"#;
    let composer: ComposerJson = serde_json::from_str(json).unwrap();

    assert_eq!(composer.name, Some("test/package".to_string()));
    assert!(composer.require.is_empty());
    assert_eq!(composer.description, None);
    assert_eq!(composer.minimum_stability, None);
}

#[test]
fn test_composer_json_with_repositories() {
    let json = r#"{
        "name": "test/package",
        "require": {},
        "repositories": [
            {
                "type": "vcs", 
                "url": "https://github.com/test/repo.git"
            }
        ]
    }"#;

    let composer: ComposerJson = serde_json::from_str(json).unwrap();
    assert!(composer.repositories.is_some());
    assert_eq!(composer.repositories.unwrap().len(), 1);
}
