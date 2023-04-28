use std::process::Command;

use super::i_adapter::{get_adapter, DecodedDevice, Device, IAdapter, OsType};

pub fn get_devices() -> Vec<Box<dyn IAdapter>> {
    let mut command = Command::new("flutter");
    command.arg("devices").arg("--machine");
    let output = command.output().expect("Something went wrong");

    let bytes = output.stdout;

    let json_content = String::from_utf8(bytes).expect("The obtained string is not valid");

    let data: Vec<DecodedDevice> =
        serde_json::from_str::<Vec<DecodedDevice>>(&json_content).expect("Invalid json");

    let devices: Vec<Box<dyn IAdapter>> = data
        .iter()
        .map(|d| Device::from_parsed(d))
        // FIXME: right now the android adapter is the only one working
        .filter(|d| d.os_type == OsType::Android)
        .map(|d| get_adapter(d))
        .collect();

    return devices;
}
