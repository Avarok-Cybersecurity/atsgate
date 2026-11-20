import { graph } from '../lib/html.mjs';
import { blueprints, cite, scoring } from '../lib/data.mjs';

const plain = (html) => html.replace(/<[^>]+>/g, '');

export default function faqPage() {
  const faqs = [
    ['Is the ATSGate score what Workday or Taleo would give my resume?', `No. Every vendor result is a ${scoring.label}. No vendor publishes its scoring, and Greenhouse and Lever document no automatic scoring at all (${cite('gh-tm-faq')}). ATSGate shows how much a strict parser could lose and how well your keywords cover a role.`],
    ['Does ATSGate use AI or a language model?', 'No language model sits in the scoring path. Extraction, parsing, matching and scoring are deterministic Rust, so the same file and blueprint always give the same report, and a fairness property can be tested rather than estimated.'],
    ['Will hiding keywords in white text raise my score?', 'No. Text a reader cannot see is removed before parsing, so it earns nothing, and it is reported as an integrity flag. With <code>fail-on-integrity-flags: true</code> it fails the gate.'],
    ['Does an employment gap lower my score?', 'No. Years of experience are the union of your dated roles, and the time between them never subtracts. A metamorphic test checks that inserting a gap leaves every score bit-identical.'],
    ['Which file formats can it read?', 'PDF, DOCX, HTML, Markdown and plain text. PDF and DOCX get full layout analysis: columns, tables, text boxes, header and footer content, image-only pages, unmapped glyphs and hidden text.'],
    ['Do I have to render my resume with ATSGate?', 'No. <code>score</code> mode scores any existing PDF or DOCX. <code>gate</code> mode renders from a career file and scores exactly what it rendered, which is the stronger guarantee.'],
    ['What if my role is not one of the built-in blueprints?', `There are ${blueprints.length} <a href="/blueprints/">built-in blueprints</a>. You can write your own YAML against the <a href="/docs/blueprint-spec/">blueprint spec</a>, or generate a draft from a job posting with <code>generate:posting.txt</code> and review it.`],
    ['Is my resume uploaded anywhere?', 'No. The action runs on your runner; it downloads only its own pinned, checksummed binary and any blueprint or posting URL you name. Nothing about your resume leaves the job unless you upload it as an artifact yourself.'],
    ['Why is it also called ATS Express?', 'ATS Express is the name of the website, ats.express. The tool, the action and the crates are called ATSGate.'],
    ['What licence is it under?', 'The code is MIT OR Apache-2.0, at your option. The embedded O*NET data is CC BY 4.0 and ESCO is reused under the European Commission’s terms; attribution is in DATA-LICENSES.md.'],
  ];
  const body = faqs.map(([q, a], i) => `<h2 id="q${i + 1}">${q}</h2>\n<p>${a}</p>`).join('\n');
  const faqNode = {
    '@type': 'FAQPage',
    '@id': 'https://ats.express/docs/faq/#faq',
    mainEntity: faqs.map(([q, a]) => ({ '@type': 'Question', name: q, acceptedAnswer: { '@type': 'Answer', text: plain(a) } })),
  };
  const page = {
    route: '/docs/faq/',
    title: 'ATSGate FAQ: ATS scores, AI, gaps and hidden text',
    h1: 'Frequently asked questions',
    description: 'Answers about ATSGate: whether its scores match real ATS vendors, AI use, hidden text, employment gaps, file formats, privacy and licensing.',
    lede: 'Short answers to what people ask most about ATSGate, the deterministic resume gate for CI.',
    body,
  };
  return { ...page, jsonld: graph(page, [faqNode]) };
}
