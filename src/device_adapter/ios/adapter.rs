use log::{error, info};

use crate::{
    device_adapter::i_adapter::{Device, DeviceStatus, IAdapter, ScreenRequest},
    utils::command_executor,
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
}

impl IAdapter for IosAdapter {
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
        if bundle_path.ends_with(".ipa") {
            return Err("Still to implement".to_string());
        } else {
            if !bundle_path.ends_with(".app") {
                error!("Invalid bundle for ios device: {}", &bundle_path);
                return Err(format!("Invalid bundle path: {}", &bundle_path));
            }

            let bundle_name = "it.clikapp.toduba.Toduba-Pastopay".to_string();
            if self.is_app_installed(&bundle_name).unwrap_or(false) {
                _ = self.uninstall_app(&bundle_name);
            }

            match command_executor::exec(&format!(
                "idb install --udid {} {}",
                self.device.id, &bundle_path
            )) {
                Ok(_) => {
                    info!("Installed bundle on ios device");
                    // TODO: get the installed package name from output
                    return Ok(bundle_name);
                }
                Err(err) => {
                    error!("Failed to install bundle on ios device: {}", err);
                    return Err(format!("Failed to install bundle on ios device: {}", err));
                }
            }
        }
    }

    fn get_device_name(&self) -> String {
        String::from(&self.device.name)
    }

    fn get_os_type(&self) -> crate::device_adapter::i_adapter::OsType {
        self.device.os_type
    }
}
