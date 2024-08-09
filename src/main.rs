use std::env;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn main() {
    println!("brsk shell: v0.0.1 (alpha)");

    loop {
        if let Err(err) = display_prompt() {
            eprint!("{}", err);
            continue;
        }

        let (command, args) = parse_input();
        let cwd = env::current_dir().unwrap_or_default();

        match command.as_str() {
            "cd" => change_directory(args, cwd),
            "exit" => exit(0),
            _ => execute_command(&command, args),
        }
    }
}

fn display_prompt() -> io::Result<()> {
    print!(">> ");
    io::stdout().flush()
}

fn parse_input() -> (String, Vec<String>) {
    let mut input = String::new();

    if let Err(err) = io::stdin().read_line(&mut input) {
        eprintln!("Error reading input: {}", err);
        return (String::new(), Vec::new());
    }

    let trimmed_input = input.trim();
    let mut tokens = trimmed_input.split_whitespace();

    let command = tokens.next().unwrap_or_default().to_string();
    let args: Vec<String> = tokens.map(String::from).collect();
    
    (command, args)
}

fn change_directory(args: Vec<String>, cwd: PathBuf) {
    // Get directory from args || default to home if none provided
    let target_path = args.get(0).map_or_else(
        || format!("/home/{}/", env::var("USER").unwrap_or_default()),
        |loc| {
            // Absolute path
            if loc.starts_with('/') {
                format!("{}{}", cwd.to_str().unwrap_or(""), loc)
            } else {
                // Relative path
                format!("{}/{}", cwd.to_str().unwrap_or(""), loc)
            }
        }
    );

    let path = Path::new(&target_path);

    if let Err(err) = env::set_current_dir(&path) {
        eprintln!("Error changing directory: {}", err);
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

