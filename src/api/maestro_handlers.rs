use std::{fs::read_dir, io::Write, path::Path};

use axum::{
    extract::Multipart,
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use log::info;
use serde_json::{json, Value};
use tracing::error;

use crate::utils::env_helper::ENV_DATA;

/// Initializes a new instance of [Router] to handle the rest APIs
pub fn initialize_router() -> Router {
    Router::new()
        .route("/", get(get_uploaded_files))
        .route("/upload", post(upload_test_file))
}

async fn get_uploaded_files() -> Result<Json<Value>, StatusCode> {
    let tests_dir = &ENV_DATA.lock().unwrap().maestro_tests_dir;

    let entries = match read_dir(&tests_dir) {
        Ok(e) => e
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap())
            .map(|e| e.file_name().to_str().unwrap().to_string())
            .collect::<Vec<String>>(),
        Err(err) => {
            error!("Failed to read tests directory: {}", err.to_string());
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    return Ok(Json(json!({ "tests": entries })));
}

/// Handles the upload of a new maestro test file (.yml file)
async fn upload_test_file(mut multipart: Multipart) -> Result<Response, StatusCode> {
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

        let tests_dir = &ENV_DATA.lock().unwrap().maestro_tests_dir;
        if !Path::new(&tests_dir).exists() {
            match std::fs::create_dir_all(&tests_dir) {
                Ok(_) => info!("Created download directory {}", &tests_dir),
                Err(err) => {
                    error!(
                        "Failed to create tests dir {}:\n{}",
                        &tests_dir,
                        err.to_string()
                    );
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }

        let temp_file_path = format!("{}/{}", &tests_dir, filename);

        let temp_file = Path::new(&temp_file_path);

        match temp_file.extension() {
            Some(extension) => {
                let ext = extension.to_str().unwrap().to_string();
                if ext != "yaml" {
                    error!("Only yamls files are accepted. Received {}", &ext);
                    return Err(StatusCode::BAD_REQUEST);
                }
            }
            None => {
                error!("Missing extension in file {}", &filename);
                return Err(StatusCode::UNPROCESSABLE_ENTITY);
            }
        };

        if temp_file.exists() {
            match std::fs::remove_file(&temp_file_path) {
                Ok(_) => info!("Already existent file {} has been removed", &filename),
                Err(err) => error!("Couldn't remove file {}:\n{}", &filename, err.to_string()),
            }
        }

        match std::fs::File::create(temp_file) {
            Ok(mut file) => match file.write(&bytes) {
                Ok(size) => info!("Written {} bytes", size),
                Err(err) => {
                    error!("Failed to write test file: {}", err.to_string());
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            },
            Err(err) => {
                error!("Failed to create new test file: {}", err.to_string());
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    return Ok(Response::default());
}
