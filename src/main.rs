use std::sync::Arc;

use axum::{
    Extension, Json, Router, extract,
    http::{self, HeaderMap, request::Parts},
    routing,
};
use bytes::{Buf, Bytes};
use clap::{Parser, Subcommand};
use tokio::sync::RwLock;

mod api;
mod error;
mod index;
mod nd_json;
mod s3;

use {
    error::Error,
    index::{Index, IndexEntry},
    nd_json::NdJson,
};

use crate::{api::PublishedCrate, s3::S3Storage};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    Run,
    PutKey { key: String, value: String },

    ListObjects,
    LoadIndex,
}

#[derive(Clone)]
pub struct IndexState(Arc<RwLock<Index>>);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let s3_storage = S3Storage::from_env().await;

    let args = Args::parse();

    match args.command.unwrap_or(Command::Run) {
        Command::Run => (),

        Command::PutKey { key, value } => {
            println!("putting {key} => {value}");
            s3_storage.put_text_object(key, value).await?;
            return Ok(());
        }

        Command::ListObjects => {
            s3_storage.list_objects().await?;
            return Ok(());
        }

        Command::LoadIndex => {
            s3_storage.load_index().await?;
            return Ok(());
        }
    }

    let index = s3_storage.load_index().await?;

    // build our application with a single route
    let app = Router::new()
        .route("/config.json", routing::get(get_config))
        .route("/{s1}/{s2}/{name}", routing::get(get_crate))
        .route("/api/v1/crates/new", routing::put(publish_crate))
        .route("/api/v1/crates", routing::get(search_crates))
        .layer(Extension(IndexState(Arc::new(RwLock::new(index)))))
        .layer(Extension(s3_storage))
        .fallback(fallback);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(serde::Deserialize, Debug)]
struct GetCrate {
    #[allow(dead_code)]
    pub s1: String,

    #[allow(dead_code)]
    pub s2: String,
    pub name: String,
}

async fn get_crate(
    extract::Path(args): extract::Path<GetCrate>,
    extract::Extension(IndexState(mtx)): extract::Extension<IndexState>,
) -> Result<(HeaderMap, NdJson<PublishedCrate>)> {
    println!("Got Crates get: {args:#?}",);

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
             }| PublishedCrate {
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

async fn publish_crate(
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

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct SearchArgs {
    q: String,
    per_page: usize,
}

async fn search_crates(
    extract::Query(args): extract::Query<SearchArgs>,
) -> Json<api::SearchResult> {
    println!("got search requests: {args:#?}");

    Json(api::SearchResult {
        crates: vec![api::CrateListItem {
            name: "hello".into(),
            max_version: "1.0.0".parse().unwrap(),
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

async fn get_config(parts: Parts) -> Json<Config> {
    println!("got request: {parts:#?}");

    Json(Config {
        dl: "http://localhost:3000/api/v1/crates/{crate}/{version}/download".into(),
        api: "http://localhost:3000".into(),
    })
}

async fn fallback(parts: Parts) -> &'static str {
    eprintln!("fallback: {parts:#?}");

    "hello"
}
