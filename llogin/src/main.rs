//! LPU WiFi Manager: A CLI tool for managing Lovely Professional University WiFi connections.
//!
//! This crate provides functionality to:
//! - Store multiple LPU WiFi credentials securely
//! - Login to LPU WiFi network
//! - Logout from LPU WiFi network
//! - List stored accounts
//! - Cross-platform support for Windows and Linux
//!
//! # Usage
//! ```bash
//! llogin --help    # Show help
//! llogin --account <myaccount>  # Login with stored account
//! llogin --logout <myaccount>   # Logout from account
//! llogin --list    # List stored accounts
//! ```
mod cli;
mod credentials;
mod login;
mod wifi;

use std::env;
use std::process;

fn main() {
    if !wifi::check_lpu_wifi() {
        println!("Not connected to LPU WiFi. Exiting.");
        process::exit(1);
    }

    let args: Vec<String> = env::args().collect();
    cli::handle_args(args);
}
