use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum::Display;
use strum_macros::EnumString;

use super::{android::adapter::AdbAdapter, ios::adapter::IosAdapter};

pub enum ScreenRequest {
    On,
    Off,
}

pub trait IAdapter: Sync + Send {
    fn get_os_type(&self) -> OsType;

    fn get_device_name(&self) -> String;

    fn toggle_screen(&self, request: &ScreenRequest);

    fn unlock_device(&self);

    fn get_device_id(&self) -> String;

    fn open_app(&self, app_name: &String);

    fn send_keyevent(&self, key_event: &String);

    fn get_device_status(&self) -> DeviceStatus;

    /// In case of [Ok] returns the name of the bundle installed
    fn install_bundle(&self, bundle_path: &String) -> Result<String, String>;
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

#[derive(Debug)]
pub enum DeviceStatus {
    Dozing,
    Awake,
    Unknown,
}

impl Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceStatus::Dozing => f.write_str("Dozing"),
            DeviceStatus::Awake => f.write_str("Awake"),
            DeviceStatus::Unknown => f.write_str("Unknown"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy, EnumString, Display)]
pub enum OsType {
    #[strum(serialize = "android")]
    Android,
    #[strum(serialize = "ios")]
    Ios,
    #[strum(serialize = "none")]
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
