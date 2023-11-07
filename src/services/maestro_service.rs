use tracing::{error, info};

use crate::utils::{
    files_manager::save_file,
    multipart_helper::ExtractedFile,
};
use crate::utils::configs::CONFIGS;

pub fn store_file(file: ExtractedFile) -> Result<(), String> {
    if !(file.name.ends_with(".yaml") || file.name.ends_with(".yml")) {
        return Err(String::from("Invalid file type"));
    }

    let download_dir = &CONFIGS.lock().unwrap().tests_config.storation_folder;

    match save_file(&file, download_dir) {
        Ok(_) => info!("Stored file {}", file.name),
        Err(err) => {
            error!("Failed to write file {}: {}", file.name, err);
            return Err(err.to_string());
        }
    }

    Ok(())
}

pub fn execute_test(test_name: &String) {
    let tests_folder = &CONFIGS.lock().unwrap().tests_config.storation_folder;
    //TODO
}
