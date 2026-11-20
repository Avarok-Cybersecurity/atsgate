// The built-in blueprint index and one page per blueprint, each with an Occupation JSON-LD node.
import { ORIGIN, code, esc, graph, pre, table } from '../lib/html.mjs';
import { blueprints } from '../lib/data.mjs';

const SRC = 'https://github.com/Avarok-Cybersecurity/atsgate/blob/main/blueprints';
const onet = (soc) => `https://www.onetonline.org/link/summary/${soc}`;
const list = (xs) => (xs.length ? xs.map(esc).join(', ') : 'none');
const route = (b) => `/blueprints/${b.id}/`;

function education(b) {
  const e = b.education;
  const level = e.min_level ? `${e.min_level.framework.toUpperCase()} level ${e.min_level.level}` : 'no minimum level';
  const fields = e.fields.length ? ` in ${list(e.fields)}` : '';
  return `${level}${fields}${e.experience_in_place_of_education ? '; relevant experience at the minimum years can stand in for it' : ''}`;
}

function skillRows(skills) {
  return skills.map((s) => [esc(s.canonical), String(s.weight), code(s.evidence_scope), s.taxonomy_ref ? code(s.taxonomy_ref) : '—']);
}

function occupation(b) {
  const skills = [...b.required_skills, ...b.preferred_skills].map((s) => ({
    '@type': 'DefinedTerm',
    name: s.canonical,
    ...(s.taxonomy_ref?.startsWith('http') ? { sameAs: s.taxonomy_ref } : {}),
  }));
  return {
    '@type': 'Occupation',
    '@id': `${ORIGIN}${route(b)}#occupation`,
    name: b.title,
    alternateName: b.title_aliases,
    occupationalCategory: { '@type': 'CategoryCode', codeValue: b.occupationalCategory, inCodeSet: 'O*NET-SOC', url: onet(b.occupationalCategory) },
    ...(b.esco_occupation ? { sameAs: b.esco_occupation } : {}),
    skills,
    educationRequirements: education(b),
    experienceRequirements: { '@type': 'OccupationalExperienceRequirements', monthsOfExperience: Math.round(b.experience.min_years * 12) },
    qualifications: b.certifications.length ? b.certifications.join(', ') : undefined,
  };
}

export function blueprintPage(b) {
  const knockouts = b.knockouts.length
    ? table('Knockouts', ['Type', 'Value', 'If unknown'], b.knockouts.map((k) => [code(k.type), esc(k.value), code(k.unknown_policy)]))
    : '<p>None: this blueprint has no hard requirements, so nothing fails the gate except the floors and, if you ask, integrity flags.</p>';
  const body = `
<h2 id="use">How do I score a resume against it?</h2>
${pre(`blueprint: builtin:${b.id}`)}
<p>Version ${esc(b.version)}, ${b.provenance.reviewed ? 'reviewed by a person' : 'not yet reviewed'}. O*NET-SOC <a href="${onet(b.occupationalCategory)}">${esc(b.occupationalCategory)}</a>${b.esco_occupation ? `, <a href="${esc(b.esco_occupation.replace('http://', 'https://'))}">ESCO occupation</a>` : ''}. Seniority: ${code(b.seniority)}. Source: <a href="${SRC}/${b.id}.yaml">${b.id}.yaml</a>.</p>

<h2 id="required">Which skills are required?</h2>
${table('Required skills (R)', ['Skill', 'Weight', 'Evidence scope', 'Taxonomy reference'], skillRows(b.required_skills))}

<h2 id="preferred">Which skills are preferred?</h2>
${b.preferred_skills.length ? table('Preferred skills (P)', ['Skill', 'Weight', 'Evidence scope', 'Taxonomy reference'], skillRows(b.preferred_skills)) : '<p>None.</p>'}

<h2 id="experience">What experience and education does it ask for?</h2>
<ul>
<li><strong>Years:</strong> at least ${b.experience.min_years} in roles titled ${list(b.experience.relevant_titles)}. Gaps between roles never subtract.</li>
<li><strong>Education:</strong> ${education(b)}.</li>
<li><strong>Titles matched as the same role:</strong> ${esc(b.title)}, ${list(b.title_aliases)}.</li>
<li><strong>Certifications:</strong> ${list(b.certifications)}. <strong>Languages:</strong> ${list(b.languages)}. <strong>Domain keywords:</strong> ${list(b.domain_keywords)}.</li>
</ul>

<h2 id="knockouts">Does it have knockouts?</h2>
${knockouts}`;
  const page = {
    route: route(b),
    title: `${b.title} blueprint · ATSGate`,
    h1: `${b.title} blueprint`,
    description: `The built-in ATSGate blueprint for a ${b.title}: required and preferred skills with taxonomy references, years, education and knockouts.`.slice(0, 155),
    lede: `What ATSGate looks for when it scores a resume against <code>builtin:${esc(b.id)}</code>: ${b.required_skills.length} required skills, ${b.preferred_skills.length} preferred skills and at least ${b.experience.min_years} years.`,
    body,
  };
  return { ...page, jsonld: graph(page, [occupation(b)]) };
}

export function blueprintIndex() {
  const rows = blueprints.map((b) => [
    `<a href="${route(b)}">${esc(b.title)}</a>`,
    code(`builtin:${b.id}`),
    esc(b.occupationalCategory),
    String(b.required_skills.length),
    String(b.experience.min_years),
  ]);
  const body = `
<h2 id="list">Which blueprints are built in?</h2>
${table('Built-in blueprints', ['Role', 'Input value', 'O*NET-SOC', 'Required skills', 'Min years'], rows)}
<h2 id="own">Can I write my own?</h2>
<p>Yes. A blueprint is a YAML or JSON file that follows the <a href="/docs/blueprint-spec/">blueprint spec</a>. You can also generate a draft from a job posting with <code>blueprint: generate:posting.txt</code>; it is marked <code>reviewed: false</code> until you review it.</p>`;
  const page = {
    route: '/blueprints/',
    title: 'Built-in job blueprints · ATSGate',
    h1: 'Built-in job blueprints',
    description: `The ${blueprints.length} reviewed job blueprints built into ATSGate, from backend engineer to paralegal, each tied to O*NET and ESCO.`,
    lede: `ATSGate ships ${blueprints.length} reviewed blueprints. Every skill in them names an O*NET, ESCO or curated taxonomy concept, and a test fails if one stops resolving.`,
    body,
  };
  return { ...page, jsonld: graph(page) };
}

export const blueprintPages = () => [blueprintIndex(), ...blueprints.map(blueprintPage)];
