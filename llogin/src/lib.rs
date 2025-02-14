// src/lib.rs

//! LPU WiFi Manager: A CLI tool for managing Lovely Professional University WiFi connections.
//!
//! This crate provides functionality to:
//! - Store multiple LPU WiFi credentials securely
//! - Login to LPU WiFi network
//! - Logout from LPU WiFi network
//! - List stored accounts
//! - Cross-platform support for Windows and Linux
//!
//! # Usage
//! ```bash
//! llogin --help    # Show help
//! llogin login <myaccount>  # Login with stored account
//! llogin logout <myaccount>   # Logout from account
//! llogin list    # List stored accounts
//! ```

pub mod cli;
pub mod credentials;
pub mod login;
pub mod wifi;
