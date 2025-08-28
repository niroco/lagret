use aws_sdk_s3::{
    error::SdkError,
    operation::{get_object::GetObjectError, put_object::PutObjectError},
};

#[derive(Debug, thiserror::Error)]
pub enum S3Error {
    #[error("PUT {status:03?}: {message}")]
    Put {
        status: Option<u16>,
        message: String,
    },

    #[error("GET {status:03?}: {message}")]
    Get {
        status: Option<u16>,
        message: String,
    },
}

impl<T> crate::error::Optional<T, S3Error> for Result<T, S3Error> {
    fn optional(self) -> Result<Option<T>, S3Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(S3Error::Get {
                status: Some(404), ..
            }) => Ok(None),

            Err(err) => Err(err),
        }
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

        Self::Put {
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

        Self::Put {
            status: err.raw_response().map(|r| r.status().as_u16()),
            message,
        }
    }
}
