// Generated from CITATIONS.md (via site/data/citations.json), the repository's source list.
import { esc, graph, table } from '../lib/html.mjs';
import { citations } from '../lib/data.mjs';

const GRADES = [
  ['PR', 'Peer-reviewed'],
  ['AX', 'arXiv preprint (not peer-reviewed)'],
  ['VD', 'Vendor documentation'],
  ['IB', 'Industry report or blog'],
  ['CR', 'Community project or repository'],
  ['LG', 'Legal text or standard'],
];

export default function citationsPage() {
  const known = GRADES.map(([g]) => g);
  const unknown = citations.filter((c) => !known.includes(c.grade)).map((c) => c.id);
  if (unknown.length) throw new Error(`citations with an undocumented grade: ${unknown}`);
  const items = citations
    .map(
      (c) => `<li id="cite-${esc(c.id)}"><strong>${esc(c.id)}</strong>: ${esc(c.authors)}. <cite>${esc(c.title)}</cite>. ${esc(c.date)}. <a href="${esc(c.url)}">${esc(new URL(c.url).hostname)}</a> (${esc(c.grade)}; accessed ${esc(c.accessed)}). Supports: ${esc(c.supports.replace(/\.$/, ''))}.</li>`,
    )
    .join('\n');
  const body = `
<p>Generated from <a href="https://github.com/Avarok-Cybersecurity/atsgate/blob/main/CITATIONS.md">CITATIONS.md</a>, the single source of truth for every source ATSGate relies on. CI fails if the README or this page drifts from it.</p>
<h2 id="grades">What do the evidence grades mean?</h2>
${table('Evidence grades', ['Grade', 'Kind of source'], GRADES)}
<h2 id="sources">Which sources does ATSGate cite?</h2>
<ol>
${items}
</ol>`;
  const page = {
    route: '/citations/',
    title: 'Citations and sources · ATSGate',
    h1: 'Citations',
    description: `The ${citations.length} sources behind ATSGate’s scoring, fairness and vendor claims, each with an evidence grade and the claim it supports.`,
    lede: 'Every claim ATSGate makes about vendors, fairness, parsing and taxonomies, and the source it rests on.',
    body,
  };
  const cites = citations.map((c) => ({ '@type': 'CreativeWork', name: c.title, url: c.url }));
  const jsonld = graph(page);
  jsonld['@graph'].find((n) => n['@id'] === 'https://ats.express/citations/').citation = cites;
  return { ...page, jsonld };
}
