pub mod dependency;
pub mod dependency_utils;
pub mod http_client;
pub mod packagist;
pub mod version;

// Re-export commonly used items
pub use dependency::solve;
pub use http_client::get_client;
pub use packagist::{
    PackageInfo, SearchResult, fetch_multiple_package_info, fetch_package_info,
    fetch_packagist_versions_bulk, search_packagist,
};
pub use version::parse_constraint;
