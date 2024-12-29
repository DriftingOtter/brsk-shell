use std::io::{stdout, Write};
use std::process::{exit, Command};
use std::env;
use std::time::Duration;
use crossterm::{
    cursor::{MoveTo, position},
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

mod setup;
mod utils;
mod inbuilts;

// Function to reset the cursor to the start of the terminal
fn reset_cursor_position() {
    let (prompt_x, prompt_y) = position().unwrap();
    execute!(stdout(), MoveTo(0, prompt_y)).unwrap();
    stdout().flush().unwrap();
}

fn main() {
    // Enable terminal raw mode
    enable_raw_mode().expect("Could not initialize terminal in raw mode.");
    let mut stdout = stdout();

    // Initialize shell
    println!("brsk shell: v0.0.1 (alpha)");

    // Find/create log file (.brsk_history)
    let log_path = setup::init_history().unwrap();

    // Find/create rc file (.brskrc)
    let _rc_path = setup::init_resource_config().unwrap();

    // Reset the cursor to the first column at initialization
    reset_cursor_position();

    loop {
        // Display entry prompt
        if let Err(err) = utils::display_prompt() {
            eprint!("{}", err);
            continue;
        }

        let mut cursor_pos = 0;
        let mut input = String::new();

        // Get the current cursor position after the prompt
        let (prompt_x, prompt_y) = position().unwrap();

        // Inline editing loop
        loop {
            // Non-blocking poll for user input
            if poll(Duration::from_millis(50)).unwrap() {
                if let Event::Key(KeyEvent { code, modifiers, .. }) = read().unwrap() {
                    match code {
                        // Handle Ctrl+C (SIGINT)
                        KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => {
                            reset_cursor_position();
                            disable_raw_mode().unwrap();
                            reset_cursor_position();
                            exit(0);
                        }
                        KeyCode::Char(c) => {
                            input.insert(cursor_pos, c);
                            cursor_pos += 1;
                        }
                        KeyCode::Left => {
                            if cursor_pos > 0 {
                                cursor_pos -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if cursor_pos < input.len() {
                                cursor_pos += 1;
                            }
                        }
                        KeyCode::Backspace => {
                            if cursor_pos > 0 {
                                cursor_pos -= 1;
                                input.remove(cursor_pos);
                            }
                        }
                        KeyCode::Delete => {
                            if cursor_pos < input.len() {
                                input.remove(cursor_pos);
                            }
                        }
                        KeyCode::Enter => {
                            break;
                        }
                        _ => {}
                    }
                }
            }

            // Clear only the input portion and redraw
            execute!(
                stdout,
                MoveTo(prompt_x + 2, prompt_y), // Start input display after the `$` prompt
                Clear(ClearType::UntilNewLine)  // Clear the line from the cursor position onward
            )
            .unwrap();

            // Redraw the input and move the cursor
            print!("{}", input);
            execute!(stdout, MoveTo(prompt_x + 2 + cursor_pos as u16, prompt_y)).unwrap();
            stdout.flush().unwrap();
        }

        // After entering the command, move the cursor to the next line
        execute!(stdout, crossterm::cursor::MoveTo(0, prompt_y + 1)).unwrap();
        stdout.flush().unwrap();

        // Parse user input into a vector of command tuples (command, args, input_redirection)
        let command_queue = utils::parse_input(input.clone());

        // Run execution on all vectored commands
        for (command, args, _input_redirect) in command_queue {
            let mut return_code = 0;

            // Handle built-in commands
            if command == "clear" {
                // Let the terminal handle the clear command, do nothing internally
                let _ = Command::new("clear")
                    .status()
                    .expect("Failed to execute clear command");
                continue; // Skip further handling for the clear command
            }

            match command.as_str() {
                "cd" => {
                    let cwd = env::current_dir().unwrap_or_default();
                    inbuilts::change_directory(args, cwd);
                }
                "exit" => {
                    disable_raw_mode().unwrap();
                    exit(0);
                }
                _ => {
                    // Execute the command
                    let output = if let Some((cmd, input)) = utils::is_input_redirect(&input) {
                        utils::execute_command(&cmd, vec![], Some(input))
                    } else {
                        utils::execute_command(&command, args, None)
                    };

                    // Print the command output
                    if let Some(output_str) = output {
                        println!("{}", output_str);
                    } else {
                        eprintln!("Error executing command");
                        return_code = -1;
                    }
                }
            }

            // Save executed command to log/history
            if utils::create_log(&log_path, &input, return_code).is_none() {
                eprintln!("Could not save executed command to log file");
            }
        }

        if input != "clear" {
            // After the command execution finishes, move the cursor to the beginning of the next line
            execute!(stdout, MoveTo(0, prompt_y + 2)).unwrap(); // Move to the beginning of the next line
            stdout.flush().unwrap();
        }
    }
}

