pub mod dependency;
pub mod packagist;
pub mod version;

// Re-export commonly used items
pub use dependency::solve;
pub use packagist::{
    PackageInfo, SearchResult, fetch_multiple_package_info, fetch_package_info, search_packagist,
};
pub use version::parse_constraint;
