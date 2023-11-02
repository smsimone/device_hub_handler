use std::{
    fs::{create_dir_all, read_dir, DirEntry},
    io::Cursor,
    path::Path,
    thread,
};

use axum::{extract::Multipart, http::StatusCode, response::Response, routing::post, Router};
use tracing::{error, info};

use crate::{
    services,
    utils::{commands::install_bundle_all, env_helper::ENV_DATA, multipart_helper},
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

        _ = services::bundle_service::extract_bundle(&file);

        let download_dir = &ENV_DATA.lock().unwrap().download_default_dir;
        if !Path::new(&download_dir).exists() {
            match std::fs::create_dir_all(&download_dir) {
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

        let temp_file_path = format!("{}/{}", &download_dir, file.name);

        let temp_file = Path::new(&temp_file_path);

        let extension = match temp_file.extension() {
            Some(extension) => {
                let ext = extension.to_str().unwrap().to_string();
                if ext != "zip" {
                    error!("Only zip files are supported. Received {}", &ext);
                    return Err(StatusCode::BAD_REQUEST);
                }
                ext
            }
            None => {
                error!("Missing extension in file {}", &file.name);
                return Err(StatusCode::UNPROCESSABLE_ENTITY);
            }
        };

        if temp_file.exists() {
            match std::fs::remove_file(&temp_file_path) {
                Ok(_) => info!("Already existent file {} has been removed", &file.name),
                Err(err) => error!("Couldn't remove file {}:\n{}", &file.name, err.to_string()),
            }
        }

        let extraction_folder = temp_file
            .parent()
            .unwrap()
            .join(file.name.replace(&format!(".{}", extension), ""))
            .join("extract");

        info!(
            "Extracting zip in folder: {}",
            &extraction_folder.to_str().unwrap().to_string()
        );

        match create_dir_all(&extraction_folder) {
            Ok(_) => info!("Created all directories to the extraction path"),
            Err(err) => {
                error!(
                    "Failed to create directories in path to extraction folder:\n{}",
                    err.to_string()
                );
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }

        match zip_extract::extract(Cursor::new(&file.bytes), &extraction_folder, true) {
            Ok(_) => info!(
                "Extracted zip file in {}",
                &extraction_folder.to_str().unwrap()
            ),
            Err(err) => {
                error!("Failed to extract zip file:\n{}", err.to_string());
                return Err(StatusCode::UNPROCESSABLE_ENTITY);
            }
        }

        let entries = match read_dir(&extraction_folder) {
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
    return Ok(Response::default());
}
