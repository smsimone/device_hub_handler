use std::{
    io::{BufReader, Cursor, Read},
    path::Path,
};

use glob::glob;
use log::{error, info};
use plist::Value;

use crate::{
    device_adapter::i_adapter::{Device, DeviceStatus, IAdapter, ScreenRequest},
    utils::{command_executor, env_helper::ENV_DATA},
};

pub struct IosAdapter {
    pub device: Device,
}

impl IosAdapter {
    fn is_app_installed(&self, package_name: &String) -> Result<bool, String> {
        info!(
            "[{}] Checking if {} is installed",
            self.device.name, package_name
        );
        match command_executor::exec(&format!(
            "idb list-apps --udid {} | grep {}",
            self.device.id, package_name
        )) {
            Ok(_) => Ok(true),
            Err(err) => {
                if err.starts_with("Failed to") {
                    return Ok(false);
                }
                return Err(err);
            }
        }
    }

    fn uninstall_app(&self, package_name: &String) -> Result<(), String> {
        info!("[{}] Uninstalling app {}", self.device.name, package_name);
        match command_executor::exec(&format!(
            "idb uninstall --udid {} {}",
            self.device.id, package_name
        )) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }

    /// I hope I will have some time to refactor this because it's pretty shitty
    fn get_bundle_name(&self, bundle_path: &String) -> Result<String, String> {
        let bundle_file = Path::new(bundle_path);

        if !bundle_file.exists() {
            return Err("The given path does not exists".to_string());
        }

        let new_file = format!("{}.zip", bundle_path);

        let result = match command_executor::exec(&format!("cp {} {}", bundle_path, &new_file)) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        };

        if result.is_err() {
            return Err(result.unwrap_err());
        }

        let extraction_path = format!(
            "{}/{}",
            ENV_DATA.lock().unwrap().extract_output_dir,
            self.device.id
        );
        info!("Extracting application into {}", extraction_path);

        let file = std::fs::File::open(&new_file).map_err(|err| err.to_string())?;
        let mut reader = BufReader::new(&file);
        let mut buffer = Vec::new();

        let dim = reader.read_to_end(&mut buffer);
        info!(
            "Read {} bytes",
            dim.map(|size| size.to_string())
                .unwrap_or("There was an error".to_string())
        );

        let extraction_folder = Path::new(&extraction_path);

        match zip_extract::extract(Cursor::new(&buffer), &extraction_folder, true) {
            Ok(_) => info!(
                "Extracted zip file in {}",
                &extraction_folder.to_str().unwrap()
            ),
            Err(err) => {
                error!("Failed to extract zip file:\n{}", err.to_string());
                return Err(err.to_string());
            }
        }

        let info_plists = glob(
            format!(
                "{}/**/Info.plist",
                &extraction_folder.as_os_str().to_str().unwrap()
            )
                .as_str(),
        )
            .expect("Failed to read glob");

        let item = info_plists.min_by_key(|f| f.as_ref().unwrap().display().to_string().len());

        let correct_plist = item.unwrap().unwrap().to_str().unwrap().to_string();

        info!("Minimum info.plist path: {}", &correct_plist);

        let info = Value::from_file(&correct_plist).expect("Must be a valid plist file");

        let bundle_name = info
            .as_dictionary()
            .and_then(|dict| dict.get("CFBundleIdentifier"))
            .and_then(|bundle| bundle.as_string())
            .expect("There should be the bundle identifier key");

        info!("Got bundle name: {}", &bundle_name);

        return Ok(bundle_name.to_string());
    }
}

impl IAdapter for IosAdapter {
    fn get_os_type(&self) -> crate::device_adapter::i_adapter::OsType {
        self.device.os_type
    }

    fn get_device_id(&self) -> String {
        String::from(&self.device.id)
    }

    fn get_device_name(&self) -> String {
        String::from(&self.device.name)
    }

    fn toggle_screen(&self, _request: &ScreenRequest) {}

    fn unlock_device(&self) {}

    fn open_app(&self, app_name: &String) {
        match command_executor::exec(&format!(
            "idb launch --udid {} {}",
            self.device.id, &app_name
        )) {
            Ok(_) => info!("[{}] Launched app {}", self.device.name, &app_name),
            Err(err) => error!(
                "[{}] Failed to launch app {}\n{}",
                self.device.name, &app_name, err
            ),
        }
    }

    fn send_keyevent(&self, _key_event: &String) {
        todo!()
    }

    fn get_device_status(&self) -> DeviceStatus {
        // TODO: fetch current device status
        DeviceStatus::Awake
    }

    fn install_bundle(&self, bundle_path: &String) -> Result<String, String> {
        if !bundle_path.ends_with(".app") && !bundle_path.ends_with(".ipa") {
            error!("Invalid bundle for ios device: {}", &bundle_path);
            return Err(format!("Invalid bundle path: {}", &bundle_path));
        }

        let bundle_name = self
            .get_bundle_name(&bundle_path)
            .expect("There should be a bundle name");

        if self.is_app_installed(&bundle_name).unwrap_or(false) {
            _ = self.uninstall_app(&bundle_name);
        }

        return match command_executor::exec(&format!(
            "idb install --udid {} {}",
            self.device.id, &bundle_path
        )) {
            Ok(_) => {
                info!("Installed bundle on ios device");
                Ok(bundle_name)
            }
            Err(err) => {
                error!("Failed to install bundle on ios device: {}", err);
                Err(format!("Failed to install bundle on ios device: {}", err))
            }
        };
    }
}
