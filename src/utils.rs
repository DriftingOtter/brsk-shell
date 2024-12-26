use std::fs;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::fs::OpenOptions;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn display_prompt() -> io::Result<()> {
    print!("$ ");
    io::stdout().flush()
}

pub fn parse_input(input: String) -> Vec<(String, Vec<String>, Option<String>)> {
    if input.is_empty() {
        return Vec::new();
    }

    let trimmed_input = input.trim();

    let mut command_list: Vec<(String, Vec<String>, Option<String>)> = Vec::new();

    // Split commands by "&&"
    let segments: Vec<&str> = trimmed_input.split("&&").map(|s| s.trim()).collect();

    for segment in segments {
        let mut tokens = segment.split_whitespace();
        let command = tokens.next().unwrap_or_default().to_string();
        let args: Vec<String> = tokens.map(String::from).collect();

        // Check if there's input redirection (<) in the command
        let mut input_redirect: Option<String> = None;
        if let Some(input_part) = command.split('<').nth(1) {
            input_redirect = Some(input_part.trim().to_string());
        }

        // Add the command, arguments, and input redirect to the list
        command_list.push((command, args, input_redirect));
    }

    return command_list;
}

pub fn execute_command(command: &str, args: Vec<String>, input_redirect: Option<String>) -> Option<isize> {
    // Create the base command
    let mut cmd = Command::new(command);
    
    // Only add arguments if there are any
    if !args.is_empty() {
        let args_slice: Vec<&OsStr> = args.iter().map(AsRef::as_ref).collect();
        cmd.args(args_slice);
    }

    // If input is provided (input redirect), handle stdin redirection
    if let Some(input) = input_redirect {
        match cmd.stdin(Stdio::piped()).spawn() {
            Ok(mut cmd_process) => {
                // Write the input to the stdin pipe of the command
                if let Some(mut stdin_handle) = cmd_process.stdin.take() {
                    if let Err(err) = stdin_handle.write_all(input.as_bytes()) {
                        eprintln!("Error writing to stdin: {}", err);
                        return Some(-1);
                    }
                }
                // Wait for the command to complete
                match cmd_process.wait() {
                    Ok(status) => status.code().map(|code| code as isize),
                    Err(err) => {
                        eprintln!("Error waiting for command: {}", err);
                        return Some(-1);
                    }
                }
            },
            Err(err) => {
                eprintln!("Error spawning command with input redirect: {}", err);
                return Some(-1);
            }
        }
    } else {
        // If no input redirect, just execute normally
        match cmd.spawn() {
            Ok(mut cmd_process) => match cmd_process.wait() {
                Ok(status) => status.code().map(|code| code as isize),
                Err(err) => {
                    eprintln!("Error waiting for command: {}", err);
                    return Some(-1);
                }
            },
            Err(err) => {
                eprintln!("Error starting command: {}", err);
                return Some(-1);
            }
        }
    }
}

pub fn is_input_redirect(command: &str) -> Option<(String, String)> {
    let mut parts: Vec<&str> = command.trim().split('<').collect();

    // Trim each part in parts
    for part in parts.iter_mut() {
        *part = part.trim();
    }

    // Check if there are exactly two parts
    if parts.len() != 2 {
        return None;
    }

    let output = parts[0].to_string(); // Command and its args
    let input_file = parts[1].to_string().trim().to_string();  // Input file path

    // Read the contents of the file specified by input
    match fs::read_to_string(input_file) {
        Ok(content) => Some((output, content)),
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            return None;
        }
    }
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

