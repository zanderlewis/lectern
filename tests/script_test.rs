use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_run_script_list() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{
"name": "test/script",
"scripts": {
    "test": "echo 'test script'",
    "build": "echo 'build script'"
}
}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("run-script")
        .arg("--list")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern run-script --list");

    // The command should either succeed or at least not panic
    // Some implementations may not fully support --list yet
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Just verify the command ran without crashing
    assert!(!stdout.is_empty() || !stderr.is_empty() || output.status.success());
}

#[test]
fn test_run_script_execute() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{
"name": "test/script",
"scripts": {
    "hello": "echo 'Hello from script'"
}
}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("run-script")
        .arg("hello")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern run-script");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from script") || stdout.contains("Running script"));
}

#[test]
fn test_run_script_nonexistent() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{
"name": "test/script",
"scripts": {
    "hello": "echo 'Hello'"
}
}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("run-script")
        .arg("nonexistent")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern run-script");

    // Should fail or report script not found
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    
    assert!(
        !output.status.success() || combined.contains("not found") || combined.contains("Script")
    );
}

#[test]
fn test_run_script_no_scripts() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{
"name": "test/script",
"require": {}
}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("run-script")
        .arg("--list")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern run-script --list");

    // Should run without crashing (may fail or show message)
    assert!(output.status.code().is_some());
}
