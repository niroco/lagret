use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;

use crate::api;

#[derive(Default, Clone)]
pub struct Store {
    crates: Arc<tokio::sync::RwLock<HashMap<String, api::PublishedCrate>>>,
}

impl Store {
    pub async fn get_crate(&self, name: &str) -> crate::Result<api::PublishedCrate> {
        let read = self.crates.read().await;
        read.get(name).cloned().ok_or(crate::Error::NotFound)
    }

    pub async fn store_published_crate(
        &self,
        meta: api::CrateMeta,
        crate_data: Bytes,
    ) -> crate::Result<()> {
        let slice: &[u8] = &crate_data;
        let cksum = sha256::digest(slice);

        let crate_name = meta.name.clone();

        let published_crate = api::PublishedCrate {
            name: meta.name,
            vers: meta.vers,
            deps: meta.deps,
            cksum,
            features: Default::default(),
            yanked: false,
            links: meta.links,
            v: 2,
            features2: meta.features,
            rust_version: meta.rust_version,
        };

        let mut write = self.crates.write().await;

        write.insert(crate_name, published_crate);

        Ok(())
    }
}
