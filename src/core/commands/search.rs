use crate::resolver::search_packagist;
use crate::utils::{print_error, print_info};
use anyhow::Result;
use std::path::Path;

/// Search for packages on Packagist
/// # Errors
/// Returns an error if the search request fails
pub async fn search_packages(terms: &[String], _working_dir: &Path) -> Result<()> {
    if terms.is_empty() {
        print_error("❌ Please provide search terms");
        return Ok(());
    }

    print_info(&format!("🔍 Searching for: {}", terms.join(" ")));

    let results = search_packagist(terms).await?;

    if results.is_empty() {
        print_info("📦 No packages found matching your search.");
        return Ok(());
    }

    println!("\n🔍 Search Results ({} found):", results.len());
    println!("{:<30} {:<50} Downloads", "Package", "Description");
    println!("{}", "-".repeat(100));

    for result in results.iter().take(15) {
        let desc = result.description.as_deref().unwrap_or("No description");
        let short_desc = if desc.len() > 47 {
            format!("{}...", &desc[..44])
        } else {
            desc.to_string()
        };

        let downloads = result
            .downloads
            .map_or_else(|| "N/A".to_string(), |d| d.to_string());

        println!("{:<30} {:<50} {}", result.name, short_desc, downloads);
    }

    Ok(())
}
