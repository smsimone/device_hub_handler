use std::{
    process::{Command, Stdio},
    sync::Arc,
};

use log::{error, info};

fn get_command_components(command: &String) -> Vec<String> {
    command
        .split(" ")
        .map(|s| String::from(s.trim()))
        .collect::<Vec<String>>()
}

fn create_command(command: &String) -> &mut Command {
    let components = get_command_components(command);

    let mut shell_command = &mut Command::new(&components[0]);
    for i in 1..components.len() {
        shell_command = (*shell_command).arg(&components[i]);
    }

    return Box::new(shell_command);
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
    info!("Checking existence of `{}`", command);

    return Command::new(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|output| {
            info!("Output:\n{}", String::from_utf8(output.stdout).unwrap());
            if output.status.success() {
                info!("`{}` has been found", command);
            } else {
                error!("`{}` was not found! Check your PATH!", command);
            }
            ()
        })
        .map_err(|err| {
            error!("Failed to spawn `{}`: {}", command, err.to_string());
            ()
        });
}
