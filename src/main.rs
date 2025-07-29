mod grux_configuration;
mod conf_structure;
mod grux_http_server;
mod grux_log;
use log::{error, info};

fn main() {
    // Initialize logging
    let _log_handle = crate::grux_log::init_logging().unwrap();

    // Starting grux
    let version = env!("CARGO_PKG_VERSION", "unknown");
    info!("Starting grux {}...", version);

    // Load configuration and check for errors
    let configuration = grux_configuration::load_configuration();
    if let Err(e) = configuration {
        error!("Failed to load configuration: {}", e);
        std::process::exit(1);
    }
    let configuration_handle = configuration.unwrap();
    info!("Configuration loaded successfully.");

    // Load the admin services endpoints


    // Init server bindings and start serving those bits
    if let Err(e) = crate::grux_http_server::initialize_server(&configuration_handle) {
        error!("Error initializing bindings: {}", e);
        error!("Make sure the port(s) is not already in use and that you have the necessary permissions to bind to it.");
        std::process::exit(1);
    }
}
