use std::env;
use std::ffi::OsStr;
use std::io;
use std::io::stdout;
use std::io::Write;
use std::process::exit;
use std::process::Command;
use std::process::Stdio;
use std::path::Path;

fn main() {
    loop {

        // Print command prompt
        print!(">> ");
        if let Err(err) = stdout().flush() {
            eprint!("{}", err);
        }

        // Parse user input
        let (command, args) = parse_input();

        // Execute built-in commands
        if command == "cd" {
            cd(args);

        } else if command == "exit" {
            exit(0);
        } else {
            // Execute external commands
            execute_command(&command, args);
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

fn execute_command(command: &str, args: Vec<String>) {
    let args_slice: Vec<&OsStr> = args.iter().map(AsRef::as_ref).collect();

    match Command::new(command).args(args_slice).spawn() {
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

    if check_pipes(input.clone().as_str()) {
        let mut commands = input.trim().split(" | ").peekable();

        while let Some(command) = commands.next() {
            let mut parts = command.trim().split_whitespace();
            let command   = parts.next().unwrap();
            let args      = parts;

            let mut prev_command = None;

            let stdin = prev_command
                .map_or(Stdio::inherit(), |output: std::process::Child| Stdio::from(output.stdout.unwrap()));

            let stdout = if commands.peek().is_some() {
                Stdio::piped()
            } else {
                Stdio::inherit()
            };

            let output = Command::new(command)
                .args(args)
                .stdin(stdin)
                .stdout(stdout)
                .spawn();

            match output {
                        Ok(output) => { prev_command = Some(output); },
                        Err(e) => {
                            prev_command = None;
                            eprintln!("{}", e);
                        },
                    };

            if let Some(mut final_command) = prev_command {
                match final_command.wait() {
                    Ok(_) => {},
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
    }

    // Parse input into command name and arguments
    let mut input_tokens = input.trim().split_whitespace();

    let command = input_tokens.next().unwrap_or_default().to_string();
    let args: Vec<String> = input_tokens.map(|s| s.to_string()).collect();

    (command, args)
}

fn check_pipes(input: &str) -> bool {
    let trimmed = input.trim();
    let segments: Vec<&str> = trimmed.split(" | ").collect();

    if segments.len() >= 2 {
        return true;
    } else {
        return false;
    }
}
