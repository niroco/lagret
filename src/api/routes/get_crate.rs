use axum::{
    extract,
    http::{self, HeaderMap},
};

use crate::{Error, IndexEntry, IndexState, NdJson, Result, api};

#[derive(serde::Deserialize, Debug)]
pub struct Args {
    #[allow(dead_code)]
    s1: String,

    #[allow(dead_code)]
    s2: String,
    name: String,
}

pub async fn get_crate(
    extract::Path(args): extract::Path<Args>,
    extract::Extension(IndexState(mtx)): extract::Extension<IndexState>,
) -> Result<(HeaderMap, NdJson<api::PublishedCrate>)> {
    let read_index = mtx.read().await;

    let Some(versions_iter) = read_index.get_crate(&args.name) else {
        return Err(Error::NotFound);
    };

    let versions_vec = versions_iter
        .into_iter()
        .map(
            |IndexEntry {
                 cksum,
                 meta,
                 yanked,
             }| api::PublishedCrate {
                name: meta.name.clone(),
                vers: meta.vers.clone(),
                deps: meta.deps.clone(),
                cksum: cksum.into(),
                features: meta.features.clone(),
                yanked: *yanked,
                links: meta.links.clone(),
                v: 2,
                features2: meta.features.clone(),
                rust_version: meta.rust_version.clone(),
            },
        )
        .collect::<Vec<_>>();

    let headers = HeaderMap::from_iter([(
        http::HeaderName::from_static("content-type"),
        http::HeaderValue::from_static("applicant/json"),
    )]);

    Ok((headers, NdJson(versions_vec)))
}
