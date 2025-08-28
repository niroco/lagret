use aws_sdk_s3::{error::SdkError, operation::put_object::PutObjectError};

#[derive(Debug, thiserror::Error)]
pub enum S3Error {
    #[error("PUT {status:03?}: {message}")]
    Put {
        status: Option<u16>,
        message: String,
    },
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
