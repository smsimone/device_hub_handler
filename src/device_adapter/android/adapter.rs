use crate::{
    device_adapter::i_adapter::{Device, DeviceStatus, IAdapter, ScreenRequest},
    utils::{
        apks_helper,
        command_executor::{self, exec},
        env_helper::ENV_DATA,
    },
};
use std::{fs::remove_file, str::FromStr};

use log::{error, info, warn};
use regex::Regex;
use strum_macros::EnumString;

pub struct AdbAdapter {
    pub device: Device,
}

#[derive(Debug, EnumString)]
enum Abi {
    #[strum(serialize = "armeabi-v7a")]
    ArmeabiV7a,
    #[strum(serialize = "arm64-v8a")]
    Arm64V8a,
    #[strum(serialize = "x86")]
    X86,
    #[strum(serialize = "x86_64")]
    X86_64,
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
        let result = exec(&format!(
            "adb -s {} shell dumpsys {} | grep {}",
            self.device.id, key, value_key
        ));

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
                error!(
                    "[{}] Failed to get current screen on: {}",
                    self.device.name, err
                );
                return default_val;
            }
        };
    }

    /// Checks wether the app is already installed or not
    fn is_app_already_installed(&self, package_name: &String) -> Result<bool, String> {
        return command_executor::exec(&format!(
            "adb -s {} shell pm list packages {}",
            self.device.id, package_name
        ))
        .map(|res| !res.is_empty())
        .map_err(|err| {
            format!(
                "[{}] Failed to check if app {} is installed: {}",
                self.device.name,
                package_name,
                err.to_string()
            )
        });
    }

    /// Gets the architecture for the device connected
    fn get_device_architecture(&self) -> Result<Abi, String> {
        let output = command_executor::exec(&format!(
            "adb -s {} shell getprop ro.product.cpu.abi",
            self.device.id
        ))
        .map(|val| val.trim().to_owned())
        .map_err(|err| err.to_string())?;

        return Abi::from_str(&output).map_err(|err| {
            format!(
                "[{}] Invalid abi {}: {}",
                self.device.name,
                &output,
                err.to_string()
            )
        });
    }

    /// Extracts the apk for the current device's architecture given the aab file
    pub fn extract_apk(&self, aab_path: &String) -> Result<String, String> {
        let arch = self.get_device_architecture()?;
        info!("[{}] Device has arch {:?}", &self.device.name, &arch);

        let output_path = format!(
            "{}/{}.apks",
            ENV_DATA.lock().unwrap().extract_output_dir,
            &self.device.id
        );

        info!(
            "[{}] Extracting apks into {}",
            self.device.name, &output_path
        );

        let config = &ENV_DATA.lock().unwrap().android_config;

        return command_executor::exec(&format!(
            "bundletool build-apks --bundle={} --output={} --connected-device --device-id {} --ks={} --ks-key-alias={} --key-pass=pass:{} --ks-pass=pass:{}",
            &aab_path, &output_path, self.device.id,config.keystore_path,  config.keystore_alias, config.keystore_pass, config.keystore_pass
        ))
            .map(|_| {
                info!("[{}] Extracted apks in {}", self.device.name, &output_path);
                String::from(&output_path)
            })
            .map_err(|err| {
                format!(
                    "[{}] failed to extract apk: {}",
                    self.device.name,
                    err.to_string()
                )
            });
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
            (_, DeviceStatus::Unknown) => warn!(
                "[{}] Unkown device status, cannot do anything",
                self.device.name
            ),
            (_, _) => info!("[{}] Screen is already set up", self.device.name),
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
        let command = exec(&format!(
            "adb -s {} shell am start -n {}/{}.MainActivity",
            self.device.id, app_name, app_name
        ));
        match command {
            Ok(_) => info!("[{}] App {} executed", self.device.name, app_name),
            Err(err) => error!("[{}] Failed to open app: {}", self.device.name, err),
        }
    }

    fn send_keyevent(&self, key_event: &String) {
        let command = exec(&format!(
            "adb -s {} shell input keyevent {}",
            self.device.id, key_event
        ));
        match command {
            Ok(_) => info!("[{}] Sent keyevent {}", self.device.name, key_event),
            Err(err) => error!("[{}] Failed to wake device: {}", self.device.name, err),
        }
    }

    fn install_bundle(&self, bundle_path: &String) -> Result<String, String> {
        let extracted_apks_path = self.extract_apk(bundle_path)?;

        info!(
            "[{}] Extracted apk at {}",
            self.device.name, &extracted_apks_path
        );

        let package_name = match apks_helper::extract_package_name(&extracted_apks_path) {
            Ok(package) => package,
            Err(err) => {
                error!(
                    "[{}] Failed to extract package: {}",
                    self.device.name,
                    err.to_string()
                );
                return Err(err.to_string());
            }
        };

        self.unlock_device();
        if self.is_app_already_installed(&package_name.to_string())? {
            info!(
                "[{}] App {} is already installed. Uninstalling old version..",
                self.device.name, &package_name
            );
            let result = command_executor::exec(&format!(
                "adb -s {} uninstall {}",
                self.device.id, package_name
            ))
            .map(|_| info!("[{}] Previous app uninstalled", self.device.name))
            .map_err(|err| {
                format!(
                    "[{}] Could not uninstall app: {}",
                    self.device.name,
                    err.to_string()
                )
            });
            if result.is_err() {
                return Err(result.unwrap_err());
            }
        }

        info!("[{}] Installing app", self.device.name);
        return command_executor::exec(&format!(
            "bundletool install-apks --apks={} --device-id {}",
            extracted_apks_path, self.device.id,
        ))
        .map(|_| {
            info!("[{}] Installed apk", self.device.name);
            match remove_file(&extracted_apks_path) {
                Ok(_) => info!(
                    "[{}] Removed file {}",
                    self.device.name, &extracted_apks_path
                ),
                Err(err) => error!(
                    "[{}] Failed to remove file {}: {}",
                    self.device.name,
                    &extracted_apks_path,
                    err.to_string()
                ),
            }
            package_name
        })
        .map_err(|err| format!("Failed to install apk: {}", err.to_string()));
    }

    fn get_device_name(&self) -> String {
        String::from(&self.device.name)
    }
}
