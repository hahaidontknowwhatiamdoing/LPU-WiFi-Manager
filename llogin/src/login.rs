// src/login.rs

use crate::credentials::read_credentials;
use colored::*;
use native_tls::TlsConnector;
use regex::Regex;
use ureq::Agent;

/// Attempts to login to LPU WiFi using stored credentials
pub fn perform_lpu_login(account_id: &str) {
    let credentials = read_credentials();

    if let Some(creds) = credentials.get(account_id) {
        // Check if there's an active session
        if let Some(full_username) = get_current_logged_in_user() {
            let current_username = full_username.trim_end_matches("@lpu.com");

            if current_username == creds.username {
                println!("{}", "✓ Already logged in as this account".bright_green());
                return;
            } else {
                // Check if the current user is from a stored account
                let current_credentials = read_credentials();
                let current_account = current_credentials
                    .iter()
                    .find(|(_, creds)| creds.username == current_username);

                match current_account {
                    Some((current_id, _)) => {
                        // Switching between stored accounts
                        println!(
                            "{}",
                            format!(
                                "→ Switching accounts: {} → {}",
                                current_id.bright_yellow(),
                                account_id.bright_green()
                            )
                        );
                    }
                    None => {
                        // Switching from temporary login
                        println!(
                            "{}",
                            format!(
                                "→ Switching accounts: {} → {}",
                                full_username.bright_yellow(),
                                account_id.bright_green()
                            )
                        );
                    }
                }

                // Attempt to logout the current session
                perform_quiet_logout(&full_username);
            }
        }

        // Proceed to log in as the new account
        login_with_credentials(&creds.username, &creds.password, Some(account_id));
    } else if account_id != "[Temporary Login]" {
        println!(
            "{}",
            format!("✗ Credentials not found for account '{}'", account_id).bright_red()
        );
    }
}

/// Performs a temporary login with provided username and password
pub fn perform_temporary_login(username: &str, password: &str) {
    // Check if there's an active session
    if let Some(full_username) = get_current_logged_in_user() {
        let current_username = full_username.trim_end_matches("@lpu.com");

        if current_username == username {
            println!("{}", "✓ Already logged in as this account".bright_green());
            return;
        } else {
            // Check if current user is from a stored account
            let current_credentials = read_credentials();
            let current_account = current_credentials
                .iter()
                .find(|(_, creds)| creds.username == current_username);

            match current_account {
                Some((current_id, _)) => {
                    println!(
                        "{}",
                        format!(
                            "→ Switching from {} to [Temporary Session]",
                            current_id.bright_yellow()
                        )
                    );
                }
                None => {
                    println!(
                        "{}",
                        format!(
                            "→ Switching from {} to temporary session",
                            full_username.bright_yellow()
                        )
                    );
                }
            }

            perform_quiet_logout(&full_username);
        }
    }

    login_with_credentials(username, password, None);
}

/// Logs in using the provided username and password
fn login_with_credentials(username: &str, password: &str, account_id: Option<&str>) {
    let data = format!(
        "mode=191&username={}%40lpu.com&password={}",
        username, password
    );

    let agent = create_http_agent();

    match agent
        .post("https://internet.lpu.in/24online/servlet/E24onlineHTTPClient")
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("Accept-Encoding", "gzip, deflate, br")
        .send_string(&data)
    {
        Ok(response) => match response.into_string() {
            Ok(text) => {
                if text.contains("To start surfing")
                    || text.contains("You have successfully logged in")
                {
                    match account_id {
                        Some(id) => println!("{}", format!("✓ Logged in as {}", id).bright_green()),
                        None => println!("{}", "✓ Logged in successfully".bright_green()),
                    }
                } else if text.contains("Invalid user") {
                    match account_id {
                        Some(id) => println!(
                            "{}",
                            format!("✗ Login failed for '{}': Invalid credentials", id)
                                .bright_red()
                        ),
                        None => println!("{}", "✗ Login failed: Invalid credentials".bright_red()),
                    }
                } else {
                    match account_id {
                        Some(id) => println!(
                            "{}",
                            format!("✗ Login failed for '{}': Unexpected response", id)
                                .bright_red()
                        ),
                        None => println!("{}", "✗ Login failed: Unexpected response".bright_red()),
                    }
                }
            }
            Err(_) => match account_id {
                Some(id) => println!(
                    "{}",
                    format!("✗ Failed to read response for '{}'", id).bright_red()
                ),
                None => println!("{}", "✗ Failed to read response".bright_red()),
            },
        },
        Err(_) => match account_id {
            Some(id) => println!(
                "{}",
                format!("✗ Network error while logging in as '{}'", id).bright_red()
            ),
            None => println!("{}", "✗ Network error while logging in".bright_red()),
        },
    }
}

/// Creates an HTTP agent with TLS certificate verification disabled
fn create_http_agent() -> Agent {
    ureq::AgentBuilder::new()
        .tls_connector(std::sync::Arc::new(
            TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap(),
        ))
        .build()
}

/// Gets the currently logged-in username
pub fn get_current_logged_in_user() -> Option<String> {
    let agent = create_http_agent();
    let html_content = fetch_client_jsp_content(&agent)?;
    extract_username_from_html(&html_content)
}

/// Fetches the content of client.jsp
fn fetch_client_jsp_content(agent: &Agent) -> Option<String> {
    match agent
        .get("https://internet.lpu.in/24online/webpages/client.jsp")
        .call()
    {
        Ok(response) => response.into_string().ok(),
        Err(_) => None,
    }
}

/// Extracts username from HTML content
fn extract_username_from_html(html: &str) -> Option<String> {
    let patterns = [
        r#"name=["']?loggedinuser["']?\s+value=["']([^"']+)"#,
        r#"name=["']?username["']?\s+value=["']([^"']+)"#,
    ];

    patterns.iter().find_map(|pattern| {
        Regex::new(pattern)
            .ok()
            .and_then(|re| re.captures(html))
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    })
}

/// Attempts to logout the currently logged-in user
pub fn logout_current_session() -> bool {
    if let Some(full_username) = get_current_logged_in_user() {
        let username = full_username.trim_end_matches("@lpu.com");
        let credentials = read_credentials();

        // Try to find if this is a stored account
        let matching_account = credentials
            .iter()
            .find(|(_, creds)| creds.username == username);

        match matching_account {
            Some((account_id, _)) => {
                // This is a stored account, use account ID
                println!(
                    "{}",
                    format!("→ Logging out {}", account_id).bright_yellow()
                );
                perform_lpu_logout(account_id);
            }
            None => {
                // This is a temporary session, use full username
                println!(
                    "{}",
                    format!("→ Logging out temporary session: {}", full_username).bright_yellow()
                );
                perform_logout_with_username(&full_username);
            }
        }
        true
    } else {
        println!("{}", "No active session detected".bright_yellow());
        false
    }
}

/// Performs logout using full username
fn perform_logout_with_username(full_username: &str) {
    let data = format!("mode=193&logout=Logout&username={}", full_username);

    let agent = create_http_agent();

    match agent
        .post("https://internet.lpu.in/24online/servlet/E24onlineHTTPClient")
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&data)
    {
        Ok(response) => match response.into_string() {
            Ok(text) => {
                if text.contains("successfully logged off")
                    || text.contains("logged out successfully")
                    || text.contains("Logout successfully")
                {
                    println!("{}", "✓ Logged out successfully".bright_green());
                } else {
                    println!("{}", "✗ Logout failed: Unexpected response".bright_red());
                }
            }
            Err(_) => println!("{}", "✗ Failed to read logout response".bright_red()),
        },
        Err(_) => {
            println!("{}", "✗ Network error while logging out".bright_red());
        }
    }
}

/// Performs a quiet logout (with minimal output)
fn perform_quiet_logout(full_username: &str) {
    let data = format!("mode=193&logout=Logout&username={}", full_username);

    let agent = create_http_agent();

    if let Ok(response) = agent
        .post("https://internet.lpu.in/24online/servlet/E24onlineHTTPClient")
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&data)
    {
        if let Ok(text) = response.into_string() {
            if !text.contains("successfully logged off")
                && !text.contains("logged out successfully")
                && !text.contains("Logout successfully")
            {
                println!("{}", "→ Previous session ended".bright_yellow());
            }
        }
    }
}

/// Logs out from LPU WiFi for the given account
pub fn perform_lpu_logout(account_id: &str) {
    let credentials = read_credentials();

    if let Some(creds) = credentials.get(account_id) {
        let data = format!(
            "mode=193&logout=Logout&username={}%40lpu.com",
            creds.username
        );

        let agent = create_http_agent();

        match agent
            .post("https://internet.lpu.in/24online/servlet/E24onlineHTTPClient")
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&data)
        {
            Ok(response) => match response.into_string() {
                Ok(text) => {
                    if text.contains("successfully logged off")
                        || text.contains("logged out successfully")
                        || text.contains("Logout successfully")
                    {
                        println!(
                            "{}",
                            format!("✓ Logged out from {}", account_id).bright_green()
                        );
                    } else {
                        println!(
                            "{}",
                            format!("✗ Logout failed for {}: Unexpected response", account_id)
                                .bright_red()
                        );
                    }
                }
                Err(_) => println!(
                    "{}",
                    format!("✗ Failed to read logout response for {}", account_id).bright_red()
                ),
            },
            Err(_) => {
                println!(
                    "{}",
                    format!("✗ Network error while logging out from {}", account_id).bright_red()
                );
            }
        }
    } else {
        println!(
            "{}",
            format!("✗ Account '{}' not found", account_id).bright_red()
        );
    }
}
