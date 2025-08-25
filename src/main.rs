use axum::{Json, Router, extract, http::request::Parts, routing};
use bytes::{Buf, Bytes};

type Result<T> = anyhow::Result<T>;

mod api;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/config.json", routing::get(config))
        .route("/api/v1/crates/new", routing::put(crates_publish))
        .route("/api/v1/crates", routing::get(crates_search))
        .fallback(fallback);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct SearchArgs {
    q: String,
    per_page: usize,
}

async fn crates_publish(mut bs: Bytes) -> Json<api::PublishResult> {
    let total_len = bs.len();

    let json_len = bs.get_u32_le() as usize;

    let meta = serde_json::from_slice::<api::CrateMeta>(&bs[0..json_len]).expect("getting meta");

    println!("read meta: {meta:#?}");

    let package_len = bs.get_u32_le() as usize;

    assert_eq!(
        total_len,
        4 + json_len + 4 + package_len,
        "the total size of the data"
    );

    Json(api::PublishResult::default())
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
