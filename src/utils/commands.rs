use std::{
    net::Ipv4Addr,
    path::Path,
    process::Command,
    thread::{self, JoinHandle},
};

use adb_client::AdbTcpConnection;
use log::{error, info};

use crate::device_adapter::{
    android,
    i_adapter::{DecodedDevice, Device, get_adapter, IAdapter, OsType},
};

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

pub fn get_android_devices() -> Result<Vec<Box<dyn IAdapter>>, String> {
    if let Ok(mut conn) = AdbTcpConnection::new(Ipv4Addr::from([127, 0, 0, 1]), 5037)
        .map_err(|err| error!("Failed to connect to adb server: {}", err))
    {
        match conn.devices() {
            Ok(devices) => {
                info!("Found {} devices with adbconnection", devices.len());
                let mut data: Vec<Box<dyn IAdapter>> = vec![];
                for element in devices {
                    data.push(Box::new(android::adapter::AdbAdapter {
                        device: Device {
                            name: String::from("device"),
                            id: element.identifier,
                            os_type: OsType::Android,
                            emulator: false,
                        },
                    }));
                }
                return Ok(data);
            }
            Err(err) => {
                error!("Failed to look for connected devices: {}", err);
                return Err(format!("Failed to look for connected devices: {}", err));
            }
        }
    }

    return Err(String::from("Failed to connect to adb"));
}

fn install_bundle(adapter: &dyn IAdapter, bundle_path: &String) -> Result<(), String> {
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
            .map_or("no_name".to_string(), |s| s.to_str().unwrap().to_string());
        error!("Missing extension on given file {}", &filename);
        return Err(format!("Missing extension on given file {}", &filename));
    }
    let ext = extension.unwrap().to_str().unwrap();
    if !vec!["aab", "ipa", "app", "apk"].contains(&ext) {
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
    info!("Found {} devices", devices.len());

    let mut handles = Vec::<JoinHandle<()>>::new();

    for device in devices.into_iter() {
        let temp_path = String::from(bundle_path);
        let handle = thread::spawn(move || {
            info!(
                "Installing against {} -> {}",
                device.get_device_name(),
                device.get_os_type().to_string()
            );
            match install_bundle(device.as_ref(), &temp_path) {
                Ok(_) => info!("installed and ran app"),
                Err(err) => {
                    error!("Failed to install and run app: {}", err);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    return Ok(());
}
