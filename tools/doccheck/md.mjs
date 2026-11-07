// Markdown, read the way the docs are written: headings, fenced blocks with
// the headings they sit under, table rows, and links. Code fences are opaque.

export function splitRow(line) {
  const cells = [];
  let cur = '';
  let tick = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === '\\' && line[i + 1] === '|') { cur += '|'; i++; continue; }
    if (c === '`') tick = !tick;
    if (c === '|' && !tick) { cells.push(cur.trim()); cur = ''; continue; }
    cur += c;
  }
  cells.push(cur.trim());
  if (cells[0] === '') cells.shift();
  if (cells[cells.length - 1] === '') cells.pop();
  return cells;
}

export function parseMd(text) {
  const lines = text.split(/\r?\n/);
  const blocks = [];
  const rows = [];
  const headings = [];
  const prose = []; // lines outside fences, with their number
  let fence = null;
  let buf = [];
  let blockStart = 0;
  let h2 = '';
  let h3 = '';
  let table = null; // header cells of the current table
  lines.forEach((l, idx) => {
    const ln = idx + 1;
    const f = l.match(/^\s*```\s*([\w-]*)/);
    if (f) {
      if (fence === null) {
        fence = f[1];
        buf = [];
        blockStart = ln + 1;
      } else {
        blocks.push({ lang: fence, text: buf.join('\n'), line: blockStart, h2, h3 });
        fence = null;
      }
      table = null;
      return;
    }
    if (fence !== null) { buf.push(l); return; }
    prose.push({ line: ln, text: l });
    const hm = l.match(/^(#{1,6}) (.*)$/);
    if (hm) {
      const lvl = hm[1].length;
      headings.push({ level: lvl, text: hm[2].trim(), line: ln });
      if (lvl <= 2) { h2 = lvl === 2 ? hm[2].trim() : ''; h3 = ''; }
      if (lvl === 3) h3 = hm[2].trim();
      table = null;
      return;
    }
    if (l.startsWith('|')) {
      const cells = splitRow(l);
      if (cells.every((c) => /^:?-{3,}:?$/.test(c))) return;
      if (!table) { table = cells; return; }
      rows.push({ line: ln, cells, header: table, h2, h3 });
      return;
    }
    table = null;
  });
  return { blocks, rows, headings, prose };
}

/** GitHub's heading anchor. */
export function slug(h) {
  const s = h.trim().toLowerCase().replace(/`/g, '');
  let out = '';
  for (const c of s) if (/[\p{L}\p{N} _-]/u.test(c)) out += c;
  return out.replace(/ /g, '-');
}

export function anchors(headings) {
  const set = new Set();
  const seen = new Map();
  for (const h of headings) {
    const b = slug(h.text.replace(/\[([^\]]*)\]\([^)]*\)/g, '$1'));
    const n = seen.get(b) ?? 0;
    seen.set(b, n + 1);
    set.add(n === 0 ? b : `${b}-${n}`);
  }
  return set;
}

/** Relative links outside code: `[text](target)`. Inline code is skipped. */
export function links(prose) {
  const out = [];
  for (const { line, text } of prose) {
    const noCode = text.replace(/`[^`]*`/g, (m) => ' '.repeat(m.length));
    for (const m of noCode.matchAll(/\]\(([^)\s]+)\)/g)) out.push({ line, target: m[1] });
  }
  return out;
}

/** The backticked names in a cell: `a`, `b::c`. */
export function ticks(cell) {
  return [...cell.matchAll(/`([^`]+)`/g)].map((m) => m[1]);
}
