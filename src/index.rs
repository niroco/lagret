use semver::Version;
use std::collections::HashMap;

use crate::api::CrateMeta;

type VersionMap = HashMap<Version, CrateMeta>;
type CrateMap = HashMap<String, VersionMap>;

#[derive(Default)]
pub struct Index {
    crates: CrateMap,
}

impl Index {
    pub fn add_crate_meta(&mut self, meta: CrateMeta) {
        let name = meta.name.clone();
        let version = meta.vers.clone();

        self.crates.entry(name).or_default().insert(version, meta);
    }

    pub fn get_crate<'a>(
        &'a self,
        crate_name: &str,
    ) -> Option<impl IntoIterator<Item = &'a CrateMeta>> {
        self.crates
            .get(crate_name)
            .map(|versions| versions.values())
    }
}
