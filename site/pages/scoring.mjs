import { code, graph, table } from '../lib/html.mjs';
import { cite, scoring } from '../lib/data.mjs';

const REPO_SRC = 'https://github.com/Avarok-Cybersecurity/atsgate/blob/main/crates/atsgate-score/src';

// Why each penalized signal costs what it costs (parse.rs). A rule the scorer charges that is not
// explained here fails the build, and so does an explanation for a rule it no longer charges.
const WHY = {
  'layout.image_only_page': 'No text to parse at all',
  'layout.multi_column': 'Row-by-row parsers interleave the columns',
  'layout.reading_order_disagreement': 'Stored order scrambles blocks the same way',
  'glyphs.missing_tounicode': 'Glyphs extract as nothing or as garbage',
  'layout.table': 'Cells are skipped or flattened',
  'layout.text_box': 'Out-of-flow boxes are skipped or misplaced',
  'layout.header_footer_content': 'Many parsers never read the margins',
  'glyphs.unmapped': 'Replacement characters break words',
  'glyphs.ligature': 'A ligature such as “ﬁ” breaks keyword matching',
  'parse.page_count': `More than ${scoring.page_limit} pages may be truncated`,
};

const num = (x, digits = 3) => String(Number(x.toFixed(digits)));

export function penaltyRows() {
  const charged = scoring.penalties.map((p) => p.rule);
  const stale = Object.keys(WHY).filter((r) => !charged.includes(r));
  if (stale.length) throw new Error(`scoring page explains rules the scorer no longer charges: ${stale}`);
  const rows = scoring.penalties.map((p) => {
    if (!WHY[p.rule]) throw new Error(`no explanation for penalty rule ${p.rule}`);
    return [code(p.rule), num(p.points), WHY[p.rule]];
  });
  const each = `${num(scoring.per_item)} each, at most ${num(scoring.per_item_cap)}`;
  rows.push([code('parse.nonstandard_heading'), each, 'Sections may not be recognized']);
  rows.push([code('parse.undated_role'), each, 'A role without dates adds no years']);
  return rows;
}

export function vendorRows() {
  return scoring.vendors.map((v) => [v.name, code(v.key), num(v.sigma), String(v.floor), v.note ?? '—']);
}

export default function scoringPage() {
  const m = scoring.match;
  const t = scoring.title;
  const weightRows = scoring.vendors.map((v) => [v.name, ...['f', 'r', 'p', 't', 'e', 'd'].map((k) => num(v.weights[k]))]);
  const body = `
<p>Every number on this page is generated from the scorer's source code by <code>cargo run -p xtask -- site --write</code>, and CI fails if the committed copy drifts from it.</p>

<h2 id="dimensions">Which dimensions does ATSGate score?</h2>
${table('Scoring dimensions', ['Dimension', 'Formula'], [
  ['R, P: required and preferred skills', `Σ w·m·c / Σ w, each skill counted once at its best occurrence. m = ${num(m.canonical)} for the canonical name, ${num(m.alias)} for a taxonomy alias, ${num(m.related)} for a broader or narrower concept. c = ${num(m.in_scope)} where the blueprint's <code>evidence_scope</code> asks for it, ${num(m.list_only)} for a skill an <code>experience_or_projects</code> blueprint finds only in the skills list or summary`],
  ['T: title', `The best title match (${num(t.same)} the same words, ${num(t.occupation)} a title of the blueprint's O*NET occupation, ${num(t.broad_group)} one in the same SOC broad group) × (1 − |seniority steps apart| / ${num(t.ladder_span)})`],
  ['E: years', 'min(1, relevant years / the blueprint minimum). Years are the union of relevant dated roles: overlaps count once and <strong>gaps never subtract</strong>'],
  ['D: education', 'Degree level and field from each education entry title. Institutions and dates are never read'],
  ['F: parse integrity', 'max(0, 100 − Σ penalty · σ), with σ the vendor parser strictness and the penalties below'],
  ['Q: quantification', `Share of bullets with a number, labelled “${scoring.quantification_label}”. Reported, never scored`],
  ['Knockouts', 'Met, unmet or unknown, never points. Unmet fails the gate unless the blueprint says <code>review</code>'],
])}
<p>Text a reader cannot see (white, tiny, off-page, zero-width or hidden runs) is removed before the resume is parsed, so it earns nothing. It is reported as an <a href="/docs/fairness/#integrity">integrity flag</a> instead.</p>

<h2 id="vendor-score">How is a vendor score computed?</h2>
<p>Each vendor's score is 100 · Σ weight · dimension over the dimensions the blueprint uses, renormalized and rounded to 0.1. A score equal to the floor passes. The gate fails when any vendor is below its floor, any knockout fails, or, with <code>fail-on-integrity-flags: true</code>, any integrity flag is raised.</p>
${table('Vendor strictness and default floors (generated from vendor.rs)', ['Vendor', 'Floor key', 'σ', 'Default floor', 'Note'], vendorRows())}
<p>The weights and σ are adapted from ats-screener (${cite('ats-screener-method')}, MIT). Its formatting and section-completeness weights become F, keyword match is split ${num(scoring.required_share * 100)}/${num((1 - scoring.required_share) * 100)} between R and P, experience relevance is split ${num(scoring.title_share * 100)}/${num((1 - scoring.title_share) * 100)} between T and E, and quantification is dropped and the rest renormalized. The default floors are its <code>thresholds.md</code> values, because its own profiles disagree with them.</p>
${table('Dimension weights per vendor, summing to 1 (generated from vendor.rs)', ['Vendor', 'F', 'R', 'P', 'T', 'E', 'D'], weightRows)}

<h2 id="penalties">What does each layout problem cost?</h2>
<p>Points are at σ = 1; a vendor charges points · σ. Each signal kind is charged once, however many pages or spans carry it.</p>
${table('Parse penalties (generated by running parse.rs)', ['Rule', 'Points', 'Why'], penaltyRows())}

<h2 id="overrides">Can I change the floors?</h2>
<p>Yes. The <code>floors</code> input takes overrides such as <code>taleo=80, lever=0</code>; vendors you do not name keep the default floors. Setting every floor to 0 turns the gate into a report that still fails on knockouts and, if you ask, on integrity flags.</p>
<p>Source: <a href="${REPO_SRC}/vendor.rs">vendor.rs</a>, <a href="${REPO_SRC}/parse.rs">parse.rs</a>, <a href="${REPO_SRC}/skills.rs">skills.rs</a> and <a href="${REPO_SRC}/title.rs">title.rs</a>.</p>`;
  const page = {
    route: '/docs/scoring/',
    type: 'TechArticle',
    title: 'Scoring formulas and vendor floors · ATSGate',
    h1: 'Scoring formulas and vendor floors',
    description: 'How ATSGate scores skills, title, years, education and parse integrity, with the vendor σ and floor table generated from the scorer.',
    lede: 'ATSGate scores a resume on seven dimensions, then weights them per simulated vendor. Every point names the rule and the evidence behind it.',
    body,
  };
  return { ...page, jsonld: graph(page) };
}
