use tracing::{error, info};

use crate::utils::{
    env_helper::ENV_DATA, files_manager::save_file, multipart_helper::ExtractedFile,
};

pub fn store_file(file: ExtractedFile) -> Result<(), String> {
    if !(file.name.ends_with(".yaml") || file.name.ends_with(".yml")) {
        return Err(String::from("Invalid file type"));
    }

    let download_dir = &ENV_DATA.lock().unwrap().tests_folder;

    match save_file(&file, download_dir) {
        Ok(_) => info!("Stored file {}", file.name),
        Err(err) => {
            error!("Failed to write file {}: {}", file.name, err);
            return Err(err.to_string());
        }
    }

    Ok(())
}
