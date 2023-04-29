//! Information about the crate at compile time

use imgui::Ui;

/// Project authors
const AUTHORS: Option<&str> = option_env!("CARGO_PKG_AUTHORS");
/// Compiled binary name
const BIN: Option<&str> = option_env!("CARGO_BIN_NAME");
/// Crate description
const DESCRIPTION: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
/// Crate name
const NAME: Option<&str> = option_env!("CARGO_PKG_NAME");
/// Crate repository
const REPO: Option<&str> = option_env!("CARGO_PKG_REPOSITORY");
/// Crate version
const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

pub(crate) fn about(ui: &Ui) {
    ui.popup("About", || {
        ui.text(format!(
            "{} v{}",
            NAME.unwrap_or("Chirp"),
            VERSION.unwrap_or("None"),
        ));
        ui.text(format!("Crafted by: {}", AUTHORS.unwrap_or("some people")));
        ui.text(REPO.unwrap_or("Repo unavailable"));
    });
}
