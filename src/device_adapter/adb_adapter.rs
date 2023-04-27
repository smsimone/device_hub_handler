use std::process::Command;

use super::i_adapter::IAdapter;

pub struct AdbAdapter {}

impl IAdapter for AdbAdapter {
    fn wake_up_device(device: &super::i_adapter::Device) {
        AdbAdapter::send_keyevent(device, &String::from("KEYCODE_WAKEUP"));
    }

    fn send_keyevent(device: &super::i_adapter::Device, key_event: &String) {
        let mut command = Command::new("adb");
        command
            .arg("-s")
            .arg(&device.id)
            .arg("shell")
            .arg("input")
            .arg("keyevent")
            .arg(key_event);
        command.output().expect(&String::from(format!(
            "Non è stato possibile inviare il comando {} a {}",
            key_event, device.name
        )));
    }
}
