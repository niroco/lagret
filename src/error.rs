use axum::{
    Json, http,
    response::{IntoResponse, Response},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not found")]
    NotFound,
}

#[derive(serde::Serialize)]
struct ErrorResponse {
    description: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code = match self {
            Self::NotFound => http::StatusCode::NOT_FOUND,
        };

        let description = self.to_string();

        eprintln!("{status_code}: {description}");

        (status_code, Json(ErrorResponse { description })).into_response()
    }
}
