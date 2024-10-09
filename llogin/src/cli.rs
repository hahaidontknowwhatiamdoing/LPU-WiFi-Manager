use std::env;
use std::io::{self, Write};
use std::process;

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
            _ => {
                println!("Error: Unknown option. Use --help for usage information.");
                process::exit(1);
            }
        }
    }
}

fn prompt_for_account_id() -> String {
    print!("Enter the account ID or Name: ");
    io::stdout().flush().unwrap();
    let mut account_id = String::new();
    io::stdin().read_line(&mut account_id).unwrap();
    account_id.trim().to_string()
}

fn show_help() {
    println!(
        "Usage: {} [OPTION]",
        env::args().next().unwrap()
    );
    println!("Manage and log in to multiple LPU WiFi accounts.");
    println!();
    println!("Options:");
    println!(" --help , -h       Show this help message and exit.");
    println!(" --version , -v    Show version information and exit.");
    println!(" --list , -l       List all stored account IDs.");
    println!(" --account , -a    Followed by the account ID you want to login as.");
}

fn show_version() {
    println!("LPU WiFi Manager 0.1.2");
}

fn list_account_ids() {
    let accounts: Vec<String> = env::vars()
        .filter_map(|(key, _)| {
            if key.starts_with("LPU_USERNAME_") {
                Some(key.strip_prefix("LPU_USERNAME_").unwrap().to_string())
            } else {
                None
            }
        })
        .collect();

    if accounts.is_empty() {
        println!("No stored accounts found.");
    } else {
        println!("Stored account IDs:");
        for account in accounts {
            println!("- {}", account);
        }
    }
}
