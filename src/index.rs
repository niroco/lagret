use semver::Version;
use std::collections::HashMap;

use crate::api::CrateMeta;

type VersionMap = HashMap<Version, IndexEntry>;
type CrateMap = HashMap<String, VersionMap>;

#[derive(Default)]
pub struct Index {
    crates: CrateMap,
}

pub struct IndexEntry {
    pub cksum: String,
    pub meta: CrateMeta,
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
}
