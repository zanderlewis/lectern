use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_diagnose_command_success() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a valid composer.json
    let composer_json = r#"{
"name": "test/diagnose",
"require": {}
}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("diagnose")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern diagnose");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostic") || stdout.contains("composer.json"));
}

#[test]
fn test_diagnose_command_missing_composer_json() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let output = Command::new(get_lectern_binary_path())
        .arg("diagnose")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern diagnose");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("composer.json not found") || stdout.contains("Issues"));
}

#[test]
fn test_diagnose_detects_missing_dependencies() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{
"name": "test/diagnose-deps",
"require": {
    "vendor/nonexistent-package": "^1.0"
}
}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("diagnose")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern diagnose");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should detect missing vendor directory or uninstalled packages
    assert!(
        stdout.contains("vendor") || stdout.contains("install") || stdout.contains("Issue")
    );
}
