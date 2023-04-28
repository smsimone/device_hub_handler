use std::{
    fs::{read_dir, remove_dir_all},
    path::Path,
};

use log::{error, info};

use super::{command_executor, env_helper::ENV_DATA};

/// Extracts the apks file in the path given and returns the app package name
pub fn extract_package_name(apks_path: &String) -> Result<String, String> {
    info!("Extracting {}", apks_path);
    let extraction_directory = extract_apks(apks_path)?;

    let path = Path::new(&extraction_directory).join("splits");
    let total_path = path.into_os_string().into_string().unwrap();

    let dir_content = read_dir(&total_path).map_err(|e| e.to_string())?;

    let apk_file_path = dir_content
        .filter(|e| match e {
            Ok(entry) => {
                let p = entry.path().into_os_string().into_string().unwrap();
                return p.ends_with(".apk");
            }
            Err(_) => false,
        })
        .map(|e| e.unwrap().path().into_os_string().into_string().unwrap())
        .collect::<Vec<String>>()
        .first()
        .unwrap()
        .to_owned();

    return command_executor::exec(&format!("aapt2 dump packagename {}", &apk_file_path))
        .map(|res| {
            match remove_dir_all(&extraction_directory) {
                Ok(_) => info!("Removed directory {}", &extraction_directory),
                Err(err) => error!(
                    "Failed to remove directory {}: {}",
                    &extraction_directory, err
                ),
            }
            res.trim().to_owned()
        })
        .map_err(|err| err.to_string());
}

fn extract_apks(apks_path: &String) -> Result<String, String> {
    let path = Path::new(apks_path);
    if !path.exists() {
        return Err("Apks file does not exsits".to_string());
    }

    let extraction_directory =
        path.file_name().map_or("temp_file".to_string(), |name| {
            match name.to_os_string().into_string() {
                Ok(name) => {
                    let cleared_name = name.replace(".apk", "");
                    format!(
                        "{}/{}",
                        ENV_DATA.lock().unwrap().extract_output_dir,
                        cleared_name
                    )
                }
                Err(_) => "temp_file".to_string(),
            }
        });

    command_executor::exec(&format!("mkdir {}", extraction_directory))
        .expect("Could not create directory");

    command_executor::exec(&format!(
        "tar -xvf {} -C {}",
        apks_path, extraction_directory
    ))
    .map(|_| {
        info!("Extracted apks in path {}", extraction_directory);
        extraction_directory
    })
    .map_err(|err| {
        error!("Failed to extract apks: {}", err.to_string());
        err.to_string()
    })
}
