// src/bin/llogin.rs

use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use llogin;

#[derive(PartialEq)]
enum LoginAction {
    UseExistingAccount(String),
    NewAccount,
    TemporaryLogin,
    Cancel,
}

enum SelectionContext {
    Login,
    Other,
}

fn main() {
    if !llogin::wifi::check_lpu_wifi() {
        println!("Not connected to LPU WiFi. Exiting.");
        std::process::exit(1);
    }

    let cli = llogin::cli::Cli::parse();

    match &cli.command {
        Some(llogin::cli::Commands::Login(args)) => {
            if args.set_default && args.account_id.is_some() {
                if let Some(id) = &args.account_id {
                    if let Err(e) = llogin::credentials::set_default_account(id) {
                        println!("{}", e.bright_red());
                        std::process::exit(1);
                    }
                    println!("{}", format!("✓ Set '{}' as default account", id).bright_green());
                }
            }
            if args.no_save {
                let username = args.username.clone().unwrap_or_else(|| prompt_username());
                let password = args.password.clone().unwrap_or_else(|| prompt_password());
                llogin::login::perform_temporary_login(&username, &password);
            } else {
                match &args.account_id {
                    Some(id) => {
                        if llogin::credentials::account_exists(id) {
                            llogin::login::perform_lpu_login(id);
                        } else {
                            println!("Account '{}' not found.", id);
                            let create = confirm_action(&format!(
                                "Would you like to create a new account with ID '{}'?",
                                id
                            ));
                            if create {
                                let new_id = llogin::credentials::store_new_account_dialogue(id);
                                let proceed = confirm_action(&format!(
                                    "Would you like to login with the newly created account '{}'?",
                                    new_id
                                ));
                                if proceed {
                                    llogin::login::perform_lpu_login(&new_id);
                                } else {
                                    println!("{}", "Login skipped.".bright_yellow());
                                }
                            } else {
                                std::process::exit(0);
                            }
                        }
                    }
                    None => {
                        match interactive_account_selection_with_context(SelectionContext::Login) {
                            LoginAction::UseExistingAccount(id) => {
                                llogin::login::perform_lpu_login(&id);
                            }
                            LoginAction::NewAccount => {
                                let new_id = llogin::credentials::store_new_account_dialogue("");
                                let proceed = confirm_action(&format!(
                                    "Would you like to login with the newly created account '{}'?",
                                    new_id
                                ));
                                if proceed {
                                    llogin::login::perform_lpu_login(&new_id);
                                } else {
                                    println!("{}", "Login skipped.".bright_yellow());
                                }
                            }
                            LoginAction::TemporaryLogin => {
                                println!("{}", "\nTemporary Login".bright_yellow().bold());
                                let username = prompt_username();
                                let password = prompt_password();
                                llogin::login::perform_temporary_login(&username, &password);
                            }
                            LoginAction::Cancel => {
                                println!("Operation cancelled.");
                                std::process::exit(0);
                            }
                        }
                    }
                }
            }
        }

        Some(llogin::cli::Commands::Logout { account_id }) => {
            if let Some(id) = account_id {
                llogin::login::perform_lpu_logout(id);
            } else {
                let logged_out = llogin::login::logout_current_session();
                if !logged_out {
                    println!("No active session detected.");
                    let proceed = confirm_action("Do you want to logout from a stored account?");
                    if proceed {
                        match interactive_account_selection_with_context(SelectionContext::Other) {
                            LoginAction::UseExistingAccount(id) => {
                                llogin::login::perform_lpu_logout(&id);
                            }
                            _ => {
                                println!("Invalid selection for logout operation.");
                                std::process::exit(1);
                            }
                        }
                    }
                }
            }
        }

        Some(llogin::cli::Commands::List) => {
            llogin::credentials::list_account_ids();
        }

        Some(llogin::cli::Commands::Remove { account_ids }) => {
            if account_ids.is_empty() {
                let selected_accounts = multi_select_accounts();
                if selected_accounts.is_empty() {
                    println!("No accounts selected for removal.");
                } else {
                    llogin::credentials::remove_accounts(&selected_accounts);
                }
            } else {
                llogin::credentials::remove_accounts(account_ids);
            }
        }

        Some(llogin::cli::Commands::Update { account_id }) => {
            if let Some(id) = account_id {
                llogin::credentials::update_account(&id);
            } else {
                match interactive_account_selection_with_context(SelectionContext::Other) {
                    LoginAction::UseExistingAccount(id) => {
                        llogin::credentials::update_account(&id);
                    }
                    _ => {
                        println!("Invalid selection for update operation.");
                        std::process::exit(1);
                    }
                }
            }
        }

        Some(llogin::cli::Commands::Show(show_args)) => {
            if let Some(id) = &show_args.account_id {
                llogin::credentials::show_account(id, show_args.show_password);
            } else {
                match interactive_account_selection_with_context(SelectionContext::Other) {
                    LoginAction::UseExistingAccount(id) => {
                        llogin::credentials::show_account(&id, show_args.show_password);
                    }
                    _ => {
                        println!("Invalid selection for show operation.");
                        std::process::exit(1);
                    }
                }
            }
        }

        Some(llogin::cli::Commands::Default { account_id, clear }) => {
            if *clear {
                llogin::credentials::clear_default_account();
                println!("{}", "✓ Cleared default account setting".bright_green());
            } else if let Some(id) = account_id {
                // Direct flag usage - just set default without asking to login
                if let Err(e) = llogin::credentials::set_default_account(id) {
                    println!("{}", e.bright_red());
                    std::process::exit(1);
                }
                println!("{}", format!("✓ Set '{}' as default account", id).bright_green());
            } else {
                // Show interactive selection with clear option
                let mut account_ids: Vec<String> = llogin::credentials::read_credentials()
                    .iter()
                    .map(|(id, creds)| {
                        if creds.is_default {
                            format!("{} {}", id, "(default)".bright_black())
                        } else {
                            id.clone()
                        }
                    })
                    .collect();
                
                // Add clear option at the end
                account_ids.push("[Clear Default]".bright_red().to_string());

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select an account to set as default")
                    .items(&account_ids)
                    .default(0)
                    .interact_opt()
                    .unwrap();

                match selection {
                    Some(index) if index == account_ids.len() - 1 => {
                        // Clear default was selected
                        llogin::credentials::clear_default_account();
                        println!("{}", "✓ Cleared default account setting".bright_green());
                    }
                    Some(index) => {
                        let selected = &account_ids[index];
                        let id = selected.split_whitespace().next().unwrap();
                        
                        if let Err(e) = llogin::credentials::set_default_account(id) {
                            println!("{}", e.bright_red());
                            std::process::exit(1);
                        }
                        println!("{}", format!("✓ Set '{}' as default account", id).bright_green());

                        // Ask if user wants to login with the new default account
                        let proceed = confirm_action(&format!(
                            "Would you like to login with the newly set default account '{}'?",
                            id
                        ));
                        if proceed {
                            llogin::login::perform_lpu_login(id);
                        }
                    }
                    None => {
                        println!("Operation cancelled.");
                        std::process::exit(0);
                    }
                }
            }
        }

        None => {
            // Check for default account first
            if let Some(default_id) = llogin::credentials::get_default_account() {
                llogin::login::perform_lpu_login(&default_id);
            } else {
                // Existing interactive selection logic
                match interactive_account_selection_with_context(SelectionContext::Login) {
                    LoginAction::UseExistingAccount(id) => {
                        llogin::login::perform_lpu_login(&id);
                    }
                    LoginAction::NewAccount => {
                        let new_id = llogin::credentials::store_new_account_dialogue("");
                        let proceed = confirm_action(&format!(
                            "Would you like to login with the newly created account '{}'?",
                            new_id
                        ));
                        if proceed {
                            llogin::login::perform_lpu_login(&new_id);
                        } else {
                            println!("{}", "Login skipped.".bright_yellow());
                        }
                    }
                    LoginAction::TemporaryLogin => {
                        println!("{}", "\nTemporary Login".bright_yellow().bold());
                        let username = prompt_username();
                        let password = prompt_password();
                        llogin::login::perform_temporary_login(&username, &password);
                    }
                    LoginAction::Cancel => {
                        println!("Operation cancelled.");
                        std::process::exit(0);
                    }
                }
            }
        }
    }
}

fn interactive_account_selection_with_context(context: SelectionContext) -> LoginAction {
    let credentials = llogin::credentials::read_credentials();
    let mut account_ids: Vec<String> = credentials
        .iter()
        .map(|(id, creds)| {
            if creds.is_default {
                format!("{} {}", id, "(default)".bright_black())
            } else {
                id.clone()
            }
        })
        .collect();

    // Only add special options for Login context
    match context {
        SelectionContext::Login => {
            account_ids.push("[New Account]".green().to_string());
            account_ids.push("[Temporary Login]".yellow().to_string());
        }
        SelectionContext::Other => {
            // No special options added for other contexts
        }
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an account")
        .items(&account_ids)
        .default(0)
        .interact_opt()
        .unwrap();

    match selection {
        Some(index) => {
            let selected = &account_ids[index];
            if selected.contains("[New Account]") {
                LoginAction::NewAccount
            } else if selected.contains("[Temporary Login]") {
                LoginAction::TemporaryLogin
            } else {
                LoginAction::UseExistingAccount(selected.clone())
            }
        }
        None => LoginAction::Cancel,
    }
}

fn prompt_username() -> String {
    use dialoguer::{theme::ColorfulTheme, Input};

    let theme = ColorfulTheme::default();
    let input = Input::with_theme(&theme)
        .with_prompt("Enter your LPU username (without @lpu.com)")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Username cannot be empty.")
            } else {
                Ok(())
            }
        })
        .allow_empty(true)
        .interact_text();

    match input {
        Ok(username) => {
            if username.trim().is_empty() {
                println!("{}", "Operation cancelled.".bright_yellow());
                std::process::exit(0);
            }
            username
        }
        Err(_) => {
            println!("{}", "Operation cancelled.".bright_yellow());
            std::process::exit(0);
        }
    }
}

fn prompt_password() -> String {
    use dialoguer::{theme::ColorfulTheme, Password};

    let theme = ColorfulTheme::default();
    let password = Password::with_theme(&theme)
        .with_prompt("Enter your LPU password")
        .allow_empty_password(true)
        .interact();

    match password {
        Ok(pass) => {
            if pass.trim().is_empty() {
                println!("{}", "Operation cancelled.".bright_yellow());
                std::process::exit(0);
            }
            pass
        }
        Err(_) => {
            println!("{}", "Operation cancelled.".bright_yellow());
            std::process::exit(0);
        }
    }
}

fn multi_select_accounts() -> Vec<String> {
    let credentials = llogin::credentials::read_credentials();
    if credentials.is_empty() {
        println!("No stored accounts to remove.");
        return Vec::new();
    }

    let account_ids: Vec<&String> = credentials.keys().collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select account(s) to remove (use space to select, enter to confirm)")
        .items(&account_ids)
        .interact_opt()
        .unwrap();

    match selections {
        Some(indices) => {
            if indices.is_empty() {
                println!("No accounts selected.");
                Vec::new()
            } else {
                indices
                    .iter()
                    .map(|&index| account_ids[index].clone())
                    .collect()
            }
        }
        None => {
            println!("Operation cancelled.");
            Vec::new()
        }
    }
}

fn confirm_action(message: &str) -> bool {
    use dialoguer::Confirm;

    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(false)
        .interact()
        .unwrap()
}