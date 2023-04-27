use std::{io::Error, process::Command};

pub fn execute_command(command: &String) -> Result<String, Error> {
    let data = command
        .split(" ")
        .map(|s| String::from(s))
        .collect::<Vec<String>>();

    let mut command = &mut Command::new(&data[0]);
    for i in 1..data.len() {
        command = (*command).arg(&data[i]);
    }

    return command
        .output()
        .map(|d| String::from_utf8(d.stdout).expect("Invalid string"));
}
