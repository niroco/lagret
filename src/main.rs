use axum::{
    Extension, Json, Router, extract,
    http::{HeaderMap, request::Parts},
    routing,
};
use bytes::{Buf, Bytes};

mod api;
mod error;
mod s3;
mod store;

use error::Error;
use store::Store;

use crate::{error::Optional, s3::S3Storage};

type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let s3_storage = S3Storage::from_env().await;

    s3_storage.load_latest_index().await?;

    if let Some((key, content)) = std::env::args().nth(1).zip(std::env::args().nth(2)) {
        println!("putting {key} => {content}");

        s3_storage.put_text_object(key, content).await?;

        return Ok(());
    }

    // build our application with a single route
    let app = Router::new()
        .route("/config.json", routing::get(config))
        .route("/{s1}/{s2}/{name}", routing::get(crates_get))
        .route("/api/v1/crates/new", routing::put(crates_publish))
        .route("/api/v1/crates", routing::get(crates_search))
        .layer(Extension(Store::default()))
        .layer(Extension(s3_storage))
        .fallback(fallback);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(serde::Deserialize, Debug)]
struct GetCrate {
    pub s1: String,
    pub s2: String,
    pub name: String,
}

async fn crates_get(
    extract::Path(args): extract::Path<GetCrate>,
    extract::Extension(store): extract::Extension<Store>,
) -> Result<Json<api::PublishedCrate>> {
    println!("Got Crates get: {args:#?}",);

    let res = store.get_crate(&args.name).await?;
    Ok(Json(res))
}

async fn crates_publish(
    headers: HeaderMap,
    extract::Extension(store): extract::Extension<S3Storage>,
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

    store.store_crate(&meta, data).await?;

    Ok(Json(api::PublishResult::default()))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct SearchArgs {
    q: String,
    per_page: usize,
}

async fn crates_search(
    extract::Query(args): extract::Query<SearchArgs>,
) -> Json<api::SearchResult> {
    println!("got search requests: {args:#?}");

    Json(api::SearchResult {
        crates: vec![api::CrateListItem {
            name: "hello".into(),
            max_version: "1.0.0".into(),
            description: "a dummy crate just here to see if we can reply to Cargo".into(),
        }],
        meta: api::SearchMeta { total: 10 },
    })
}

#[derive(serde::Serialize)]
struct Config {
    dl: String,
    api: String,
}

async fn config(parts: Parts) -> Json<Config> {
    println!("got request: {parts:#?}");

    Json(Config {
        dl: "http://localhost:3000/api/v1/crates/{crate}/{version}/download".into(),
        api: "http://localhost:3000".into(),
    })
}

async fn fallback(parts: Parts) -> &'static str {
    println!("fallback: {parts:#?}");

    "hello"
}
