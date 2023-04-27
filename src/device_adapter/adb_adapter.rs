use crate::utils::command_parser::execute_command;

use super::i_adapter::{Device, DeviceStatus, IAdapter, ScreenRequest};
use regex::Regex;

pub struct AdbAdapter {
    pub device: Device,
}

trait FromString: Sized {
    fn from_string(data: &str) -> Self;
}

impl FromString for DeviceStatus {
    fn from_string(data: &str) -> DeviceStatus {
        let temp = String::from(data);
        if temp == "Awake" {
            return DeviceStatus::Awake;
        } else if temp == "Dozing" {
            return DeviceStatus::Dozing;
        }
        return DeviceStatus::Unknown;
    }
}

impl FromString for bool {
    fn from_string(data: &str) -> Self {
        let temp = String::from(data);
        if temp == "true" {
            return true;
        }
        return false;
    }
}

impl AdbAdapter {
    fn dump_sys_value<T>(&self, key: &String, value_key: &String, default_val: T) -> T
    where
        T: FromString,
    {
        let result = execute_command(&String::from(format!(
            "adb -s {} shell dumpsys {} | grep {}",
            self.device.id, key, value_key
        )));

        match result {
            Ok(output) => {
                let regex_value = format!(r"{}=(\w+)", value_key);
                let re = Regex::new(&regex_value).unwrap();

                return re
                    .captures(&output)
                    .expect(&format!("Missing {}", value_key))
                    .get(1)
                    .map_or(default_val, |m| T::from_string(m.as_str()));
            }
            Err(err) => {
                println!(
                    "Failed to get current screen on for {}: {}",
                    self.device.name, err
                );
                return default_val;
            }
        };
    }
}

impl IAdapter for AdbAdapter {
    fn get_device_status(&self) -> DeviceStatus {
        self.dump_sys_value::<DeviceStatus>(
            &String::from("power"),
            &String::from("mWakefulness"),
            DeviceStatus::Unknown,
        )
    }

    fn toggle_screen(&self, request: &ScreenRequest) {
        let device_status = self.get_device_status();

        match (request, device_status) {
            (ScreenRequest::On, DeviceStatus::Dozing) => self.send_keyevent(&String::from("26")),
            (ScreenRequest::Off, DeviceStatus::Awake) => self.send_keyevent(&String::from("26")),
            (_, DeviceStatus::Unknown) => println!(
                "Unkown device status for {}, cannot do anything",
                self.device.name
            ),
            (_, _) => println!("Device {} is already ok", self.device.name),
        }
    }

    fn unlock_device(&self) {
        self.toggle_screen(&ScreenRequest::On);

        let is_on_lockscreen = self.dump_sys_value::<bool>(
            &String::from("window"),
            &String::from("mDreamingLockscreen"),
            false,
        );

        if is_on_lockscreen {
            self.send_keyevent(&String::from("82"));
        }
    }

    fn open_app(&self, app_name: &String) {
        let command = execute_command(&String::from(format!(
            "adb -s {} -n {}",
            self.device.id, app_name
        )));
        match command {
            Ok(_) => println!("App {} executed on {}", app_name, self.device.name),
            Err(err) => println!("Failed to open app on {}: {}", self.device.name, err),
        }
    }

    fn send_keyevent(&self, key_event: &String) {
        let command = execute_command(&String::from(format!(
            "adb -s {} shell input keyevent {}",
            self.device.id, key_event
        )));
        match command {
            Ok(_) => println!("Sent keyevent {} to {}", key_event, self.device.name),
            Err(err) => println!("Failed to wake ios device: {}", err),
        }
    }
}
