use std::{
    path::Path,
    process::Command,
    thread::{self, JoinHandle},
};

use log::{error, info};

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
        .filter(|d| d.os_type != OsType::Invalid)
        .map(|d| get_adapter(d))
        .collect();

    return devices;
}

pub fn find_android_devices() {
    let bytes = Command::new("adb")
        .arg("devices")
        .arg("-l")
        .output()
        .expect("Something went wrong")
        .stdout;

    let output_value = String::from_utf8(bytes).expect("The obtained string is not valid");
    let lines = output_value
        .split("\n")
        .map(|item| String::from(item))
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.split(" ")
                .map(|l| l.to_string())
                .filter(|l| !l.is_empty())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>();

    for line in lines {
        for component in line {
            print!("{}, ", component);
        }
        println!("");
    }
}

fn install_bundle(adapter: &Box<dyn IAdapter>, bundle_path: &String) -> Result<(), String> {
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

/// Installs the given bundle_path against all the devices connected
///
/// The [OsType] is computed from the extension of the file given:
/// - aab: [OsType::Android]
/// - app/ipa: [OsType::Ios]
pub fn install_bundle_all(bundle_path: &String) -> Result<(), String> {
    let file = Path::new(bundle_path);
    if !file.exists() {
        return Err("The given path does not exists".to_string());
    }

    let extension = file.extension();
    if extension.is_none() {
        let filename = file
            .file_name()
            .map(|s| s.to_str().unwrap().to_string())
            .unwrap_or("no_name".to_string());
        error!("Missing extension on given file {}", &filename);
        return Err(format!("Missing extension on given file {}", &filename));
    }
    let ext = extension.unwrap().to_str().unwrap();
    if !vec!["aab", "ipa", "app"].contains(&ext) {
        error!("Invalid file extension: {}", &ext);
        return Err(format!("Invalid file extension: {}", &ext));
    }

    let os_device: Option<OsType>;

    if ext == "aab" {
        os_device = Some(OsType::Android);
    } else {
        os_device = Some(OsType::Ios);
    }

    let devices = find_devices(os_device);

    let mut handles = Vec::<JoinHandle<()>>::new();

    for device in devices.into_iter() {
        let temp_path = String::from(bundle_path);
        let handle = thread::spawn(move || match install_bundle(&device, &temp_path) {
            Ok(_) => info!("installed and ran app"),
            Err(err) => {
                error!("Failed to install and run app: {}", err);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    return Ok(());
}
