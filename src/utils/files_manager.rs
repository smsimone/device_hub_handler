use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

use tracing::{error, info};

use super::multipart_helper::ExtractedFile;

pub fn save_file(file: &ExtractedFile, directory: &String) -> Result<(), String> {
    let temp_file_path = format!("{}/{}", directory, file.name);
    let temp_file = Path::new(&temp_file_path);

    if temp_file.exists() {
        match std::fs::remove_file(&temp_file_path) {
            Ok(_) => info!("Removed old file {}", file.name),
            Err(err) => error!("Failed to remove file {}: {}", file.name, err.to_string()),
        }
    }

    let dir_path = Path::new(directory);
    if !dir_path.exists() {
        match create_dir_all(dir_path) {
            Ok(_) => info!("Created destination folder {}", &directory),
            Err(err) => error!(
                "Failed to create destination folder {}: {}",
                &directory, err
            ),
        }
    }

    match File::create(&temp_file_path)
        .map_err(|err| err.to_string())
        .map(|mut cf| cf.write_all(&file.bytes.to_vec()))
    {
        Ok(_) => {
            info!("Written test file {}", file.name);
            return Ok(());
        }
        Err(err) => {
            error!(
                "Failed to write test file {}: {}",
                file.name,
                err.to_string()
            );
            return Err(err);
        }
    }
}
