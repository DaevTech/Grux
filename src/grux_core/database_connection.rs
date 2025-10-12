
pub fn get_database_connection() -> Result<sqlite::Connection, String> {
    let connection = sqlite::open("./grux.db").map_err(|e| format!("Failed to open database connection: {}", e))?;
    Ok(connection)
}
