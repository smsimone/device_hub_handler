use std::{path::Path, thread};

use axum::{extract::Multipart, http::StatusCode, response::Response, routing::post, Router};
use log::info;
use tracing::error;

use crate::utils::{commands::install_bundle_all, env_helper::ENV_DATA};

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

        let download_dir = &ENV_DATA.lock().unwrap().download_default_dir;
        if !Path::new(&download_dir).exists() {
            match std::fs::create_dir(&download_dir) {
                Ok(_) => info!("Created download directory {}", &download_dir),
                Err(err) => {
                    error!(
                        "Failed to create download dir {}:\n{}",
                        &download_dir,
                        err.to_string()
                    );
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }

        let temp_file_path = format!("{}/{}", &download_dir, filename);

        if Path::new(&temp_file_path).exists() {
            match std::fs::remove_file(&temp_file_path) {
                Ok(_) => info!("Already existent file {} has been removed", &filename),
                Err(err) => error!("Couldn't remove file {}:\n{}", &filename, err.to_string()),
            }
        }

        match std::fs::write(&temp_file_path, &bytes) {
            Ok(_) => {
                info!(
                    "Written file {}. Starting with the installation process",
                    &filename
                );
                thread::spawn(move || match install_bundle_all(&temp_file_path) {
                    Ok(_) => info!("Installed bundle againts all devices"),
                    Err(err) => error!("Failed to install bundle:\n{}", err),
                });
            }
            Err(err) => {
                error!("Failed to write file {}:\n{}", &filename, err.to_string());
                return Err(StatusCode::UNPROCESSABLE_ENTITY);
            }
        }
    }
    return Ok(Response::default());
}
