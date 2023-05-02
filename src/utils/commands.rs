use std::process::Command;

use log::error;

use crate::device_adapter::i_adapter::{get_adapter, DecodedDevice, Device, IAdapter, OsType};

/// Find all devices with the same os defined in filter. If filter is [None], all device types will
/// be returned
pub fn find_devices(filter: Option<OsType>) -> Vec<Box<dyn IAdapter>> {
    let bytes = Command::new("flutter")
        .arg("devices")
        .arg("--machine")
        .output()
        .expect("Something went wrong")
        .stdout;

    let json_content = String::from_utf8(bytes).expect("The obtained string is not valid");

    let data: Vec<DecodedDevice> =
        serde_json::from_str::<Vec<DecodedDevice>>(&json_content).expect("Invalid json");

    let devices: Vec<Box<dyn IAdapter>> = data
        .iter()
        .map(|d| Device::from_parsed(d))
        .filter(|d| match filter {
            None => true,
            Some(os) => d.os_type == os,
        })
        .map(|d| get_adapter(d))
        .collect();

    return devices;
}

pub fn install_bundle(adapter: &Box<dyn IAdapter>, bundle_path: &String) -> Result<(), String> {
    return adapter
        .install_bundle(bundle_path)
        .map(|package_name| {
            adapter.open_app(&package_name);
            ()
        })
        .map_err(|err| {
            error!("Failed: {}", err);
            err
        });
}
