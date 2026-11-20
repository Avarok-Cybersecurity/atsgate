import { code, graph, table } from '../lib/html.mjs';
import { cite, scoring } from '../lib/data.mjs';

const TESTS = 'https://github.com/Avarok-Cybersecurity/atsgate/blob/main/crates/atsgate-score/tests/fairness.rs';

// What each integrity rule detects. A rule the extractor raises that is not described fails the build.
const INTEGRITY = {
  'integrity.hidden.white': 'Text coloured like the page background',
  'integrity.hidden.tiny': 'Text too small to read',
  'integrity.hidden.zero_width': 'Zero-width characters inside words',
  'integrity.hidden.off_page': 'Text placed outside the visible page',
  'integrity.hidden.vanished': 'DOCX runs marked hidden (<code>w:vanish</code>)',
  'integrity.metadata_only': 'Terms present only in document metadata, never in the visible text',
};

export default function fairnessPage() {
  const rows = scoring.integrity_rules.map((rule) => {
    if (!INTEGRITY[rule]) throw new Error(`fairness page lacks a description of ${rule}`);
    return [code(rule), INTEGRITY[rule]];
  });
  const body = `
<h2 id="never-read">What does ATSGate never read?</h2>
${table('Never a scoring input', ['Input', 'How it is kept out'], [
  ['Name and contact lines', 'The preamble before the first heading and the page headers are not scored'],
  ['Photos', 'Images are counted for the image-only-page check, never read for content'],
  ['Street address', 'Not scored; the career file refuses a phone, street or postal code outright'],
  ['School or institution name', 'Education D reads only each entry’s degree title'],
  ['Graduation year and dates of study', 'Education dates are never read'],
  ['Employment gaps', 'Years are the union of dated roles; the time between roles never subtracts'],
])}

<h2 id="why">Why does this matter?</h2>
<p>Language-model screeners carry name, school and gap bias. Name-based bias varies by model (${cite('faire')}), education bias persists (${cite('iso-bias')}), college names act as a demographic proxy despite anti-bias prompts (${cite('karvonen')}), and the direction of racial bias flips across model generations (${cite('llm-hire-fairly')}). Recruiting systems also exclude candidates for a gap in employment (${cite('hidden-workers')}).</p>
<p>ATSGate is a deterministic program, so these properties are checkable rather than statistical. A resume audit finds that objective rules narrow discrimination gaps where discretion widens them (${cite('task-audit')}); ATSGate is a set of objective rules you can read.</p>

<h2 id="tests">How is it tested?</h2>
<p>Metamorphic tests in <a href="${TESTS}">fairness.rs</a> change one protected input at a time and require every score to stay bit-identical, over every built-in blueprint:</p>
<ul>
<li>Swapping the name, the school or the graduation year changes nothing.</li>
<li>Inserting an employment gap between two roles changes nothing.</li>
<li>Competence tests check the other direction: a mismatched resume and blueprint score below the floors.</li>
</ul>
<p>An invariance test that passes because nothing is scored would prove nothing, so a guard test checks that the metamorphic resume exercises every dimension.</p>

<h2 id="integrity">What counts as hidden text?</h2>
<p>About 1% of real resumes carry hidden prompt injections (${cite('inj-measure')}), and concealment is itself a detectable signal (${cite('resumeshield')}). ATSGate removes text a reader cannot see before the resume is parsed, so it earns nothing, and reports each case as an integrity flag with its evidence. With <code>fail-on-integrity-flags: true</code> any flag fails the gate.</p>
${table('Integrity flags (generated from the extractor)', ['Rule', 'What it detects'], rows)}

<h2 id="not-a-hiring-tool">Is ATSGate a hiring tool?</h2>
<p>No. ATSGate is built for candidates and their CI, to check their own resume. Using any automated tool to rank or reject applicants is regulated in places: New York City requires bias audits of automated employment decision tools (${cite('ll144')}), and the EU AI Act lists recruitment AI as high-risk (${cite('eu-aia-a3')}).</p>`;
  const page = {
    route: '/docs/fairness/',
    type: 'TechArticle',
    title: 'Fairness: what ATSGate never reads · ATSGate',
    h1: 'Fairness: what ATSGate never reads',
    description: 'ATSGate never scores names, photos, addresses, schools, graduation years or employment gaps, and metamorphic tests prove each score is unchanged.',
    lede: 'Names, photos, addresses, schools, graduation years and gaps are never scoring inputs. Changing any of them leaves every score bit-identical.',
    body,
  };
  return { ...page, jsonld: graph(page) };
}
