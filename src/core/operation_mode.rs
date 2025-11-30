use std::sync::OnceLock;

use crate::core::command_line_args::cmd_get_operation_mode;

// Operation mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationMode {
    DEV,
    DEBUG,
    PRODUCTION,
    SPEEDTEST,
}
// Set the operation mode here
// Change this to OperationMode::Production when deploying to production
// Or set via an environment variable or config file as needed
pub static GRUX_OPERATION_MODE: OperationMode = OperationMode::PRODUCTION;

pub fn load_operation_mode() -> OperationMode {
    // Parse command line args
    let opmode = cmd_get_operation_mode();

    match opmode.as_str() {
        "DEV" => OperationMode::DEV,
        "DEBUG" => OperationMode::DEBUG,
        "PRODUCTION" => OperationMode::PRODUCTION,
        "SPEEDTEST" => OperationMode::SPEEDTEST,
        _ => OperationMode::PRODUCTION,
    }
}

static OPERATION_MODE_SINGLETON: OnceLock<OperationMode> = OnceLock::new();

pub fn get_operation_mode() -> OperationMode {
    *OPERATION_MODE_SINGLETON.get_or_init(|| load_operation_mode())
}
