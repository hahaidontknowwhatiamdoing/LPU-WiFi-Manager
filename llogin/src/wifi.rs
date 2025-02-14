// src/wifi.rs

//! WiFi connection detection module for LPU WiFi Manager
//!
//! This module handles:
//! - Cross-platform WiFi connection detection
//! - Linux detection using nmcli, iwconfig, iw, or /proc/net/wireless
//! - Windows detection using netsh
//! - macOS detection using airport utility

use std::fs;
use std::process::Command;

/// Checks if the system is currently connected to LPU WiFi or Block WiFi
///
/// # Returns
/// `true` if connected to an SSID starting with "LPU" or "Block", `false` otherwise
///
/// # Platform Support
/// - Linux: Uses multiple methods (nmcli, iwconfig, iw, /proc/net/wireless)
/// - Windows: Uses netsh to check connection status
/// - macOS: Uses airport utility to check connection status
pub fn check_lpu_wifi() -> bool {
    if cfg!(target_os = "windows") {
        check_lpu_wifi_windows()
    } else if cfg!(target_os = "macos") {
        check_lpu_wifi_macos()
    } else {
        check_lpu_wifi_linux()
    }
}

/// Checks LPU/Block WiFi connection on Linux systems using various methods
///
/// # Implementation Details
/// Tries multiple methods in order:
/// 1. nmcli (NetworkManager)
/// 2. iwconfig
/// 3. iw
/// 4. /proc/net/wireless
///
/// # Returns
/// `true` if connected to LPU/Block WiFi, `false` if not connected or on error
fn check_lpu_wifi_linux() -> bool {
    // Try nmcli first
    if let Some(connected) = check_wifi_nmcli() {
        return connected;
    }

    // Try iwconfig if nmcli fails
    if let Some(connected) = check_wifi_iwconfig() {
        return connected;
    }

    // Try iw if iwconfig fails
    if let Some(connected) = check_wifi_iw() {
        return connected;
    }

    // Last resort: check /proc/net/wireless
    check_wifi_proc()
}

/// Checks WiFi using nmcli
fn check_wifi_nmcli() -> Option<bool> {
    if let Ok(output) = Command::new("nmcli")
        .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
    {
        if let Ok(output_string) = String::from_utf8(output.stdout) {
            return Some(output_string.lines().any(|line| {
                line.starts_with("yes:")
                    && (line[4..].starts_with("LPU") || line[4..].starts_with("Block"))
            }));
        }
    }
    None
}

/// Checks WiFi using iwconfig
fn check_wifi_iwconfig() -> Option<bool> {
    if let Ok(output) = Command::new("iwconfig").output() {
        if let Ok(output_string) = String::from_utf8(output.stdout) {
            return Some(output_string.lines().any(|line| {
                line.contains("ESSID:") && (line.contains("\"LPU") || line.contains("\"Block"))
            }));
        }
    }
    None
}

/// Checks WiFi using iw
fn check_wifi_iw() -> Option<bool> {
    // First, list wireless interfaces
    if let Ok(output) = Command::new("iw").args(&["dev"]).output() {
        if let Ok(output_string) = String::from_utf8(output.stdout) {
            // Find the first wireless interface
            if let Some(interface) = output_string
                .lines()
                .find(|line| line.contains("Interface"))
                .and_then(|line| line.split_whitespace().nth(1))
            {
                // Check the link information for this interface
                if let Ok(link_output) = Command::new("iw")
                    .args(&["dev", interface, "link"])
                    .output()
                {
                    if let Ok(link_string) = String::from_utf8(link_output.stdout) {
                        return Some(link_string.lines().any(|line| {
                            line.contains("SSID:")
                                && (line.contains("LPU") || line.contains("Block"))
                        }));
                    }
                }
            }
        }
    }
    None
}

/// Checks WiFi using /proc/net/wireless
fn check_wifi_proc() -> bool {
    if let Ok(contents) = fs::read_to_string("/proc/net/wireless") {
        // If there's any wireless interface with non-zero link quality
        return contents.lines().skip(2).any(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                // Check if link quality (third field) is greater than 0
                if let Ok(link_quality) = parts[2].trim_end_matches('.').parse::<f32>() {
                    return link_quality > 0.0;
                }
            }
            false
        });
    }
    false
}

/// Checks LPU/Block WiFi connection on Windows systems using netsh
fn check_lpu_wifi_windows() -> bool {
    if let Ok(output) = Command::new("netsh")
        .args(&["wlan", "show", "interfaces"])
        .output()
    {
        if let Ok(output_string) = String::from_utf8(output.stdout) {
            return output_string
                .lines()
                .any(|line| line.contains("LPU") || line.contains("Block"));
        }
    }
    false
}

/// Checks LPU/Block WiFi connection on macOS systems using airport utility
fn check_lpu_wifi_macos() -> bool {
    // First try the primary method using airport utility
    if let Some(connected) = check_wifi_airport() {
        return connected;
    }

    // Fallback to networksetup if airport utility is not available
    check_wifi_networksetup()
}

/// Checks WiFi using the airport utility on macOS
fn check_wifi_airport() -> Option<bool> {
    let airport_path =
        "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport";

    if let Ok(output) = Command::new(airport_path)
        .arg("-I") // Get current WiFi information
        .output()
    {
        if let Ok(output_string) = String::from_utf8(output.stdout) {
            // Look for SSID line in the output
            for line in output_string.lines() {
                if line.contains(" SSID: ") {
                    let ssid = line.split("SSID: ").nth(1).unwrap_or("").trim();
                    return Some(ssid.starts_with("LPU") || ssid.starts_with("Block"));
                }
            }
        }
    }
    None
}

/// Fallback method using networksetup on macOS
fn check_wifi_networksetup() -> bool {
    // First, get the WiFi device name
    if let Ok(output) = Command::new("networksetup")
        .args(&["-listallhardwareports"])
        .output()
    {
        if let Ok(output_string) = String::from_utf8(output.stdout) {
            // Find the Wi-Fi device name
            let mut wifi_device = None;
            let lines: Vec<&str> = output_string.lines().collect();

            for (i, line) in lines.iter().enumerate() {
                if line.contains("Wi-Fi") || line.contains("AirPort") {
                    if i + 1 < lines.len() {
                        if let Some(device) = lines[i + 1].strip_prefix("Device: ") {
                            wifi_device = Some(device.to_string());
                            break;
                        }
                    }
                }
            }

            // If we found the WiFi device, check its current network
            if let Some(device) = wifi_device {
                if let Ok(ssid_output) = Command::new("networksetup")
                    .args(&["-getairportnetwork", &device])
                    .output()
                {
                    if let Ok(ssid_string) = String::from_utf8(ssid_output.stdout) {
                        return ssid_string.contains("LPU") || ssid_string.contains("Block");
                    }
                }
            }
        }
    }
    false
}
