import { code, esc, graph, pre, table } from '../lib/html.mjs';
import { action } from '../lib/data.mjs';

const CLI = `atsgate gate --career career.json --tailoring tailoring/general.yaml --contact public --out dist \\
  --blueprint builtin:software-engineer-backend --fail-on-integrity-flags true \\
  --as-of today --report atsgate/report.json --summary atsgate/report.md
atsgate score --resume my-resume.pdf --blueprint generate:posting.txt --floors "taleo=80" \\
  --fail-on-integrity-flags true --as-of today --report report.json --summary report.md`;

const required = (f) => (f.required ? 'yes' : 'no');
const fallback = (f) => (f.default == null ? '—' : code(f.default));

export default function actionPage() {
  const inputs = action.inputs.map((f) => [code(f.name), required(f), fallback(f), esc(f.description)]);
  const outputs = action.outputs.map((f) => [code(f.name), esc(f.description)]);
  const body = `
<h2 id="modes">Which mode should I use?</h2>
${table('Modes', ['Mode', 'What it does'], [
  [code('build'), 'Renders the resume PDF, DOCX, HTML partials and JSON-LD from a career file and a tailoring file. No scoring'],
  [code('score'), 'Scores an existing PDF or DOCX against a blueprint and applies the gate'],
  [code('gate'), 'Builds, then scores exactly the PDF it rendered, so what ships is what was scored'],
])}

<h2 id="inputs">What inputs does the action take?</h2>
<p>Generated from <code>action.yml</code>. The prefix of each description names the modes that read it. <code>mode</code> and <code>fail-on-integrity-flags</code> have no default on purpose: whether hidden text fails your build is a decision you state.</p>
${table('Action inputs (from action.yml)', ['Input', 'Required', 'Default', 'Description'], inputs)}

<h2 id="outputs">What outputs does it set?</h2>
${table('Action outputs (from action.yml)', ['Output', 'Description'], outputs)}
<p>“Present” in a role's dates counts through the month the job runs in.</p>

<h2 id="blueprint-input">How do I name a blueprint?</h2>
<ul>
<li>${code('builtin:ID')}: one of the <a href="/blueprints/">built-in blueprints</a>.</li>
<li>A path to a blueprint YAML or JSON file in your repository, checked against the <a href="/docs/blueprint-spec/">blueprint spec</a>.</li>
<li>An ${code('https://')} URL to one.</li>
<li>${code('generate:FILE-OR-URL')}: a job posting. The generated blueprint is written beside the report and marked <code>reviewed: false</code>, so review it before it gates anything.</li>
</ul>

<h2 id="cli">Can I run it outside GitHub Actions?</h2>
<p>Yes. The action wraps the <code>atsgate</code> command, the only crate that reads files or the network:</p>
${pre(CLI)}
<p>Exit status: 0 when the gate passes, 1 when it fails (the report is still written) or on an error, 2 on a usage error.</p>

<h2 id="binaries">Where do the binaries come from?</h2>
<p>Outside its own repository the action downloads the <code>atsgate</code> binary for the runner (Linux x86_64 and arm64, macOS arm64, Windows x86_64) from the GitHub release named in <code>action/release-pins.txt</code>, and refuses it unless its sha256 matches the pin. Each binary is built by the release workflow on a <code>v*</code> tag with a build-provenance attestation you can check with <code>gh attestation verify</code>. A revision with no pins refuses to run rather than fetch an unverified binary.</p>
<p>In CI a private contact file is refused unless <code>allow-private-contact</code> is <code>true</code>, so a phone number or street address cannot reach a public artifact by accident.</p>`;
  const page = {
    route: '/docs/action/',
    type: 'TechArticle',
    title: 'GitHub Action inputs and outputs · ATSGate',
    h1: 'GitHub Action reference',
    description: 'Every input and output of the ATSGate GitHub Action, generated from action.yml, plus the CLI, exit codes and how release binaries are verified.',
    lede: 'ATSGate runs as a composite GitHub Action in three modes: build, score and gate. This page is generated from action.yml.',
    body,
  };
  return { ...page, jsonld: graph(page) };
}
