// Page chrome, escaping and the identity JSON-LD shared by every page of ats.express.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

export const ORIGIN = 'https://ats.express';
export const REPO = 'https://github.com/Avarok-Cybersecurity/atsgate';
export const ORG_ID = 'https://avarok.net/#organization';
export const AUTHOR_ID = 'https://thomaspbraun.com/#person';
export const SITE_ID = `${ORIGIN}/#website`;
export const APP_ID = `${ORIGIN}/#software`;
// Filled in by GeoHarness from the git date of each page's sources (content.sources).
export const LASTMOD = '{{geo.lastmod}}';

const ESCAPES = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' };
export const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ESCAPES[c]);
export const code = (s) => `<code>${esc(s)}</code>`;

export function table(caption, headers, rows) {
  const head = headers.map((h) => `<th scope="col">${h}</th>`).join('');
  const body = rows.map((r) => `<tr>${r.map((c) => `<td>${c}</td>`).join('')}</tr>`).join('\n');
  return `<div class="table" role="region" aria-label="${esc(caption)}" tabindex="0"><table>
<caption>${esc(caption)}</caption>
<thead><tr>${head}</tr></thead>
<tbody>
${body}
</tbody></table></div>`;
}

export const pre = (text) => `<pre tabindex="0"><code>${esc(text)}</code></pre>`;

const here = import.meta.dirname;
const minify = (css) =>
  css
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/\s+/g, ' ')
    .replace(/\s*([{}:;,>])\s*/g, '$1')
    .replace(/;}/g, '}')
    .trim();
export const CSS = minify(readFileSync(join(here, '..', 'theme.css'), 'utf8'));

const LOGO = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="36" height="36" aria-hidden="true" focusable="false"><rect class="lg" x="6" y="8" width="52" height="8" rx="2"/><rect class="lg" x="10" y="16" width="9" height="42" rx="1.5"/><rect class="lg" x="45" y="16" width="9" height="42" rx="1.5"/><path class="la" d="M24 38 l7 7 l11 -15" fill="none" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/></svg>`;

const NAV = [
  ['/docs/', 'Docs'],
  ['/blueprints/', 'Blueprints'],
  ['/citations/', 'Citations'],
  [REPO, 'GitHub'],
];

const header = () => `<a class="skip" href="#content">Skip to content</a>
<header class="site"><div class="wrap">
<a class="brand" href="/">${LOGO}<span>ATSGate</span></a>
<nav aria-label="Primary"><ul>
${NAV.map(([href, label]) => `<li><a href="${href}">${label}</a></li>`).join('\n')}
</ul></nav>
</div></header>`;

const footer = () => `<footer class="site"><div class="wrap">
<p>ATSGate (also known as ATS Express) is open source under MIT OR Apache-2.0, published by <a href="https://avarok.net/about/">Avarok Cybersecurity</a> and written by <a href="https://thomaspbraun.com" rel="author">Thomas Braun</a>. Source on <a href="${REPO}">GitHub</a>.</p>
<p><a href="mailto:contact@avarok.net">Contact</a> · <a href="https://avarok.net/privacy/">Privacy notice</a> · <a href="${REPO}/security/policy">Security policy</a> · Built and deployed with <a href="https://geo.avarok.net">GeoHarness</a>. Agents: send <code>Accept: text/markdown</code>, or read <a href="/llms.txt">llms.txt</a>.</p>
</div></footer>`;

export const organization = () => ({
  '@type': 'Organization',
  '@id': ORG_ID,
  name: 'Avarok',
  alternateName: 'Avarok Cybersecurity',
  url: 'https://avarok.net',
  logo: 'https://avarok.net/icon-512.png',
  email: 'contact@avarok.net',
  sameAs: [
    'https://github.com/Avarok-Cybersecurity',
    'https://blog.avarok.net/',
    'https://www.linkedin.com/company/avarok/',
    'https://crates.io/crates/citadel_sdk',
  ],
});

export const author = () => ({ '@type': 'Person', '@id': AUTHOR_ID, name: 'Thomas Braun', url: 'https://thomaspbraun.com' });

export const website = () => ({
  '@type': 'WebSite',
  '@id': SITE_ID,
  name: 'ATSGate',
  alternateName: 'ATS Express',
  url: `${ORIGIN}/`,
  inLanguage: 'en',
  publisher: { '@id': ORG_ID },
});

export const SUMMARY =
  'ATSGate is a deterministic, evidence-carrying ATS gate for GitHub Actions: it renders a resume from one career file and fails CI on simulated parse and keyword risk.';

export const software = () => ({
  '@type': 'SoftwareApplication',
  '@id': APP_ID,
  name: 'ATSGate',
  alternateName: 'ATS Express',
  description: SUMMARY,
  url: `${ORIGIN}/`,
  applicationCategory: 'DeveloperApplication',
  operatingSystem: 'Linux, macOS, Windows',
  license: ['https://opensource.org/licenses/MIT', 'https://www.apache.org/licenses/LICENSE-2.0'],
  isAccessibleForFree: true,
  offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
  author: { '@id': AUTHOR_ID },
  publisher: { '@id': ORG_ID },
  sameAs: [REPO],
});

/** The JSON-LD of one page: the shared entities plus the page node and any extra nodes. */
export function graph(page, extra = []) {
  const url = `${ORIGIN}${page.route}`;
  const node = {
    '@type': page.type ?? 'WebPage',
    '@id': url,
    url,
    name: page.title,
    headline: page.type === 'TechArticle' ? page.title : undefined,
    description: page.description,
    inLanguage: 'en',
    isPartOf: { '@id': SITE_ID },
    about: { '@id': APP_ID },
    author: { '@id': AUTHOR_ID },
    publisher: { '@id': ORG_ID },
    dateModified: LASTMOD,
  };
  return { '@context': 'https://schema.org', '@graph': [organization(), author(), website(), software(), node, ...extra] };
}

// `<` is escaped so no string in the data can close the script element.
const jsonld = (value) => JSON.stringify(value).replace(/</g, '\\u003c');

/** A complete HTML document. `page`: { route, title, description, lede, body, jsonld }. */
export function layout(page) {
  const canonical = `${ORIGIN}${page.route}`;
  if (page.description.length > 155) throw new Error(`${page.route}: description is ${page.description.length} characters (max 155)`);
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(page.title)}</title>
<meta name="description" content="${esc(page.description)}">
<link rel="canonical" href="${canonical}">
<meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)">
<meta name="theme-color" content="#0d1117" media="(prefers-color-scheme: dark)">
<link rel="icon" href="/favicon-light.svg" type="image/svg+xml" media="(prefers-color-scheme: light)">
<link rel="icon" href="/favicon-dark.svg" type="image/svg+xml" media="(prefers-color-scheme: dark)">
<link rel="me" href="${REPO}">
<meta property="og:type" content="website">
<meta property="og:site_name" content="ATSGate">
<meta property="og:title" content="${esc(page.title)}">
<meta property="og:description" content="${esc(page.description)}">
<meta property="og:url" content="${canonical}">
<style>${CSS}</style>
<script type="application/ld+json">${jsonld(page.jsonld)}</script>
</head>
<body>
${header()}
<main id="content"><div class="wrap">
<h1>${esc(page.h1 ?? page.title)}</h1>
<p class="lede">${page.lede}</p>
<p class="meta">Updated <time datetime="${LASTMOD}">${LASTMOD}</time></p>
${page.body.trim()}
</div></main>
${footer()}
</body>
</html>
`;
}
