//! Helper functions used across the application.

use dialoguer::{theme::ColorfulTheme, Confirm};

/// Prompts the user to confirm an action.
///
/// # Arguments
/// * `message` - The confirmation message to display
///
/// # Returns
/// `true` if the user confirms, `false` otherwise
pub fn confirm_action(message: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(false)
        .interact()
        .unwrap()
} 