use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Composer JSON format
#[derive(Debug, Deserialize, Serialize)]
pub struct ComposerJson {
    pub name: Option<String>,
    #[serde(default)]
    pub require: BTreeMap<String, String>,
    #[serde(default, rename = "require-dev")]
    pub require_dev: BTreeMap<String, String>,
    #[serde(default)]
    pub autoload: Option<Autoload>,
    #[serde(default)]
    pub repositories: Option<Vec<Repository>>,
    #[serde(default)]
    pub minimum_stability: Option<String>,
    #[serde(default)]
    pub prefer_stable: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Autoload {
    #[serde(default)]
    pub psr4: BTreeMap<String, String>,
    #[serde(default)]
    pub classmap: Vec<String>,
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Repository {
    Composer { url: String },
    Vcs { url: String },
    Path { url: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lock {
    pub packages: Vec<LockedPackage>,
    #[serde(default)]
    pub packages_dev: Vec<LockedPackage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub dist_url: Option<String>,
    pub dist_type: Option<String>,
    pub dist_shasum: Option<String>,
    pub source_url: Option<String>,
    pub source_reference: Option<String>,
    pub source_path: Option<String>,
}
