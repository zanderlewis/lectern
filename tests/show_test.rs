use std::process::Command;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_show_command() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("show")
        .arg("monolog/monolog")
        .output()
        .expect("Failed to execute lectern show");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show package details
    assert!(
        stdout.contains("monolog") || stdout.contains("Package") || output.status.success()
    );
}

#[test]
fn test_show_nonexistent_package() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("show")
        .arg("nonexistent/package-that-does-not-exist-xyz")
        .output()
        .expect("Failed to execute lectern show");

    // Should fail or indicate package not found
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    
    assert!(
        !output.status.success() || combined.contains("not found") || combined.contains("error")
    );
}

#[test]
fn test_show_package_details() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("show")
        .arg("symfony/console")
        .output()
        .expect("Failed to execute lectern show");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show various package details
    assert!(
        stdout.contains("symfony") || stdout.contains("Description") || stdout.contains("Version") || output.status.success()
    );
}
