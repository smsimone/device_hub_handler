use std::thread;

use axum::{extract::Multipart, http::StatusCode, response::Response, routing::post, Router};
use log::info;
use tracing::error;

use crate::api::services::{bundle_service, device_service};

/// Initializes a new instance of [Router] to handle the rest APIs
pub fn initialize_router() -> Router {
    Router::new().route("/upload", post(upload_bundle))
}

/// Handles the upload of a given bundle and starts the installation process
async fn upload_bundle(mut multipart: Multipart) -> Result<Response, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = match field.file_name() {
            Some(name) => name.to_string(),
            None => {
                error!("Missing filename for the uploaded file");
                continue;
            }
        };

        let bytes = match field.bytes().await {
            Ok(data) => data,
            Err(err) => {
                error!("Failed to get bytes from multipart: {}", err.to_string());
                continue;
            }
        };

        match bundle_service::extract_zip_file(&filename, &bytes.to_vec()) {
            Ok(entries) => {
                for bundle_file in entries {
                    let path = bundle_file.path().to_str().unwrap().to_string();
                    thread::spawn(move || match device_service::install_bundle_all(&path) {
                        Ok(_) => info!("Installed bundle againts all devices"),
                        Err(err) => error!("Failed to install bundle:\n{}", err),
                    });
                }
            }
            Err(_err) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    return Ok(Response::default());
}
