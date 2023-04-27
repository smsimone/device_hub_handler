use serde::{Deserialize, Serialize};

use super::{adb_adapter::AdbAdapter, ios_adapter::IosAdapter};

pub enum ScreenRequest {
    On,
    Off,
}

pub trait IAdapter {
    fn toggle_screen(&self, request: &ScreenRequest);

    fn unlock_device(&self);

    fn open_app(&self, app_name: &String);

    fn send_keyevent(&self, key_event: &String);
}

pub fn get_adapter(device: Device) -> Box<dyn IAdapter> {
    match device.os_type {
        OsType::Android => return Box::new(AdbAdapter { device }),
        OsType::Ios => return Box::new(IosAdapter { device }),
        OsType::Invalid => todo!(),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub name: String,
    pub id: String,
    pub os_type: OsType,
    pub emulator: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum OsType {
    Android,
    Ios,
    Invalid,
}

impl Device {
    pub fn from_parsed(device: &DecodedDevice) -> Device {
        return Device {
            name: String::from(&device.name),
            id: String::from(&device.id),
            os_type: OsType::from_sdk(&device.sdk),
            emulator: device.emulator,
        };
    }
}

impl OsType {
    fn from_sdk(sdk: &String) -> OsType {
        match sdk.contains("Android") {
            true => OsType::Android,
            false => {
                if sdk.contains("iOS") {
                    return OsType::Ios;
                }
                return OsType::Invalid;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecodedDevice {
    name: String,
    id: String,
    sdk: String,
    emulator: bool,
}
