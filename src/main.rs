use std::{io::Error, path::Path};

use dotenv::dotenv;

use crate::device_adapter::adapter::get_devices;

mod device_adapter;
mod utils;

fn main() -> Result<(), Error> {
    let devices = get_devices();

    let aab_path = String::from(
        "/Users/smaso/Development/toduba/app/build/app/outputs/bundle/release/app-release.aab",
    );

    if !Path::new(&aab_path).exists() {
        panic!("The path pointing to the aab file is wrong");
    }

    match dotenv().ok() {
        None => panic!("Missing .env file"),
        Some(_) => {}
    }

    devices
        .iter()
        .for_each(|adapter| match adapter.install_bundle(&aab_path) {
            Ok(package_name) => adapter.open_app(&package_name),
            Err(err) => println!("Failed: {}", err),
        });

    // devices.iter().for_each(|d| {
    //     d.unlock_device();
    //     d.open_app(&String::from("it.clikapp.toduba"))
    // });
    Ok(())
}
