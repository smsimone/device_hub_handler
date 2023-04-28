use crate::utils::command_executor::exec;

use super::i_adapter::{Device, IAdapter, ScreenRequest};

pub struct IosAdapter {
    pub device: Device,
}

impl IAdapter for IosAdapter {
    fn toggle_screen(&self, request: &ScreenRequest) {}

    fn unlock_device(&self) {}

    fn open_app(&self, app_name: &String) {}

    fn send_keyevent(&self, key_event: &String) {
        let command = exec(&format!(
            "adb -s {} shell input keyevent {}",
            self.device.id, key_event
        ));

        match command {
            Ok(_) => println!("Woke up device {}", self.device.name),
            Err(err) => println!("Failed to wake ios device: {}", err),
        }
    }

    fn get_device_status(&self) -> super::i_adapter::DeviceStatus {
        todo!()
    }

    fn install_bundle(&self, bundle_path: &String) -> Result<String, String> {
        todo!()
    }
}
