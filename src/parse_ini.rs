use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Lines};
use std::iter::Peekable;

/// Configuration options for the INI parser.
#[derive(Debug, Clone)]
pub struct IniParserConfig {
    pub allow_inline_comments: bool,
    pub allow_bom: bool,
    pub ignore_whitespace: bool,
    pub allow_quoted_with_equals: bool,
    pub allow_multiline: bool,
    pub strip_multiline_leading_ws: bool, // needs to be used with allow_multiline
}

impl Default for IniParserConfig {
    fn default() -> Self {
        IniParserConfig {
            allow_bom: false,
            allow_multiline: true,
            allow_inline_comments: true,
            allow_quoted_with_equals: false,
            ignore_whitespace: true,
            strip_multiline_leading_ws: false,
        }
    }
}

impl IniParserConfig {
    fn skip_bom(&self, line: String) -> String {
        if self.allow_bom && line.starts_with('\u{FEFF}') {
            line.trim_start_matches('\u{FEFF}').to_string()
        } else {
            line
        }
    }

    fn remove_inline_comments(&self, line: String) -> String {
        let mut value = line;
        if self.allow_inline_comments {
            // Skip if it starts with a comment
            if value.starts_with(';') || value.starts_with('#') {
                return value;
            }

            // Otherwise, strip inline comments (first occurrence of ';' or '#')
            value = strip_inline_comment(&value).to_string();
        }
        value
    }

    fn clean_quoted_value(&self, line: String) -> String {
        if self.allow_quoted_with_equals && line.contains('=') {
            let stripped = strip_comment_outside_quotes(&line);
            strip_outer_quotes(stripped).to_string()
        } else {
            line
        }
    }

    fn multi_line(
        &self,
        lines: &mut Peekable<Lines<&mut BufReader<File>>>,
        mut value: String,
    ) -> Result<String, std::io::Error> {
        if self.allow_multiline {
            while value.ends_with('\\') {
                value.pop(); // remove the trailing backslash

                if let Some(Ok(next_line)) = lines.next() {
                    let next_line = strip_inline_comment(&next_line);

                    if self.strip_multiline_leading_ws {
                        value += next_line.trim();
                    } else {
                        value += next_line.trim_end();
                    }
                } else {
                    break;
                }
            }
        }

        Ok(value)
    }
}

pub fn parse_ini_with_config(
    filename: &str,
    config: &IniParserConfig,
) -> Result<HashMap<String, HashMap<String, String>>, std::io::Error> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut lines = std::io::BufRead::lines(&mut reader).peekable();
    let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current_section = "default".to_string();

    while let Some(Ok(mut line)) = lines.next() {
        line = line.trim().to_string();
        line = config.skip_bom(line);

        match classify_line(&line) {
            LineType::Comment => continue,
            LineType::Section(name) => {
                current_section = name;
            }
            LineType::KeyValue(key, mut value) => {
                // Multiline continuation and inline comments
                value = config.multi_line(&mut lines, value)?;

                // parse quoted with equals strings, e.g.
                // handles values like: value = "a=very=custom=user" 
                value = config.clean_quoted_value(value);

                // alllow inline comments
                value = config.remove_inline_comments(value);

                // final result
                result
                    .entry(current_section.clone())
                    .or_default()
                    .insert(key, value);
            }
        }
    }
    Ok(result)
}

/// Classifies a line in the INI file.
enum LineType {
    /// Comment line or empty line.
    Comment,
    /// Section header like [section].
    Section(String),
    /// Key-value pair like key=value.
    KeyValue(String, String),
}

fn classify_line(line: &str) -> LineType {
    if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
        LineType::Comment
    } else if line.starts_with('[') && line.ends_with(']') {
        LineType::Section(line[1..line.len() - 1].to_string())
    } else if let Some(pos) = line.find('=') {
        let key = line[..pos].trim().to_string();
        let value = line[pos + 1..].trim().to_string();
        LineType::KeyValue(key, value)
    } else {
        LineType::Comment
    }
}

fn strip_inline_comment(line: &str) -> &str {
    line.split_terminator(|c| c == ';' || c == '#')
        .next()
        .unwrap_or(line)
        .trim_end()
}

fn strip_comment_outside_quotes(line: &str) -> &str {
    let mut in_quotes = false;

    for (i, c) in line.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            ';' | '#' if !in_quotes => return &line[..i].trim_end(),
            _ => {}
        }
    }

    line.trim_end()
}

fn strip_outer_quotes(line: &str) -> &str {
    let trimmed = line.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

/// parse_ini with default config settings
pub fn parse_ini(
    filename: &str,
) -> Result<HashMap<String, HashMap<String, String>>, std::io::Error> {
    let config = IniParserConfig::default();
    parse_ini_with_config(filename, &config)
}
