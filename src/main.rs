use serde::Serialize;
use std::env;

#[derive(Debug, Clone, Serialize)]
struct Translation {
    original: String,
    translated: String,
    replacements: Vec<Replacement>,
}

#[derive(Debug, Clone, Serialize)]
struct Replacement {
    from: &'static str,
    to: &'static str,
    count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    TypographyToEnglish,
    EnglishToTypography,
}

fn typography_map() -> Vec<(&'static str, &'static str)> {
    vec![
        (" -- ", "—"),
        (" - ", "–"),
        ("...", "…"),
        ("*", "•"),
        ("(TM)", "™"),
        ("(R)", "®"),
        ("(C)", "©"),
        ("1/2", "½"),
        ("1/4", "¼"),
        ("3/4", "¾"),
        ("fi", "ﬁ"),
        ("fl", "ﬂ"),
    ]
}

fn english_map() -> Vec<(&'static str, &'static str)> {
    vec![
        ("“", "\""),
        ("”", "\""),
        ("‘", "'"),
        ("’", "'"),
        ("—", " -- "),
        ("–", " - "),
        ("…", "..."),
        ("•", "*"),
        ("™", "(TM)"),
        ("®", "(R)"),
        ("©", "(C)"),
        ("½", "1/2"),
        ("¼", "1/4"),
        ("¾", "3/4"),
        ("ﬁ", "fi"),
        ("ﬂ", "fl"),
        (" ", " "), // non-breaking space
    ]
}

fn apply_contextual_quotes(text: &str, replacements: &mut Vec<Replacement>) -> String {
    let mut result = String::new();
    let mut double_quote_open = true;
    let mut single_quote_open = true;
    let mut double_count = 0;
    let mut single_count = 0;

    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                result.push_str(if double_quote_open { "\u{201c}" } else { "\u{201d}" });
                double_quote_open = !double_quote_open;
                double_count += 1;
            }
            '\'' => {
                // Check if this is an apostrophe (surrounded by word characters)
                let prev_is_word = result.chars().last().map_or(false, |c| c.is_alphanumeric());
                let next_is_word = chars.peek().map_or(false, |c| c.is_alphanumeric());

                if prev_is_word || next_is_word {
                    // It's an apostrophe - use right single quote
                    result.push_str("\u{2019}");
                } else {
                    // It's a paired quote - alternate between opening and closing
                    result.push_str(if single_quote_open { "\u{2018}" } else { "\u{2019}" });
                    single_quote_open = !single_quote_open;
                }
                single_count += 1;
            }
            _ => result.push(ch),
        }
    }

    if double_count > 0 {
        replacements.push(Replacement {
            from: "\"",
            to: "\u{201c}/\u{201d}",
            count: double_count,
        });
    }
    if single_count > 0 {
        replacements.push(Replacement {
            from: "'",
            to: "\u{2018}/\u{2019}",
            count: single_count,
        });
    }

    result
}

fn translate(input: &str, direction: Direction) -> Translation {
    let mut text = input.to_string();
    let mut replacements = Vec::new();

    match direction {
        Direction::EnglishToTypography => {
            // Apply contextual quotes first
            text = apply_contextual_quotes(&text, &mut replacements);

            // Then apply other replacements
            for (from, to) in typography_map() {
                let count = text.matches(from).count();
                if count > 0 {
                    text = text.replace(from, to);
                    replacements.push(Replacement { from, to, count });
                }
            }
        }
        Direction::TypographyToEnglish => {
            // Apply all replacements
            for (from, to) in english_map() {
                let count = text.matches(from).count();
                if count > 0 {
                    text = text.replace(from, to);
                    replacements.push(Replacement { from, to, count });
                }
            }
        }
    }

    Translation {
        original: input.to_string(),
        translated: normalize_spacing(&text),
        replacements,
    }
}

fn normalize_spacing(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
}

fn to_json(t: &Translation) -> String {
    serde_json::to_string_pretty(t).unwrap_or_else(|e| format!("JSON serialization error: {}", e))
}

fn to_markdown(t: &Translation, direction: Direction) -> String {
    let mut output = String::new();
    output.push_str("# Translation\n\n");
    output.push_str(&format!("## Direction\n> {}\n\n", direction_label(direction)));
    output.push_str("## Original\n");
    output.push_str(&format!("> {}\n\n", t.original));
    output.push_str("## Translated\n");
    output.push_str(&format!("> {}\n\n", t.translated));
    output.push_str("## Replacements\n");

    if t.replacements.is_empty() {
        output.push_str("- None\n");
    } else {
        for r in &t.replacements {
            output.push_str(&format!("- `{}` -> `{}` (x{})\n", r.from, r.to, r.count));
        }
    }
    output
}

fn to_toon(t: &Translation, direction: Direction) -> String {
    let total_swaps: usize = t.replacements.iter().map(|r| r.count).sum();
    format!(
        "POW! {} translated!\n\
         --------------------------------\n\
         BEFORE: {}\n\
         AFTER : {}\n\
         SWAPS : {}\n\
         --------------------------------\n\
         ZAP! Done.\n",
        direction_label(direction),
        t.original,
        t.translated,
        total_swaps
    )
}

fn direction_label(direction: Direction) -> &'static str {
    match direction {
        Direction::TypographyToEnglish => "Typography to English",
        Direction::EnglishToTypography => "English to Typography",
    }
}

fn usage() -> &'static str {
    "Usage: typography-to-english --input \"text\" --format <plain|json|markdown|toon> --direction <typography-to-english|english-to-typography>"
}

fn parse_args(args: &[String]) -> Result<(String, String, Direction), String> {
    let mut input = None;
    let mut format = String::from("plain");
    let mut direction = Direction::TypographyToEnglish;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing value for --input".to_string());
                }
                input = Some(args[i].clone());
            }
            "--format" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing value for --format".to_string());
                }
                format = args[i].to_lowercase();
            }
            "--direction" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing value for --direction".to_string());
                }
                direction = match args[i].as_str() {
                    "typography-to-english" => Direction::TypographyToEnglish,
                    "english-to-typography" => Direction::EnglishToTypography,
                    _ => return Err("Invalid value for --direction".to_string()),
                };
            }
            "--help" | "-h" => return Ok(("--help".to_string(), String::new(), direction)),
            other => return Err(format!("Unknown argument: {other}")),
        }
        i += 1;
    }

    let input = input.ok_or_else(|| "--input is required".to_string())?;
    Ok((input, format, direction))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (input, format, direction) = match parse_args(&args) {
        Ok(v) => {
            if v.0 == "--help" {
                println!("{}", usage());
                std::process::exit(0);
            }
            v
        }
        Err(e) => {
            eprintln!("{e}\n{}", usage());
            std::process::exit(1);
        }
    };

    let translated = translate(&input, direction);
    let rendered = match format.as_str() {
        "plain" => translated.translated,
        "json" => to_json(&translated),
        "markdown" => to_markdown(&translated, direction),
        "toon" => to_toon(&translated, direction),
        _ => {
            eprintln!("Invalid format: {format}\n{}", usage());
            std::process::exit(1);
        }
    };

    println!("{rendered}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_typographic_quotes_and_ellipsis() {
        let input = "“Hello…” she said — and smiled.";
        let out = translate(input, Direction::TypographyToEnglish);
        assert_eq!(out.translated, "\"Hello...\" she said -- and smiled.");
        assert!(!out.replacements.is_empty());
    }

    #[test]
    fn translates_english_to_typography() {
        let input = "\"Hello...\" (TM)";
        let out = translate(input, Direction::EnglishToTypography);
        assert_eq!(out.translated, "“Hello…” ™");
        assert!(!out.replacements.is_empty());
    }

    #[test]
    fn renders_json_output() {
        let out = translate("© 2026", Direction::TypographyToEnglish);
        let json = to_json(&out);
        assert!(json.contains("\"translated\": \"(C) 2026\""));
    }

    #[test]
    fn renders_markdown_output() {
        let out = translate("• item", Direction::TypographyToEnglish);
        let md = to_markdown(&out, Direction::TypographyToEnglish);
        assert!(md.contains("# Translation"));
        assert!(md.contains("`•` -> `*`"));
    }

    #[test]
    fn parse_args_requires_input() {
        let args = vec!["app".to_string(), "--format".to_string(), "json".to_string()];
        let result = parse_args(&args);
        assert!(result.is_err());
    }
}
