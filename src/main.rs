use std::env;

#[derive(Debug, Clone)]
struct Translation {
    original: String,
    translated: String,
    replacements: Vec<Replacement>,
}

#[derive(Debug, Clone)]
struct Replacement {
    from: &'static str,
    to: &'static str,
    count: usize,
}

fn typography_map() -> Vec<(&'static str, &'static str)> {
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

fn translate(input: &str) -> Translation {
    let mut text = input.to_string();
    let mut replacements = Vec::new();

    for (from, to) in typography_map() {
        let count = text.matches(from).count();
        if count > 0 {
            text = text.replace(from, to);
            replacements.push(Replacement { from, to, count });
        }
    }

    Translation {
        original: input.to_string(),
        translated: normalize_spacing(&text),
        replacements,
    }
}

fn normalize_spacing(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn to_json(t: &Translation) -> String {
    let replacements = t
        .replacements
        .iter()
        .map(|r| {
            format!(
                "{{\"from\":\"{}\",\"to\":\"{}\",\"count\":{}}}",
                escape_json(r.from),
                escape_json(r.to),
                r.count
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"original\":\"{}\",\"translated\":\"{}\",\"replacements\":[{}]}}",
        escape_json(&t.original),
        escape_json(&t.translated),
        replacements
    )
}

fn to_markdown(t: &Translation) -> String {
    let mut output = String::new();
    output.push_str("# Typography to English Translation\n\n");
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

fn to_toon(t: &Translation) -> String {
    format!(
        "POW! Typography translated!\n\
         --------------------------------\n\
         BEFORE: {}\n\
         AFTER : {}\n\
         SWAPS : {}\n\
         --------------------------------\n\
         ZAP! Done.\n",
        t.original,
        t.translated,
        t.replacements.len()
    )
}

fn usage() -> &'static str {
    "Usage: typography-to-english --input \"text\" --format <plain|json|markdown|toon>"
}

fn parse_args(args: &[String]) -> Result<(String, String), String> {
    let mut input = None;
    let mut format = String::from("plain");

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
            "--help" | "-h" => return Err(usage().to_string()),
            other => return Err(format!("Unknown argument: {other}")),
        }
        i += 1;
    }

    let input = input.ok_or_else(|| "--input is required".to_string())?;
    Ok((input, format))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (input, format) = match parse_args(&args) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}\n{}", usage());
            std::process::exit(1);
        }
    };

    let translated = translate(&input);
    let rendered = match format.as_str() {
        "plain" => translated.translated,
        "json" => to_json(&translated),
        "markdown" => to_markdown(&translated),
        "toon" => to_toon(&translated),
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
        let out = translate(input);
        assert_eq!(out.translated, "\"Hello...\" she said -- and smiled.");
        assert!(!out.replacements.is_empty());
    }

    #[test]
    fn renders_json_output() {
        let out = translate("© 2026");
        let json = to_json(&out);
        assert!(json.contains("\"translated\":\"(C) 2026\""));
    }

    #[test]
    fn renders_markdown_output() {
        let out = translate("• item");
        let md = to_markdown(&out);
        assert!(md.contains("# Typography to English Translation"));
        assert!(md.contains("`•` -> `*`"));
    }

    #[test]
    fn parse_args_requires_input() {
        let args = vec!["app".to_string(), "--format".to_string(), "json".to_string()];
        let result = parse_args(&args);
        assert!(result.is_err());
    }
}
