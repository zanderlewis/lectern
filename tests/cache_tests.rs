use lectern::core::cache;
use lectern::core::cache_utils::{get_cache_dir, get_cache_file_path};

#[tokio::test]
async fn test_cache_set_and_get_meta() {
    let key = "test:meta:key";
    let value = serde_json::json!({"test": "data"});

    cache::cache_set_meta(key, value.clone()).await;

    let retrieved = cache::cache_get_meta(key).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), value);
}

#[tokio::test]
async fn test_cache_get_nonexistent() {
    let key = "test:nonexistent:key:12345";
    let retrieved = cache::cache_get_meta(key).await;
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_cache_set_and_get_package_info() {
    let key = "test:package:info";
    let value = serde_json::json!({
        "name": "test/package",
        "version": "1.0.0"
    });

    cache::cache_set_package_info(key, value.clone()).await;

    let retrieved = cache::cache_get_package_info(key).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), value);
}

#[tokio::test]
async fn test_cache_set_and_get_search() {
    let key = "test:search:query";
    let value = serde_json::json!([
        {"name": "package1"},
        {"name": "package2"}
    ]);

    cache::cache_set_search(key, value.clone()).await;

    let retrieved = cache::cache_get_search(key).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), value);
}

#[tokio::test]
async fn test_cache_get_multiple_package_info() {
    let key1 = "test:pkg1";
    let key2 = "test:pkg2";
    let value1 = serde_json::json!({"name": "pkg1"});
    let value2 = serde_json::json!({"name": "pkg2"});

    cache::cache_set_package_info(key1, value1.clone()).await;
    cache::cache_set_package_info(key2, value2.clone()).await;

    let keys = vec![key1.to_string(), key2.to_string()];
    let retrieved = cache::cache_get_multiple_package_info(&keys).await;

    assert_eq!(retrieved.len(), 2);
    assert_eq!(retrieved.get(key1).unwrap(), &value1);
    assert_eq!(retrieved.get(key2).unwrap(), &value2);
}

#[tokio::test]
async fn test_cache_set_multiple_package_info() {
    let mut data = std::collections::HashMap::new();
    data.insert("test:multi:pkg1".to_string(), serde_json::json!({"name": "pkg1"}));
    data.insert("test:multi:pkg2".to_string(), serde_json::json!({"name": "pkg2"}));

    cache::cache_set_multiple_package_info(data.clone()).await;

    for (key, expected_value) in data.iter() {
        let retrieved = cache::cache_get_package_info(key).await;
        assert!(retrieved.is_some());
        assert_eq!(&retrieved.unwrap(), expected_value);
    }
}

#[test]
fn test_cache_dir_path() {
    let cache_dir = get_cache_dir();
    assert!(cache_dir.to_string_lossy().contains("lectern"));
}

#[test]
fn test_cache_file_path_format() {
    let path = get_cache_file_path("test_type", "test_key");
    
    // Should contain the cache type
    assert!(path.to_string_lossy().contains("test_type"));
    
    // Should end with .json
    assert!(path.to_string_lossy().ends_with(".json"));
}

#[test]
fn test_cache_file_path_different_keys() {
    let path1 = get_cache_file_path("type", "key1");
    let path2 = get_cache_file_path("type", "key2");
    
    // Different keys should produce different paths
    assert_ne!(path1, path2);
}

#[test]
fn test_cache_file_path_same_key() {
    let path1 = get_cache_file_path("type", "same_key");
    let path2 = get_cache_file_path("type", "same_key");
    
    // Same keys should produce same paths
    assert_eq!(path1, path2);
}
