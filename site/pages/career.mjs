import { code, graph, pre, table } from '../lib/html.mjs';
import { cite } from '../lib/data.mjs';

const SCHEMAS = 'https://github.com/Avarok-Cybersecurity/atsgate/blob/main/schemas';

const TAILORING = `# tailoring/backend.yaml: one resume, selected from career.json
version: 1
blueprint: builtin:software-engineer-backend
title: { variant: backend }
summary: basics
max_pages: 1
work:
  - { id: northwind, highlights: [0, 1], stats: [0] }
  - { id: contoso, highlights: [1, 0], stats: [] }
projects:
  - { id: ledgerline, highlights: [0], stats: [0, 1] }
skills: [Languages, Data]
education: [state-university]`;

const COMMANDS = `# Convert a JSON Resume into a career file; every field is mapped.
atsgate import --from json-resume --input resume.json --out career.json

# Render one tailoring of it. --contact is public or a private contact file.
atsgate build --career career.json --tailoring tailoring/backend.yaml --contact public --out dist

# Fill <!-- atsgate:NAME --> ... <!-- /atsgate:NAME --> markers in a page with the partials.
atsgate inject --partials dist/partials --page site/index.html --out site/index.html`;

export default function careerPage() {
  const body = `
<h2 id="ssot">What is the career file?</h2>
<p>The career file is the single source of truth for everything you have done: a JSON Resume document (${cite('jsonresume')}) plus an <code>x-atsgate</code> block. The block adds what a resume generator needs to stay honest:</p>
<ul>
<li><strong>Sources:</strong> where each fact came from, as a URL or a repository path. A source with neither is an unlocated statement and reads as unverified.</li>
<li><strong>Statistics with a required source:</strong> a number on your resume must say where it came from.</li>
<li><strong>Families:</strong> related products shown together.</li>
<li><strong>Variants:</strong> alternative phrasings of a title or summary, chosen per resume.</li>
</ul>
<p>A phone number, street or postal code in the career file is an error. Those live in a separate private contact file that never needs to be committed.</p>

<h2 id="tailoring">What is a tailoring file?</h2>
<p>A tailoring file picks one resume out of the career file: which entries, highlights, statistics and variants it shows, in which order, and which blueprint it targets. It only selects, orders and chooses. It has no field that could carry a new fact, so tailoring for a role can never invent experience.</p>
${pre(TAILORING)}

<h2 id="render">What does a build produce?</h2>
${table('Files written by build and gate', ['File', 'Contents'], [
  [code('resume.pdf'), 'One column, real text, typeset by Typst embedded as a library: standard headings, contact details in the body, no tables, dates as <code>Mon YYYY - Mon YYYY</code>'],
  [code('resume.docx'), 'The same resume as DOCX'],
  [code('partials/*.html'), 'Experience, projects, skills and education for your website'],
  [code('partials/jsonld.json'), 'schema.org <code>Person</code> with <code>hasOccupation</code>, <code>worksFor</code> and <code>alumniOf</code>'],
  [code('manifest.json'), 'The SHA-256 of each file; output is byte-for-byte reproducible'],
])}
<p>Before anything is written, the PDF and the DOCX are read back through ATSGate's own extractor and resume parser. Every section, entry, date, bullet and skill must come back in order, with no integrity flag, no layout-risk signal and no unrecognized heading, or the build fails. Career text reaches the template only as data, never as markup.</p>

<h2 id="contact">How is contact information handled?</h2>
<p>The <code>public</code> profile prints city and region, email, website and identity profiles. A private contact file adds the phone and street. In GitHub Actions a private contact file is refused unless <code>allow-private-contact: true</code> is set, so a public artifact cannot carry your phone number by accident.</p>

<h2 id="commands">Which commands work with career files?</h2>
${pre(COMMANDS)}
<p>Schemas: <a href="${SCHEMAS}/career.schema.json">career</a>, <a href="${SCHEMAS}/tailoring.schema.json">tailoring</a> and <a href="${SCHEMAS}/contact.schema.json">private contact</a>, all generated from the Rust types.</p>`;
  const page = {
    route: '/docs/career-and-tailoring/',
    type: 'TechArticle',
    title: 'Career SSOT and tailoring files · ATSGate',
    h1: 'Career file and tailoring',
    description: 'One career file renders every resume: tailoring files select and order facts but cannot add them, and each PDF and DOCX is checked by a parse round trip.',
    lede: 'Keep every fact in one career file. Each resume is a tailoring of it that selects and orders, and can never add a fact.',
    body,
  };
  return { ...page, jsonld: graph(page) };
}
