use axum::{Json, extract, http::HeaderMap};

use bytes::{Buf, Bytes};

use crate::{IndexState, Result, S3Storage, api};

pub async fn publish_crate(
    headers: HeaderMap,
    extract::Extension(store): extract::Extension<S3Storage>,
    extract::Extension(IndexState(mtx)): extract::Extension<IndexState>,
    mut bs: Bytes,
) -> Result<Json<api::PublishResult>> {
    println!("Publish: {headers:#?}");
    let total_len = bs.len();
    let json_len = bs.get_u32_le() as usize;
    let json_data = bs.split_to(json_len);

    let meta = match serde_json::from_slice::<api::CrateMeta>(&json_data) {
        Ok(meta) => meta,

        Err(err) => {
            eprintln!("deserializing CrateMeta: {err}");
            eprintln!(
                "raw: {}",
                str::from_utf8(&json_data).expect("json must be utf8")
            );
            panic!("shit");
        }
    };

    // check if the crate exists
    {
        let idx_read = mtx.read().await;
        if idx_read.get_crate_version(&meta.name, &meta.vers).is_some() {
            return Err(crate::error::Error::CrateExists {
                name: meta.name,
                version: meta.vers,
            });
        }
    }

    println!("read meta: {meta:#?}");

    let data_len = bs.get_u32_le() as usize;
    println!("pacakge_len: {data_len}");

    let data = bs.split_to(data_len);

    println!("read {}bs of .crate", data.len());

    assert_eq!(
        total_len,
        4 + json_len + 4 + data_len,
        "the total size of the data"
    );

    let index_entry = store.store_crate(meta, data).await?;

    let mut index_write = mtx.write().await;

    index_write.add_crate_meta(index_entry);

    Ok(Json(api::PublishResult::default()))
}
