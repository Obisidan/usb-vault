//! USB Vault — encrypt USB drives with a password.
//!
//! Run without arguments to launch the TUI.
//! Run with --encrypt / --decrypt / --wipe for CLI mode.

mod crypto;
mod tui;
mod usb;
mod warning;

use std::env;
use std::path::Path;

fn print_usage() {
    println!("USB Vault — pure Rust USB encryption suite");
    println!();
    println!("Usage:");
    println!("  usb-vault                         Launch interactive TUI");
    println!("  usb-vault encrypt -d <device>     Encrypt a USB drive");
    println!("  usb-vault decrypt -d <device>     Decrypt a USB drive");
    println!("  usb-vault wipe -d <device>        Wipe vault header");
    println!();
    println!("Options:");
    println!("  -d, --device <path>   USB device path (e.g., /dev/sdX)");
    println!("  -p, --password <pw>   Password (interactive if not provided)");
    println!();
    println!("Examples:");
    println!("  sudo usb-vault encrypt -d /dev/sdb");
    println!("  sudo usb-vault decrypt -d /dev/sdb -p mypassword");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Launch TUI
        if let Err(e) = tui::launch() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    match args[1].as_str() {
        "encrypt" => {
            let (device, password) = parse_args(&args);
            if device.is_empty() {
                eprintln!("Error: --device is required");
                print_usage();
                std::process::exit(1);
            }
            let password = match password {
                Some(p) => p,
                None => rpassword::read_password().expect("Failed to read password"),
            };
            let path = Path::new(&device);
            println!("Encrypting {}...", device);
            println!("WARNING: This will encrypt the ENTIRE drive. Without the password, data is LOST FOREVER.");
            match usb::encrypt_device(path, &password) {
                Ok(()) => {
                    println!("Done! Drive encrypted successfully.");
                    println!("IMPORTANT: Write your password down on paper and store it safely!");
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "decrypt" => {
            let (device, password) = parse_args(&args);
            if device.is_empty() {
                eprintln!("Error: --device is required");
                print_usage();
                std::process::exit(1);
            }
            let password = match password {
                Some(p) => p,
                None => rpassword::read_password().expect("Failed to read password"),
            };
            let path = Path::new(&device);
            println!("Decrypting {}...", device);
            match usb::decrypt_device(path, &password) {
                Ok(()) => {
                    println!("Done! Drive decrypted successfully.");
                    println!("Note: The vault header is still present. Use 'wipe' to remove it.");
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "wipe" => {
            let (device, _) = parse_args(&args);
            if device.is_empty() {
                eprintln!("Error: --device is required");
                print_usage();
                std::process::exit(1);
            }
            println!("Wiping vault header from {}...", device);
            println!("This will destroy the encrypted data permanently.");
            let path = Path::new(&device);
            match usb::wipe_device(path) {
                Ok(()) => println!("Done! Vault header removed. Drive can be reformatted."),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "help" | "-h" | "--help" => {
            print_usage();
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn parse_args(args: &[String]) -> (String, Option<String>) {
    let mut device = String::new();
    let mut password: Option<String> = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--device" => {
                if i + 1 < args.len() {
                    device = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-p" | "--password" => {
                if i + 1 < args.len() {
                    password = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    (device, password)
}
