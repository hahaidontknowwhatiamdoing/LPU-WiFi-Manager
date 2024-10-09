use std::env;
use ureq;
use native_tls;

pub fn perform_lpu_login(account_id: &str) {
    let username_var = format!("LPU_USERNAME_{}", account_id);
    let password_var = format!("LPU_PASSWORD_{}", account_id);

    let username = env::var(&username_var).unwrap_or(String::new());
    let password = env::var(&password_var).unwrap_or(String::new());

    if !username.is_empty() && !password.is_empty() {
        let data = format!(
            "mode=191&username={}%40lpu.com&password={}",
            username, password
        );

        let agent = ureq::AgentBuilder::new()
            .tls_connector(std::sync::Arc::new(native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap()))
            .build();

        match agent.post("https://10.10.0.1/24online/servlet/E24onlineHTTPClient")
            .set("Content-Type", "application/x-www-form-urlencoded")
            .set("Accept-Encoding", "gzip, deflate, br")
            .send_string(&data)
        {
            Ok(response) => {
                match response.into_string() {
                    Ok(text) => {
                        if text.contains("To start surfing") {
                            println!("Login successful for account {}.", account_id);
                        } else if text.contains("Invalid user") {
                            println!("Login failed for account {}. Invalid username or password.", account_id);
                        } else {
                            println!("Login failed for account {}. Unexpected response.", account_id);
                        }
                    },
                    Err(_) => println!("Failed to read response for account {}.", account_id),
                }
            },
            Err(_) => {
                println!("Login failed for account {}. Network error or server unavailable.", account_id);
            },
        }
    } else {
        println!(
            "LPU username or password not set for account {}.",
            account_id
        );
        crate::credentials::store_lpu_credentials();
    }
}
