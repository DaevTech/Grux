use std::sync::OnceLock;

use crate::core::command_line_args::get_command_line_args;

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
    let cli = get_command_line_args();

    cli.opmode.map(|s| match s.as_str() {
        "DEV" => OperationMode::DEV,
        "DEBUG" => OperationMode::DEBUG,
        "PRODUCTION" => OperationMode::PRODUCTION,
        "SPEEDTEST" => OperationMode::SPEEDTEST,
        _ => OperationMode::DEV,
    }).unwrap_or(OperationMode::PRODUCTION)
}

static OPERATION_MODE_SINGLETON: OnceLock<OperationMode> = OnceLock::new();

pub fn get_operation_mode() -> OperationMode {
    *OPERATION_MODE_SINGLETON.get_or_init(|| load_operation_mode())
}