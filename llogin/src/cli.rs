//! Command-line interface module for the LPU WiFi Manager
//!
//! This module handles:
//! - Command line argument parsing
//! - User input prompts
//! - Help and version information
//! - Account management commands

use std::env;
use std::io::{self, Write};
use std::process;

/// Handles command line arguments and executes appropriate actions
///
/// # Arguments
/// * `args` - Vector of command line arguments
///
/// # Example
/// ```no_run
/// use std::env;
/// let args: Vec<String> = env::args().collect();
/// llogin::cli::handle_args(args);
/// ```
///
/// # Command Line Options
/// - `--help, -h`: Show help message
/// - `--version, -v`: Show version information
/// - `--list, -l`: List stored accounts
/// - `--account, -a <id>`: Login with specified account
/// - `--logout, -lg <id>`: Logout specified account
///
/// # Exit Codes
/// - 0: Success
/// - 1: Error (invalid arguments, network error, etc)
pub fn handle_args(args: Vec<String>) {
    if args.len() == 1 {
        println!("No account ID provided.");
        let account_id = prompt_for_account_id();
        crate::login::perform_lpu_login(&account_id);
    } else {
        match args[1].as_str() {
            "--help" | "-h" => show_help(),
            "--version" | "-v" => show_version(),
            "--account" | "-a" => {
                if args.len() != 3 {
                    println!("Error: Please provide an account ID.");
                    process::exit(1);
                } else {
                    crate::login::perform_lpu_login(&args[2]);
                }
            }
            "--list" | "-l" => list_account_ids(),
            "--logout" | "-lg" => {
                if args.len() != 3 {
                    println!("Error: Please provide an account ID.");
                    process::exit(1);
                } else {
                    crate::login::perform_lpu_logout(&args[2]);
                }
            }
            _ => {
                println!("Error: Unknown option. Use --help for usage information.");
                process::exit(1);
            }
        }
    }
}

/// Prompts user for an account identifier
///
/// # Returns
/// String containing the user-entered account ID
///
/// # Panics
/// Panics if:
/// - Unable to flush stdout
/// - Unable to read from stdin
fn prompt_for_account_id() -> String {
    print!("Enter the account ID or Name: ");
    io::stdout().flush().unwrap();
    let mut account_id = String::new();
    io::stdin().read_line(&mut account_id).unwrap();
    account_id.trim().to_string()
}

/// Displays program usage information
///
/// Shows:
/// - Basic usage syntax
/// - Available commands
/// - Command descriptions
fn show_help() {
    println!("Usage: {} [OPTION]", env::args().next().unwrap());
    println!("Manage and log in to multiple LPU WiFi accounts.");
    println!();
    println!("Options:");
    println!(" --help    , -h    Show this help message and exit.");
    println!(" --version , -v    Show version information and exit.");
    println!(" --list    , -l    List all stored account IDs.");
    println!(" --account , -a    Followed by the account ID you want to login as.");
    println!(" --logout  , -lg   Followed by the account ID you want to logout from.");
}

/// Displays program version information
fn show_version() {
    println!("LPU WiFi Manager 0.2.1");
}

/// Lists all stored account identifiers
///
/// Reads credentials from storage and displays:
/// - Account IDs in a list format
/// - Message if no accounts are stored
fn list_account_ids() {
    let credentials = crate::credentials::read_credentials();

    if credentials.is_empty() {
        println!("No stored accounts found.");
    } else {
        println!("Stored account IDs:");
        for account_id in credentials.keys() {
            println!("- {}", account_id);
        }
    }
}
