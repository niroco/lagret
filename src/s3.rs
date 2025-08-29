use std::sync::Arc;

use crate::api;
use aws_sdk_s3::{
    Client, config::Credentials, operation::put_object::builders::PutObjectFluentBuilder,
    primitives::ByteStream,
};
use bytes::Bytes;

mod error;

pub use error::S3Error;

pub type S3Result<T> = std::result::Result<T, S3Error>;

#[derive(Clone)]
pub struct S3Storage {
    c: Client,
    bucket_name: Arc<String>,
}

static CRATES_BUCKET_DIR: &str = "crates";

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

    fn put(&self, key: impl Into<String>) -> PutObjectFluentBuilder {
        self.c
            .put_object()
            .bucket(self.bucket_name.as_str())
            .key(key)
    }

    pub async fn load_index(&self) -> S3Result<crate::Index> {
        let mut objects_paginator = self
            .c
            .list_objects_v2()
            .bucket(self.bucket_name.as_str())
            .into_paginator()
            .page_size(50)
            .send();

        let mut index = crate::Index::default();

        while let Some(page) = objects_paginator.next().await.transpose()? {
            for object in page.contents.into_iter().flatten() {
                let Some(key) = object.key.as_deref() else {
                    continue;
                };

                let mut split = key.split('/');

                if Some(CRATES_BUCKET_DIR) != split.next() {
                    continue;
                }

                let crate_name = split
                    .next()
                    .ok_or_else(|| S3Error::key_split(key, "finding crate name"))?;

                let crate_version = split
                    .next()
                    .ok_or_else(|| S3Error::key_split(key, "finding crate version"))?;

                let filename = split
                    .next()
                    .ok_or_else(|| S3Error::key_split(key, "finding filename"))?;

                // Only add entyr once per crate
                if !filename.ends_with("json") {
                    continue;
                }

                let meta_key = format!(
                    "{CRATES_BUCKET_DIR}/{crate_name}/{crate_version}/{crate_name}-{crate_version}.json"
                );

                //let crate_key = format!(
                //    "{CRATES_BUCKET_DIR}/{crate_name}/{crate_version}/{crate_name}-{crate_version}.crate"
                //);

                let fetched_meta = self
                    .c
                    .get_object()
                    .bucket(self.bucket_name.as_str())
                    .key(meta_key)
                    .send()
                    .await?;

                let bs = fetched_meta.body.collect().await.expect("collecting body");
                let crate_meta = serde_json::from_slice::<crate::api::CrateMeta>(&bs.into_bytes())
                    .expect("parsing meta file");

                println!("adding {} {}", crate_meta.name, crate_meta.vers);
                index.add_crate_meta(crate_meta)
            }
        }

        Ok(index)
    }

    pub async fn list_objects(&self) -> S3Result<()> {
        let res = self
            .c
            .list_objects_v2()
            .bucket(self.bucket_name.as_str())
            .send()
            .await?;

        if let Some(contents) = &res.contents {
            for obj in contents {
                let Some(key) = obj.key.as_deref() else {
                    continue;
                };

                println!("Object: {key}")
            }
        }

        Ok(())
    }

    pub async fn put_text_object(&self, key: String, content: String) -> S3Result<()> {
        self.put(key)
            .body(ByteStream::from(content.into_bytes()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn store_crate(&self, meta: &api::CrateMeta, data: Bytes) -> S3Result<()> {
        let base = format!("crates/{}/{}", meta.name, meta.vers);
        let crate_key = format!("{}/{}-{}.crate", base, meta.name, meta.vers);
        let meta_key = format!("{}/{}-{}.json", base, meta.name, meta.vers);

        let json_vec = serde_json::to_vec(&meta).expect("serializing crate meta");

        self.put(crate_key)
            .body(ByteStream::from(data))
            .send()
            .await?;

        self.put(meta_key)
            .body(ByteStream::from(json_vec))
            .send()
            .await?;

        Ok(())
    }
}
