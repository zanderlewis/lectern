use lectern::core::utils::*;
use std::path::PathBuf;

#[test]
fn test_normalize_path_absolute() {
    let absolute = "/usr/local/bin";
    let result = normalize_path(absolute).unwrap();
    assert_eq!(result, PathBuf::from(absolute));
}

#[test]
fn test_normalize_path_relative() {
    let relative = "src/lib";
    let result = normalize_path(relative).unwrap();
    assert!(result.is_absolute(), "Relative path should be converted to absolute");
}

#[test]
fn test_strip_first_component_single() {
    let path = "component/file.txt";
    let result = strip_first_component(path);
    assert_eq!(result, PathBuf::from("file.txt"));
}

#[test]
fn test_strip_first_component_multiple() {
    let path = "first/second/third/file.txt";
    let result = strip_first_component(path);
    assert_eq!(result, PathBuf::from("second/third/file.txt"));
}

#[test]
fn test_strip_first_component_single_component() {
    let path = "file.txt";
    let result = strip_first_component(path);
    // Should return empty path when only one component
    assert_eq!(result, PathBuf::from(""));
}

#[test]
fn test_strip_first_component_empty() {
    let path = "";
    let result = strip_first_component(path);
    assert_eq!(result, PathBuf::from(""));
}

#[test]
fn test_print_functions_dont_panic() {
    // These should not panic
    print_success("Test success message");
    print_info("Test info message");
    print_error("Test error message");
    print_warning("Test warning message");
    print_step("Test step message");
}

#[test]
fn test_is_prerelease_version() {
    use lectern::utils::is_prerelease_version;
    
    assert!(is_prerelease_version("1.0.0-alpha"));
    assert!(is_prerelease_version("2.0.0-beta.1"));
    assert!(is_prerelease_version("3.0.0-rc"));
    assert!(is_prerelease_version("v1.0.0-dev"));
    
    assert!(!is_prerelease_version("1.0.0"));
    assert!(!is_prerelease_version("2.5.3"));
    assert!(!is_prerelease_version("v3.0.0"));
}

#[test]
fn test_version_comparison() {
    use lectern::utils::is_prerelease_version;
    
    let stable_versions = vec!["1.0.0", "2.0.0", "3.5.7"];
    let prerelease_versions = vec!["1.0.0-alpha", "2.0.0-beta", "3.0.0-rc1"];
    
    for version in stable_versions {
        assert!(!is_prerelease_version(version), "{} should be stable", version);
    }
    
    for version in prerelease_versions {
        assert!(is_prerelease_version(version), "{} should be prerelease", version);
    }
}
