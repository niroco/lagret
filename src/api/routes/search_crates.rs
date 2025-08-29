use axum::{Json, extract};

use crate::{IndexState, api};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Args {
    q: String,
    per_page: usize,
}

pub async fn search_crates(
    extract::Query(args): extract::Query<Args>,
    extract::Extension(IndexState(mtx)): extract::Extension<IndexState>,
) -> Json<api::SearchResult> {
    let idx_read = mtx.read().await;

    let res = idx_read.search_crates(&args.q, args.per_page);

    Json(res)
}
