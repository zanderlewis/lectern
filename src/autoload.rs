use crate::installer::InstalledPackage;
use crate::model::ComposerJson;
use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Generate vendor/autoload.php, `autoload_psr4.php`, `autoload_classmap.php`
/// # Errors
/// Returns an error if the autoload files cannot be written
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
pub async fn write_autoload_files(
    project_dir: &Path,
    composer: &ComposerJson,
    installed: &Vec<InstalledPackage>,
) -> Result<()> {
    let vendor = project_dir.join("vendor");
    let composer_dir = vendor.join("composer");
    tokio::fs::create_dir_all(&composer_dir).await?;

    // generate autoload_psr4 from top-level composer.json + vendor packages' composer.json
    let mut psr4_map: Vec<(String, String)> = Vec::new();

    if let Some(a) = &composer.autoload {
        for (k, v) in &a.psr4 {
            psr4_map.push((k.clone(), v.clone()));
        }
    }

    // scan installed packages for autoload psr-4 entries
    for pkg in installed {
        let pkg_path = pkg.path.as_std_path();
        let cj = pkg_path.join("composer.json");
        if cj.exists() {
            if let Ok(s) = fs::read_to_string(&cj) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                    if let Some(a) = v.get("autoload").and_then(|x| x.get("psr-4")) {
                        if let Some(map) = a.as_object() {
                            for (k, val) in map {
                                if let Some(dir) = val.as_str() {
                                    let base = pkg_path.join(dir);
                                    psr4_map.push((k.clone(), base.to_string_lossy().into_owned()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // write autoload_psr4.php
    let mut s = String::from("<?php\nreturn [\n");
    for (ns, dir) in psr4_map {
        use std::fmt::Write;
        writeln!(
            &mut s,
            "  '{}' => '{}',",
            ns.replace('\'', "\\'"),
            dir.replace('\'', "\\'")
        ).unwrap();
    }
    s.push_str("];\n");
    tokio::fs::write(composer_dir.join("autoload_psr4.php"), s).await?;

    // classmap: top-level + vendor classmap directive
    let mut classmap_entries: Vec<String> = Vec::new();
    if let Some(a) = &composer.autoload {
        for entry in &a.classmap {
            let p = project_dir.join(entry);
            if p.exists() {
                for e in WalkDir::new(&p).into_iter().filter_map(std::result::Result::ok) {
                    if e.file_type().is_file()
                        && e.path().extension().is_some_and(|e| e == "php")
                    {
                        classmap_entries.push(e.path().to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    // vendor packages classmap
    for pkg in installed {
        let pkg_path = pkg.path.as_std_path();
        let cj = pkg_path.join("composer.json");
        if cj.exists() {
            if let Ok(s) = fs::read_to_string(&cj) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                    if let Some(cm) = v.get("autoload").and_then(|x| x.get("classmap")) {
                        if let Some(arr) = cm.as_array() {
                            for it in arr {
                                if let Some(dir) = it.as_str() {
                                    let root = pkg_path.join(dir);
                                    if root.exists() {
                                        for e in
                                            WalkDir::new(&root).into_iter().filter_map(std::result::Result::ok)
                                        {
                                            if e.file_type().is_file()
                                                && e.path()
                                                    .extension()
                                                    .is_some_and(|e| e == "php")
                                            {
                                                classmap_entries
                                                    .push(e.path().to_string_lossy().to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // write classmap
    let mut cm = String::from("<?php\nreturn [\n");
    for p in classmap_entries {
        use std::fmt::Write;
        writeln!(
            &mut cm,
            "  '{}' => '{}',",
            p.replace('\'', "\\'"),
            p.replace('\'', "\\'")
        ).unwrap();
    }
    cm.push_str("];\n");
    tokio::fs::write(composer_dir.join("autoload_classmap.php"), cm).await?;

    // autoload.php shim
    let autoload_php = r#"<?php
// Generated by Lectern
$loader = require __DIR__ . '/autoload_psr4.php';
spl_autoload_register(function($class) use ($loader) {
    foreach ($loader as $prefix => $baseDir) {
        $len = strlen($prefix);
        if (strncmp($prefix, $class, $len) !== 0) continue;
        $relative = str_replace('\\', '/', substr($class, $len)) . '.php';
        $file = rtrim($baseDir, '/').'/'.$relative;
        if (file_exists($file)) { require $file; return true; }
    }
    $classmap = require __DIR__ . '/autoload_classmap.php';
    if (isset($classmap[$class]) && file_exists($classmap[$class])) { require $classmap[$class]; return true; }
    return false;
});
return $loader;
"#;
    tokio::fs::write(
        project_dir.join("vendor").join("autoload.php"),
        autoload_php,
    )
    .await?;
    Ok(())
}
