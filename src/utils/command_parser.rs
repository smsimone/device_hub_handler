use std::process::Command;

pub fn parse_command(command: &String) -> Command {
    let data = command
        .split(" ")
        .map(|s| String::from(s))
        .collect::<Vec<String>>();

    let mut command = &Command::new(&data[0]);
    for i in 1..data.len() {
        command = (*command).arg(String::from(data[i]));
    }

    return *command;
}
