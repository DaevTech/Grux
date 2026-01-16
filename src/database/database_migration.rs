use sqlite::Connection;

use crate::{
    core::database_connection::get_database_connection,
    database::database_schema::{get_schema_version, set_schema_version},
};

pub fn migrate_database() -> i32 {
    // Get our current schema version from db
    let mut schema_version = get_schema_version();
    if schema_version < 1 {
        return 0;
    }

    let connection_result = get_database_connection();
    if let Err(_) = connection_result {
        panic!("Failed to get database connection for migration");
    }
    let connection = connection_result.unwrap();

    // Migration from 2 to 3
    if schema_version == 2 {
        migrate_db_2_to_3(&connection);
        schema_version = 3;
    }

    schema_version
}

fn migrate_db_2_to_3(connection: &Connection) {
    // Add "server_software_spoof" to "php_processors" table
    let alter_table_result = connection.execute("ALTER TABLE php_processors ADD COLUMN server_software_spoof TEXT NOT NULL DEFAULT '';");
    if let Err(e) = alter_table_result {
        panic!("Failed to migrate database from version 2 to 3: {}", e);
    }

    set_schema_version(3).expect("Failed to set schema version to 3 after migration");
}
