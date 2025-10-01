// Operation mode
#[derive(Debug, Clone, Copy)]
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
    // Here you can implement logic to set the operation mode based on environment variables or config files
    // For example, read an environment variable and set GRUX_OPERATION_MODE accordingly
    OperationMode::DEV
}