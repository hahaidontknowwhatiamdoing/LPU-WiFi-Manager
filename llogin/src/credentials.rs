// src/credentials.rs

use colored::*;
use dialoguer::{console::Term, theme::ColorfulTheme, Input, Password};
use dirs_next::config_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

/// Represents stored credentials for an LPU WiFi account
#[derive(Serialize, Deserialize)]
pub struct Credentials {
    /// LPU username without @lpu.com suffix
    pub username: String,
    /// Account password
    pub password: String,
}

/// Type alias for a HashMap storing account IDs mapped to their credentials
pub type CredentialsMap = HashMap<String, Credentials>;

/// Gets the path to the credentials file in the system config directory
fn get_credentials_file_path() -> PathBuf {
    let config_dir = config_dir().expect("Failed to get config directory");
    let llogin_dir = config_dir.join("llogin");
    if !llogin_dir.exists() {
        fs::create_dir_all(&llogin_dir).expect("Failed to create llogin directory");
    }
    llogin_dir.join("credentials.json")
}

/// Reads stored credentials from the credentials file
pub fn read_credentials() -> CredentialsMap {
    let path = get_credentials_file_path();
    if path.exists() {
        let file = File::open(path).expect("Failed to open credentials file");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Failed to read credentials")
    } else {
        HashMap::new()
    }
}

/// Writes credentials to the credentials file
fn write_credentials(credentials: &CredentialsMap) {
    let path = get_credentials_file_path();
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .expect("Failed to open credentials file");
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, credentials).expect("Failed to write credentials");
}

/// Checks if an account ID exists
pub fn account_exists(account_id: &str) -> bool {
    let credentials = read_credentials();
    credentials.contains_key(account_id)
}

/// Stores new LPU WiFi credentials using dialoguer
pub fn store_new_account_dialogue(default_account_id: &str) -> String {
    let mut credentials = read_credentials();
    let theme = ColorfulTheme::default();
    let term = Term::stdout();

    println!("{}", "\nCreating New Account".bright_blue().bold());

    // Prompt for a unique account ID
    let account_id: String = loop {
        let input: String = Input::with_theme(&theme)
            .with_prompt("Enter a unique identifier for this account")
            .with_initial_text(default_account_id)
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("Account ID cannot be empty.")
                } else if credentials.contains_key(input.trim()) {
                    Err("An account with this ID already exists. Please choose a different ID.")
                } else {
                    Ok(())
                }
            })
            .interact_text_on(&term)
            .unwrap();

        break input.trim().to_string();
    };

    // Prompt for username
    let username: String = Input::with_theme(&theme)
        .with_prompt("Enter your LPU username (without @lpu.com)")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Username cannot be empty.")
            } else {
                Ok(())
            }
        })
        .interact_text_on(&term)
        .unwrap();

    // Prompt for password with confirmation
    let password: String = Password::with_theme(&theme)
        .with_prompt("Enter your LPU password")
        .with_confirmation("Confirm your password", "Passwords do not match.")
        .interact_on(&term)
        .unwrap();

    credentials.insert(
        account_id.clone(),
        Credentials {
            username: username.trim().to_string(),
            password: password.trim().to_string(),
        },
    );

    write_credentials(&credentials);

    println!(
        "{}",
        format!("✓ Account '{}' has been stored successfully", account_id).bright_green()
    );
    println!(); // Add a blank line for better readability

    account_id
}

/// Lists all stored account identifiers
pub fn list_account_ids() {
    use crate::login::get_current_logged_in_user;
    let credentials = read_credentials();

    if credentials.is_empty() {
        println!("{}", "No stored accounts found.".bright_yellow());
    } else {
        println!("{}", "Stored account IDs:".bright_blue());

        // Get currently logged in user if any
        let current_user =
            get_current_logged_in_user().map(|u| u.trim_end_matches("@lpu.com").to_string());

        for (account_id, creds) in &credentials {
            let display_text = format!("- {} ({})", account_id, creds.username);

            // If this account matches the currently logged in user
            if let Some(ref current_username) = current_user {
                if current_username == &creds.username {
                    println!(
                        "{} {}",
                        display_text.bright_green(),
                        "← Currently logged in".bright_green().italic()
                    );
                    continue;
                }
            }

            // Normal display for other accounts
            println!("{}", display_text.bright_white());
        }
    }
}

/// Removes specified accounts from the stored credentials
pub fn remove_accounts(account_ids: &[String]) {
    let mut credentials = read_credentials();

    let mut removed = false;
    for account_id in account_ids {
        if credentials.remove(account_id).is_some() {
            println!(
                "{}",
                format!("✗ Removed account '{}'", account_id).bright_red()
            );
            removed = true;
        } else {
            println!(
                "{}",
                format!("! Account '{}' not found", account_id).bright_yellow()
            );
        }
    }

    if removed {
        write_credentials(&credentials);
        println!("{}", "✓ Credentials updated successfully".bright_green());
    } else {
        println!("{}", "! No accounts were removed".bright_yellow());
    }
}

/// Updates the details of a stored account
pub fn update_account(account_id: &str) {
    let mut credentials = read_credentials();

    if let Some(creds) = credentials.get(account_id) {
        let original_account_id = account_id.to_string();

        println!(
            "\n{}",
            format!("Updating account '{}'", account_id)
                .bright_blue()
                .bold()
        );
        let theme = ColorfulTheme::default();
        let term = Term::stdout();

        // Prompt for new account ID (name)
        let new_account_id: String = Input::with_theme(&theme)
            .with_prompt("Enter new account ID (press Enter to keep current)")
            .default(original_account_id.clone())
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("Account ID cannot be empty.")
                } else if input.trim() != original_account_id
                    && credentials.contains_key(input.trim())
                {
                    Err("An account with this ID already exists. Please choose a different ID.")
                } else {
                    Ok(())
                }
            })
            .interact_text_on(&term)
            .unwrap();

        // Prompt for new username
        let new_username: String = Input::with_theme(&theme)
            .with_prompt("Enter new LPU username (press Enter to keep current)")
            .default(creds.username.clone())
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("Username cannot be empty.")
                } else {
                    Ok(())
                }
            })
            .interact_text_on(&term)
            .unwrap();

        // Prompt for new password
        let new_password = Password::with_theme(&theme)
            .with_prompt("Enter new LPU password (leave blank to keep current)")
            .with_confirmation("Confirm your password", "Passwords do not match.")
            .allow_empty_password(true)
            .interact()
            .unwrap();

        // Prepare updated credentials
        let updated_creds = Credentials {
            username: new_username.trim().to_string(),
            password: if new_password.is_empty() {
                creds.password.clone()
            } else {
                new_password
            },
        };

        // Update credentials map
        if new_account_id != original_account_id {
            credentials.remove(account_id);
            credentials.insert(new_account_id.clone(), updated_creds);
            println!(
                "{}",
                format!(
                    "✓ Account ID changed from '{}' to '{}'",
                    original_account_id, new_account_id
                )
                .bright_green()
            );
        } else {
            credentials.insert(account_id.to_string(), updated_creds);
        }

        // Write updated credentials
        write_credentials(&credentials);
        println!("{}", "✓ Account updated successfully".bright_green());
    } else {
        println!(
            "{}",
            format!("✗ Account '{}' not found", account_id).bright_red()
        );
    }
}

/// Shows the details of a stored account
pub fn show_account(account_id: &str, show_password: bool) {
    let credentials = read_credentials();

    if let Some(creds) = credentials.get(account_id) {
        println!("\n{}", "Account Details:".bright_blue().bold());
        println!("Account ID: {}", account_id.bright_white());
        println!("Username: {}", creds.username.bright_white());
        if show_password {
            // Provide a confirmation before displaying the password
            use dialoguer::Confirm;
            let proceed = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Are you sure you want to display the password in plaintext?")
                .default(false)
                .interact()
                .unwrap();

            if proceed {
                println!("Password: {}", creds.password.bright_white());
            } else {
                println!("Password: {}", "[hidden]".bright_yellow());
            }
        } else {
            println!("Password: {}", "[hidden]".bright_yellow());
        }
    } else {
        println!(
            "{}",
            format!("✗ Account '{}' not found", account_id).bright_red()
        );
    }
}
