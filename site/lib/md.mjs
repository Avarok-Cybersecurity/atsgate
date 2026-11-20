// The Markdown subset the repository's docs use (headings, paragraphs, lists, tables, code spans,
// bold, italics and links), rendered without a dependency. Anything else fails the build rather
// than rendering wrongly.
import { esc, table } from './html.mjs';

export function inline(text) {
  const parts = text.split(/(`[^`]+`)/);
  return parts
    .map((part) => {
      if (part.startsWith('`') && part.endsWith('`') && part.length > 1) return `<code>${esc(part.slice(1, -1))}</code>`;
      return esc(part)
        .replace(/\[([^\]]+)\]\((https:\/\/[^)\s]+|\/[^)\s]*)\)/g, '<a href="$2">$1</a>')
        .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
        .replace(/(^|[\s(])\*([^*\s][^*]*)\*/g, '$1<em>$2</em>');
    })
    .join('');
}

const cells = (line) =>
  line
    .trim()
    .replace(/^\|/, '')
    .replace(/\|$/, '')
    .split('|')
    .map((c) => c.trim());

/**
 * Renders `markdown` below a page's own h1: the document's h1 is dropped, and heading levels are
 * clamped so none skips a level. `slug` turns heading text into an id.
 */
export function render(markdown, { caption = (h) => h } = {}) {
  const lines = markdown.replace(/<!--[\s\S]*?-->/g, '').split('\n');
  const out = [];
  let level = 1;
  let lastHeading = '';
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) {
      i += 1;
      continue;
    }
    const heading = line.match(/^(#{1,6}) (.+)$/);
    if (heading) {
      i += 1;
      if (heading[1].length === 1) continue;
      level = Math.min(heading[1].length, level + 1);
      lastHeading = heading[2];
      const id = heading[2].toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
      out.push(`<h${level} id="${id}">${inline(heading[2])}</h${level}>`);
      continue;
    }
    if (line.startsWith('|')) {
      const block = [];
      while (i < lines.length && lines[i].startsWith('|')) block.push(lines[i++]);
      if (!/^\|[-| :]+\|$/.test(block[1] ?? '')) throw new Error(`markdown table without a separator: ${block[0]}`);
      out.push(table(caption(lastHeading), cells(block[0]).map(inline), block.slice(2).map((r) => cells(r).map(inline))));
      continue;
    }
    if (/^- /.test(line)) {
      const items = [];
      while (i < lines.length && /^(- |  \S)/.test(lines[i])) {
        if (lines[i].startsWith('- ')) items.push(lines[i].slice(2));
        else items[items.length - 1] += ` ${lines[i].trim()}`;
        i += 1;
      }
      out.push(`<ul>\n${items.map((t) => `<li>${inline(t)}</li>`).join('\n')}\n</ul>`);
      continue;
    }
    if (/^(```|>|\d+\. |<)/.test(line)) throw new Error(`unsupported markdown block: ${line}`);
    const para = [];
    while (i < lines.length && lines[i].trim() && !/^(#|\||- )/.test(lines[i])) para.push(lines[i++].trim());
    out.push(`<p>${inline(para.join(' '))}</p>`);
  }
  return out.join('\n');
}
