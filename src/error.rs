use axum::{
    Json, http,
    response::{IntoResponse, Response},
};

use crate::{api, s3::S3Error};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("crate `{name}-{version}` is already published")]
    CrateExists { name: String, version: api::Version },

    #[error("S3: {0}")]
    S3(#[from] S3Error),
}

#[derive(serde::Serialize)]
struct ErrorResponse {
    description: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::NotFound => http::StatusCode::NOT_FOUND,
            Self::CrateExists { .. } => http::StatusCode::BAD_REQUEST,
            Self::S3(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let description = self.to_string();

        eprintln!("{status_code}: {description}");

        (status_code, Json(ErrorResponse { description })).into_response()
    }
}

#[allow(dead_code)]
pub trait Optional<T, E>
where
    Self: Sized,
{
    fn optional(self) -> std::result::Result<Option<T>, E>;
}

impl<T> Optional<T, Error> for Result<T, Error> {
    fn optional(self) -> std::result::Result<Option<T>, Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(Error::S3(S3Error::Non2xx {
                status: Some(404), ..
            })) => Ok(None),

            Err(err) => Err(err),
        }
    }
}
