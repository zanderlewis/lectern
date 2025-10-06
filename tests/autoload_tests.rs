use lectern::core::autoload::*;
use lectern::models::model::ComposerJson;
use lectern::installer::InstalledPackage;
use std::collections::BTreeMap;
use std::fs;
use tempfile::TempDir;
use camino::Utf8PathBuf;

#[tokio::test]
async fn test_write_autoload_files_basic() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    let composer = ComposerJson {
        name: Some("test/autoload".to_string()),
        require: BTreeMap::new(),
        require_dev: BTreeMap::new(),
        autoload: None,
        autoload_dev: None,
        description: None,
        version: None,
        package_type: None,
        keywords: None,
        homepage: None,
        readme: None,
        time: None,
        license: None,
        authors: None,
        support: None,
        conflict: None,
        replace: None,
        provide: None,
        suggest: None,
        include_path: None,
        target_dir: None,
        repositories: None,
        config: None,
        scripts: None,
        extra: None,
        minimum_stability: None,
        prefer_stable: None,
        bin: None,
    };
    
    let installed = vec![];
    
    let result = write_autoload_files(temp_path, &composer, &installed).await;
    assert!(result.is_ok());
    
    // Check that autoload.php was created
    let autoload_file = temp_path.join("vendor").join("autoload.php");
    assert!(autoload_file.exists(), "autoload.php should be created");
}

#[tokio::test]
async fn test_write_autoload_files_with_psr4() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    let mut psr4_map = BTreeMap::new();
    psr4_map.insert("App\\".to_string(), "src/".to_string());
    
    let composer = ComposerJson {
        name: Some("test/psr4".to_string()),
        require: BTreeMap::new(),
        require_dev: BTreeMap::new(),
        autoload: Some(lectern::models::model::Autoload {
            psr4: psr4_map,
            classmap: vec![],
            files: vec![],
        }),
        autoload_dev: None,
        description: None,
        version: None,
        package_type: None,
        keywords: None,
        homepage: None,
        readme: None,
        time: None,
        license: None,
        authors: None,
        support: None,
        conflict: None,
        replace: None,
        provide: None,
        suggest: None,
        include_path: None,
        target_dir: None,
        repositories: None,
        config: None,
        scripts: None,
        extra: None,
        minimum_stability: None,
        prefer_stable: None,
        bin: None,
    };
    
    let installed = vec![];
    
    let result = write_autoload_files(temp_path, &composer, &installed).await;
    assert!(result.is_ok());
    
    let autoload_file = temp_path.join("vendor").join("autoload.php");
    assert!(autoload_file.exists());
    
    // Just verify the file exists and has content
    let content = fs::read_to_string(autoload_file).unwrap();
    assert!(!content.is_empty(), "autoload.php should not be empty");
}

#[tokio::test]
async fn test_write_autoload_files_with_packages() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    let composer = ComposerJson {
        name: Some("test/packages".to_string()),
        require: BTreeMap::new(),
        require_dev: BTreeMap::new(),
        autoload: None,
        autoload_dev: None,
        description: None,
        version: None,
        package_type: None,
        keywords: None,
        homepage: None,
        readme: None,
        time: None,
        license: None,
        authors: None,
        support: None,
        conflict: None,
        replace: None,
        provide: None,
        suggest: None,
        include_path: None,
        target_dir: None,
        repositories: None,
        config: None,
        scripts: None,
        extra: None,
        minimum_stability: None,
        prefer_stable: None,
        bin: None,
    };
    
    let installed = vec![
        InstalledPackage {
            name: "vendor/package1".to_string(),
            version: "1.0.0".to_string(),
            path: Utf8PathBuf::from("vendor/vendor/package1"),
        },
        InstalledPackage {
            name: "vendor/package2".to_string(),
            version: "2.0.0".to_string(),
            path: Utf8PathBuf::from("vendor/vendor/package2"),
        },
    ];
    
    let result = write_autoload_files(temp_path, &composer, &installed).await;
    assert!(result.is_ok());
    
    let autoload_file = temp_path.join("vendor").join("autoload.php");
    assert!(autoload_file.exists());
}

#[test]
fn test_autoload_structure() {
    use lectern::models::model::Autoload;
    
    let mut psr4 = BTreeMap::new();
    psr4.insert("Test\\".to_string(), "tests/".to_string());
    
    let autoload = Autoload {
        psr4,
        classmap: vec!["src/helpers.php".to_string()],
        files: vec!["src/functions.php".to_string()],
    };
    
    assert_eq!(autoload.psr4.len(), 1);
    assert_eq!(autoload.classmap.len(), 1);
    assert_eq!(autoload.files.len(), 1);
}
