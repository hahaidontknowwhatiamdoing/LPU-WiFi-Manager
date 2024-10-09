mod wifi;
mod credentials;
mod login;
mod cli;

use std::env;
use std::process;

fn main() {
    if !wifi::check_lpu_wifi() {
        println!("Not connected to LPU WiFi. Exiting.");
        process::exit(1);
    }

    let args: Vec<String> = env::args().collect();
    cli::handle_args(args);
}
