use std::sync::Arc;

use axum::{Extension, Router, http::request::Parts, routing};
use clap::{Parser, Subcommand};
use tokio::sync::RwLock;

mod api;
mod error;
mod index;
mod nd_json;
mod s3;
mod store;

use {
    error::Error,
    index::{Index, IndexEntry},
    nd_json::NdJson,
    s3::S3Storage,
    store::CrateFile,
};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    Run,
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
        .route("/config.json", routing::get(api::routes::get_config))
        .route("/{s1}/{s2}/{name}", routing::get(api::routes::get_crate))
        .route(
            "/{crate_name}/{version}/download",
            routing::get(api::routes::download_crate),
        )
        .route(
            "/api/v1/crates/new",
            routing::put(api::routes::publish_crate),
        )
        .route("/api/v1/crates", routing::get(api::routes::search_crates))
        .layer(Extension(IndexState(Arc::new(RwLock::new(index)))))
        .layer(Extension(s3_storage))
        .fallback(fallback);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn fallback(parts: Parts) -> &'static str {
    eprintln!("fallback: {parts:#?}");

    "hello"
}
