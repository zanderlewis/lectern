use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Once;
use tempfile::TempDir;

static INIT: Once = Once::new();

/// Ensure the lectern binary is built before running tests
fn ensure_lectern_binary() {
    INIT.call_once(|| {
        let build_output = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(get_project_root())
            .output()
            .expect("Failed to build lectern");

        if !build_output.status.success() {
            panic!(
                "Failed to build lectern: {}",
                String::from_utf8_lossy(&build_output.stderr)
            );
        }
    });
}

/// Test that Lectern can install packages in a temporary directory
#[test]
fn test_lectern_install_command() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a simple composer.json
    let composer_json = r#"{
"name": "test/integration",
"require": {
    "monolog/monolog": "^3.0"
}
}"#;

    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    // Run lectern install
    let output = Command::new(get_lectern_binary_path())
        .arg("install")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern install");

    // Check that it succeeded
    if !output.status.success() {
        eprintln!("Lectern install failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    // Check that composer.lock was created
    assert!(temp_path.join("composer.lock").exists());

    // Check that vendor directory was created
    assert!(temp_path.join("vendor").exists());
    assert!(temp_path.join("vendor").is_dir());
}

/// Test that Lectern can show package information
#[test]
fn test_lectern_search_command() {
    ensure_lectern_binary();

    // Run lectern search
    let output = Command::new(get_lectern_binary_path())
        .args(["search", "monolog"])
        .output()
        .expect("Failed to execute lectern search");

    // Check that it succeeded
    if !output.status.success() {
        eprintln!("Lectern search failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    // Check that output contains monolog
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("monolog"),
        "Expected search results to contain 'monolog'"
    );
}

/// Test that Lectern can show help
#[test]
fn test_lectern_help_command() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute lectern --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Lectern"));
    assert!(stdout.contains("install"));
    assert!(stdout.contains("search"));
    assert!(stdout.contains("outdated"));
}

/// Test that Lectern can show version information
#[test]
fn test_lectern_version_command() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute lectern --version");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lectern") || stdout.contains("0.1.0"));
}

/// Test that Lectern can handle invalid commands gracefully
#[test]
fn test_lectern_invalid_command() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("invalid-command-that-does-not-exist")
        .output()
        .expect("Failed to execute lectern with invalid command");

    // Should exit with error
    assert!(!output.status.success());

    // Should show help or error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "Expected error message for invalid command"
    );
}

/// Test that Lectern install fails gracefully without composer.json
#[test]
fn test_lectern_install_without_composer_json() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Don't create composer.json

    let output = Command::new(get_lectern_binary_path())
        .arg("install")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern install");

    // Should fail gracefully
    assert!(!output.status.success());

    // Should give meaningful error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("composer.json") || stderr.contains("not found"));
}

/// Test that Lectern can show status of installed packages
#[test]
fn test_lectern_status_with_packages() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a simple composer.json
    let composer_json = r#"{
"name": "test/status",
"require": {
    "monolog/monolog": "^3.0"
}
}"#;

    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    // Install packages first
    let install_output = Command::new(get_lectern_binary_path())
        .arg("install")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern install");

    if !install_output.status.success() {
        eprintln!(
            "Install failed: {}",
            String::from_utf8_lossy(&install_output.stderr)
        );
    }
    assert!(install_output.status.success());

    // Now test status
    let output = Command::new(get_lectern_binary_path())
        .arg("status")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern status");

    if !output.status.success() {
        eprintln!("Status failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("monolog"),
        "Status should show installed monolog package"
    );
}

fn get_project_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn get_lectern_binary_path() -> String {
    let root = get_project_root();
    root.join("target")
        .join("release")
        .join("lectern")
        .to_string_lossy()
        .into_owned()
}
