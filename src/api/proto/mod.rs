use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
pub struct CrateListItem {
    pub name: String,
    pub max_version: String,
    pub description: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SearchResult {
    pub crates: Vec<CrateListItem>,
    pub meta: SearchMeta,
}

#[derive(Debug, serde::Serialize)]
pub struct SearchMeta {
    pub total: u64,
}

#[derive(Default, Debug, serde::Serialize)]
pub struct PublishResult {
    pub warnings: PublishWarnings,
}

#[derive(Default, Debug, serde::Serialize)]
pub struct PublishWarnings {
    pub invalid_categories: Vec<String>,
    pub invalid_badges: Vec<String>,
    pub other: Vec<String>,
}

#[derive(Clone, Default, Debug, serde::Serialize)]
pub struct PublishedCrate {
    pub name: String,
    pub vers: String,
    pub deps: Vec<CrateDep>,
    pub cksum: String,
    pub features: HashMap<String, Vec<String>>,
    pub yanked: bool,
    pub links: Option<String>,
    pub v: u8,
    pub features2: HashMap<String, Vec<String>>,
    pub rust_version: Option<String>,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrateDepKind {
    Dev,
    Build,
    Normal,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CrateDep {
    pub name: String,
    pub version_req: String,
    pub features: String,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: CrateDepKind,
    pub registry: Option<String>,
    pub explicit_name_in_toml: Option<String>,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct CrateMeta {
    pub name: String,
    pub vers: String,
    pub deps: Vec<CrateDep>,
    pub features: HashMap<String, Vec<String>>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub readme: Option<String>,
    pub readme_file: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub badges: HashMap<String, HashMap<String, String>>,
    pub links: Option<String>,
    pub rust_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal_crate_meta() {
        let s = r#"
{"name":"dummy","vers":"0.1.0","deps":[],"features":{},"authors":[],"description":null,"documentation":null,"homepage":null,"readme":null,"readme_file":null,"keywords":[],"categories":[],"license":null,"license_file":null,"repository":null,"badges":{},"links":null,"rust_version":null}
"#;

        serde_json::from_str::<CrateMeta>(s).expect("deserializing minial CrateMeta");
    }
}
