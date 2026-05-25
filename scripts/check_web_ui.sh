#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

python - <<'PY'
import re
from pathlib import Path

html = Path('index.html').read_text(encoding='utf-8')
script = re.search(r"<script>([\s\S]*?)</script>", html)
if not script:
    raise SystemExit('No inline script found in index.html')
Path('/tmp/index.html.js').write_text(script.group(1), encoding='utf-8')
print('Extracted inline script from index.html')
PY

node --check /tmp/index.html.js

node - <<'NODE'
const fs = require('fs');
const vm = require('vm');
const code = fs.readFileSync('/tmp/index.html.js', 'utf8');

const listeners = {};
const input = { value: '', addEventListener: (e, cb) => { listeners[`input_${e}`] = cb; } };
const direction = { value: 'english-to-typography', addEventListener: (e, cb) => { listeners[`direction_${e}`] = cb; } };
const output = { textContent: '' };
const json = { textContent: '' };
const convert = { addEventListener: (e, cb) => { listeners[`convert_${e}`] = cb; } };

const document = {
  readyState: 'complete',
  getElementById: (id) => ({ input, output, json, direction, convert }[id]),
  addEventListener: () => {}
};

vm.runInNewContext(code, { document, JSON, setTimeout });

input.value = '"Hello..." (TM)';
listeners.input_input();
if (output.textContent !== '“Hello…” ™') {
  throw new Error(`Expected english->typography output, got: ${output.textContent}`);
}

input.value = '“Rust™ — ﬁne… really”';
listeners.convert_click();
if (output.textContent === '"Rust(TM) -- fine... really"') {
  throw new Error(`Expected typography output, got english output: ${output.textContent}`);
}

console.log('Web UI translation smoke test passed');
NODE

printf '\nAll web UI checks passed.\n'
