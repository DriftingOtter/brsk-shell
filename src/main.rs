use std::env;
use std::thread;
use std::fs::{self, OpenOptions};
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::time::{SystemTime, UNIX_EPOCH};
use signal_hook::{consts::SIGINT, iterator::Signals};

fn main() {
    // capture SIGINT && Other Unix Signals
    set_signal_tap();

    // init text
    println!("brsk shell: v0.0.1 (alpha)");

    // Creating log file for command history
    let mut log_path: String = String::new();
    let mut rc_path:  String = String::new();

    // Find/create log file (.brsk_history)
    log_path = match find_history_log() {
        Some(path) => path,
        None => match create_log_file() {
            Some(path) => path,
            None => {
                eprintln!("Could not create log file");
                return;
            },
        },
    };

    // Find/create rc file (.brskrc)
    rc_path = match find_rc_file() {
        Some(path) => path,
        None => match create_rc_file() {
            Some(path) => path,
            None => {
                eprintln!("Could not create .brskrc file");
                return;
            },
        },
    };

    loop {
        // Display entry prompt
        if let Err(err) = display_prompt() {
            eprint!("{}", err);
            continue;
        }

        // Read user input
        let mut input = String::new();
        if let Err(err) = io::stdin().read_line(&mut input) {
            eprintln!("{}", err);
            continue;
        }

        // Parse user input into a vector of command tuples
        let command_queue = parse_input(input.clone());

        let mut return_code: isize = 0;

        // Run execution on all vectored commands
        for (command, args, input) in command_queue {
            match command.as_str() {
                "cd" => {
                    // Save current working directory for preserved memory of location
                    let cwd = env::current_dir().unwrap_or_default();
                    change_directory(args, cwd);
                }
                "exit" => exit(0),
                _ => return_code = execute_command(&command, args).unwrap(),
            }

            // Save executed command to log/history
            if create_log(&log_path, &input, return_code).is_none() {
                eprintln!("Could not save executed command to log file");
            }
        }
    }
}

fn display_prompt() -> io::Result<()> {
    print!(">> ");
    io::stdout().flush()
}

fn parse_input(input: String) -> Vec<(String, Vec<String>, String)> {
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

fn execute_command(command: &str, args: Vec<String>) -> Option<isize> {
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

fn find_history_log() -> Option<String> {
    let default_log_path: String = format!("/home/{}/.brsk_history", 
        env::var("USER").unwrap().trim());

    match fs::metadata(default_log_path.clone()) {
        Ok(_)  => return Some(default_log_path),
        Err(_) => return None,
    }
}

fn create_log_file() -> Option<String> {
    let default_log_path: String = format!("/home/{}/.brsk_history", 
        env::var("USER").unwrap().trim());

    match fs::write(default_log_path.clone(), "") {
        Ok(_)  => return Some(default_log_path),
        Err(_) => return None,
    }
}

fn create_log(log_path: &str, command: &str, return_code: isize) -> Option<()> {
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

fn find_rc_file() -> Option<String> {
    let default_log_path: String = format!("/home/{}/.brskrc", 
        env::var("USER").unwrap().trim());
    match fs::metadata(default_log_path.clone()) {
        Ok(_)  => return Some(default_log_path),
        Err(_) => return None,
    }
}

fn create_rc_file() -> Option<String> {
    let default_log_path: String = format!("/home/{}/.brskrc", 
        env::var("USER").unwrap().trim());

    match fs::write(default_log_path.clone(), "") {
        Ok(_)  => return Some(default_log_path),
        Err(_) => return None,
    }
}

fn set_signal_tap() {
    let mut signals = Signals::new([SIGINT]).unwrap();

    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGINT => {
                    println!("");
                    exit(0);
                }
                15 => {
                    println!("");
                    return;
                },
                9  => {
                    println!("");
                    exit(0);
                },
                _  => exit(-1),
            }
        }
    });
}

