use lectern::core::installer::installer_utils::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_get_package_cache_dir() {
    let cache_dir = get_package_cache_dir();
    assert!(cache_dir.to_string_lossy().contains("lectern"));
    assert!(cache_dir.to_string_lossy().contains("packages"));
}

#[test]
fn test_get_cached_package_path() {
    let name = "vendor/package";
    let version = "1.0.0";
    let url = "https://example.com/package.zip";
    
    let path = get_cached_package_path(name, version, url);
    
    // Should be a valid path
    assert!(path.to_string_lossy().ends_with(".zip"));
    assert!(path.to_string_lossy().contains("packages"));
}

#[test]
fn test_get_cached_package_path_consistency() {
    let name = "vendor/package";
    let version = "1.0.0";
    let url = "https://example.com/package.zip";
    
    let path1 = get_cached_package_path(name, version, url);
    let path2 = get_cached_package_path(name, version, url);
    
    // Same inputs should produce same path
    assert_eq!(path1, path2);
}

#[test]
fn test_get_cached_package_path_different_inputs() {
    let path1 = get_cached_package_path("vendor/package1", "1.0.0", "https://example.com/1.zip");
    let path2 = get_cached_package_path("vendor/package2", "1.0.0", "https://example.com/2.zip");
    
    // Different inputs should produce different paths
    assert_ne!(path1, path2);
}

#[test]
fn test_extract_archive_with_invalid_path() {
    let temp_dir = TempDir::new().unwrap();
    let dest = temp_dir.path();
    let nonexistent = Path::new("/nonexistent/file.zip");
    
    let result = extract_archive_ultra_fast(nonexistent, dest);
    assert!(result.is_err(), "Should fail with nonexistent archive");
}

#[test]
fn test_extract_zip_with_invalid_archive() {
    let temp_dir = TempDir::new().unwrap();
    let dest = temp_dir.path();
    
    // Create an invalid zip file
    let invalid_zip = temp_dir.path().join("invalid.zip");
    fs::write(&invalid_zip, b"not a zip file").unwrap();
    
    let result = extract_zip_ultra_fast(&invalid_zip, dest);
    assert!(result.is_err(), "Should fail with invalid zip");
}

#[test]
fn test_extract_tar_gz_with_invalid_archive() {
    let temp_dir = TempDir::new().unwrap();
    let dest = temp_dir.path();
    
    // Create an invalid tar.gz file
    let invalid_tar = temp_dir.path().join("invalid.tar.gz");
    fs::write(&invalid_tar, b"not a tar.gz file").unwrap();
    
    let result = extract_tar_gz_ultra_fast(&invalid_tar, dest);
    assert!(result.is_err(), "Should fail with invalid tar.gz");
}

#[tokio::test]
async fn test_copy_local_path_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let dest = temp_dir.path().join("dest");
    
    let result = copy_local_path_optimized("/nonexistent/path", &dest).await;
    assert!(result.is_err(), "Should fail with nonexistent source");
}

#[tokio::test]
async fn test_copy_local_path_file_not_dir() {
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("file.txt");
    fs::write(&temp_file, "test").unwrap();
    
    let dest = temp_dir.path().join("dest");
    
    let result = copy_local_path_optimized(temp_file.to_str().unwrap(), &dest).await;
    assert!(result.is_err(), "Should fail when source is not a directory");
}
