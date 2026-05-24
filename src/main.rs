use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;

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
        ("“", "\""), ("”", "\""), ("‘", "'"), ("’", "'"), ("—", " -- "), ("–", " - "),
        ("…", "..."), ("•", "*"), ("™", "(TM)"), ("®", "(R)"), ("©", "(C)"), ("½", "1/2"),
        ("¼", "1/4"), ("¾", "3/4"), ("ﬁ", "fi"), ("ﬂ", "fl"), (" ", " "),
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
    Translation { original: input.to_string(), translated: normalize_spacing(&text), replacements }
}

fn normalize_spacing(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
}

fn escape_json(s: &str) -> String { s.replace('\\', "\\\\").replace('"', "\\\"") }

fn to_json(t: &Translation) -> String {
    let replacements = t.replacements.iter().map(|r| {
        format!("{{\"from\":\"{}\",\"to\":\"{}\",\"count\":{}}}", escape_json(r.from), escape_json(r.to), r.count)
    }).collect::<Vec<_>>().join(",");
    format!("{{\"original\":\"{}\",\"translated\":\"{}\",\"replacements\":[{}]}}", escape_json(&t.original), escape_json(&t.translated), replacements)
}

fn to_markdown(t: &Translation) -> String {
    let mut output = String::new();
    output.push_str("# Typography to English Translation\n\n## Original\n");
    output.push_str(&format!("> {}\n\n## Translated\n> {}\n\n## Replacements\n", t.original, t.translated));
    if t.replacements.is_empty() { output.push_str("- None\n"); }
    else { for r in &t.replacements { output.push_str(&format!("- `{}` -> `{}` (x{})\n", r.from, r.to, r.count)); } }
    output
}

fn to_toon(t: &Translation) -> String {
    format!("POW! Typography translated!\n--------------------------------\nBEFORE: {}\nAFTER : {}\nSWAPS : {}\n--------------------------------\nZAP! Done.\n", t.original, t.translated, t.replacements.len())
}

enum Mode { Cli { input: String, format: String }, Serve { port: u16 } }

fn usage() -> &'static str { "Usage:\n  typography-to-english --input \"text\" --format <plain|json|markdown|toon>\n  typography-to-english --serve [--port 8080]" }

fn parse_args(args: &[String]) -> Result<Mode, String> {
    let (mut input, mut format, mut serve, mut port) = (None, String::from("plain"), false, 8080u16);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => { i += 1; if i >= args.len() { return Err("Missing value for --input".into()); } input = Some(args[i].clone()); }
            "--format" => { i += 1; if i >= args.len() { return Err("Missing value for --format".into()); } format = args[i].to_lowercase(); }
            "--serve" => serve = true,
            "--port" => { i += 1; if i >= args.len() { return Err("Missing value for --port".into()); }
                port = args[i].parse::<u16>().map_err(|_| "--port must be 0-65535".to_string())?; }
            "--help" | "-h" => return Err(usage().to_string()),
            other => return Err(format!("Unknown argument: {other}")),
        }
        i += 1;
    }
    if serve { return Ok(Mode::Serve { port }); }
    Ok(Mode::Cli { input: input.ok_or_else(|| "--input is required unless --serve is used".to_string())?, format })
}

fn index_html() -> &'static str { r#"<!doctype html><html><head><meta charset='UTF-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>Typography to English</title><style>:root{--bg:#1f140f;--panel:#2b1b14;--accent:#ff8c2a;--text:#f8efe8;--muted:#d9bba4;--border:#5c3a24}*{box-sizing:border-box}body{margin:0;font-family:system-ui;background:linear-gradient(140deg,#180f0a,#2d1a12);color:var(--text);min-height:100vh;display:grid;place-items:center;padding:24px}.card{width:min(900px,100%);background:var(--panel);border:1px solid var(--border);border-radius:14px;padding:20px}h1{margin:0 0 8px;color:var(--accent)}p{margin:0 0 16px;color:var(--muted)}textarea,pre{width:100%;border-radius:10px;border:1px solid var(--border);background:#1a110d;color:var(--text);padding:12px}textarea{min-height:120px}button{border:0;border-radius:10px;background:var(--accent);color:#28170d;padding:11px 16px;font-weight:700;cursor:pointer;margin:12px 0}pre{white-space:pre-wrap;color:#ffd9b2}</style></head><body><main class='card'><h1>Typography to English</h1><p>Simple and clean converter with dark orange theme.</p><textarea id='input' placeholder='Paste text like: “Rust™ — ﬁne… really”'></textarea><button id='convert'>Translate</button><h3>Translated</h3><pre id='output'></pre><h3>JSON</h3><pre id='json'></pre></main><script>const i=document.getElementById('input'),o=document.getElementById('output'),j=document.getElementById('json');document.getElementById('convert').onclick=async()=>{const r=await fetch('/api/translate',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({text:i.value||''})});const d=await r.json();o.textContent=d.translated;j.textContent=JSON.stringify(d,null,2);};</script></body></html>"# }

fn parse_text_from_body(body: &str) -> String {
    if let Some(pos) = body.find("\"text\":") {
        let rest = &body[pos + 7..];
        if let Some(start) = rest.find('"') {
            let s = &rest[start + 1..];
            if let Some(end) = s.find('"') { return s[..end].replace("\\\"", "\"").replace("\\n", "\n"); }
        }
    }
    String::new()
}

fn http_response(status: &str, content_type: &str, body: &str) -> String {
    format!("HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body)
}

fn run_server(port: u16) {
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("bind failed");
    println!("Web UI running at http://127.0.0.1:{port}");
    for stream in listener.incoming() {
        let mut stream = match stream { Ok(s) => s, Err(_) => continue };
        let mut buf = [0u8; 8192];
        let n = match stream.read(&mut buf) { Ok(n) => n, Err(_) => continue };
        let req = String::from_utf8_lossy(&buf[..n]);
        let response = if req.starts_with("GET / ") {
            http_response("200 OK", "text/html; charset=utf-8", index_html())
        } else if req.starts_with("POST /api/translate ") {
            let body = req.split("\r\n\r\n").nth(1).unwrap_or("");
            let text = parse_text_from_body(body);
            let out = to_json(&translate(&text));
            http_response("200 OK", "application/json", &out)
        } else {
            http_response("404 Not Found", "text/plain; charset=utf-8", "Not found")
        };
        let _ = stream.write_all(response.as_bytes());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = match parse_args(&args) { Ok(v) => v, Err(e) => { eprintln!("{e}\n{}", usage()); std::process::exit(1); } };
    match mode {
        Mode::Serve { port } => run_server(port),
        Mode::Cli { input, format } => {
            let translated = translate(&input);
            let rendered = match format.as_str() {
                "plain" => translated.translated,
                "json" => to_json(&translated),
                "markdown" => to_markdown(&translated),
                "toon" => to_toon(&translated),
                _ => { eprintln!("Invalid format: {format}\n{}", usage()); std::process::exit(1); }
            };
            println!("{rendered}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn translates_typographic_quotes_and_ellipsis(){let out=translate("“Hello…” she said — and smiled.");assert_eq!(out.translated,"\"Hello...\" she said -- and smiled.");}
    #[test] fn renders_json_output(){let out=translate("© 2026");assert!(to_json(&out).contains("\"translated\":\"(C) 2026\""));}
    #[test] fn renders_markdown_output(){let md=to_markdown(&translate("• item"));assert!(md.contains("`•` -> `*`"));}
    #[test] fn parse_args_requires_input_unless_serving(){assert!(parse_args(&["app".into(),"--format".into(),"json".into()]).is_err());assert!(parse_args(&["app".into(),"--serve".into()]).is_ok());}
}
