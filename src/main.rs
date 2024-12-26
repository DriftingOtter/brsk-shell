use std::io;
use std::env;
use std::process::exit;

mod setup;
mod utils;
mod inbuilts;
mod signal_handler;

fn main() {
    // capture SIGINT && Other Unix Signals
    signal_handler::set_signal_tap();

    // init text
    println!("brsk shell: v0.0.1 (alpha)");

    // Find/create log file (.brsk_history)
    let log_path = setup::init_history().unwrap();

    // Find/create rc file (.brskrc)
    let _rc_path = setup::init_resource_config().unwrap();

    loop {
        // Display entry prompt
        if let Err(err) = utils::display_prompt() {
            eprint!("{}", err);
            continue;
        }

        // Read user input
        let mut input = String::new();
        if let Err(err) = io::stdin().read_line(&mut input) { 
            eprintln!("{}", err);
            continue;
        }

        // Parse user input into a vector of command tuples (command, args, input_redirection)
        let command_queue = utils::parse_input(input.clone());

        // Run execution on all vectored commands
        for (command, args, input_redirect) in command_queue {
            let mut return_code = 0;
            
            // Handle built-in commands
            match command.as_str() {
                "cd" => {
                    let cwd = env::current_dir().unwrap_or_default(); // Get the current working directory
                    inbuilts::change_directory(args, cwd);
                }
                "exit" => {
                    exit(0);
                }
                _ => {
                    println!("command: {}, args: {:?}, input: {:?}", command, args, input_redirect);
                    
                    // Check for input redirection
                    if let Some((cmd, input)) = utils::is_input_redirect(&input) {
                        // Execute command with input redirection
                        match utils::execute_command(&cmd, args, Some(input)) {
                            Some(code) => return_code = code,
                            None => {
                                eprintln!("Error executing command with input redirection");
                                return_code = -1;
                            }
                        }
                    } else {
                        // Execute command without input redirection
                        match utils::execute_command(&command, args, None) {
                            Some(code) => return_code = code,
                            None => {
                                eprintln!("Error executing command");
                                return_code = -1;
                            }
                        }
                    }
                }
            }

            // Save executed command to log/history
            if utils::create_log(&log_path, &input, return_code).is_none() {
                eprintln!("Could not save executed command to log file");
            }
        }
    }
}

