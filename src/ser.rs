use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Serializes a nested `HashMap<String, HashMap<String, String>>` into INI format and writes to a file.
pub fn to_ini_file<P: AsRef<Path>>(
    path: P,
    data: &HashMap<String, HashMap<String, String>>,
) -> io::Result<()> {
    let mut file = File::create(path)?;

    for (section, kv_pairs) in data {
        writeln!(file, "[{}]", section)?;
        for (key, value) in kv_pairs {
            writeln!(file, "{} = {}", key, value)?;
        }
        writeln!(file)?; // extra newline between sections
    }

    Ok(())
}
