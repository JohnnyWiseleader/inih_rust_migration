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

trait IgnoreWhitespace {
    fn trim_line(&self, line: &str) -> String;
}

impl IgnoreWhitespace for IniParserConfig {
    fn trim_line(&self, line: &str) -> String {
        if self.ignore_whitespace {
            line.trim().to_string()
        } else {
            line.to_string()
        }
    }
}

trait AllowBOM {
    fn skip_bom(&self, line: String) -> String;
}

impl AllowBOM for IniParserConfig {
    fn skip_bom(&self, line: String) -> String {
        if self.allow_bom && line.starts_with('\u{FEFF}') {
            line.trim_start_matches('\u{FEFF}').to_string()
        } else {
            line
        }
    }
}

trait AllowInlineComments {
    fn remove_inline_comments(&self, line: String) -> String;
}

impl AllowInlineComments for IniParserConfig {
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
}

trait AllowQuotedWithEquals {
    fn quoted_with_equals(&self, line: String) -> String;
}

impl AllowQuotedWithEquals for IniParserConfig {
    fn quoted_with_equals(&self, line: String) -> String {
        let mut value = line;
        if self.allow_quoted_with_equals {
            if value.contains('=') {
                value = strip_inline_comment(&value.to_string()).to_string();
                value = value.trim().replace("\"", "").to_string();
            }
        }
        value
    }
}
trait AllowMultiline {
    fn multi_line(
        &self,
        lines: &mut Peekable<Lines<&mut BufReader<File>>>,
        value: String,
    ) -> Result<String, std::io::Error>;
}

impl AllowMultiline for IniParserConfig {
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
        line = config.trim_line(&line);
        line = config.skip_bom(line);

        match classify_line(&line) {
            LineType::Comment | LineType::Empty => continue,
            LineType::Section(name) => {
                current_section = name;
            }
            LineType::KeyValue(key, mut value) => {
                // Multiline continuation and inline comments
                value = config.multi_line(&mut lines, value)?;

                // parse quoted with equals strings, e.g.
                // value = "a=very=custom=user"
                value = config.quoted_with_equals(value);

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

enum LineType {
    Empty,
    Comment,
    Section(String),
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
        LineType::Empty
    }
}

fn strip_inline_comment(line: &str) -> &str {
    line.split_terminator(|c| c == ';' || c == '#')
        .next()
        .unwrap_or(line)
        .trim_end()
}

/// parse_ini with default config settings
pub fn parse_ini(
    filename: &str,
) -> Result<HashMap<String, HashMap<String, String>>, std::io::Error> {
    let config = IniParserConfig::default();
    parse_ini_with_config(filename, &config)
}
