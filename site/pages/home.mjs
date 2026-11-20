import { SUMMARY, code, graph, pre, table } from '../lib/html.mjs';
import { blueprints, cite, scoring } from '../lib/data.mjs';

const WORKFLOW = `name: resume
on: [push, pull_request]
jobs:
  resume:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: Avarok-Cybersecurity/atsgate@<release tag or commit>
        id: atsgate
        with:
          mode: gate
          career: career.json
          tailoring: tailoring/general.yaml
          contact: public
          out: dist
          blueprint: builtin:software-engineer-backend
          fail-on-integrity-flags: "true"
          report: atsgate/report.json
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: resume
          path: |
            dist/
            atsgate/`;

export default function home() {
  const vendors = scoring.vendors.map((v) => v.name).join(', ');
  const description =
    'ATSGate renders a resume PDF and DOCX from one career file and gates CI on simulated ATS parse and keyword risk, with evidence for every point.';
  const body = `
<h2 id="what">What is ATSGate?</h2>
<p>ATSGate (also called ATS Express) is an open-source GitHub Action and command-line tool written in pure Rust by Avarok Cybersecurity. It reads a resume as PDF, DOCX, HTML, Markdown or plain text and scores it against a standardized job blueprint. It also renders the resume itself, from one career file, so the PDF you ship is the PDF that was scored.</p>
<ul>
<li><strong>Deterministic.</strong> The same input always gives the same score. No language model sits in the scoring path.</li>
<li><strong>Evidence-carrying.</strong> Every point names the rule that awarded it and the page, bounding box and text span it came from.</li>
<li><strong>One source of truth.</strong> A career file (JSON Resume plus an <code>x-atsgate</code> block) renders the resume PDF, the DOCX, HTML partials for your site and schema.org JSON-LD.</li>
<li><strong>Fair by construction.</strong> Names, photos, addresses, schools, graduation years and employment gaps are never scoring inputs, and metamorphic tests prove it.</li>
</ul>

<h2 id="honest">What does the score mean, and what does it not mean?</h2>
<p>Every vendor result is labelled a <strong>${scoring.label}</strong>. It is not a vendor's score, because no applicant tracking system publishes its scoring. Greenhouse and Lever document no automatic scoring or rejection at all (${cite('gh-tm-faq')}, ${cite('ats-screener-research')}).</p>
<p>ATSGate simulates how strict parsers in the style of ${vendors} would read your file and how well its keywords cover a role. It tells you that a two-column layout may be read out of order, or that a required skill appears only in the skills list. It cannot tell you whether a recruiter will call.</p>

<h2 id="quick-start">How do I gate a resume in CI?</h2>
<p>Add one step to a workflow. In <code>gate</code> mode the action renders the resume from your career file, scores exactly the PDF it rendered, and fails the job when any simulated vendor falls below its floor, a knockout fails, or the file carries hidden text.</p>
${pre(WORKFLOW)}
<p>Pin the action to a release tag or a full commit SHA. Release binaries are verified against sha256 pins committed in the repository, and a revision with no pins refuses to run rather than download an unverified binary. Every input and output is listed on the <a href="/docs/action/">action reference</a>.</p>

<h2 id="output">What does a run produce?</h2>
${table('What gate mode writes', ['File', 'Contents'], [
  [code('dist/resume.pdf'), 'One-column PDF typeset by Typst, read back through the extractor before it is written'],
  [code('dist/resume.docx'), 'The same resume as DOCX, also checked by a parse round trip'],
  [code('dist/partials/'), 'Experience, projects, skills and education as HTML, plus <code>jsonld.json</code> (schema.org Person)'],
  [code('atsgate/report.json'), 'The resume sha256, every dimension with evidence, six vendor results, knockouts, integrity flags, the verdict'],
  [code('atsgate/report.md'), 'A markdown summary, also appended to the job summary'],
])}

<h2 id="next">Where do I go next?</h2>
<ul>
<li><a href="/docs/scoring/">Scoring formulas and the vendor table</a>, generated from the scorer's source.</li>
<li><a href="/blueprints/">The ${blueprints.length} built-in blueprints</a>, from backend engineer to paralegal.</li>
<li><a href="/docs/fairness/">What ATSGate never reads</a>, and how that is tested.</li>
<li><a href="/docs/faq/">Frequently asked questions</a>.</li>
</ul>`;
  const page = {
    route: '/',
    title: 'ATSGate: deterministic ATS resume gate for CI',
    h1: 'Gate your resume in CI before an ATS reads it',
    description,
    lede: SUMMARY,
    body,
  };
  return { ...page, jsonld: graph(page) };
}
