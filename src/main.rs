use inih_rust_migration::parse_ini_file;

fn main() {
    match parse_ini_file("tests/sample.ini") {
        Some(data) => {
            println!("{:#?}", data);
            println!("DB user: {}", data["database"]["user"]);
            println!("DB password: {}", data["database"]["password"]);
        }
        None => {
            eprintln!("Failed to parse INI file");
        }
    }
}