// The generated data under site/data/ (`cargo run -p xtask -- site --write`; CI runs `--check`).
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { esc } from './html.mjs';

const dir = join(import.meta.dirname, '..', 'data');
const load = (name) => JSON.parse(readFileSync(join(dir, name), 'utf8'));

export const scoring = load('scoring.json');
export const schema = load('blueprint-schema.json');
export const action = load('action.json');
export const citations = load('citations.json');
export const evaluation = load('evaluation.json');
export const blueprints = readdirSync(join(dir, 'blueprints'))
  .filter((f) => f.endsWith('.json'))
  .sort()
  .map((f) => load(join('blueprints', f)));

const ids = new Set(citations.map((c) => c.id));
/** A link to a CITATIONS.md entry; the build fails on an id it does not define. */
export function cite(id) {
  if (!ids.has(id)) throw new Error(`unknown citation id ${id}`);
  return `<a href="/citations/#cite-${esc(id)}">${esc(id)}</a>`;
}
