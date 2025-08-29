use axum::Json;

#[derive(serde::Serialize)]
pub struct Config {
    dl: String,
    api: String,
}

pub async fn get_config() -> Json<Config> {
    Json(Config {
        dl: "http://localhost:3000/{crate}/{version}/download".into(),
        api: "http://localhost:3000".into(),
    })
}
