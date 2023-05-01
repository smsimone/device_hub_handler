use std::process::{Command, Stdio};

use log::error;

fn get_command_components(command: &String) -> Vec<String> {
    command
        .split(" ")
        .map(|s| String::from(s.trim()))
        .collect::<Vec<String>>()
}

/// Executes the command given and returns its output
pub fn exec(command: &String) -> Result<String, String> {
    let data = get_command_components(command);

    let mut shell_command = &mut Command::new(&data[0]);
    for i in 1..data.len() {
        shell_command = (*shell_command).arg(&data[i]);
    }

    let result = shell_command.output();
    match result {
        Ok(d) => {
            if !d.status.success() {
                let err = String::from_utf8(d.stderr).expect("Invalid string");
                return Err(format!("Failed to execute command: {}\n{}", command, err));
            }
            let output = String::from_utf8(d.stdout).expect("Invalid string");
            return Ok(output);
        }
        Err(err) => Err(err.to_string()),
    }
}

pub fn command_exists(command: &String) -> Result<(), ()> {
    let components = get_command_components(command);

    if components.len() != 1 {
        error!("invalid command: {}", command);
        return Err(());
    }

    let command = &components[0];

    return Command::new("which")
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|output| {
            if !output.status.success() {
                error!("`{}` was not found! Check your PATH!", command);
            }
            ()
        })
        .map_err(|err| {
            error!("Failed to spawn `{}`: {}", command, err.to_string());
            ()
        });
}
