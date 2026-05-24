# Typography Translator (Two-way)

A Rust-based English ↔ Typography translator with CLI outputs (`plain`, `json`, `markdown`, `toon`) plus a GitHub Pages-compatible web UI.

## GitHub Pages setup

1. Push this repo to GitHub.
2. Go to **Settings → Pages**.
3. Under **Build and deployment**, choose **Deploy from a branch**.
4. Select your branch and set folder to **`/docs`**.
5. Save. Your app will be served as a static site from `docs/index.html`.

The Pages web UI is fully client-side JavaScript and does not require the Rust HTTP server.

## CLI usage

```bash
cargo run -- --input '“Rust™ — ﬁne… really”' --format json --direction typography-to-english
cargo run -- --input '“Rust™ — ﬁne… really”' --format markdown --direction typography-to-english
cargo run -- --input '"Rust..." (TM) is fine' --format plain --direction english-to-typography
```

## Local web server usage (optional)

```bash
cargo run -- --serve --port 8080
```
