use log::{error, info};

use crate::{
    device_adapter::i_adapter::{Device, DeviceStatus, IAdapter, ScreenRequest},
    utils::command_executor::{self, exec},
};

pub struct IosAdapter {
    pub device: Device,
}

impl IAdapter for IosAdapter {
    fn toggle_screen(&self, request: &ScreenRequest) {}

    fn unlock_device(&self) {}

    fn open_app(&self, app_name: &String) {
        match command_executor::exec(&format!("idb launch {}", &app_name)) {
            Ok(_) => info!("[{}] Launched app {}", self.device.name, &app_name),
            Err(err) => error!(
                "[{}] Failed to launch app {}\n{}",
                self.device.name, &app_name, err
            ),
        }
    }

    fn send_keyevent(&self, key_event: &String) {
        let command = exec(&format!(
            "adb -s {} shell input keyevent {}",
            self.device.id, key_event
        ));

        match command {
            Ok(_) => info!("Woke up device {}", self.device.name),
            Err(err) => error!("Failed to wake ios device: {}", err),
        }
    }

    fn get_device_status(&self) -> DeviceStatus {
        todo!()
    }

    fn install_bundle(&self, bundle_path: &String) -> Result<String, String> {
        todo!()
    }
}
