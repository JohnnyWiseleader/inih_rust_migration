use inih_rust_migration::ser::to_ini_file;
use std::collections::HashMap;

fn main() {
    let mut server = HashMap::new();
    server.insert("port".to_string(), 8088.to_string());

    let mut database = HashMap::new();
    database.insert("is_admin".to_string(), "true".to_string());
    database.insert("user".to_string(), "user123".to_string());
    database.insert("password".to_string(), "asdfghjkl".to_string());

    let mut config = HashMap::new();
    config.insert("server".to_string(), server);
    config.insert("database".to_string(), database);

    to_ini_file("tests/output.ini", &config).expect("Failed to write INI file");
}
