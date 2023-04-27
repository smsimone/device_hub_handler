use crate::utils::command_parser::parse_command;

use super::i_adapter::IAdapter;

pub struct IosAdapter {}

impl IAdapter for IosAdapter {
    fn wake_up_device(device: &super::i_adapter::Device) {
        todo!();
    }

    fn send_keyevent(device: &super::i_adapter::Device, key_event: &String) {
        let mut command = parse_command(&String::from(format!(
            "adb -s {} shell input keyevent {}",
            &device.id, key_event
        )));
        command.output().expect(&String::from(format!(
            "Non è stato possibile inviare il comando {} a {}",
            key_event, device.name
        )));
    }
}
