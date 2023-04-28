use std::process::Command;

/// Executes the command given and returns its output
pub fn exec(command: &String) -> Result<String, String> {
    let data = command
        .split(" ")
        .map(|s| String::from(s))
        .collect::<Vec<String>>();

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
