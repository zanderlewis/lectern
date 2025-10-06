// Common test utilities shared across all command tests
use std::process::Command;
use std::sync::Once;

static INIT: Once = Once::new();

/// Ensure the lectern binary is built before running tests
pub fn ensure_lectern_binary() {
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

/// Get the project root directory
pub fn get_project_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Get the path to the lectern binary
pub fn get_lectern_binary_path() -> std::path::PathBuf {
    get_project_root()
        .join("target")
        .join("release")
        .join("lectern")
}
