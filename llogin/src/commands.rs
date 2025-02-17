//! Command handlers for CLI operations.

use colored::*;
use std::process;

use crate::{
    cli::{self, Commands, LoginArgs},
    credentials,
    dialogues::{self, prompt_password, prompt_username},
    helpers::confirm_action,
    login,
    types::{LoginAction, SelectionContext},
};

/// Handles the main CLI command dispatch.
pub fn handle_command(cli: cli::Cli) {
    match &cli.command {
        Some(command) => match command {
            Commands::Login(args) => handle_login(args),
            Commands::Logout { account_id } => handle_logout(account_id),
            Commands::List => handle_list(),
            Commands::Remove { account_ids } => handle_remove(account_ids),
            Commands::Update { account_id } => handle_update(account_id),
            Commands::Show(args) => handle_show(args),
            Commands::Default { account_id, clear } => handle_default(account_id, *clear),
        },
        None => handle_default_action(),
    }
}

/// Handles the login command.
fn handle_login(args: &LoginArgs) {
    if args.set_default && args.account_id.is_some() {
        if let Some(id) = &args.account_id {
            if let Err(e) = credentials::set_default_account(id) {
                println!("{}", e.bright_red());
                process::exit(1);
            }
            println!("{}", format!("✓ Set '{}' as default account", id).bright_green());
        }
    }

    if args.no_save {
        let username = args.username.clone().unwrap_or_else(prompt_username);
        let password = args.password.clone().unwrap_or_else(prompt_password);
        login::perform_temporary_login(&username, &password);
    } else {
        match &args.account_id {
            Some(id) => handle_login_with_id(id),
            None => handle_interactive_login(),
        }
    }
}

/// Handles login with a specific account ID.
fn handle_login_with_id(id: &str) {
    if credentials::account_exists(id) {
        login::perform_lpu_login(id);
    } else {
        println!("Account '{}' not found.", id);
        let create = confirm_action(&format!(
            "Would you like to create a new account with ID '{}'?",
            id
        ));
        if create {
            let new_id = credentials::store_new_account_dialogue(id);
            let proceed = confirm_action(&format!(
                "Would you like to login with the newly created account '{}'?",
                new_id
            ));
            if proceed {
                login::perform_lpu_login(&new_id);
            } else {
                println!("{}", "Login skipped.".bright_yellow());
            }
        } else {
            process::exit(0);
        }
    }
}

/// Handles the default action when no command is specified.
fn handle_default_action() {
    // Check for default account first
    if let Some(default_id) = credentials::get_default_account() {
        login::perform_lpu_login(&default_id);
    } else {
        match dialogues::interactive_account_selection_with_context(SelectionContext::Login) {
            LoginAction::UseExistingAccount(id) => {
                login::perform_lpu_login(&id);
            }
            LoginAction::NewAccount => {
                let new_id = credentials::store_new_account_dialogue("");
                let proceed = confirm_action(&format!(
                    "Would you like to login with the newly created account '{}'?",
                    new_id
                ));
                if proceed {
                    login::perform_lpu_login(&new_id);
                } else {
                    println!("{}", "Login skipped.".bright_yellow());
                }
            }
            LoginAction::TemporaryLogin => {
                println!("{}", "\nTemporary Login".bright_yellow().bold());
                let username = prompt_username();
                let password = prompt_password();
                login::perform_temporary_login(&username, &password);
            }
            LoginAction::ClearDefaults => {
                println!("{}", "Invalid operation for login context.".bright_red());
                process::exit(1);
            }
            LoginAction::Cancel => {
                println!("{}", "Operation cancelled.".bright_yellow());
                process::exit(0);
            }
        }
    }
}

/// Handles the logout command.
fn handle_logout(account_id: &Option<String>) {
    if let Some(id) = account_id {
        login::perform_lpu_logout(id);
    } else {
        let logged_out = login::logout_current_session();
        if !logged_out {
            println!("{}", "No active session detected.".bright_yellow());
            let proceed = confirm_action("Do you want to logout from a stored account?");
            if proceed {
                match dialogues::interactive_account_selection_with_context(SelectionContext::Other) {
                    LoginAction::UseExistingAccount(id) => {
                        login::perform_lpu_logout(&id);
                    }
                    _ => {
                        println!("{}", "Invalid selection for logout operation.".bright_red());
                        process::exit(1);
                    }
                }
            }
        }
    }
}

/// Handles the list command.
fn handle_list() {
    credentials::list_account_ids();
}

/// Handles the remove command.
fn handle_remove(account_ids: &[String]) {
    if account_ids.is_empty() {
        let selected_accounts = dialogues::multi_select_accounts();
        if selected_accounts.is_empty() {
            println!("{}", "No accounts selected for removal.".bright_yellow());
        } else {
            credentials::remove_accounts(&selected_accounts);
        }
    } else {
        credentials::remove_accounts(account_ids);
    }
}

/// Handles the update command.
fn handle_update(account_id: &Option<String>) {
    if let Some(id) = account_id {
        credentials::update_account(id);
    } else {
        match dialogues::interactive_account_selection_with_context(SelectionContext::Other) {
            LoginAction::UseExistingAccount(id) => {
                credentials::update_account(&id);
            }
            _ => {
                println!("{}", "Invalid selection for update operation.".bright_red());
                process::exit(1);
            }
        }
    }
}

/// Handles the show command.
fn handle_show(args: &cli::ShowArgs) {
    if let Some(id) = &args.account_id {
        credentials::show_account(id, args.show_password);
    } else {
        match dialogues::interactive_account_selection_with_context(SelectionContext::Other) {
            LoginAction::UseExistingAccount(id) => {
                credentials::show_account(&id, args.show_password);
            }
            _ => {
                println!("{}", "Invalid selection for show operation.".bright_red());
                process::exit(1);
            }
        }
    }
}

/// Handles the default command.
fn handle_default(account_id: &Option<String>, clear: bool) {
    if clear {
        credentials::clear_default_account();
        println!("{}", "✓ Cleared default account setting".bright_green());
    } else if let Some(id) = account_id {
        if let Err(e) = credentials::set_default_account(id) {
            println!("{}", e.bright_red());
            process::exit(1);
        }
        println!("{}", format!("✓ Set '{}' as default account", id).bright_green());
    } else {
        handle_interactive_default_selection();
    }
}

/// Handles interactive default account selection.
fn handle_interactive_default_selection() {
    match dialogues::interactive_account_selection_with_context(SelectionContext::Default) {
        LoginAction::UseExistingAccount(id) => {
            let id = id.split_whitespace().next().unwrap();
            if let Err(e) = credentials::set_default_account(id) {
                println!("{}", e.bright_red());
                process::exit(1);
            }
            println!("{}", format!("✓ Set '{}' as default account", id).bright_green());

            let proceed = confirm_action(&format!(
                "Would you like to login with the newly set default account '{}'?",
                id
            ));
            if proceed {
                login::perform_lpu_login(id);
            }
        }
        LoginAction::ClearDefaults => {
            credentials::clear_default_account();
            println!("{}", "✓ Cleared default account setting".bright_green());
        }
        _ => {
            println!("{}", "Invalid selection for default operation.".bright_red());
            process::exit(1);
        }
    }
}

/// Handles interactive login flow.
fn handle_interactive_login() {
    match dialogues::interactive_account_selection_with_context(SelectionContext::Login) {
        LoginAction::UseExistingAccount(id) => {
            login::perform_lpu_login(&id);
        }
        LoginAction::NewAccount => {
            let new_id = credentials::store_new_account_dialogue("");
            let proceed = confirm_action(&format!(
                "Would you like to login with the newly created account '{}'?",
                new_id
            ));
            if proceed {
                login::perform_lpu_login(&new_id);
            }
        }
        LoginAction::TemporaryLogin => {
            println!("{}", "\nTemporary Login".bright_yellow().bold());
            let username = prompt_username();
            let password = prompt_password();
            login::perform_temporary_login(&username, &password);
        }
        LoginAction::ClearDefaults => {
            println!("{}", "Invalid operation for login context.".bright_red());
            process::exit(1);
        }
        LoginAction::Cancel => {
            println!("{}", "Operation cancelled.".bright_yellow());
            process::exit(0);
        }
    }
} 