use std::sync::Arc;

use crate::api;
use aws_sdk_s3::{Client, config::Credentials, primitives::ByteStream};
use bytes::Bytes;

mod error;

pub use error::S3Error;

pub type S3Result<T> = std::result::Result<T, S3Error>;

#[derive(Clone)]
pub struct S3Storage {
    c: Client,
    bucket_name: Arc<String>,
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

        Self { c, bucket_name }
    }

    pub async fn put_text_object(&self, key: String, content: String) -> S3Result<()> {
        self.c
            .put_object()
            .bucket(self.bucket_name.as_str())
            .key(key)
            .body(ByteStream::from(content.into_bytes()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn store_crate(&self, meta: &api::CrateMeta, data: Bytes) -> S3Result<()> {
        let key = format!("crates/{}/{}", meta.name, meta.vers);
        self.c
            .put_object()
            .bucket(self.bucket_name.as_str())
            .key(key)
            .body(ByteStream::from(data))
            .send()
            .await?;

        Ok(())
    }
}
