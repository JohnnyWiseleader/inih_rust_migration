# INIH Rust Migration

This is a Rust wrapper (and eventual full port) of the [inih](https://github.com/benhoyt/inih) C INI parser.

## License

This project is licensed under either:

- MIT license (see `LICENSE`)
- or BSD 3-Clause license (see `LICENSE-BSD`, if included)

## Example

```rust
let parsed = inih_rust_migration::parse_ini_file("config.ini").unwrap();
println!("{}", parsed["server"]["port"]);
