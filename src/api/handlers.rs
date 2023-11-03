use std::{
    fs::{read_dir, DirEntry},
    thread,
};

use axum::{extract::Multipart, http::StatusCode, response::Response, routing::post, Router};
use tracing::{error, info};

use crate::{
    services,
    utils::{commands::install_bundle_all, multipart_helper},
};

/// Initializes a new instance of [Router] to handle the rest APIs
pub fn initialize_router() -> Router {
    Router::new().route("/upload", post(upload_bundle))
}

/// Handles the upload of a given bundle and starts the installation process
async fn upload_bundle(mut multipart: Multipart) -> Result<Response, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file = match multipart_helper::extract_file(field).await {
            Some(f) => f,
            None => continue,
        };

        if let Ok(extraction_path) = services::bundle_service::extract_bundle(&file) {
            let entries = match read_dir(&extraction_path) {
                Ok(e) => e
                    .filter(|item| item.is_ok())
                    .map(|item| item.unwrap())
                    .collect::<Vec<DirEntry>>(),
                Err(err) => {
                    error!("Failed to read extraction directory: {}", err.to_string());
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            for bundle_file in entries {
                let path = bundle_file.path().to_str().unwrap().to_string();
                thread::spawn(move || match install_bundle_all(&path) {
                    Ok(_) => info!("Installed bundle againts all devices"),
                    Err(err) => error!("Failed to install bundle:\n{}", err),
                });
            }
        }
    }
    return Ok(Response::default());
}
