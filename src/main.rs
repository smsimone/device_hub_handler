use clap::{Arg, Command};
use device_adapter::i_adapter::OsType;
use std::{
    fs::{create_dir, read_dir, remove_dir_all, DirEntry},
    io::Error,
    path::Path,
    process::exit,
    str::FromStr,
    thread::{self, JoinHandle},
};

use dialoguer::Confirm;

use log::{error, info, warn};
use utils::{
    command_executor::command_exists,
    commands::{find_devices, install_bundle},
    env_helper::ENV_DATA,
};

mod device_adapter;
mod utils;

fn main() -> Result<(), Error> {
    env_logger::init();

    validate_depdencies();
    match check_extraction_path() {
        Ok(_) => {}
        Err(err) => {
            error!("Could not check extraction path: {}", err.to_string());
            exit(1);
        }
    }

    let matches = Command::new("dhh")
        .author("smsimone")
        .version("0.0.1")
        .subcommand(
            Command::new("find_devices")
                .about("Find all connected devices")
                .arg(
                    Arg::new("filter")
                        .short('f')
                        .long("filter")
                        .help("Select the type of device you want to find")
                        .required(false)
                        .value_parser(["android", "ios"]),
                ),
        )
        .subcommand(
            Command::new("install_bundle")
                .about("Install the given bundle to the devices")
                .arg(
                    Arg::new("aab_path")
                        .short('a')
                        .long("aab_path")
                        .help("Path to the aab file that must be installed")
                        .required(false),
                )
                .arg(
                    Arg::new("ipa_path")
                        .short('i')
                        .long("ipa_path")
                        .help("Path to the ipa file that must be installed")
                        .required(false),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("find_devices", args)) => {
            let os_type = match args.get_one::<String>("filter") {
                Some(filter) => OsType::from_str(&filter)
                    .map(|f| if f == OsType::Invalid { None } else { Some(f) })
                    .unwrap_or(None),
                None => None,
            };
            for d in find_devices(os_type) {
                info!("{} - {}", d.get_device_name(), d.get_os_type());
            }
        }
        Some(("install_bundle", args)) => {
            let mut handles = Vec::<JoinHandle<()>>::new();
            let aab_path = args.get_one::<String>("aab_path").map(|s| String::from(s));

            if aab_path.is_some() {
                let path = aab_path.unwrap();
                if !Path::new(&path).exists() {
                    error!("The path given to the aab file does not point to anything");
                    exit(1);
                }

                let android_devices = find_devices(Some(OsType::Android));

                for device in android_devices.into_iter() {
                    let temp_path = String::from(&path);
                    let handle = thread::spawn(move || match install_bundle(&device, &temp_path) {
                        Ok(_) => info!("installed and ran app"),
                        Err(err) => {
                            error!("Failed to install and run app: {}", err);
                        }
                    });
                    handles.push(handle);
                }
            }

            let ipa_path = args.get_one::<String>("ipa_path").map(|s| String::from(s));
            if ipa_path.is_some() {
                let path = ipa_path.unwrap();
                if !Path::new(&path).exists() {
                    error!("The path given to the ipa file does not point to anything");
                    exit(1);
                }

                let ios_devices = find_devices(Some(OsType::Ios));

                for device in ios_devices.into_iter() {
                    let temp_path = String::from(&path);
                    let handle = thread::spawn(move || match install_bundle(&device, &temp_path) {
                        Ok(_) => info!("installed and ran app"),
                        Err(err) => {
                            error!("Failed to install and run app: {}", err);
                        }
                    });
                    handles.push(handle);
                }
            }

            for handle in handles {
                handle.join().unwrap();
            }
        }
        _ => {}
    }

    Ok(())
}

fn check_extraction_path() -> Result<(), Error> {
    let extract_path = String::from(&ENV_DATA.lock().unwrap().extract_output_dir);

    let dir_path = Path::new(&extract_path);
    if !dir_path.exists() {
        match create_dir(dir_path) {
            Ok(_) => {
                info!("Created directory {}", &extract_path);
                Ok(())
            }
            Err(err) => {
                error!("Failed to create directory: {}", err);
                Err(err)
            }
        }
    } else {
        if !(Path::new(&extract_path).is_dir()) {
            error!("The path {} is not a directory", &extract_path);
            exit(1);
        }

        let content = match read_dir(&extract_path) {
            Ok(val) => val,
            Err(_) => panic!("Could not read the directory"),
        };
        let content = content.collect::<Vec<Result<DirEntry, Error>>>();
        if !content.is_empty() {
            let should_erase = Confirm::new()
                .with_prompt(format!(
                    "The directory {} is not empty, do you want to delete its content?",
                    &extract_path
                ))
                .interact()?;

            if should_erase {
                match remove_dir_all(&extract_path) {
                    Ok(_) => {
                        info!("Directory {} erased", &extract_path);
                        return Ok(());
                    }
                    Err(err) => {
                        error!(
                            "Failed to erase directory {}: {}",
                            &extract_path,
                            err.to_string()
                        );
                        exit(1);
                    }
                }
            } else {
                warn!("Cannot continue if {} is not empty", &extract_path);
                exit(1);
            }
        }
        return Ok(());
    }
}

/// Checks whether all the required binaries are installed and present in PATH
fn validate_depdencies() {
    // FIXME: should remove this dependency to use just adb and idb
    // Used to find all connected devices
    if command_exists(&"flutter".to_string()).is_err() {
        exit(1);
    }

    // Used to interact with android devices
    if command_exists(&"adb".to_string()).is_err() {
        exit(1);
    }

    // Used to manage android appbundles
    if command_exists(&"bundletool".to_string()).is_err() {
        exit(1);
    }

    // Used to interact with ios devices just like adb
    if command_exists(&"idb".to_string()).is_err() {
        exit(1);
    }

    // Used to extract packagename from an apk
    if command_exists(&"aapt2".to_string()).is_err() {
        exit(1);
    }

    if command_exists(&"tar".to_string()).is_err() {
        exit(1);
    }
}
