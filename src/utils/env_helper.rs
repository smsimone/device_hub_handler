use std::{path::Path, sync::Mutex};

use dotenv::dotenv;
use once_cell::sync::Lazy;

pub static ENV_DATA: Lazy<Mutex<EnvData>> = Lazy::new(|| Mutex::new(EnvData::load().unwrap()));

pub struct EnvData {
    pub android_config: AndroidConfig,
    pub extract_output_dir: String,
    pub download_default_dir: String,
}

pub struct AndroidConfig {
    pub keystore_path: String,
    pub keystore_alias: String,
    pub keystore_pass: String,
}

impl EnvData {
    pub fn load() -> Result<EnvData, String> {
        if dotenv().ok().is_none() {
            panic!("Failed to load .env file");
        }

        let android_keystore_path =
            dotenv::var("ANDROID_KEYSTORE_PATH").map_err(|err| err.to_string())?;
        let android_keystore_alias =
            dotenv::var("ANDROID_KEYSTORE_KEY_ALIAS").map_err(|err| err.to_string())?;
        let android_keystore_pass =
            dotenv::var("ANDROID_KEYSTORE_KEY_PASS").map_err(|err| err.to_string())?;
        let extract_output_dir =
            dotenv::var("EXTRACT_DEFAULT_DIR").map_err(|err| err.to_string())?;
        let download_default_dir =
            dotenv::var("DOWNLOAD_DEFAULT_DIR").map_err(|err| err.to_string())?;

        if !Path::new(&extract_output_dir).is_absolute() {
            panic!("EXTRACT_DEFAULT_DIR must be absolute");
        }

        if !Path::new(&download_default_dir).is_absolute() {
            panic!("DOWNLOAD_DEFAULT_DIR must be absolute");
        }

        Ok(EnvData {
            extract_output_dir,
            download_default_dir,
            android_config: AndroidConfig {
                keystore_path: android_keystore_path,
                keystore_alias: android_keystore_alias,
                keystore_pass: android_keystore_pass,
            },
        })
    }
}
