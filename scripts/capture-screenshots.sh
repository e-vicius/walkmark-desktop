#!/usr/bin/env bash
# Render README / docs screenshots with Playwright (static HTML mocks of the app UI).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HTML="$ROOT/scripts/screenshots"
OUT="$ROOT/docs/screenshots"

mkdir -p "$OUT"

python3 "$ROOT/scripts/generate_screenshot_assets.py"

node --input-type=module <<EOF
import { chromium } from 'playwright';
import path from 'node:path';

const html = '$HTML';
const out = '$OUT';

async function capture(filename, pageHtml, width, height) {
  const browser = await chromium.launch();
  const page = await browser.newPage({
    viewport: { width, height },
    deviceScaleFactor: 2
  });
  await page.goto(\`file://\${path.join(html, pageHtml)}\`, { waitUntil: 'networkidle' });
  await page.waitForTimeout(400);
  await page.locator('.window').screenshot({ path: path.join(out, filename), type: 'png' });
  await browser.close();
  console.log('  wrote', filename);
}

console.log('Capturing screenshots → docs/screenshots/');
await capture('library.png', 'capture-library.html', 1200, 900);
await capture('editor.png', 'capture-document.html', 1280, 900);
await capture('export.png', 'capture-export.html', 1280, 720);
await capture('step-record.png', 'capture-step-record.html', 720, 480);
await capture('step-tidy.png', 'capture-step-tidy.html', 720, 480);
await capture('step-export.png', 'capture-step-ship.html', 720, 480);
console.log('Done.');
EOF
