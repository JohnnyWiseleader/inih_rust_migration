use inih_rust_migration::de::from_ini_file;
use inih_rust_migration::parse_ini::IniParserConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    server: ServerSection,
    database: DatabaseSection,
}

#[derive(Debug, Deserialize)]
struct ServerSection {
    port: u16, // matches "port" in the INI
}

#[derive(Debug, Deserialize)]
struct DatabaseSection {
    user: String,     // matches "user" in the INI
    password: String, // matches "password" in the INI
}

fn main() {
    let config = IniParserConfig::default();
    let parsed: Config = from_ini_file("tests/sample.ini", &config).unwrap();
    println!("{:#?}", parsed);
    println!("Database user {}", parsed.database.user);
    println!("Port number {}", parsed.server.port);
    println!("Database password {}", parsed.database.password);
}
