use inih_rust_migration::{parse_ini, parse_ini_with_config, IniParserConfig};

fn get_test_path(filename: &str) -> String {
    format!("./tests/test_data/{}", filename)
}

#[test]
fn test_basic_ini() {
    let config = IniParserConfig::default();
    let parsed = parse_ini_with_config(&get_test_path("test_default.ini"), &config).unwrap();

    assert_eq!(parsed["server"]["host"], "localhost");
    assert_eq!(parsed["server"]["port"], "8080");
}

#[test]
fn test_bom_handling() {
    let mut config = IniParserConfig::default();
    config.allow_bom = true;

    let parsed = parse_ini_with_config(&get_test_path("bom.ini"), &config).unwrap();
    assert_eq!(parsed["database"]["user"], "admin");
}

#[test]
fn test_multiline_values() {
    let mut config = IniParserConfig::default();
    config.allow_multiline = true;

    let parsed = parse_ini_with_config(&get_test_path("test_multiline.ini"), &config).unwrap();
    assert_eq!(
        parsed["description"]["text"],
        "This is a long line that    continues on the next line and even more."
    );
}

#[test]
fn test_multiline_values_leading_ws_removed() {
    let mut config = IniParserConfig::default();
    // Note: in order to use strip_multiline_leading_ws
    //          you must also use allow_multiline config
    config.allow_multiline = true;
    config.strip_multiline_leading_ws = true;

    let parsed = parse_ini_with_config(&get_test_path("test_multiline.ini"), &config).unwrap();
    assert_eq!(
        parsed["description"]["text"],
        "This is a long line thatcontinues on the nextline and even more."
    );
}

#[test]
fn test_inline_comments_ignored() {
    let mut config = IniParserConfig::default();
    config.allow_inline_comments = false;

    let parsed = parse_ini_with_config(&get_test_path("inline_comments.ini"), &config).unwrap();
    assert_eq!(
        parsed["network"]["address"],
        "192.168.1.1 ; default gateway"
    );
    assert_eq!(parsed["network"]["dns"], "8.8.8.8 # google dns");
}

#[test]
fn test_inline_comments_trimmed() {
    let mut config = IniParserConfig::default();
    config.allow_inline_comments = true;

    let parsed = parse_ini_with_config(&get_test_path("inline_comments.ini"), &config).unwrap();
    assert_eq!(parsed["network"]["address"], "192.168.1.1");
    assert_eq!(parsed["network"]["dns"], "8.8.8.8");
}
#[test]
fn test_quoted_string_with_equals() {
    let mut config = IniParserConfig::default();
    config.allow_quoted_with_equals = true;

    let parsed = parse_ini_with_config(&get_test_path("quoted_equals.ini"), &config).unwrap();
  
    assert_eq!(parsed["user"]["description"], "a=very=custom\"=user");
}


////////////////////////////////////////////////////////////////////
#[test]
fn test_empty_values() {
    let parsed = parse_ini(
        &get_test_path("empty_values.ini"),
    )
    .unwrap();
    assert_eq!(parsed["settings"]["empty"], "");
    assert_eq!(parsed["settings"]["nonempty"], "value");
}

#[test]
fn test_multiple_sections() {
    let parsed = parse_ini(
        &get_test_path("multi_section.ini"),
    )
    .unwrap();
    assert_eq!(parsed["dev"]["key"], "value1");
    assert_eq!(parsed["prod"]["key"], "value2");
}

#[test]
fn test_multiline_with_inline_comments() {
    let mut config = IniParserConfig::default();
    config.allow_multiline = true;
    config.allow_inline_comments = true;

    let parsed =
        parse_ini_with_config(&get_test_path("multiline_inline_comment.ini"), &config).unwrap();
    assert_eq!(
        parsed["notes"]["content"],
        "Line one continues Line two ends here"
    );
}
