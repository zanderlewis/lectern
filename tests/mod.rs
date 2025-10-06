// Core test modules
mod dependency_tests;
mod integration_tests;
mod model_tests;
mod packagist_tests;
mod strict_test_utils;
mod version_tests;

// Component test modules
mod cache_tests;
mod http_client_tests;
mod utils_tests;
mod installer_tests;
mod autoload_tests;

// Shared test utilities (available to all test files)
pub mod common;

// Individual command test modules (one per command)
mod browse_test;
mod clear_cache_test;
mod depends_test;
mod diagnose_test;
mod fund_test;
mod init_test;
mod licenses_test;
mod outdated_test;
mod prohibits_test;
mod project_test;
mod script_test;
mod search_test;
mod show_test;
mod status_test;
mod suggests_test;
mod validate_test;

// Re-export strict testing utilities for use in other test modules
pub use strict_test_utils::*;
