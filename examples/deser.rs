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
    admin: bool,
    user: String,
    password: String,
}

fn main() {
    let config = IniParserConfig::default();
    let parsed: Config = from_ini_file("tests/sample.ini", &config).unwrap();
    println!("{:#?}", parsed);
    println!("Port number = {}", parsed.server.port);
    println!("Database user is {}", parsed.database.user);
    println!("Database user is admin {}", parsed.database.admin);
    println!("Database password is {}", parsed.database.password);
}
