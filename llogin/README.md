# LPU WiFi Manager

LPU WiFi Manager is a command-line tool for managing Lovely Professional University WiFi connections.

[![Crates.io](https://img.shields.io/crates/v/llogin.svg)](https://crates.io/crates/llogin)
[![Documentation](https://docs.rs/llogin/badge.svg)](https://docs.rs/llogin)

## Features

- Store multiple LPU WiFi credentials securely
- Cross-platform support for Windows and Linux
- Login to LPU WiFi network
- Logout from LPU WiFi network
- List stored accounts
- Secure credential storage in system config directory

## Usage

Run the program without any arguments to be prompted for an account ID and perform LPU login:

```bash
llogin
```

## You can also use the following options:

- `--help` or `-h`: Show the help message.
- `--version` or `-v`: Show the version of the program.
- `--account <account_id>` or `-a <account_id>`: Perform LPU login with the provided account ID.
- `--list` or `-l`: List all stored account IDs.
- `--logout <account_id>` or `-lg <account_id>`: Perform logout with the provided account ID.

## Installation

To install LPU WiFi Manager, you need to have Rust installed on your machine. You can then use cargo to install the package:

```bash
cargo install llogin
```

## License

This project is licensed under the MIT License.


