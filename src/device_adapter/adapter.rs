use std::process::Command;

use super::{
    adb_adapter::AdbAdapter,
    i_adapter::{DecodedDevice, Device, IAdapter, OsType},
    ios_adapter::IosAdapter,
};

pub fn get_devices() -> Vec<Device> {
    let mut command = Command::new("flutter");
    command.arg("devices").arg("--machine");
    let output = command.output().expect("Something went wrong");

    let bytes = output.stdout;

    let json_content = String::from_utf8(bytes).expect("The obtained string is not valid");

    let data: Vec<DecodedDevice> =
        serde_json::from_str::<Vec<DecodedDevice>>(&json_content).expect("Invalid json");

    let devices: Vec<Device> = data
        .iter()
        .map(|d| Device::from_parsed(d))
        .filter(|d| d.os_type != OsType::Invalid)
        .collect();

    return devices;
}

/// Sveglia tutti i device passati
pub fn wake_up_devices(devices: &Vec<Device>) {
    devices.iter().for_each(|device| match device.os_type {
        OsType::Android => AdbAdapter::wake_up_device(&device),
        OsType::Ios => IosAdapter::wake_up_device(&device),
        OsType::Invalid => todo!(),
    });
}
