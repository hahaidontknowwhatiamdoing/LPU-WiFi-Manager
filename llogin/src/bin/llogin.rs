use llogin;
use std::env;
use std::process;

fn main() {
    if !llogin::wifi::check_lpu_wifi() {
        println!("Not connected to LPU WiFi. Exiting.");
        process::exit(1);
    }

    let args: Vec<String> = env::args().collect();
    llogin::cli::handle_args(args);
}