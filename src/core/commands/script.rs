use crate::cli::RunScriptArgs;
use crate::io::read_composer_json;
use crate::utils::{print_info, print_step, print_success};
use anyhow::{Result, anyhow};
use std::path::Path;

/// Run a script defined in composer.json
pub async fn run_script(args: &RunScriptArgs, working_dir: &Path) -> Result<()> {
    let composer_path = working_dir.join("composer.json");
    let composer = read_composer_json(&composer_path)?;

    if args.list {
        if let Some(scripts) = &composer.scripts {
            println!("\n📜 Available scripts:");
            for (name, _) in scripts.iter() {
                println!("  • {name}");
            }
        } else {
            print_info("No scripts defined in composer.json");
        }
        return Ok(());
    }

    if let Some(scripts) = &composer.scripts {
        if let Some(script_value) = scripts.get(&args.script) {
            print_step(&format!("🚀 Running script: {}", args.script));

            // Scripts can be either a string or array of strings
            let commands: Vec<String> = match script_value {
                crate::models::model::ScriptDefinition::String(s) => vec![s.clone()],
                crate::models::model::ScriptDefinition::Array(arr) => arr.clone(),
            };

            for cmd in commands {
                print_info(&format!("  > {cmd}"));
                let status = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .current_dir(working_dir)
                    .status()?;

                if !status.success() {
                    return Err(anyhow!(
                        "Script '{}' failed with exit code: {:?}",
                        args.script,
                        status.code()
                    ));
                }
            }

            print_success("✅ Script completed successfully");
        } else {
            return Err(anyhow!(
                "Script '{}' not found in composer.json",
                args.script
            ));
        }
    } else {
        return Err(anyhow!("No scripts defined in composer.json"));
    }

    Ok(())
}
