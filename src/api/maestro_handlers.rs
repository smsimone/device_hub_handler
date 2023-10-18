use axum::{extract::Multipart, http::StatusCode, response::Response, routing::post, Router};
use tracing::error;

use crate::utils::{env_helper::ENV_DATA, files_manager::save_file, multipart_helper};

pub fn initialize_router() -> Router {
    Router::new().route("/upload", post(upload_test_file))
}

async fn upload_test_file(mut multipart: Multipart) -> Result<Response, StatusCode> {
    let download_dir = &ENV_DATA.lock().unwrap().tests_folder;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let file = match multipart_helper::extract_file(field).await {
            Some(file) => file,
            None => continue,
        };

        match save_file(&file, download_dir) {
            Err(err) => error!("Failed to write file {}: {}", file.name, err),
            Ok(_) => {}
        }
    }

    return Ok(Response::default());
}
