//! Common types used throughout the application.

/// Represents possible actions when selecting an account to log in or manage.
#[derive(Debug, PartialEq)]
pub enum LoginAction {
    /// Login using an existing account.
    UseExistingAccount(String),
    /// Create a new account.
    NewAccount,
    /// Do a one-time login without saving credentials.
    TemporaryLogin,
    /// Clear default account settings.
    ClearDefaults,
    /// Cancel the current operation.
    Cancel,
}

/// Indicates the broader context in which an account selection occurs.
#[derive(Debug)]
pub enum SelectionContext {
    /// Context for login-related operations
    Login,
    /// Context for default account operations
    Default,
    /// Context for other operations like update, show, etc.
    Other,
} 