use crate::device_adapter::adapter::get_devices;

mod device_adapter;
mod utils;

fn main() {
    let devices = get_devices();
    devices.iter().for_each(|d| {
        d.unlock_device();
        d.open_app(&String::from("it.clikapp.toduba"))
    });
}
