use axum::{
    extract,
    http::{HeaderMap, HeaderName, HeaderValue},
};
use bytes::Bytes;

use crate::{Result, S3Storage, api::Version};

#[derive(serde::Deserialize)]
pub struct Args {
    crate_name: String,
    version: Version,
}

pub async fn download_crate(
    extract::Path(args): extract::Path<Args>,
    extract::Extension(store): extract::Extension<S3Storage>,
) -> Result<(HeaderMap, Bytes)> {
    let crate_file = store.download(&args.crate_name, &args.version).await?;

    let len_str = format!("{}", crate_file.size);

    Ok((
        HeaderMap::from_iter([(
            HeaderName::from_static("content-length"),
            HeaderValue::from_str(&len_str).expect("valid header"),
        )]),
        crate_file.data,
    ))
}
