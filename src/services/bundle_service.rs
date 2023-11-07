use std::{
    fs::create_dir_all,
    io::Cursor,
    path::{Path, PathBuf},
};

use tracing::{error, info};

use crate::utils::multipart_helper::ExtractedFile;
use crate::utils::configs::CONFIGS;

/// Extracts the given file and if it's all ok returns the path to the folder extracted
pub fn extract_bundle(file: &ExtractedFile) -> Result<PathBuf, String> {
    let download_dir = &CONFIGS.lock().unwrap().bundles_config.download_folder;

    if let Err(err) = ensure_directory_exists(download_dir) {
        return Err(err);
    }

    let temp_file_path = format!("{}/{}", &download_dir, file.name);

    let temp_file = Path::new(&temp_file_path);

    let extension = match temp_file.extension() {
        Some(extension) => {
            let ext = extension.to_str().unwrap().to_string();
            if ext != "zip" {
                return Err(format!("Received a {} instead of a zip file", &ext));
            }
            ext
        }
        None => {
            return Err(format!("Missing extension in file {}", &file.name));
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
            return Err(format!(
                "Failed to create directories to extraction folder: {}",
                err
            ));
        }
    }

    match zip_extract::extract(Cursor::new(&file.bytes), &extraction_folder, true) {
        Ok(_) => info!(
            "Extracted zip file in {}",
            &extraction_folder.to_str().unwrap()
        ),
        Err(err) => {
            error!("Failed to extract zip file:\n{}", err.to_string());
            return Err(format!("Failed to extract zip folder: {}", err));
        }
    }

    Ok(extraction_folder)
}

fn ensure_directory_exists(dir_path: &String) -> Result<(), String> {
    if !Path::new(&dir_path).exists() {
        match create_dir_all(&dir_path) {
            Ok(_) => info!("Created directory {}", &dir_path),
            Err(err) => {
                error!(
                    "Failed to create directory {}:\n{}",
                    &dir_path,
                    err.to_string()
                );
                return Err(format!("Failed to create directory {}", dir_path));
            }
        }
    }
    Ok(())
}
