use std::thread;
use std::process::exit;
use signal_hook::{consts::SIGINT, consts::SIGTERM, consts::SIGKILL, iterator::Signals};

pub fn set_signal_tap() {
    let mut signals = Signals::new([SIGINT]).unwrap();

    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGINT => {
                    println!("");
                    exit(0);
                }
                SIGTERM => {
                    println!("");
                    return;
                },
                SIGKILL => {
                    println!("");
                    exit(0);
                },
                _  => exit(-1),
            }
        }
    });
}

