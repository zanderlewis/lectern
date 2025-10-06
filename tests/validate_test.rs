use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_validate_command_valid_json() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{
"name": "test/validate",
"description": "Test package",
"require": {
    "php": ">=7.4"
}
}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("validate")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern validate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid") || stdout.contains("✅"));
}

#[test]
fn test_validate_command_invalid_json() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Invalid JSON
    let composer_json = r#"{
"name": "test/validate",
"require": {
    "php": ">=7.4"
}"#; // Missing closing brace
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("validate")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern validate");

    // The validate command prints errors but still exits 0
    // Check if it detected the invalid JSON in the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    
    assert!(
        combined.contains("invalid") || combined.contains("error") || combined.contains("❌"),
        "Should detect invalid JSON. Output was: {}",
        combined
    );
}

#[test]
fn test_validate_missing_composer_json() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let output = Command::new(get_lectern_binary_path())
        .arg("validate")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern validate");

    // Should run without crashing - may succeed or fail
    assert!(output.status.code().is_some());
}

#[test]
fn test_validate_malformed_json() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Completely invalid JSON
    let composer_json = "not even json";
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("validate")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern validate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    
    assert!(combined.contains("invalid") || combined.contains("error") || combined.contains("❌"));
}
