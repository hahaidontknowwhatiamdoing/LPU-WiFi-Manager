use dirs_next::home_dir;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn store_lpu_credentials() {
    print!("Enter a unique identifier for this account: ");
    io::stdout().flush().unwrap();
    let account_id = prompt_for_account_id();

    let username_var = format!("LPU_USERNAME_{}", account_id);
    let password_var = format!("LPU_PASSWORD_{}", account_id);

    if env::var(&username_var).is_ok() || env::var(&password_var).is_ok() {
        println!("Credentials already exist for account ID '{}'.", account_id);
        return;
    }

    print!("Enter your LPU username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();

    print!("Enter your LPU password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();

    env::set_var(&username_var, username.trim());
    env::set_var(&password_var, password.trim());

    // Write credentials to file
    let home_dir = home_dir().expect("Failed to get home directory");
    let lpu_creds_path = home_dir.join(".lpu_creds");
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(lpu_creds_path)
        .unwrap();

    if let Err(e) = writeln!(file, "export {}=\"{}\"", username_var, username.trim()) {
        eprintln!("Couldn't write to file: {}", e);
    }
    if let Err(e) = writeln!(file, "export {}=\"{}\"", password_var, password.trim()) {
        eprintln!("Couldn't write to file: {}", e);
    }

    update_shell_config();

    println!("LPU username and password have been stored securely. Reload the current shell or open a new one to use it :)");
}

fn prompt_for_account_id() -> String {
    let mut account_id = String::new();
    io::stdin().read_line(&mut account_id).unwrap();
    account_id.trim().to_string()
}

fn update_shell_config() {
    let home_dir = home_dir().expect("Failed to get home directory");
    let shell_config = determine_shell_config(&home_dir);

    if let Some(config_path) = shell_config {
        let file = File::open(&config_path).unwrap();
        let reader = BufReader::new(file);

        if !reader
            .lines()
            .any(|line| line.unwrap() == "source ~/.lpu_creds")
        {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&config_path)
                .unwrap();

            if let Err(e) = writeln!(file, "source ~/.lpu_creds") {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
    }
}

fn determine_shell_config(home_dir: &Path) -> Option<std::path::PathBuf> {
    let zshrc_in_home = home_dir.join(".zshrc");
    let zshrc_in_config = home_dir.join(".config/zsh/.zshrc");

    match env::var("SHELL") {
        Ok(val) => {
            if val.contains("bash") {
                Some(home_dir.join(".bashrc"))
            } else if val.contains("zsh") {
                if zshrc_in_home.exists() {
                    Some(zshrc_in_home)
                } else if zshrc_in_config.exists() {
                    Some(zshrc_in_config)
                } else {
                    println!("Could not find .zshrc file. Please manually set the environment variables.");
                    None
                }
            } else if val.contains("fish") {
                Some(home_dir.join(".config/fish/config.fish"))
            } else {
                println!("Unsupported shell. Please manually set the environment variables.");
                None
            }
        }
        Err(_) => {
            println!("Couldn't determine shell.");
            None
        }
    }
}
