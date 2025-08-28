use axum::{
    Json, http,
    response::{IntoResponse, Response},
};

use crate::s3::S3Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not found")]
    NotFound,

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
            Self::S3(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let description = self.to_string();

        eprintln!("{status_code}: {description}");

        (status_code, Json(ErrorResponse { description })).into_response()
    }
}
