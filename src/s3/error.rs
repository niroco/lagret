use aws_sdk_s3::{
    error::SdkError,
    operation::{
        get_object::GetObjectError, list_objects_v2::ListObjectsV2Error, put_object::PutObjectError,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum S3Error {
    #[error("Non2xx reply: {status:?}: {message}")]
    Non2xx {
        status: Option<u16>,
        message: String,
    },

    #[error("splitting key `{key}`: {message}")]
    KeySplit { key: String, message: String },

    #[error("Streaming data error: {0}")]
    StreamError(String),
}

impl S3Error {
    pub fn key_split(key: impl Into<String>, message: impl Into<String>) -> Self {
        Self::KeySplit {
            key: key.into(),
            message: message.into(),
        }
    }
}

impl<T> crate::error::Optional<T, S3Error> for Result<T, S3Error> {
    fn optional(self) -> Result<Option<T>, S3Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(S3Error::Non2xx {
                status: Some(404), ..
            }) => Ok(None),

            Err(err) => Err(err),
        }
    }
}

impl From<aws_sdk_s3::primitives::ByteStreamError> for S3Error {
    fn from(err: aws_sdk_s3::primitives::ByteStreamError) -> Self {
        Self::StreamError(err.to_string())
    }
}

impl From<SdkError<PutObjectError>> for S3Error {
    fn from(err: SdkError<PutObjectError>) -> Self {
        println!("{err:#?}");

        let message = err
            .as_service_error()
            .and_then(|err| err.meta().message())
            .unwrap_or("-")
            .to_string();

        println!("message: {message}",);

        Self::Non2xx {
            status: err.raw_response().map(|r| r.status().as_u16()),
            message,
        }
    }
}

impl From<SdkError<ListObjectsV2Error>> for S3Error {
    fn from(err: SdkError<ListObjectsV2Error>) -> Self {
        println!("{err:#?}");

        let message = err
            .as_service_error()
            .and_then(|err| err.meta().message())
            .unwrap_or("-")
            .to_string();

        println!("message: {message}",);

        Self::Non2xx {
            status: err.raw_response().map(|r| r.status().as_u16()),
            message,
        }
    }
}

impl From<SdkError<GetObjectError>> for S3Error {
    fn from(err: SdkError<GetObjectError>) -> Self {
        println!("{err:#?}");

        let message = err
            .as_service_error()
            .and_then(|err| err.meta().message())
            .unwrap_or("-")
            .to_string();

        println!("message: {message}",);

        Self::Non2xx {
            status: err.raw_response().map(|r| r.status().as_u16()),
            message,
        }
    }
}
