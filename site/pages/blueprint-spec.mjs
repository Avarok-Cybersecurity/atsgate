import { code, esc, graph, pre, table } from '../lib/html.mjs';
import { schema } from '../lib/data.mjs';
import { inline } from '../lib/md.mjs';

// The meaning of each field, in the order the page lists them. The build fails when the schema
// (generated from the Rust types) gains a field this page does not explain, or loses one it does.
const TOP = {
  $schema: 'Optional schema URL for editors; not read by the scorer.',
  id: 'Stable identifier; built-ins are addressed as `builtin:<id>`.',
  version: 'The blueprint’s own version.',
  title: 'The role title, the first target of the title score T.',
  title_aliases: 'Other titles for the same role, matched as the same words.',
  occupationalCategory: 'schema.org `occupationalCategory`, as an O*NET-SOC code. T credits titles of this occupation and of its SOC broad group.',
  esco_occupation: 'The matching ESCO occupation URI, or `null`.',
  seniority: 'The level the role is at; T is scaled down by the distance on the seniority ladder, half as much for a resume above it as below (see scoring). `unspecified` when the posting states none, which applies no seniority factor.',
  required_skills: 'Skills scored as R.',
  preferred_skills: 'Skills scored as P.',
  knockouts: 'Hard requirements that pass, fail or go to review; never points.',
  education: 'Degree level and fields scored as D.',
  experience: 'Minimum years and the titles whose dated roles count towards E.',
  certifications: 'Certifications the role names.',
  domain_keywords: 'Domain terms the role names.',
  languages: 'Languages the role requires.',
  provenance: 'Where the blueprint came from and whether a person reviewed it.',
};

const SUB = {
  'Skill.canonical': 'The canonical skill name, as the taxonomy spells it.',
  'Skill.aliases': 'Extra spellings, matched like taxonomy aliases.',
  'Skill.evidence_scope': 'Where the skill must appear to earn full credit.',
  'Knockout.type': 'What kind of requirement it is.',
  'Knockout.value': 'The requirement, for example a degree level or a certification name.',
  'Knockout.unknown_policy': 'What an undecidable knockout resolves to.',
  'EducationRequirement.min_level': 'The lowest degree level that earns full level credit, or `null`.',
  'EducationRequirement.fields': 'Fields of study; a degree title naming one earns field credit.',
  'EducationLevel.framework': 'ISCED 2011 or EQF.',
  'ExperienceRequirement.min_years': 'Years at which E reaches 1.',
  'ExperienceRequirement.relevant_titles': 'Titles whose dated roles count as relevant years.',
  'Provenance.source': 'How the blueprint was made.',
  'Provenance.reviewed': 'Whether a person reviewed it. Generated blueprints are always `false`.',
};

const refName = (ref) => ref.split('/').pop();
function typeOf(p) {
  if (p.$ref) return `<a href="#${refName(p.$ref).toLowerCase()}">${refName(p.$ref)}</a>`;
  if (p.anyOf) return p.anyOf.map(typeOf).join(' or ');
  if (p.type === 'array') return `array of ${typeOf(p.items)}`;
  if (Array.isArray(p.type)) return p.type.join(' or ');
  return esc(p.type);
}

function checkKeys(where, actual, explained) {
  const missing = actual.filter((k) => !explained.includes(k));
  const stale = explained.filter((k) => !actual.includes(k));
  if (missing.length || stale.length) throw new Error(`${where}: unexplained ${missing}; no longer in the schema ${stale}`);
}

function objectTable(name, def) {
  const rows = Object.entries(def.properties).map(([field, p]) => {
    const meaning = def.properties[field].description ?? SUB[`${name}.${field}`];
    if (!meaning) throw new Error(`blueprint spec: no description for ${name}.${field}`);
    return [code(field), typeOf(p), def.required.includes(field) ? 'yes' : 'no', inline(meaning)];
  });
  return `<h3 id="${name.toLowerCase()}">${name}</h3>\n${table(`${name} fields`, ['Field', 'Type', 'Required', 'Meaning'], rows)}`;
}

// A string enum: plain (`enum`), or `oneOf` when schemars keeps a documented variant apart.
function enumParts(name, d) {
  if (d.enum) return [{ values: d.enum, description: d.description }];
  if (d.oneOf) return d.oneOf.flatMap((v) => enumParts(name, v));
  if (d.const !== undefined) return [{ values: [d.const], description: d.description }];
  throw new Error(`blueprint spec: ${name} is neither an object nor a string enum`);
}

function enumRow(name, d) {
  const parts = enumParts(name, d);
  const values = parts.flatMap((p) => p.values);
  const notes = parts.filter((p) => p.description).map((p) => `${p.values.map((v) => `\`${v}\``).join(', ')}: ${p.description.replace(/\n/g, ' ')}`);
  if (d.description) notes.unshift(d.description);
  return [`<span id="${name.toLowerCase()}">${name}</span>`, values.map(code).join(', '), notes.length ? notes.map(inline).join('<br>') : '—'];
}

const EXAMPLE = `required_skills:
  - canonical: "PostgreSQL"
    taxonomy_ref: "onet-tech:PostgreSQL"
    aliases: []
    weight: 1.0
    evidence_scope: experience_or_projects
knockouts:
  - { type: authorization, value: "US work authorization", unknown_policy: review }`;

export default function blueprintSpecPage() {
  checkKeys('blueprint spec', Object.keys(schema.properties), Object.keys(TOP));
  const subKeys = Object.keys(SUB).map((k) => k.split('.'));
  const stale = subKeys.filter(([d, f]) => !schema.$defs[d]?.properties?.[f]).map((k) => k.join('.'));
  if (stale.length) throw new Error(`blueprint spec explains fields no longer in the schema: ${stale}`);
  const top = Object.entries(TOP).map(([field, meaning]) => {
    const p = schema.properties[field];
    return [code(field), typeOf(p), schema.required.includes(field) ? 'yes' : 'no', inline(meaning)];
  });
  const defs = Object.entries(schema.$defs);
  const objects = defs.filter(([, d]) => d.type === 'object').map(([n, d]) => objectTable(n, d)).join('\n');
  const enums = defs.filter(([, d]) => d.type !== 'object').map(([n, d]) => enumRow(n, d));
  const body = `
<p>The tables below are generated from <code>schemas/blueprint.schema.json</code>, which is itself generated from the Rust types the parser uses, so the spec, the schema and the parser cannot drift. Every key is required; a value that does not apply is <code>null</code> or <code>[]</code>, never left out. Unknown fields are errors.</p>

<h2 id="fields">What fields does a blueprint have?</h2>
${table('Blueprint fields', ['Field', 'Type', 'Required', 'Meaning'], top)}

<h2 id="objects">What do the nested objects contain?</h2>
${objects}

<h2 id="enums">Which values are allowed?</h2>
${table('Enumerations', ['Type', 'Values', 'Meaning'], enums)}

<h2 id="example">What does a skill and a knockout look like?</h2>
${pre(EXAMPLE)}
<p>Every built-in skill names a taxonomy concept (O*NET 31.0, ESCO v1.2.1 or the curated table), and a test fails if any stops resolving. A blueprint you write yourself may use <code>taxonomy_ref: null</code> for a skill no taxonomy has.</p>

<h2 id="generate">Can I generate a blueprint from a job posting?</h2>
<p>Yes: <code>blueprint: generate:posting.txt</code>. The generator is deterministic. Section cues decide required versus preferred, and it also reads years, degrees, certifications and the seniority in the title, writing <code>unspecified</code> when the posting states none. Its output is always <code>reviewed: false</code>; its measured precision and recall are on the <a href="/docs/evaluation/">evaluation page</a>.</p>`;
  const page = {
    route: '/docs/blueprint-spec/',
    type: 'TechArticle',
    title: 'Job blueprint specification · ATSGate',
    h1: 'Job blueprint specification',
    description: 'Every field of an ATSGate job blueprint, generated from the JSON Schema: skills, knockouts, education, experience and provenance.',
    lede: 'A blueprint is a standardized description of what a role requires, aligned with schema.org JobPosting. ATSGate scores a resume against one.',
    body,
  };
  return { ...page, jsonld: graph(page) };
}
