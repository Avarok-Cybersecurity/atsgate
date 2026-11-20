import { code, graph, table } from '../lib/html.mjs';
import { cite, scoring } from '../lib/data.mjs';
import { vendorRows } from './scoring.mjs';

// What the cited vendor documentation says about parsing, per floor key. A vendor the scorer has
// that is not described here fails the build.
const NOTES = {
  taleo: ['Oracle documents the attachment formats Taleo accepts.', ['taleo-attach']],
  workday: ['Workday says parse results vary with format and word order, and advises against image resumes.', ['wd-parse']],
  successfactors: ['SAP documents limits on which resume fields map onto the candidate profile.', ['sap-kba']],
  icims: ['iCIMS describes resume parsing in a vendor blog post; ATSGate treats it as context only.', ['icims-blog']],
  greenhouse: ['Greenhouse says Talent Matching does not auto-reject and excludes names and contact details. Unrecognized characters break its resume import.', ['gh-tm-faq', 'gh-import', 'gh-types']],
  lever: ['Research notes report that Lever does not auto-score or auto-reject applicants.', ['ats-screener-research']],
};

export default function vendorsPage() {
  const missing = scoring.vendors.filter((v) => !NOTES[v.key]).map((v) => v.key);
  if (missing.length) throw new Error(`vendors page lacks notes for ${missing}`);
  const sections = scoring.vendors
    .map((v) => {
      const [text, ids] = NOTES[v.key];
      const sources = [v.citation, ...ids.filter((id) => id !== v.citation)].map(cite).join(', ');
      return `<h3 id="${v.key}">${v.name}</h3>
<p>${text} Simulated with σ = ${v.sigma} and a default floor of ${v.floor}${v.note ? ` (${v.note})` : ''}. Sources: ${sources}.</p>`;
    })
    .join('\n');
  const body = `
<h2 id="simulated">Why are vendor results simulated?</h2>
<p>No applicant tracking system publishes how it ranks resumes, so ATSGate does not claim to reproduce one. Each vendor profile is a parser strictness σ and a set of dimension weights. The result is labelled a <strong>${scoring.label}</strong>: how much of your resume a strict parser could lose, and how well its keywords cover the blueprint.</p>
<p>The profiles are adapted from the open-source ats-screener project (${cite('ats-screener-method')}). Its own pass floors contradict each other between two of its documents, so ATSGate takes the <code>thresholds.md</code> values and says so.</p>
${table('Simulated vendors (generated from vendor.rs)', ['Vendor', 'Floor key', 'σ', 'Default floor', 'Note'], vendorRows())}

<h2 id="per-vendor">What does each vendor's documentation say?</h2>
${sections}

<h2 id="what-to-do">What should I change when a vendor fails?</h2>
<ul>
<li><strong>F is low:</strong> the report names the layout signal and its page and bounding box. One column, no tables or text boxes, and contact details in the body fix most of it.</li>
<li><strong>R or P is low:</strong> the report lists each unmatched skill and the spellings that would have matched. Add the skill where you used it, in a role or project, not only in a skills list.</li>
<li><strong>T or E is low:</strong> compare your titles and dated roles with the blueprint's <code>relevant_titles</code> and <code>min_years</code>. A role without dates adds no years.</li>
</ul>
<p>Override a floor with the <code>floors</code> input, for example ${code('greenhouse=0')}, when a vendor does not apply to you.</p>`;
  const page = {
    route: '/docs/vendors/',
    type: 'TechArticle',
    title: 'Simulated ATS vendors: Workday, Taleo, iCIMS · ATSGate',
    h1: 'Simulated vendor profiles',
    description: 'How ATSGate simulates Workday, Taleo, SuccessFactors, iCIMS, Greenhouse and Lever parsing, what their documentation says, and why none is a real score.',
    lede: 'ATSGate reports six simulated vendor results. Each is a parse and keyword risk under one parser strictness, never the vendor’s own score.',
    body,
  };
  return { ...page, jsonld: graph(page) };
}
