//! Login module for LPU WiFi Manager
//!
//! This module handles:
//! - WiFi login/logout functionality
//! - Credential management
//! - Network requests to LPU authentication servers

use dirs_next::config_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// Represents stored credentials for an LPU WiFi account
#[derive(Serialize, Deserialize)]
struct Credentials {
    /// LPU username without @lpu.com suffix
    username: String,
    /// Account password
    password: String,
}

/// Type alias for a HashMap storing account IDs mapped to their credentials
type CredentialsMap = HashMap<String, Credentials>;

/// Gets the path to the credentials file in the system config directory
///
/// # Returns
/// A PathBuf pointing to credentials.json in the system-specific config location
///
/// # Panics
/// Panics if unable to determine system config directory
fn get_credentials_file_path() -> PathBuf {
    let config_dir = config_dir().expect("Failed to get config directory");
    config_dir.join("llogin").join("credentials.json")
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
fn read_credentials() -> CredentialsMap {
    let path = get_credentials_file_path();
    if path.exists() {
        let file = File::open(path).expect("Failed to open credentials file");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Failed to read credentials")
    } else {
        HashMap::new()
    }
}

/// Attempts to login to LPU WiFi using stored credentials
///
/// # Arguments
/// * `account_id` - The identifier for the stored credentials to use
///
/// # Example
/// ```no_run
/// llogin::login::perform_lpu_login("myaccount");
/// ```
///
/// # Notes
/// - Will prompt for credentials if account not found
/// - Uses HTTPS with certificate verification disabled
/// - Attempts login to LPU WiFi portal at 10.10.0.1
pub fn perform_lpu_login(account_id: &str) {
    let credentials = read_credentials();

    if let Some(creds) = credentials.get(account_id) {
        let data = format!(
            "mode=191&username={}%40lpu.com&password={}",
            creds.username, creds.password
        );

        let agent = ureq::AgentBuilder::new()
            .tls_connector(std::sync::Arc::new(
                native_tls::TlsConnector::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .unwrap(),
            ))
            .build();

        match agent
            .post("https://10.10.0.1/24online/servlet/E24onlineHTTPClient")
            .set("Content-Type", "application/x-www-form-urlencoded")
            .set("Accept-Encoding", "gzip, deflate, br")
            .send_string(&data)
        {
            Ok(response) => match response.into_string() {
                Ok(text) => {
                    if text.contains("To start surfing") {
                        println!("Login successful for account {}.", account_id);
                    } else if text.contains("Invalid user") {
                        println!(
                            "Login failed for account {}. Invalid username or password.",
                            account_id
                        );
                    } else {
                        println!(
                            "Login failed for account {}. Unexpected response.",
                            account_id
                        );
                    }
                }
                Err(_) => println!("Failed to read response for account {}.", account_id),
            },
            Err(_) => {
                println!(
                    "Login failed for account {}. Network error or server unavailable.",
                    account_id
                );
            }
        }
    } else {
        println!(
            "LPU username or password not set for account {}.",
            account_id
        );
        crate::credentials::store_lpu_credentials();
    }
}

/// Logs out from LPU WiFi for the given account
///
/// # Arguments
/// * `account_id` - The identifier for the stored credentials to use
///
/// # Example
/// ```no_run
/// llogin::login::perform_lpu_logout("myaccount");
/// ```
///
/// # Notes
/// - Uses stored credentials for the account
/// - Sends logout request to internet.lpu.in
/// - Uses HTTPS with certificate verification disabled
pub fn perform_lpu_logout(account_id: &str) {
    let credentials = read_credentials();

    if let Some(creds) = credentials.get(account_id) {
        let data = format!(
            "mode=193&logout=Logout&username={}%40lpu.com",
            creds.username
        );

        let agent = ureq::AgentBuilder::new()
            .tls_connector(std::sync::Arc::new(
                native_tls::TlsConnector::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .unwrap(),
            ))
            .build();

        match agent
            .post("https://internet.lpu.in/24online/servlet/E24onlineHTTPClient")
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&data)
        {
            Ok(response) => match response.into_string() {
                Ok(_) => {
                    println!("Logout successful for account {}.", account_id);
                }
                Err(_) => println!("Failed to read response for account {}.", account_id),
            },
            Err(_) => {
                println!(
                    "Logout failed for account {}. Network error or server unavailable.",
                    account_id
                );
            }
        }
    } else {
        println!(
            "LPU username or password not set for account {}.",
            account_id
        );
    }
}

