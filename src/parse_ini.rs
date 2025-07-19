use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Configuration options for the INI parser.
#[derive(Debug, Clone)]
pub struct IniParserConfig {
    pub allow_inline_comments: bool,
    pub allow_bom: bool,
    pub ignore_whitespace: bool,
    pub allow_multiline: bool,
    pub strip_multiline_leading_ws: bool, // needs to be used with allow_multiline
}

impl Default for IniParserConfig {
    fn default() -> Self {
        IniParserConfig {
            allow_bom: false,
            allow_multiline: true,
            allow_inline_comments: true,
            ignore_whitespace: true,
            strip_multiline_leading_ws: false,
        }
    }
}

pub fn parse_ini_with_config(
    filename: &str,
    config: &IniParserConfig,
) -> Result<HashMap<String, HashMap<String, String>>, std::io::Error> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current_section = String::from("default");

    let mut line_buf = String::new();
    while reader.read_line(&mut line_buf)? > 0 {
        let mut line = line_buf.trim_end().to_string(); // remove newline
        line_buf.clear(); // prepare for next read

        if config.ignore_whitespace {
            line = line.trim().to_string();
        }

        // Handle BOM
        if config.allow_bom && line.starts_with('\u{FEFF}') {
            line = line.trim_start_matches('\u{FEFF}').to_string();
        }

        // Skip comments and blank lines
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
        } else if let Some(pos) = line.find('=') {
            line = strip_inline_comment(&line, config.allow_inline_comments);
            let key = line[..pos].trim().to_string();
            let mut value = line[pos + 1..].trim().replace("\"", "").to_string();

            if config.allow_multiline {
                loop {
                    if !value.ends_with('\\') {
                        break;
                    }

                    value.pop(); // remove the trailing backslash

                    let mut next_line = String::new();
                    if reader.read_line(&mut next_line)? == 0 {
                        break;
                    }
                    next_line = strip_inline_comment(&next_line, config.allow_inline_comments);

                    if config.strip_multiline_leading_ws {
                        value += next_line.trim();
                    } else {
                        value += next_line.trim_end();
                    }
                }
            }

            result
                .entry(current_section.clone())
                .or_default()
                .insert(key, value);
        }
    }

    Ok(result)
}

fn strip_inline_comment(line: &str, allow_inline_comments: bool) -> String {
    if !allow_inline_comments {
        return line.to_string();
    }

    let comment_markers = [';', '#'];
    let mut min_pos = line.len();

    for marker in comment_markers {
        if let Some(pos) = line.find(marker) {
            min_pos = min_pos.min(pos);
        }
    }

    line[..min_pos].trim_end().to_string()
}

/// parse_ini with default config settings
pub fn parse_ini(
    filename: &str,
) -> Result<HashMap<String, HashMap<String, String>>, std::io::Error> {
    let config = IniParserConfig::default();
    parse_ini_with_config(filename, &config)
}
