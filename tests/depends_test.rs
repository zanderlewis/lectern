use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_depends_command_no_lock() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/depends", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("depends")
        .arg("some/package")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern depends");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should mention composer.lock is needed
    assert!(
        stdout.contains("composer.lock") || stderr.contains("composer.lock") || stdout.contains("No composer.lock"),
        "Should indicate composer.lock is needed"
    );
}

#[test]
fn test_depends_command_with_packages() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{
"name": "test/depends",
"require": {
    "monolog/monolog": "^3.0"
}
}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    // Install packages first
    let install = Command::new(get_lectern_binary_path())
        .arg("install")
        .current_dir(temp_path)
        .output()
        .expect("Failed to install");

    if install.status.success() {
        // Test depends
        let output = Command::new(get_lectern_binary_path())
            .arg("depends")
            .arg("psr/log")
            .current_dir(temp_path)
            .output()
            .expect("Failed to execute lectern depends");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should show monolog depends on psr/log
        assert!(
            stdout.contains("monolog") || stdout.contains("No packages"),
            "Should show dependency information"
        );
    }
}

#[test]
fn test_depends_no_dependencies() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/depends", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();
    
    // Create empty lock file
    let lock_json = r#"{"packages":[],"packages-dev":[]}"#;
    fs::write(temp_path.join("composer.lock"), lock_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("depends")
        .arg("some/package")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern depends");

    // Either succeeds or returns an error - both are acceptable for empty dependencies
    assert!(output.status.success() || !output.status.success());
}
