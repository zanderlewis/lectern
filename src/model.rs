use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Composer JSON format - fully compatible
#[derive(Debug, Deserialize, Serialize)]
pub struct ComposerJson {
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default, rename = "type")]
    pub package_type: Option<String>,
    #[serde(default)]
    pub keywords: Option<Vec<String>>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub readme: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub license: Option<Vec<String>>,
    #[serde(default)]
    pub authors: Option<Vec<Author>>,
    #[serde(default)]
    pub support: Option<Support>,
    #[serde(default)]
    pub require: BTreeMap<String, String>,
    #[serde(default, rename = "require-dev")]
    pub require_dev: BTreeMap<String, String>,
    #[serde(default)]
    pub conflict: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub replace: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub provide: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub suggest: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub autoload: Option<Autoload>,
    #[serde(default, rename = "autoload-dev")]
    pub autoload_dev: Option<Autoload>,
    #[serde(default)]
    pub include_path: Option<Vec<String>>,
    #[serde(default, rename = "target-dir")]
    pub target_dir: Option<String>,
    #[serde(default)]
    pub repositories: Option<Vec<Repository>>,
    #[serde(default)]
    pub config: Option<Config>,
    #[serde(default)]
    pub scripts: Option<BTreeMap<String, ScriptDefinition>>,
    #[serde(default)]
    pub extra: Option<serde_json::Value>,
    #[serde(default, rename = "minimum-stability")]
    pub minimum_stability: Option<String>,
    #[serde(default, rename = "prefer-stable")]
    pub prefer_stable: Option<bool>,
    #[serde(default)]
    pub bin: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Author {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Support {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub issues: Option<String>,
    #[serde(default)]
    pub forum: Option<String>,
    #[serde(default)]
    pub wiki: Option<String>,
    #[serde(default)]
    pub irc: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub docs: Option<String>,
    #[serde(default)]
    pub rss: Option<String>,
    #[serde(default)]
    pub chat: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default, rename = "vendor-dir")]
    pub vendor_dir: Option<String>,
    #[serde(default, rename = "bin-dir")]
    pub bin_dir: Option<String>,
    #[serde(default, rename = "process-timeout")]
    pub process_timeout: Option<u32>,
    #[serde(default, rename = "use-include-path")]
    pub use_include_path: Option<bool>,
    #[serde(default, rename = "preferred-install")]
    pub preferred_install: Option<serde_json::Value>,
    #[serde(default, rename = "notify-on-install")]
    pub notify_on_install: Option<bool>,
    #[serde(default, rename = "github-protocols")]
    pub github_protocols: Option<Vec<String>>,
    #[serde(default, rename = "github-oauth")]
    pub github_oauth: Option<BTreeMap<String, String>>,
    #[serde(default, rename = "gitlab-oauth")]
    pub gitlab_oauth: Option<BTreeMap<String, String>>,
    #[serde(default, rename = "gitlab-token")]
    pub gitlab_token: Option<BTreeMap<String, String>>,
    #[serde(default, rename = "http-basic")]
    pub http_basic: Option<BTreeMap<String, HttpBasicAuth>>,
    #[serde(default, rename = "store-auths")]
    pub store_auths: Option<bool>,
    #[serde(default)]
    pub platform: Option<BTreeMap<String, String>>,
    #[serde(default, rename = "archive-format")]
    pub archive_format: Option<String>,
    #[serde(default, rename = "archive-dir")]
    pub archive_dir: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpBasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ScriptDefinition {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
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
    Composer { 
        url: String,
        #[serde(default)]
        canonical: Option<bool>,
        #[serde(default)]
        only: Option<Vec<String>>,
        #[serde(default)]
        exclude: Option<Vec<String>>,
        #[serde(default)]
        options: Option<serde_json::Value>,
    },
    Vcs { 
        url: String,
        #[serde(default)]
        canonical: Option<bool>,
        #[serde(default)]
        only: Option<Vec<String>>,
        #[serde(default)]
        exclude: Option<Vec<String>>,
        #[serde(default)]
        options: Option<serde_json::Value>,
    },
    Path { 
        url: String,
        #[serde(default)]
        canonical: Option<bool>,
        #[serde(default)]
        only: Option<Vec<String>>,
        #[serde(default)]
        exclude: Option<Vec<String>>,
        #[serde(default)]
        options: Option<serde_json::Value>,
    },
    Package {
        package: serde_json::Value,
        #[serde(default)]
        canonical: Option<bool>,
        #[serde(default)]
        only: Option<Vec<String>>,
        #[serde(default)]
        exclude: Option<Vec<String>>,
    },
    Artifact { 
        url: String,
        #[serde(default)]
        canonical: Option<bool>,
        #[serde(default)]
        only: Option<Vec<String>>,
        #[serde(default)]
        exclude: Option<Vec<String>>,
    },
    Pear { 
        url: String,
        #[serde(default)]
        canonical: Option<bool>,
        #[serde(default)]
        only: Option<Vec<String>>,
        #[serde(default)]
        exclude: Option<Vec<String>>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lock {
    #[serde(default)]
    pub _readme: Vec<String>,
    #[serde(rename = "content-hash")]
    pub content_hash: String,
    pub packages: Vec<LockedPackage>,
    #[serde(default, rename = "packages-dev")]
    pub packages_dev: Vec<LockedPackage>,
    #[serde(default)]
    pub aliases: Vec<serde_json::Value>,
    #[serde(default, rename = "minimum-stability")]
    pub minimum_stability: String,
    #[serde(default, rename = "stability-flags")]
    pub stability_flags: BTreeMap<String, i32>,
    #[serde(default, rename = "prefer-stable")]
    pub prefer_stable: bool,
    #[serde(default, rename = "prefer-lowest")]
    pub prefer_lowest: bool,
    #[serde(default)]
    pub platform: BTreeMap<String, String>,
    #[serde(default, rename = "platform-dev")]
    pub platform_dev: BTreeMap<String, String>,
    #[serde(default)]
    pub plugin_api_version: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub source: Option<SourceInfo>,
    #[serde(default)]
    pub dist: Option<DistInfo>,
    #[serde(default)]
    pub require: Option<BTreeMap<String, String>>,
    #[serde(default, rename = "require-dev")]
    pub require_dev: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub conflict: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub replace: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub provide: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub suggest: Option<BTreeMap<String, String>>,
    #[serde(default, rename = "type")]
    pub package_type: Option<String>,
    #[serde(default)]
    pub extra: Option<serde_json::Value>,
    #[serde(default)]
    pub autoload: Option<Autoload>,
    #[serde(default, rename = "autoload-dev")]
    pub autoload_dev: Option<Autoload>,
    #[serde(default, rename = "notification-url")]
    pub notification_url: Option<String>,
    #[serde(default)]
    pub license: Option<Vec<String>>,
    #[serde(default)]
    pub authors: Option<Vec<Author>>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub keywords: Option<Vec<String>>,
    #[serde(default)]
    pub support: Option<Support>,
    #[serde(default)]
    pub funding: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub bin: Option<Vec<String>>,
    #[serde(default, rename = "include-path")]
    pub include_path: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceInfo {
    #[serde(rename = "type")]
    pub source_type: String,
    pub url: String,
    pub reference: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistInfo {
    #[serde(rename = "type")]
    pub dist_type: String,
    pub url: String,
    pub reference: String,
    #[serde(default)]
    pub shasum: String,
}
