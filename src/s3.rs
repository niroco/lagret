use std::sync::Arc;

use crate::{
    api::{self, PublishedCrate},
    error::Optional,
};
use aws_sdk_s3::{
    Client, config::Credentials, operation::put_object::builders::PutObjectFluentBuilder,
    primitives::ByteStream,
};
use bytes::{Bytes, BytesMut};

mod error;

pub use error::S3Error;
use tokio::sync::RwLock;

pub type S3Result<T> = std::result::Result<T, S3Error>;

#[derive(Clone)]
pub struct S3Storage {
    c: Client,
    bucket_name: Arc<String>,

    index: Arc<RwLock<Vec<PublishedCrate>>>,
}

impl S3Storage {
    pub async fn from_env() -> Self {
        let access_key =
            std::env::var("LAGRET_AWS_ACCESS_KEY").expect("loading LAGRET_AWS_ACCESS_KEY");
        let secret_access_key =
            std::env::var("LAGRET_AWS_SECRET_ACCESS_KEY").expect("LAGRET_AWS_SECRET_ACCESS_KEY");

        let bucket_name =
            Arc::new(std::env::var("LAGRET_AWS_S3_BUCKET").expect("LAGRET_AWS_S3_BUCKET"));

        let config = aws_config::from_env()
            .credentials_provider(Credentials::new(
                access_key,
                secret_access_key,
                None,
                None,
                "from env",
            ))
            .app_name(aws_config::AppName::new("lagret").expect("app name"))
            .load()
            .await;

        let c = Client::new(&config);

        Self {
            c,
            bucket_name,
            index: Default::default(),
        }
    }

    fn put(&self, key: impl Into<String>) -> PutObjectFluentBuilder {
        self.c
            .put_object()
            .bucket(self.bucket_name.as_str())
            .key(key)
    }

    pub async fn load_latest_index(&self) -> S3Result<()> {
        let mut write_lock = self.index.write().await;
        let index = self.get_index().await.optional()?;

        *write_lock = index.unwrap_or_default();

        Ok(())
    }

    pub async fn get_index(&self) -> S3Result<Vec<PublishedCrate>> {
        let mut res = self
            .c
            .get_object()
            .bucket(self.bucket_name.as_str())
            .key("index.json")
            .send()
            .await?;

        let mut buf = BytesMut::new();

        while let Ok(bs) = res.body.next().await.expect("getting bs") {
            buf.extend(bs);
        }

        let res = serde_json::from_slice(&buf).expect("deser");

        Ok(res)
    }

    pub async fn put_text_object(&self, key: String, content: String) -> S3Result<()> {
        self.put(key)
            .body(ByteStream::from(content.into_bytes()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn store_crate(&self, meta: &api::CrateMeta, data: Bytes) -> S3Result<()> {
        let key = format!("crates/{}/{}", meta.name, meta.vers);
        self.put(key).body(ByteStream::from(data)).send().await?;

        Ok(())
    }
}
