use crate::cli::BrowseArgs;
use crate::resolver::fetch_package_info;
use crate::utils::{print_info, print_warning};
use anyhow::Result;

/// Open package repository URL
pub async fn browse_package(args: &BrowseArgs) -> Result<()> {
    print_info(&format!("📦 Fetching information for {}...", args.package));

    let package_info = fetch_package_info(&args.package).await?;

    let url = if args.homepage {
        // Get homepage from package details
        if let Some(versions) = &package_info.package.versions {
            versions.values().next().and_then(|_v| {
                // Try to find homepage in version details
                None::<String> // Placeholder - would need to parse from version details
            })
        } else {
            None
        }
    } else {
        package_info.package.repository.clone()
    };

    if let Some(url) = url {
        if args.show {
            println!("{url}");
        } else {
            print_info(&format!("Opening {}...", url));
            // Try different browsers
            let browsers = ["xdg-open", "open", "start"];
            let mut opened = false;

            for browser in &browsers {
                if let Ok(status) = std::process::Command::new(browser).arg(&url).status() {
                    if status.success() {
                        opened = true;
                        break;
                    }
                }
            }

            if !opened {
                println!("Could not open browser. URL: {url}");
            }
        }
    } else {
        print_warning(&format!(
            "No {} URL found for {}",
            if args.homepage {
                "homepage"
            } else {
                "repository"
            },
            args.package
        ));
    }

    Ok(())
}
