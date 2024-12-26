use std::env;
use std::fs;

pub fn init_history() -> Option<String> {
    // Find/create log file (.brsk_history)
    let log_path = match find_history_log() {
        Some(path) => path,
        None => match create_log_file() {
            Some(path) => path,
            None => {
                eprintln!("Could not create log file");
                return None;
            },
        },
    };

    return Some(log_path);
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

pub fn init_resource_config() -> Option<String> {
    // Find/create rc file (.brskrc)
    let rc_path = match find_rc_file() {
        Some(path) => path,
        None => match create_rc_file() {
            Some(path) => path,
            None => {
                eprintln!("Could not create .brskrc file");
                return None;
            },
        },
    };

    return Some(rc_path);
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
