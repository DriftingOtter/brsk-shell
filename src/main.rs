use std::env;
use std::ffi::OsStr;
use std::io;
use std::io::stdout;
use std::io::Write;
use std::process::Command;
use std::path::Path;

fn main() {
    loop {

        // Print command prompt
        print!(">> ");
        if let Err(err) = stdout().flush() {
            eprint!("{}", err);
        }

        // Parse user input
        let (command_name, args) = parse_input();

        // Execute built-in commands
        if command_name == "cd" {
            cd(args);
        } else {
            // Execute external commands
            execute_command(&command_name, args);
        }
    }
}

fn cd(args: Vec<String>) -> bool {
    let default_directory = "/".to_string();

    let directory = args.get(0).unwrap_or(&default_directory);
    let root = Path::new(directory);

    match env::set_current_dir(&root) {
        Ok(_) => true,
        Err(err) => {
            eprintln!("Error changing directory: {}", err);
            false
        }
    }
}

fn execute_command(command_name: &str, args: Vec<String>) {
    let args_slice: Vec<&OsStr> = args.iter().map(AsRef::as_ref).collect();

    match Command::new(command_name).args(args_slice).spawn() {
        Ok(mut cmd) => {
            if let Err(err) = cmd.wait() {
                eprintln!("Error executing command: {}", err);
            }
        }
        Err(err) => eprintln!("Error starting command: {}", err),
    }
}

fn parse_input() -> (String, Vec<String>) {
    let mut input = String::new();
    if let Err(err) = io::stdin().read_line(&mut input) {
        eprintln!("Error reading input: {}", err);
    }

    // Parse input into command name and arguments
    let mut sanitized_input = input.trim().split_whitespace();

    let command_name = sanitized_input.next().unwrap_or_default().to_string();
    let args: Vec<String> = sanitized_input.map(|s| s.to_string()).collect();

    (command_name, args)
}
