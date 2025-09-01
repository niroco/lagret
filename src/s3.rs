use std::sync::Arc;

use crate::{
    api::{self, CrateMeta},
    index::IndexEntry,
    store::CrateFile,
};
use aws_sdk_s3::{
    Client,
    config::Credentials,
    operation::{
        get_object::builders::GetObjectFluentBuilder, put_object::builders::PutObjectFluentBuilder,
    },
    primitives::ByteStream,
};
use bytes::Bytes;

mod error;

pub use error::S3Error;
use semver::Version;

pub type S3Result<T> = std::result::Result<T, S3Error>;

#[derive(Clone)]
pub struct S3Storage {
    c: Client,
    bucket_name: Arc<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct S3CrateMeta {
    cksum: String,
    meta: CrateMeta,
    yanked: bool,
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

    fn get(&self, key: impl Into<String>) -> GetObjectFluentBuilder {
        self.c
            .get_object()
            .bucket(self.bucket_name.as_str())
            .key(key)
    }

    fn crate_path(&self, crate_name: &str, version: &Version) -> String {
        format!("{CRATES_BUCKET_DIR}/{crate_name}/{version}/{crate_name}-{version}.crate")
    }

    fn crate_meta_path(&self, crate_name: &str, version: &Version) -> String {
        format!("crates/{crate_name}/{version}/{crate_name}-{version}.json")
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

                let version = split
                    .next()
                    .ok_or_else(|| S3Error::key_split(key, "finding crate version"))?
                    .parse::<Version>()
                    .map_err(|_| S3Error::key_split(key, "invalid version"))?;

                let filename = split
                    .next()
                    .ok_or_else(|| S3Error::key_split(key, "finding filename"))?;

                // Only add entyr once per crate
                if !filename.ends_with("json") {
                    continue;
                }

                let meta_key = self.crate_meta_path(crate_name, &version);
                let fetched_meta = self.get(meta_key).send().await?;

                let bs = fetched_meta.body.collect().await.expect("collecting body");
                let S3CrateMeta {
                    cksum,
                    meta,
                    yanked,
                } = serde_json::from_slice::<S3CrateMeta>(&bs.into_bytes())
                    .expect("parsing meta file");

                println!("adding {} {}", meta.name, meta.vers);
                index.add_crate_meta(IndexEntry {
                    cksum,
                    meta,
                    yanked,
                })
            }
        }

        Ok(index)
    }

    pub async fn download(
        &self,
        crate_name: impl AsRef<str>,
        version: &Version,
    ) -> S3Result<crate::CrateFile> {
        let key = self.crate_path(crate_name.as_ref(), version);
        let res = self.get(key).send().await?;

        let size = res.content_length().expect("missing content len") as usize;
        let data = res.body.collect().await?;

        Ok(CrateFile {
            size,
            data: data.into_bytes(),
        })
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

    pub async fn store_crate(&self, meta: api::CrateMeta, data: Bytes) -> S3Result<IndexEntry> {
        let crate_name = &meta.name;
        let crate_version = &meta.vers;

        let crate_key = self.crate_path(&meta.name, crate_version);
        let meta_key =
            format!("crates/{crate_name}/{crate_version}/{crate_name}-{crate_version}.json");

        let cksum = sha256::digest(data.as_ref());

        let s3_entry = S3CrateMeta {
            cksum,
            meta,
            yanked: false,
        };

        let json_vec = serde_json::to_vec(&s3_entry).expect("serializing crate meta");

        self.put(crate_key)
            .body(ByteStream::from(data))
            .send()
            .await?;

        self.put(meta_key)
            .body(ByteStream::from(json_vec))
            .send()
            .await?;

        Ok(IndexEntry {
            cksum: s3_entry.cksum,
            meta: s3_entry.meta,
            yanked: s3_entry.yanked,
        })
    }
}
