use serde::{Deserialize, Serialize};
use std::process::{Command, ExitStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    name: String,
    id: String,
    os_type: OsType,
    emulator: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OsType {
    Android,
    Ios,
}

pub fn get_devices() -> Vec<Device> {
    let mut command = Command::new("flutter");
    command.arg("devices").arg("--machine");
    let output = command.output().expect("Something went wrong");

    let bytes = output.stdout;

    let json_content = String::from_utf8(bytes).expect("The obtained string is not valid");

    serde_json::from_str::<Device>(&json_content);

    dbg!("Got output: {}", output);
    return Vec::<Device>::new();
}
