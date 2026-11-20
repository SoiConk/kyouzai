#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod alpha_mask;
mod application;
mod input;
mod persistence;
mod pet;
mod sleep_manager;
mod tray;
mod window;

use application::Application;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    Application::new().run()
}
