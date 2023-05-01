use std::{
    fs::{create_dir, read_dir, remove_dir_all, DirEntry},
    io::Error,
    path::Path,
    process::exit,
};

use dialoguer::Confirm;

use dotenv::dotenv;
use log::{error, info, warn};
use utils::{command_executor::command_exists, env_helper::ENV_DATA};

use crate::device_adapter::adapter::get_devices;

mod device_adapter;
mod utils;

fn main() -> Result<(), Error> {
    env_logger::init();

    validate_depdencies();

    let aab_path = String::from(
        "/Users/smaso/Development/toduba/app/build/app/outputs/bundle/release/app-release.aab",
    );

    if !Path::new(&aab_path).exists() {
        error!("The path given to the aab file does not point to anything");
        exit(1);
    }

    let extract_path = String::from(&ENV_DATA.lock().unwrap().extract_output_dir);

    let dir_path = Path::new(&extract_path);
    if !dir_path.exists() {
        match create_dir(dir_path) {
            Ok(_) => info!("Created directory {}", &extract_path),
            Err(err) => error!("Failed to create directory: {}", err),
        }
    } else {
        if !(Path::new(&extract_path).is_dir()) {
            error!("The path {} is not a directory", &extract_path);
            exit(1);
        }

        let content = match read_dir(&extract_path) {
            Ok(val) => val,
            Err(_) => panic!("Could not read the directory"),
        };
        if content.collect::<Vec<Result<DirEntry, Error>>>().len() != 0 {
            let should_erase = Confirm::new()
                .with_prompt(format!(
                    "The directory {} is not empty, do you want to delete its content?",
                    &extract_path
                ))
                .interact()?;

            if should_erase {
                match remove_dir_all(&extract_path) {
                    Ok(_) => info!("Directory {} erased", &extract_path),
                    Err(err) => {
                        error!(
                            "Failed to erase directory {}: {}",
                            &extract_path,
                            err.to_string()
                        );
                        exit(1);
                    }
                }
            } else {
                warn!("Cannot continue if {} is not empty", &extract_path);
                exit(1);
            }
        }
    }

    match dotenv().ok() {
        None => panic!("Missing .env file"),
        Some(_) => {}
    }

    get_devices()
        .iter()
        .for_each(|adapter| match adapter.install_bundle(&aab_path) {
            Ok(package_name) => adapter.open_app(&package_name),
            Err(err) => error!("Failed: {}", err),
        });

    Ok(())
}

fn validate_depdencies() {
    // FIXME: should remove this dependency to use just adb and idb
    // Used to find all connected devices
    if command_exists(&"flutter".to_string()).is_err() {
        exit(1);
    }

    // Used to interact with android devices
    if command_exists(&"adb".to_string()).is_err() {
        exit(1);
    }

    // Used to manage android appbundles
    if command_exists(&"bundletool".to_string()).is_err() {
        exit(1);
    }

    // Used to interact with ios devices just like adb
    if command_exists(&"idb".to_string()).is_err() {
        exit(1);
    }

    // Used to extract packagename from an apk
    if command_exists(&"aapt2".to_string()).is_err() {
        exit(1);
    }

    if command_exists(&"tar".to_string()).is_err() {
        exit(1);
    }
}
