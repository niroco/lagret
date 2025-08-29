use semver::Version;
use std::collections::HashMap;

use crate::api;

type VersionMap = HashMap<Version, IndexEntry>;
type CrateMap = HashMap<String, VersionMap>;

#[derive(Default)]
pub struct Index {
    crates: CrateMap,
}

pub struct IndexEntry {
    pub cksum: String,
    pub meta: api::CrateMeta,
    pub yanked: bool,
}

impl Index {
    pub fn add_crate_meta(&mut self, entry: IndexEntry) {
        let name = entry.meta.name.clone();
        let version = entry.meta.vers.clone();

        self.crates.entry(name).or_default().insert(version, entry);
    }

    pub fn get_crate<'a>(
        &'a self,
        crate_name: &str,
    ) -> Option<impl IntoIterator<Item = &'a IndexEntry>> {
        self.crates
            .get(crate_name)
            .map(|versions| versions.values())
    }

    pub fn get_crate_version<'a>(
        &'a self,
        crate_name: &str,
        version: &Version,
    ) -> Option<&'a IndexEntry> {
        self.crates
            .get(crate_name)
            .and_then(|versions| versions.get(version))
    }

    pub fn search_crates(&self, q: impl AsRef<str>, max_count: usize) -> api::SearchResult {
        let iter = self
            .crates
            .iter()
            .filter(|(name, _)| name.contains(q.as_ref()));

        let total = iter.clone().count();

        let crates = iter
            .filter_map(|(_, versions)| {
                let max_version = versions.keys().max()?;

                versions
                    .get(max_version)
                    .map(|IndexEntry { meta, .. }| api::CrateListItem {
                        name: meta.name.clone(),
                        max_version: meta.vers.clone(),
                        description: meta.description.clone().unwrap_or_else(String::new),
                    })
            })
            .take(max_count)
            .collect();

        api::SearchResult {
            crates,
            meta: api::SearchMeta { total },
        }
    }
}
