use crate::device_adapter::adapter::{get_devices, wake_up_devices};

mod device_adapter;
mod utils;

fn main() {
    let devices = get_devices();
    dbg!("Found devices: {}", &devices);

    wake_up_devices(&devices);
}
