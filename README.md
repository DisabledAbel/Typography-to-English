# Typography Translator (Two-way)

A Rust-based English ↔ Typography translator with CLI outputs (`plain`, `json`, `markdown`, `toon`) plus a static web UI.

## Vercel deployment

1. Push this repo to GitHub.
2. In Vercel, import the GitHub repository.
3. Keep the default settings (no build command required).
4. Deploy.

The app is fully client-side and ships with `vercel.json` rewrites so all routes resolve to `index.html`.

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
