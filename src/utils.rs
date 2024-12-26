use std::fs;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::fs::OpenOptions;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn execute_command(command: &str, args: Vec<String>) -> Option<isize> {
    let args_slice: Vec<&OsStr> = args.iter().map(AsRef::as_ref).collect();

    match Command::new(command).args(args_slice).spawn() {
        Ok(mut cmd) => match cmd.wait() {
            Ok(status) => status.code().map(|code| code as isize),
            Err(err) => {
                eprintln!("Error waiting for command: {}", err);
                return Some(-1);
            }
        },
        Err(err) => {
            eprintln!("Error starting command: {}", err);
            return Some(-1);
        },
    }
}

pub fn display_prompt() -> io::Result<()> {
    print!("$ ");
    io::stdout().flush()
}

pub fn parse_input(input: String) -> Vec<(String, Vec<String>, String)> {
    if input.is_empty() {
        return Vec::new();
    }

    let trimmed_input = input.trim();

    let mut command_list: Vec<(String, Vec<String>, String)> = Vec::new();

    // Split commands by "&&"
    let segments: Vec<&str> = trimmed_input.split("&&").map(|s| s.trim()).collect();

    for segment in segments {
        let mut tokens = segment.split_whitespace();
        let command = tokens.next().unwrap_or_default().to_string();
        let args: Vec<String> = tokens.map(String::from).collect();

        // Use the original input as a placeholder for the third part
        command_list.push((command, args, input.clone()));
    }

    return command_list;
}

pub fn is_input_redirect(command: &str) -> Option<String> {
    if !(command.to_string().split("<").collect::<Vec<_>>()).is_empty() {
        let parts: Vec<&str> = command.split('<').collect();

        if parts.len() != 2 {
            return None;
        }

        let output = parts[0].trim().to_string();
        let input = parts[1].trim().to_string();

        let input_contents: Option<String> = match fs::read_to_string(input) {
            Ok(content) => Some(content),
            Err(err)    => {
                eprintln!("{}", err);
                None
            }
        };

        let parsed_command: String = format!("{}{}", output, input_contents.unwrap());
        return Some(parsed_command);
    } else {
        return None;
    }
}

pub fn is_output_redirect(command: &str) -> Option<isize> {
    let command = command.trim();

    // Determine the type of redirection and split the command
    let (cmd_part, file, append) = if command.contains(">>") {
        let parts: Vec<&str> = command.split(">>").collect();
        if parts.len() == 2 {
            (parts[0].trim(), parts[1].trim(), true)
        } else {
            eprintln!("Invalid command syntax");
            return None;
        }
    } else if command.contains('>') {
        let parts: Vec<&str> = command.split('>').collect();
        if parts.len() == 2 {
            (parts[0].trim(), parts[1].trim(), false)
        } else {
            eprintln!("Invalid command syntax");
            return None;
        }
    } else {
        return None;
    };

    // Parse the command part
    let command_queue = parse_input(cmd_part.to_string());

    for (command, args, _input) in command_queue {
        // Prepare command arguments
        let args_slice: Vec<&OsStr> = args.iter().map(AsRef::as_ref).collect();

        // Execute the command
        let output = match Command::new(&command).args(args_slice).output() {
            Ok(output) => output,
            Err(err) => {
                eprintln!("Error starting command: {}", err);
                return None;
            }
        };

        // Check command execution status
        if !output.status.success() {
            eprintln!("Command failed with status: {:?}", output.status);
            return Some(-1);
        }

        // Prepare file options based on the type of redirection
        let mut file_options = OpenOptions::new();
        file_options.create(true);

        if append {
            file_options.append(true);
        } else {
            file_options.write(true).truncate(true);
        }

        // Write output to the file
        if let Err(err) = file_options
            .open(file)
            .and_then(|mut file| file.write_all(&output.stdout))
        {
            eprintln!("Error writing to file: {}", err);
            return Some(-1);
        }

        return output.status.code().map(|code| code as isize);
    }

    return None;
}

pub fn create_log(log_path: &str, command: &str, return_code: isize) -> Option<()> {
    let unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // credit to zsh_history for this format
    let log = format!("{},{},{}", unix_time, return_code, command);

    match OpenOptions::new().append(true).open(log_path) {
        Ok(mut file) => {
            if let Err(_) = file.write_all(log.as_bytes()) {
                return None;
            } else {
                return Some(());
            }
        }
        Err(_) => return None,
    }
}

