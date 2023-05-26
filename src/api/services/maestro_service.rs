use std::{fs::read_dir, io::Read, path::Path};

use axum::{body, http::StatusCode, response::Response};
use log::{error, info};

use crate::{
    api::controllers::device_handlers::BundleName,
    device_adapter::i_adapter::{IAdapter, OsType, ScreenRequest},
    utils::{command_executor, env_helper::ENV_DATA},
};

use super::device_service;

/// Executes the test on the specified device
pub async fn run_test(
    test_name: &String,
    device_id: &String,
    bundle_name: &BundleName,
) -> Result<Response, StatusCode> {
    if !get_available_tests().iter().any(|f| f == test_name) {
        error!("Test {} not found", test_name);
        return Err(StatusCode::NOT_FOUND);
    }
    info!("Test file {} has been found", test_name);

    let opt_device = device_service::find_devices(None)
        .into_iter()
        .find(|d| d.get_device_id() == String::from(device_id));

    if opt_device.is_none() {
        error!("Device {} not found", device_id);
        return Err(StatusCode::NOT_FOUND);
    }

    let device = opt_device.unwrap();

    let bundle = match device.get_os_type() {
        OsType::Android => String::from(&bundle_name.android_bundle),
        OsType::Ios => String::from(&bundle_name.ios_bundle),
        OsType::Invalid => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    info!(
        "Device {} has been found. Using bundle {}",
        device_id, &bundle
    );

    let tests_folder = &ENV_DATA.lock().unwrap().maestro_tests_dir;
    let test_file = format!("{}/{}", &tests_folder.trim(), test_name);

    device.toggle_screen(&ScreenRequest::On);

    let test_result = command_executor::exec(&format!(
        "maestro --device {} test -e APP_TO_LAUNCH={} {}",
        device_id, bundle, test_file
    ));

    match test_result {
        Ok(val) => {
            info!("Executed test with success: {}", val);
            return Ok(Response::builder().body(body::boxed(val)).unwrap());
        }
        Err(err) => {
            error!("Failed to execute test: {}", err);
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    }
}

pub fn get_test_content(test_name: &String) -> Result<String, StatusCode> {
    let tests_dir = &ENV_DATA.lock().unwrap().maestro_tests_dir;
    let file_path = format!("{}/{}", tests_dir, test_name);

    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    match std::fs::File::open(&file_path) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect("Failed to read file");
            return Ok(content);
        }
        Err(error) => {
            error!("Failed to open file: {}", error.to_string());
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

pub fn get_available_tests() -> Vec<String> {
    let tests_dir = &ENV_DATA.lock().unwrap().maestro_tests_dir;
    match read_dir(&tests_dir) {
        Ok(e) => e
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap())
            .map(|e| e.file_name().to_str().unwrap().to_string())
            .collect::<Vec<String>>(),
        Err(err) => {
            error!("Failed to read tests directory: {}", err.to_string());
            Vec::new()
        }
    }
}
