use crate::cli::ClearCacheArgs;
use crate::utils::{print_info, print_step, print_success};
use anyhow::Result;

/// Clear Lectern caches
pub async fn clear_cache(args: &ClearCacheArgs) -> Result<()> {
    let cache_dir = crate::core::cache_utils::get_cache_dir();

    if !cache_dir.exists() {
        print_info("No cache directory found");
        return Ok(());
    }

    let cache_type = args.cache_type.as_deref().unwrap_or("all");

    match cache_type {
        "all" => {
            print_step("🗑️  Clearing all caches...");
            std::fs::remove_dir_all(&cache_dir)?;
            std::fs::create_dir_all(&cache_dir)?;
            print_success("✅ All caches cleared");
        }
        "repo" => {
            print_step("🗑️  Clearing repository cache...");
            let repo_cache = cache_dir.join("meta");
            if repo_cache.exists() {
                std::fs::remove_dir_all(&repo_cache)?;
            }
            print_success("✅ Repository cache cleared");
        }
        "files" => {
            print_step("🗑️  Clearing package files cache...");
            let files_cache = cache_dir.join("files");
            if files_cache.exists() {
                std::fs::remove_dir_all(&files_cache)?;
            }
            print_success("✅ Files cache cleared");
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown cache type: {cache_type}. Use: all, repo, or files"
            ));
        }
    }

    Ok(())
}
