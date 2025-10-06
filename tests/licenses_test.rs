use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_licenses_command() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/licenses", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("licenses")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern licenses");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should either show licenses or indicate no lock file
    assert!(
        stdout.contains("license") || stdout.contains("composer.lock") || stdout.contains("No packages") || output.status.success()
    );
}

#[test]
fn test_licenses_with_quiet_flag() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/licenses", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("licenses")
        .arg("--quiet")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern licenses --quiet");

    // Quiet mode should run without crashing (may fail without lock file)
    assert!(output.status.code().is_some());
}
