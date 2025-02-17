//! User interaction functions for the application.

use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Password, Select};
use std::process;

use crate::{credentials, types::{LoginAction, SelectionContext}};

/// Prompts the user for their LPU username.
pub fn prompt_username() -> String {
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
                process::exit(0);
            }
            username
        }
        Err(_) => {
            println!("{}", "Operation cancelled.".bright_yellow());
            process::exit(0);
        }
    }
}

/// Prompts the user for their LPU password.
pub fn prompt_password() -> String {
    let theme = ColorfulTheme::default();
    let password = Password::with_theme(&theme)
        .with_prompt("Enter your LPU password")
        .allow_empty_password(true)
        .interact();

    match password {
        Ok(pass) => {
            if pass.trim().is_empty() {
                println!("{}", "Operation cancelled.".bright_yellow());
                process::exit(0);
            }
            pass
        }
        Err(_) => {
            println!("{}", "Operation cancelled.".bright_yellow());
            process::exit(0);
        }
    }
}

/// Presents an interactive account selection menu.
pub fn interactive_account_selection_with_context(context: SelectionContext) -> LoginAction {
    let credentials = credentials::read_credentials();
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

    match context {
        SelectionContext::Login => {
            account_ids.push("[New Account]".green().to_string());
            account_ids.push("[Temporary Login]".yellow().to_string());
        }
        SelectionContext::Default => {
            // Add Clear Defaults option only for default account selection
            if !account_ids.is_empty() {
                account_ids.push("[Clear Defaults]".red().to_string());
            }
        }
        SelectionContext::Other => {
            // No additional options for other contexts
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
            } else if selected.contains("[Clear Defaults]") {
                LoginAction::ClearDefaults
            } else {
                LoginAction::UseExistingAccount(selected.split_whitespace().next().unwrap().to_string())
            }
        }
        None => LoginAction::Cancel,
    }
}

/// Presents a multi-select menu for account selection.
pub fn multi_select_accounts() -> Vec<String> {
    let credentials = credentials::read_credentials();
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