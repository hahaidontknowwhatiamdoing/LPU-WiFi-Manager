//! WiFi connection detection module for LPU WiFi Manager
//!
//! This module handles:
//! - Cross-platform WiFi connection detection
//! - Linux detection using nmcli
//! - Windows detection using netsh

use std::process::Command;

/// Checks if the system is currently connected to LPU WiFi
///
/// # Returns
/// `true` if connected to an SSID starting with "LPU", `false` otherwise
///
/// # Platform Support
/// - Linux: Uses nmcli to check connection status
/// - Windows: Uses netsh to check connection status
///
/// # Examples
/// ```no_run
/// if llogin::wifi::check_lpu_wifi() {
///     println!("Connected to LPU WiFi");
/// } else {
///     println!("Not connected to LPU WiFi");
/// }
/// ```
pub fn check_lpu_wifi() -> bool {
    if cfg!(target_os = "windows") {
        check_lpu_wifi_windows()
    } else {
        check_lpu_wifi_linux()
    }
}

/// Checks LPU WiFi connection on Linux systems using nmcli
///
/// # Implementation Details
/// - Uses nmcli command line tool
/// - Checks for active WiFi connections
/// - Looks for SSID starting with "LPU"
///
/// # Returns
/// `true` if connected to LPU WiFi, `false` if not connected or on error
fn check_lpu_wifi_linux() -> bool {
    if let Ok(output) = Command::new("nmcli")
        .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
    {
        if let Ok(output_string) = String::from_utf8(output.stdout) {
            return output_string
                .lines()
                .any(|line| line.starts_with("yes:LPU"));
        }
    }
    false
}

/// Checks LPU WiFi connection on Windows systems using netsh
///
/// # Implementation Details
/// - Uses Windows netsh command
/// - Checks wireless interface state
/// - Looks for SSID containing "LPU"
///
/// # Returns
/// `true` if connected to LPU WiFi, `false` if not connected or on error
fn check_lpu_wifi_windows() -> bool {
    if let Ok(output) = Command::new("netsh")
        .args(&["wlan", "show", "interfaces"])
        .output()
    {
        if let Ok(output_string) = String::from_utf8(output.stdout) {
            return output_string.lines().any(|line| line.contains("LPU"));
        }
    }
    false
}
