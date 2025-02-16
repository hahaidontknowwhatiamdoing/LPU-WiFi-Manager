// src/cli.rs

use clap::{Args, Parser, Subcommand};

/// LPU WiFi Manager: A CLI tool for managing Lovely Professional University WiFi connections.
#[derive(Parser)]
#[command(
    author = "smazmi",
    version,
    about = "Manage and log in to multiple LPU WiFi accounts",
    long_about = "LPU WiFi Manager allows you to store credentials for multiple accounts, log in to the LPU WiFi network, and manage your connections."
)]
pub struct Cli {
    /// Command to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Login using a stored account ID or create a new one
    #[command(about = "Login using a stored account ID or create a new one")]
    Login(LoginArgs),
    /// Logout from a specified account or the current session
    #[command(about = "Logout from a specified account or the current session")]
    Logout {
        /// Account ID or Name to logout from
        #[arg(value_name = "ACCOUNT_ID", help = "The account ID to logout from")]
        account_id: Option<String>,
    },
    /// List all stored account IDs
    #[command(about = "List all stored account IDs")]
    List,
    /// Remove stored accounts
    #[command(about = "Remove stored account(s)")]
    Remove {
        /// Account IDs or Names to remove
        #[arg(value_name = "ACCOUNT_ID", help = "The account ID(s) to remove")]
        account_ids: Vec<String>,
    },
    /// Update stored account details
    #[command(about = "Update stored account(s)")]
    Update {
        /// Account ID or Name to update
        #[arg(value_name = "ACCOUNT_ID", help = "The account ID to update")]
        account_id: Option<String>,
    },
    /// Show stored account details
    #[command(about = "Show stored account details")]
    Show(ShowArgs),
    /// Manage default account settings
    #[command(about = "Set or clear default account for quick login")]
    Default {
        /// Account ID to set as default. If not provided, shows selection menu
        #[arg(value_name = "ACCOUNT_ID")]
        account_id: Option<String>,
        
        /// Clear default account setting
        #[arg(long, conflicts_with = "account_id")]
        clear: bool,
    },
}

#[derive(Args)]
pub struct LoginArgs {
    /// Account ID or Name to login with
    #[arg(
        value_name = "ACCOUNT_ID",
        help = "The account ID to login with",
        required = false
    )]
    pub account_id: Option<String>,

    /// Do not save credentials
    #[arg(long, help = "Login without saving credentials")]
    pub no_save: bool,

    /// LPU username (without @lpu.com), used with --no-save
    #[arg(
        long,
        requires = "no_save",
        help = "LPU username (used with --no-save)"
    )]
    pub username: Option<String>,

    /// LPU password, used with --no-save
    #[arg(
        long,
        requires = "no_save",
        help = "LPU password (used with --no-save)"
    )]
    pub password: Option<String>,

    /// Set this account as default after login
    #[arg(long, help = "Set this account as default for quick login")]
     pub set_default: bool,
}

#[derive(Args)]
pub struct ShowArgs {
    /// Account ID or Name to show
    #[arg(
        value_name = "ACCOUNT_ID",
        help = "The account ID to show",
        required = false
    )]
    pub account_id: Option<String>,

    /// Show the password in plaintext
    #[arg(short, long, help = "Show the password in plaintext")]
    pub show_password: bool,
}
