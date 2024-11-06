//! Credential management module for LPU WiFi Manager
//!
//! This module handles secure storage and retrieval of WiFi credentials:
//! - Stores credentials in system-specific config directories
//! - Manages multiple account credentials
//! - Provides JSON-based persistent storage

use dirs_next::config_dir;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
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
type CredentialsMap = HashMap<String, Credentials>;

/// Gets the path to the credentials file in the system config directory
///
/// # Returns
/// A PathBuf pointing to credentials.json in the system-specific config location:
/// - Linux: ~/.config/llogin/credentials.json
/// - Windows: %APPDATA%/llogin/credentials.json
///
/// # Panics
/// Panics if:
/// - Unable to determine system config directory
/// - Unable to create llogin directory
fn get_credentials_file_path() -> PathBuf {
    let config_dir = config_dir().expect("Failed to get config directory");
    let llogin_dir = config_dir.join("llogin");
    if !llogin_dir.exists() {
        fs::create_dir_all(&llogin_dir).expect("Failed to create llogin directory");
    }
    llogin_dir.join("credentials.json")
}

/// Reads stored credentials from the credentials file
///
/// # Returns
/// A HashMap containing account IDs mapped to their credentials.
/// Returns an empty HashMap if no credentials file exists.
///
/// # Panics
/// Panics if:
/// - Unable to open credentials file
/// - File contains invalid JSON
/// - JSON doesn't match expected credential format
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
///
/// # Arguments
/// * `credentials` - HashMap containing account IDs and their credentials
///
/// # Panics
/// Panics if:
/// - Unable to open/create credentials file
/// - Unable to write to file
/// - Unable to serialize credentials to JSON
fn write_credentials(credentials: &CredentialsMap) {
    let path = get_credentials_file_path();
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .expect("Failed to open credentials file");
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, credentials).expect("Failed to write credentials");
}

/// Prompts user for and stores new LPU WiFi credentials
///
/// Interactively asks for:
/// - Account identifier
/// - LPU username
/// - LPU password
///
/// The password is read securely without echo.
/// Credentials are stored in the system config directory.
///
/// # Example
/// ```no_run
/// llogin::credentials::store_lpu_credentials();
/// // Enter a unique identifier for this account: myaccount
/// // Enter your LPU username: 12345678
/// // Enter your LPU password: ****
/// ```
///
/// # Notes
/// - Will not overwrite existing credentials for the same account ID
/// - Password is never displayed on screen
pub fn store_lpu_credentials() {
    print!("Enter a unique identifier for this account: ");
    io::stdout().flush().unwrap();
    let account_id = prompt_for_account_id();

    let mut credentials = read_credentials();

    if credentials.contains_key(&account_id) {
        println!("Credentials already exist for account ID '{}'.", account_id);
        return;
    }

    print!("Enter your LPU username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();

    print!("Enter your LPU password: ");
    io::stdout().flush().unwrap();
    let password = read_password().unwrap();

    credentials.insert(
        account_id.clone(),
        Credentials {
            username: username.trim().to_string(),
            password: password.trim().to_string(),
        },
    );

    write_credentials(&credentials);

    println!("LPU username and password have been stored securely.");
}

/// Prompts for and reads an account identifier from stdin
///
/// # Returns
/// The trimmed account identifier string entered by the user
///
/// # Panics
/// Panics if unable to read from stdin
fn prompt_for_account_id() -> String {
    let mut account_id = String::new();
    io::stdin().read_line(&mut account_id).unwrap();
    account_id.trim().to_string()
}
