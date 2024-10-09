use std::process::Command;

pub fn check_lpu_wifi() -> bool {
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
