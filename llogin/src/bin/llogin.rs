//! Main binary entry point for the LPU WiFi Manager.

use clap::Parser;
use colored::*;
use llogin::{cli::Cli, commands, wifi};
use std::process;

fn main() {
    // Check WiFi connectivity
    if !wifi::check_lpu_wifi() {
        println!("{}", "Not connected to LPU WiFi. Exiting.".bright_red());
        process::exit(1);
    }

    // Parse CLI arguments and handle commands
    let cli = Cli::parse();
    commands::handle_command(cli);
}