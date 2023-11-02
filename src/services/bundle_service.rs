use std::path::Path;

use tracing::{error, info};

use crate::utils::{env_helper::ENV_DATA, multipart_helper::ExtractedFile};

pub fn extract_bundle(file: &ExtractedFile) -> Result<(), String> {
    let download_dir = &ENV_DATA.lock().unwrap().download_default_dir;

    if let Err(err) = ensure_directory_exists(download_dir) {
        return Err(err);
    }

    let temp_file_path = format!("{}/{}", &download_dir, file.name);

    let temp_file = Path::new(&temp_file_path);
    //TODO: extract file in temporary file

    Ok(())
}

fn ensure_directory_exists(dir_path: &String) -> Result<(), String> {
    if !Path::new(&dir_path).exists() {
        match std::fs::create_dir_all(&dir_path) {
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
