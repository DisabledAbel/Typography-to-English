#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Keep GitHub Pages copy in sync.
cp index.html docs/index.html

python - <<'PY'
import re
from pathlib import Path

for file in ["index.html", "docs/index.html"]:
    html = Path(file).read_text(encoding="utf-8")
    if 'id="convert"' in html:
        raise SystemExit(f"Found removed translate button in {file}")

    script = re.search(r"<script>([\s\S]*?)</script>", html)
    if not script:
        raise SystemExit(f"No inline script found in {file}")

    Path(f"/tmp/{Path(file).name}.js").write_text(script.group(1), encoding="utf-8")

print("HTML checks passed for index.html and docs/index.html")
PY

node --check /tmp/index.html.js
node --check /tmp/index.html.js

node - <<'NODE'
const fs = require('fs');
const vm = require('vm');
const code = fs.readFileSync('/tmp/index.html.js', 'utf8');

const listeners = {};
const input = { value: '', addEventListener: (e, cb) => { listeners[e] = cb; } };
const direction = { value: 'english-to-typography', addEventListener: (e, cb) => { listeners[`dir_${e}`] = cb; } };
const output = { textContent: '' };
const json = { textContent: '' };

const document = {
  readyState: 'complete',
  getElementById: (id) => ({ input, output, json, direction }[id]),
  addEventListener: () => {}
};

vm.runInNewContext(code, { document, JSON });

input.value = '"Hello..." (TM)';
listeners.input();
if (output.textContent !== '“Hello…” ™') {
  throw new Error(`Expected english->typography output, got: ${output.textContent}`);
}

direction.value = 'typography-to-english';
listeners.dir_change();
input.value = '“Hello…” ™';
listeners.input();
if (output.textContent !== '"Hello..." (TM)') {
  throw new Error(`Expected typography->english output, got: ${output.textContent}`);
}

console.log('Real-time translation smoke test passed');
NODE

printf '\nAll web UI checks passed.\n'
