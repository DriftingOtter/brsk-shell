use std::env;
use std::path::{Path, PathBuf};

pub fn change_directory(args: Vec<String>, cwd: PathBuf) {
    // Get directory from args || default to home if none provided
    let target_path = args.get(0).map_or_else(
        || format!("/home/{}/", env::var("USER").unwrap_or_default()),
        |location| {
            // Absolute path
            if location.starts_with('/') {
                format!("{}{}", cwd.to_str().unwrap_or(""), location)
            } else {
                // Relative path
                format!("{}/{}", cwd.to_str().unwrap_or(""), location)
            }
        }
    );

    let path = Path::new(&target_path);

    if let Err(err) = env::set_current_dir(&path) {
        eprintln!("Error changing directory: {}", err);
    }
}

