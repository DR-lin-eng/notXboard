#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

node - "${ROOT_DIR}" <<'NODE'
const fs = require('fs');
const path = require('path');

const root = process.argv[2];
const maintainableFiles = [
  'rust-gateway/resources/public/theme/Maintainable/app.js',
  'public/theme/Maintainable/app.js',
  'theme/Maintainable/app.js',
];
const portalFiles = [
  'rust-gateway/resources/public/theme/portal/assets/umi.js',
  'theme/portal/assets/umi.js',
];

const maintainableChecks = [
  ['raw render error text', /<div class="notice error">\$\{errorText\}<\/div>/],
  ['raw exception message', /<div class="notice error">\$\{e\.message\s*\|\|/],
  ['raw checkout URL attribute', /href="\$\{out\.data\}"/],
  ['raw checkout JSON in HTML', /\$\{JSON\.stringify\(out,\s*null,\s*2\)\}/],
  ['raw checkout form in HTML', /innerHTML\s*=\s*`[^`]*支付表单[^`]*`\s*\+\s*(?:data|out\.data)/],
  ['legacy user API key in node deploy modal', /id="deployApiKey"|copyDeployApiKey/],
  ['raw route hash in HTML', /未知页面：\$\{hash\}/],
  ['raw ticket subject cell', /<td>\$\{(?:t|ticket)\.subject\s*\|\|/],
  ['raw plan name cell', /<td>\$\{(?:[a-zA-Z]+\.)?plan\?\.name\s*\|\|/],
  ['raw refund evidence', /<div>\$\{(?:e|evidence)\.content\s*\|\|/],
];

const portalChecks = [
  ['user plan content injected as HTML', 'dangerouslySetInnerHTML:{__html:e.content}'],
  ['user notice content injected as HTML', 'dangerouslySetInnerHTML:{__html:t.content}'],
  ['subscriber notice injected as HTML', 'dangerouslySetInnerHTML:{__html:this.state.notice.content||""}'],
];

let failures = 0;
for (const relative of maintainableFiles) {
  const source = fs.readFileSync(path.join(root, relative), 'utf8');
  if (!source.includes('function submitSafeEpayPostForm(')) {
    console.error(`${relative}: missing allowlisted EPay form reconstruction`);
    failures += 1;
  }
  for (const [label, pattern] of maintainableChecks) {
    if (pattern.test(source)) {
      console.error(`${relative}: ${label}`);
      failures += 1;
    }
  }
}

for (const relative of portalFiles) {
  const source = fs.readFileSync(path.join(root, relative), 'utf8');
  for (const [label, needle] of portalChecks) {
    if (source.includes(needle)) {
      console.error(`${relative}: ${label}`);
      failures += 1;
    }
  }
}

if (failures > 0) {
  console.error(`Theme XSS boundary verification failed with ${failures} finding(s).`);
  process.exit(1);
}

console.log('Theme XSS boundary verification passed.');
NODE
