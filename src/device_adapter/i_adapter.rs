use serde::{Deserialize, Serialize};

use super::{adb_adapter::AdbAdapter, ios_adapter::IosAdapter};

pub trait IAdapter: Sized {
    fn wake_up_device(device: &Device);

    fn send_keyevent(device: &Device, key_event: &String);
}

pub fn get_adapter(os_type: OsType) -> Result<dyn IAdapter, ()> {
    match os_type {
        OsType::Android => return Ok(AdbAdapter {}),
        OsType::Ios => return Ok(IosAdapter {}),
        OsType::Invalid => todo!(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub id: String,
    pub os_type: OsType,
    pub emulator: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
