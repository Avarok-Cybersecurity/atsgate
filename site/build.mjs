// The ats.express build (GeoHarness adapter "command": `node build.mjs`, output-dir dist).
// Pages are rendered from site/data/ (generated from the code by `cargo run -p xtask -- site`)
// and from the page modules in pages/. No dependencies: Node's standard library only.
import { createHash } from 'node:crypto';
import { cpSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { CSS, layout } from './lib/html.mjs';
import home from './pages/home.mjs';
import scoring from './pages/scoring.mjs';
import vendors from './pages/vendors.mjs';
import fairness from './pages/fairness.mjs';
import action from './pages/action.mjs';
import blueprintSpec from './pages/blueprint-spec.mjs';
import career from './pages/career.mjs';
import faq from './pages/faq.mjs';
import citations from './pages/citations.mjs';
import { docsIndex, evaluationPage, notFound } from './pages/docs.mjs';
import { blueprintPages } from './pages/blueprints.mjs';

const here = import.meta.dirname;
const dist = join(here, 'dist');
const brand = join(here, '..', 'brand');

const pages = [
  home(),
  docsIndex(),
  scoring(),
  vendors(),
  fairness(),
  action(),
  blueprintSpec(),
  career(),
  evaluationPage(),
  faq(),
  citations(),
  ...blueprintPages(),
  notFound(),
];

const styleHash = createHash('sha256').update(CSS).digest('base64');
const HEADERS = `/*
  Content-Security-Policy: default-src 'none'; style-src 'sha256-${styleHash}'; img-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'; require-trusted-types-for 'script'; upgrade-insecure-requests
  Strict-Transport-Security: max-age=63072000; includeSubDomains
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  Permissions-Policy: camera=(), microphone=(), geolocation=()
  Cross-Origin-Opener-Policy: same-origin
  X-Frame-Options: DENY
`;

rmSync(dist, { recursive: true, force: true });
mkdirSync(dist, { recursive: true });
const seen = new Set();
for (const page of pages) {
  if (seen.has(page.route)) throw new Error(`two pages render ${page.route}`);
  seen.add(page.route);
  const file = page.route.endsWith('.html') ? join(dist, page.route) : join(dist, page.route, 'index.html');
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, layout(page));
}
writeFileSync(join(dist, '_headers'), HEADERS);
cpSync(join(brand, 'logo-light.svg'), join(dist, 'favicon-light.svg'));
cpSync(join(brand, 'logo-dark.svg'), join(dist, 'favicon-dark.svg'));
console.log(`site: built ${pages.length} pages into ${dist}`);
