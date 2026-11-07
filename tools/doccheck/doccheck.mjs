#!/usr/bin/env node
// doccheck proves that the docs and the code agree (ADR-0020).
//
// On every run it re-extracts the inventory from docs/: the types, functions
// and constants of the ```rust blocks of 10 and 11, the wire types of 10 (doc
// fields = Rust fields = TS fields), the commands, events and files of 12, the
// collector flags of 05, the test catalog of 13 and frontend/06, the
// traceability matrix of the requisitos, the iTero decisions of 07, every
// relative link and every cited ID. It looks for each item in the code, and
// for each public item of the code in the docs.
//
// An item is done when it resolves, deferred when tools/doccheck/deferred.txt
// names it with a reason, pending otherwise. Problems that are not items
// (a deferral that matches nothing, a broken BUILT row) are warnings.
// The ledger goes to build/ledger.md. -strict exits 1 on any pending item or
// warning; -closed exits 1 on any OPEN row of build/audit.md.

import { existsSync, readFileSync, readdirSync, statSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { scanRust, modulePath } from './rust.mjs';
import { scanTs } from './ts.mjs';
import { parseMd, anchors, links, ticks } from './md.mjs';

export const DOCS = {
  model: 'docs/arquitetura/10-modelo-de-dados.md',
  catalog: 'docs/arquitetura/11-catalogo-de-funcoes.md',
  contract: 'docs/arquitetura/12-contrato-da-casca.md',
  tests: 'docs/arquitetura/13-testes.md',
  collector: 'docs/arquitetura/05-coletor.md',
  uitests: 'docs/frontend/06-testes.md',
  reqs: 'docs/produto/requisitos.md',
  rules: 'docs/produto/regras-e-pesos.md',
  questions: 'docs/produto/perguntas-em-aberto.md',
  lessons: 'docs/arquitetura/09-licoes-do-prototipo.md',
  itero: 'docs/referencia/itero/05-funcionalidades.md',
  cross: 'docs/referencia/itero/07-cruzamento.md',
  gaps: 'docs/frontend/07-lacunas-da-casca.md',
  audit: 'build/audit.md',
};

const CRATES = [
  ['core', 'core/src'],
  ['casca', 'src-tauri/src'],
  ['coletor', 'tools/ingest/src'],
];

export const KINDS = ['type', 'func', 'const', 'wire', 'command', 'event', 'file', 'flag', 'test', 'ui-test', 'trace', 'itero', 'undoc', 'todo'];
export const DECISIONS = ['copiar', 'mudar', 'rejeitar', 'adiar', 'fora'];
const TYPE_KINDS = new Set(['struct', 'enum', 'trait', 'type']);

// ---------------------------------------------------------------------------
// Files
// ---------------------------------------------------------------------------

function walk(dir, keep) {
  if (!existsSync(dir)) return [];
  const out = [];
  for (const f of readdirSync(dir)) {
    if (f === 'node_modules' || f === 'target' || f === '.git') continue;
    const p = join(dir, f);
    if (statSync(p).isDirectory()) out.push(...walk(p, keep));
    else if (keep(p)) out.push(p);
  }
  return out.sort();
}

const posix = (p) => p.split(sep).join('/');
const read = (root, rel) => (existsSync(join(root, rel)) ? readFileSync(join(root, rel), 'utf8') : null);

// ---------------------------------------------------------------------------
// Code
// ---------------------------------------------------------------------------

export function scanCode(root) {
  const code = { items: [], tests: [], commands: [], handlers: [], strings: { core: [], casca: [], coletor: [] }, comments: [] };
  for (const [crate, dir] of CRATES) {
    const base = join(root, dir);
    for (const f of walk(base, (p) => p.endsWith('.rs'))) {
      const rel = posix(relative(root, f));
      const r = scanRust(readFileSync(f, 'utf8'), { file: rel, module: modulePath(crate, posix(relative(base, f))) });
      code.items.push(...r.items.map((x) => ({ ...x, crate })));
      code.tests.push(...r.tests);
      code.commands.push(...r.commands);
      code.handlers.push(...r.handlers);
      code.strings[crate].push(...r.strings.map((s) => ({ ...s, file: rel })));
      code.comments.push(...r.comments.map((c) => ({ ...c, file: rel })));
    }
  }
  code.ts = { interfaces: [], unions: [], invokes: [], strings: [], tests: [] };
  for (const f of walk(join(root, 'src'), (p) => /\.(ts|tsx)$/.test(p))) {
    const rel = posix(relative(root, f));
    const r = scanTs(readFileSync(f, 'utf8'), { file: rel });
    code.ts.interfaces.push(...r.interfaces);
    code.ts.unions.push(...r.unions);
    code.ts.invokes.push(...r.invokes);
    code.ts.strings.push(...r.strings.map((s) => ({ ...s, file: rel })));
    code.comments.push(...r.comments.map((c) => ({ ...c, file: rel })));
    if (/\.(test|spec)\.tsx?$/.test(f)) {
      const text = readFileSync(f, 'utf8');
      for (const m of text.matchAll(/\bTF-[A-Z]+-\d+\b/g)) code.ts.tests.push({ id: m[0], file: rel, line: text.slice(0, m.index).split('\n').length });
    }
  }
  return code;
}

// ---------------------------------------------------------------------------
// Docs
// ---------------------------------------------------------------------------

const MODULE_IN_HEADING = /`((?:core|casca|coletor)(?:::\w+)*)`/;

function moduleOf(block) {
  return block.h3.match(MODULE_IN_HEADING)?.[1] ?? block.h2.match(MODULE_IN_HEADING)?.[1] ?? null;
}

function section(heading) {
  const m = heading.match(/^([\d.]+)\s/);
  return m ? `§${m[1].replace(/\.$/, '')}` : heading.slice(0, 40);
}

function docKind(k) {
  if (TYPE_KINDS.has(k)) return 'type';
  if (k === 'fn' || k === 'method') return 'func';
  if (k === 'const') return 'const';
  return null;
}

export function inventory(root, warn) {
  const items = [];
  const docs = {};
  for (const [key, rel] of Object.entries(DOCS)) {
    const text = read(root, rel);
    docs[key] = text === null ? null : parseMd(text);
  }
  const short = (rel) => rel.replace(/^docs\//, '').replace(/\.md$/, '');

  // types, functions, constants, wire types: the rust blocks of 10 and 11
  const wires = [];
  for (const key of ['model', 'catalog']) {
    const md = docs[key];
    if (!md) continue;
    for (const b of md.blocks) {
      if (b.lang !== 'rust') continue;
      const mod = moduleOf(b);
      if (!mod) {
        warn(`${DOCS[key]}:${b.line}: bloco rust sem módulo no título (\`core::…\`, \`casca::…\`, \`coletor::…\`)`);
        continue;
      }
      const r = scanRust(b.text, { module: mod });
      for (const it of r.items) {
        const kind = docKind(it.kind);
        if (!kind) continue;
        items.push({ kind, name: `${it.module}::${it.name}`, module: it.module, local: it.name, source: `${short(DOCS[key])} ${section(b.h3 || b.h2)}`, docLine: b.line + it.line - 1, doc: it });
      }
      const w = b.h3.match(/`(\w+)`\s*↔\s*`(\w+)`/);
      if (w && key === 'model') {
        const decl = r.items.find((x) => x.name === w[1] && (x.kind === 'struct' || x.kind === 'enum'));
        if (!decl) warn(`${DOCS[key]}:${b.line}: o título liga \`${w[1]}\` ↔ \`${w[2]}\`, e o bloco não declara \`${w[1]}\``);
        else wires.push({ kind: 'wire', name: `${w[1]} ↔ ${w[2]}`, module: mod, rust: w[1], ts: w[2], doc: decl, source: `${short(DOCS[key])} ${section(b.h3)}` });
      }
    }
  }
  items.push(...wires);

  // commands, events, files: the tables of 12, by section
  if (docs.contract) {
    for (const row of docs.contract.rows) {
      const first = ticks(row.cells[0] ?? '')[0];
      if (!first) continue;
      const src = `${short(DOCS.contract)} ${section(row.h2)}`;
      if (/comandos/i.test(row.h2) && /^[a-z_]+$/.test(first)) items.push({ kind: 'command', name: first, source: src, docLine: row.line });
      else if (/eventos/i.test(row.h2) && /^presage:\/\/[a-z-]+$/.test(first)) items.push({ kind: 'event', name: first, source: src, docLine: row.line });
      else if (/disco/i.test(row.h2) && /\.json$/.test(first)) items.push({ kind: 'file', name: first, source: src, docLine: row.line });
    }
  }

  // collector flags: the rows of 05 whose first cell is `--flag`
  if (docs.collector) {
    for (const row of docs.collector.rows) {
      const first = ticks(row.cells[0] ?? '')[0];
      if (first && /^--[a-z][a-z-]*$/.test(first)) items.push({ kind: 'flag', name: first, source: `${short(DOCS.collector)} ${section(row.h2)}`, docLine: row.line });
    }
  }

  // the test catalog: | T-AREA-n | `fn`, `fn` | …
  const catalog = new Map();
  if (docs.tests) {
    for (const row of docs.tests.rows) {
      const id = row.cells[0]?.match(/^(T-[A-Z]+-\d+)$/)?.[1];
      if (!id) continue;
      if (catalog.has(id)) warn(`${DOCS.tests}:${row.line}: ${id} aparece duas vezes no catálogo`);
      const names = ticks(row.cells[1] ?? '');
      catalog.set(id, names);
      items.push({ kind: 'test', name: id, tests: names, source: `${short(DOCS.tests)} ${section(row.h2)}`, docLine: row.line });
    }
  }
  const uiCatalog = new Set();
  if (docs.uitests) {
    for (const row of docs.uitests.rows) {
      const id = row.cells[0]?.match(/^(TF-[A-Z]+-\d+)$/)?.[1];
      if (!id) continue;
      uiCatalog.add(id);
      items.push({ kind: 'ui-test', name: id, source: `${short(DOCS.uitests)} ${section(row.h2)}`, docLine: row.line });
    }
  }

  // requirements and the matrix
  const reqs = new Map();
  const matrix = new Map();
  if (docs.reqs) {
    for (const h of docs.reqs.headings) {
      const m = h.text.match(/^(R-\d+)\b/);
      if (m) reqs.set(m[1], h.line);
    }
    for (const row of docs.reqs.rows) {
      const id = row.cells[0]?.match(/^\**((?:R|C|RNF)-\d+)\b/)?.[1];
      if (!id) continue;
      if (/matriz/i.test(row.h2)) matrix.set(id, row);
      else if (!id.startsWith('R-')) reqs.set(id, row.line);
    }
    for (const [id] of reqs) items.push({ kind: 'trace', name: id, row: matrix.get(id), source: `${short(DOCS.reqs)} matriz`, docLine: matrix.get(id)?.line ?? reqs.get(id) });
    for (const [id, row] of matrix) if (!reqs.has(id)) warn(`${DOCS.reqs}:${row.line}: a matriz cita ${id}, que nenhum requisito define`);
  }

  // iTero decisions
  if (docs.cross) {
    for (const row of docs.cross.rows) {
      const id = row.cells[0]?.match(/^(I-\d+)$/)?.[1];
      if (!id) continue;
      const col = row.header.findIndex((h) => /decis/i.test(h));
      const note = row.header.findIndex((h) => /nota|motivo/i.test(h));
      items.push({ kind: 'itero', name: id, decision: col >= 0 ? row.cells[col] : null, note: note >= 0 ? row.cells[note] : '', source: `${short(DOCS.cross)} ${section(row.h2)}`, docLine: row.line });
    }
  }

  // ID definitions, for the cross-reference check
  const defs = { T: new Set(catalog.keys()), TF: uiCatalog, R: new Set([...reqs.keys()].filter((k) => k.startsWith('R-'))), C: new Set([...reqs.keys()].filter((k) => k.startsWith('C-'))), RNF: new Set([...reqs.keys()].filter((k) => k.startsWith('RNF-'))) };
  const fromHeadings = (md, re) => new Set((md?.headings ?? []).map((h) => h.text.match(re)?.[1]).filter(Boolean));
  const fromRows = (md, re) => new Set((md?.rows ?? []).map((r) => r.cells[0]?.match(re)?.[1]).filter(Boolean));
  defs.RN = fromHeadings(docs.rules, /^(RN-\d+)\b/);
  defs.P = fromHeadings(docs.questions, /^(P-\d+)\b/);
  defs.L = fromRows(docs.lessons, /^(L-\d+)$/);
  defs.I = fromRows(docs.itero, /^(I-\d+)$/);
  defs.G = new Set([...fromRows(docs.cross, /^(G-\d+)$/), ...fromHeadings(docs.cross, /^(G-\d+)\b/)]);
  defs.GC = fromHeadings(docs.gaps, /^(GC-\d+)\b/);

  return { items, docs, catalog, defs };
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

export function readDeferred(text, warn) {
  const out = new Map();
  if (!text) return out;
  text.split(/\r?\n/).forEach((l, i) => {
    const t = l.trim();
    if (!t || t.startsWith('#')) return;
    const m = t.match(/^(\S+)\s+(.+?)\s*\|\s*(.*)$/);
    if (!m || !m[3].trim()) { warn(`tools/doccheck/deferred.txt:${i + 1}: adiamento sem motivo: ${t}`); return; }
    out.set(`${m[1]} ${m[2]}`, { reason: m[3].trim(), used: false, line: i + 1 });
  });
  return out;
}

function wireCompare(item, code, warnings) {
  const rust = code.items.find((x) => x.module === item.module && x.name === item.rust && (x.kind === 'struct' || x.kind === 'enum'));
  if (!rust) return `\`${item.rust}\` não existe em \`${item.module}\``;
  const problems = [];
  const same = (a, b) => a.length === b.length && a.every((x, i) => x === b[i]);
  if (rust.kind === 'struct') {
    if (!rust.fields) return null; // tuple struct: nothing to compare
    const docNames = (item.doc.fields ?? []).map((f) => f.name).sort();
    const rustNames = rust.fields.map((f) => f.name).sort();
    if (!same(docNames, rustNames)) problems.push(`doc ≠ Rust: ${diff(docNames, rustNames)}`);
    const docWire = (item.doc.fields ?? []).map((f) => f.wire).filter(Boolean).sort();
    const rustWire = rust.fields.map((f) => f.wire).filter(Boolean).sort();
    if (same(docNames, rustNames) && !same(docWire, rustWire)) problems.push(`serde do doc ≠ serde do Rust: ${diff(docWire, rustWire)}`);
    const ts = tsFields(item.ts, code);
    if (!ts) problems.push(`\`${item.ts}\` não existe em src/`);
    else if (!same(rustWire, [...ts].sort())) problems.push(`Rust ≠ TS: ${diff(rustWire, [...ts].sort())}`);
  } else {
    const docV = (item.doc.variants ?? []).map((v) => v.name).sort();
    const rustV = (rust.variants ?? []).map((v) => v.name).sort();
    if (!same(docV, rustV)) problems.push(`doc ≠ Rust: ${diff(docV, rustV)}`);
    const u = code.ts.unions.find((x) => x.name === item.ts);
    const rustW = (rust.variants ?? []).filter((v) => v.unit && v.wire).map((v) => v.wire).sort();
    if (!u) problems.push(`união \`${item.ts}\` não existe em src/`);
    else if (!same(rustW, [...u.values].sort())) problems.push(`Rust ≠ TS: ${diff(rustW, [...u.values].sort())}`);
  }
  return problems.length ? problems.join('; ') : null;
}

function tsFields(name, code, seen = new Set()) {
  if (seen.has(name)) return new Set();
  seen.add(name);
  const it = code.ts.interfaces.find((x) => x.name === name);
  if (!it) return null;
  const out = new Set(it.fields);
  for (const e of it.extends) for (const f of tsFields(e, code, seen) ?? []) out.add(f);
  return out;
}

function diff(a, b) {
  const A = new Set(a);
  const B = new Set(b);
  const only = (x, y) => [...x].filter((v) => !y.has(v));
  const parts = [];
  if (only(A, B).length) parts.push(`só no primeiro: ${only(A, B).join(', ')}`);
  if (only(B, A).length) parts.push(`só no segundo: ${only(B, A).join(', ')}`);
  return parts.join(' / ') || 'ordem';
}

const where = (x) => `${x.file}:${x.line}`;

export function resolveAll(inv, code, deferred, warn) {
  const { items } = inv;
  const byKey = new Map();
  for (const x of code.items) {
    const k = `${x.module}::${x.name}`;
    if (!byKey.has(k)) byKey.set(k, []);
    byKey.get(k).push(x);
  }
  const testsByName = new Map();
  for (const t of code.tests) if (!testsByName.has(t.name)) testsByName.set(t.name, t);
  const commands = new Map(code.commands.map((c) => [c.name, c]));
  const handlers = new Set(code.handlers);
  const invoked = new Set(code.ts.invokes.map((x) => x.name));
  const lit = (crates) => crates.flatMap((c) => code.strings[c]);
  const tsLit = new Set(code.ts.strings.map((s) => s.value));

  for (const it of items) {
    let found = null;
    let why = '';
    switch (it.kind) {
      case 'type':
      case 'func':
      case 'const': {
        const cand = (byKey.get(it.name) ?? []).filter((x) => docKind(x.kind) === it.kind);
        if (cand.length) found = where(cand[0]);
        else why = `não existe em \`${it.module}\``;
        break;
      }
      case 'wire': {
        const p = wireCompare(it, code);
        const rust = code.items.find((x) => x.module === it.module && x.name === it.rust);
        if (!p && rust) found = where(rust);
        else why = p;
        break;
      }
      case 'command': {
        const c = commands.get(it.name);
        if (!c) why = 'sem `#[tauri::command]`';
        else if (!handlers.has(it.name)) why = 'fora do `generate_handler!`';
        else if (!invoked.has(it.name)) why = 'nenhum `invoke` em src/';
        else found = where(c);
        break;
      }
      case 'event': {
        const s = lit(['casca']).find((x) => x.value === it.name);
        if (!s) why = 'a casca não emite';
        else if (!tsLit.has(it.name)) why = 'nenhum `listen` em src/';
        else found = `${s.file}:${s.line}`;
        break;
      }
      case 'file': {
        const s = lit(['casca', 'coletor']).find((x) => x.value === it.name || x.value.endsWith(`/${it.name}`) || x.value.endsWith(`\\${it.name}`));
        if (s) found = `${s.file}:${s.line}`;
        else why = 'nenhum literal com esse nome na casca ou no coletor';
        break;
      }
      case 'flag': {
        const s = lit(['coletor']).find((x) => x.value === it.name);
        if (s) found = `${s.file}:${s.line}`;
        else why = 'o coletor não lê essa opção';
        break;
      }
      case 'test': {
        if (!it.tests.length) { why = 'a linha não nomeia nenhum teste'; break; }
        const missing = it.tests.filter((n) => !testsByName.has(n));
        if (missing.length) why = `teste inexistente: ${missing.join(', ')}`;
        else found = where(testsByName.get(it.tests[0]));
        break;
      }
      case 'ui-test': {
        const t = code.ts.tests.find((x) => x.id === it.name);
        if (t) found = where(t);
        else why = 'nenhum teste do front carrega o ID';
        break;
      }
      case 'trace': {
        if (!it.row) { why = 'sem linha na matriz'; break; }
        const cited = [...(it.row.cells[2] ?? '').matchAll(/\bT-[A-Z]+-\d+\b/g)].map((m) => m[0]);
        const bad = cited.filter((t) => !inv.catalog.has(t));
        if (bad.length) why = `T-ID fora do catálogo: ${bad.join(', ')}`;
        else if (!cited.length) why = 'nenhum teste na matriz';
        else found = `${DOCS.reqs}:${it.row.line}`;
        break;
      }
      case 'itero': {
        const d = (it.decision ?? '').replace(/[*_]/g, '').trim().split(/[\s·,(]/)[0].toLowerCase();
        if (it.decision === null) why = 'a tabela não tem coluna de decisão';
        else if (!DECISIONS.includes(d)) why = `decisão "${it.decision}" fora de ${DECISIONS.join('/')}`;
        else if (!it.note.trim() && !/\bR-\d+/.test(it.decision)) why = 'decisão sem motivo';
        else found = `${DOCS.cross}:${it.docLine}`;
        break;
      }
      default:
        break;
    }
    if (found) {
      it.state = 'done';
      it.where = found;
      continue;
    }
    const d = deferred.get(`${it.kind} ${it.name}`);
    if (d) {
      d.used = true;
      it.state = 'deferred';
      it.where = d.reason;
    } else {
      it.state = 'pending';
      it.where = why;
    }
  }

  // reverse: what the code has and the docs do not
  const docNames = new Set(items.filter((x) => ['type', 'func', 'const'].includes(x.kind)).map((x) => x.name));
  const docKinds = (k) => new Set(items.filter((x) => x.kind === k).map((x) => x.name));
  const undoc = [];
  for (const x of code.items) {
    const coreApi = x.crate === 'core' && x.isPub && !x.trait && docKind(x.kind);
    const wire = x.crate === 'casca' && TYPE_KINDS.has(x.kind) && x.attrs.some((a) => /^derive\(.*\bSerialize\b/.test(a));
    if ((coreApi || wire) && !docNames.has(`${x.module}::${x.name}`)) undoc.push({ kind: 'undoc', name: `${x.module}::${x.name}`, source: where(x), why: `${x.kind} público fora do 10/11` });
  }
  const cmdDoc = docKinds('command');
  for (const c of code.commands) if (!cmdDoc.has(c.name)) undoc.push({ kind: 'undoc', name: `command ${c.name}`, source: where(c), why: 'comando fora do 12' });
  const evDoc = docKinds('event');
  const seenEv = new Set();
  for (const s of code.strings.casca) if (/^presage:\/\/[a-z-]+$/.test(s.value) && !evDoc.has(s.value) && !seenEv.has(s.value)) { seenEv.add(s.value); undoc.push({ kind: 'undoc', name: `event ${s.value}`, source: `${s.file}:${s.line}`, why: 'evento fora do 12' }); }
  const fileDoc = docKinds('file');
  const seenF = new Set();
  for (const s of lit(['casca', 'coletor'])) {
    const base = s.value.split(/[\\/]/).pop();
    if (/^[\w.-]+\.json$/.test(base) && s.value === base && !fileDoc.has(base) && !seenF.has(base) && !/^(package|tauri\.conf|latest)\.json$/.test(base)) { seenF.add(base); undoc.push({ kind: 'undoc', name: `file ${base}`, source: `${s.file}:${s.line}`, why: 'arquivo fora do 12' }); }
  }
  const flagDoc = docKinds('flag');
  const seenFl = new Set();
  for (const s of code.strings.coletor) if (/^--[a-z][a-z-]*$/.test(s.value) && !flagDoc.has(s.value) && !seenFl.has(s.value)) { seenFl.add(s.value); undoc.push({ kind: 'undoc', name: `flag ${s.value}`, source: `${s.file}:${s.line}`, why: 'opção fora do 05' }); }
  for (const u of undoc) {
    const d = deferred.get(`undoc ${u.name}`);
    if (d) { d.used = true; u.state = 'deferred'; u.where = d.reason; }
    else { u.state = 'pending'; u.where = u.why; }
  }
  items.push(...undoc);

  // TODOs in code
  for (const c of code.comments) {
    const m = c.text.match(/\bTODO(\([^)]*\))?/);
    if (!m) continue;
    const name = `${c.file} TODO${m[1] ?? ''}`;
    const it = { kind: 'todo', name, source: `${c.file}:${c.line}` };
    const d = deferred.get(`todo ${name}`);
    if (/deferred:/.test(c.text)) { it.state = 'deferred'; it.where = c.text.split('deferred:')[1].trim(); }
    else if (d) { d.used = true; it.state = 'deferred'; it.where = d.reason; }
    else { it.state = 'pending'; it.where = c.text.replace(/^\/\/[!/]?\s*/, '').slice(0, 90); }
    items.push(it);
  }

  for (const [k, d] of deferred) if (!d.used) warn(`tools/doccheck/deferred.txt:${d.line}: o adiamento \`${k}\` não casa com nenhum item`);
  return items;
}

// ---------------------------------------------------------------------------
// Prose checks: links, IDs, audit
// ---------------------------------------------------------------------------

export function checkLinks(root, warn) {
  const files = [
    ...walk(join(root, 'docs'), (p) => p.endsWith('.md')),
    ...walk(join(root, 'build'), (p) => p.endsWith('.md')),
    ...walk(join(root, '.github'), (p) => p.endsWith('.md')),
    ...['README.md', 'CLAUDE.md'].map((f) => join(root, f)).filter(existsSync),
  ];
  const cache = new Map();
  const anchorsOf = (p) => {
    if (!cache.has(p)) cache.set(p, anchors(parseMd(readFileSync(p, 'utf8')).headings));
    return cache.get(p);
  };
  let total = 0;
  let bad = 0;
  for (const f of files) {
    const md = parseMd(readFileSync(f, 'utf8'));
    for (const l of links(md.prose)) {
      if (/^[a-z][a-z0-9+.-]*:/i.test(l.target)) continue;
      total++;
      const [target, anchor] = l.target.split('#');
      const tp = target ? resolve(dirname(f), decodeURIComponent(target)) : f;
      const rel = posix(relative(root, f));
      if (!existsSync(tp)) { warn(`${rel}:${l.line}: link quebrado ${l.target}`); bad++; continue; }
      if (anchor && tp.endsWith('.md') && !anchorsOf(tp).has(decodeURIComponent(anchor))) { warn(`${rel}:${l.line}: âncora inexistente ${l.target}`); bad++; }
    }
  }
  return { total, bad };
}

const ID_PATTERNS = [
  ['T', /\bT-[A-Z]+-\d+\b/g],
  ['TF', /\bTF-[A-Z]+-\d+\b/g],
  ['RNF', /\bRNF-\d+\b/g],
  ['RN', /\bRN-\d+\b/g],
  ['R', /\bR-\d+\b/g],
  ['C', /\bC-\d+\b/g],
  ['P', /\bP-\d+\b/g],
  ['L', /\bL-\d+\b/g],
  ['I', /\bI-\d+\b/g],
  ['GC', /\bGC-\d+\b/g],
  ['G', /\bG-\d+\b/g],
];

export function checkIds(root, defs, warn) {
  let cited = 0;
  for (const f of walk(join(root, 'docs'), (p) => p.endsWith('.md'))) {
    const rel = posix(relative(root, f));
    const md = parseMd(readFileSync(f, 'utf8'));
    for (const { line, text } of md.prose) {
      for (const [k, re] of ID_PATTERNS) {
        const set = defs[k];
        if (!set || set.size === 0) continue; // the defining doc does not exist yet
        for (const m of text.matchAll(re)) {
          cited++;
          if (!set.has(m[0])) warn(`${rel}:${line}: ${m[0]} não está definido`);
        }
      }
    }
  }
  return cited;
}

export function readAudit(text, warn) {
  const rows = [];
  if (!text) return rows;
  const seen = new Set();
  text.split(/\r?\n/).forEach((l, i) => {
    const m = l.match(/^\| ([ABDFP]\d+) \|/);
    if (!m) return;
    const cells = l.split('|').slice(1, -1).map((c) => c.trim());
    const state = cells.find((c) => ['OPEN', 'BUILT', 'DEFERRED', 'DOC'].includes(c));
    if (seen.has(m[1])) warn(`build/audit.md:${i + 1}: ${m[1]} duplicado`);
    seen.add(m[1]);
    if (!state) warn(`build/audit.md:${i + 1}: ${m[1]} sem estado (OPEN, BUILT, DEFERRED, DOC)`);
    rows.push({ id: m[1], state, proof: cells[cells.length - 1], line: i + 1 });
  });
  return rows;
}

export function auditProblems(rows, items, warn) {
  const done = new Set(items.filter((x) => (x.kind === 'test' || x.kind === 'ui-test') && x.state === 'done').map((x) => x.name));
  for (const r of rows) {
    if (r.state === 'BUILT') {
      const ids = [...r.proof.matchAll(/\bTF?-[A-Z]+-\d+\b/g)].map((m) => m[0]);
      if (!ids.length) warn(`build/audit.md:${r.line}: ${r.id} BUILT sem T-ID na prova`);
      for (const id of ids) if (!done.has(id)) warn(`build/audit.md:${r.line}: ${r.id} cita ${id}, que não está feito no catálogo`);
    }
    if ((r.state === 'DEFERRED' || r.state === 'DOC') && !r.proof) warn(`build/audit.md:${r.line}: ${r.id} ${r.state} sem prova`);
  }
}

// ---------------------------------------------------------------------------
// Ledger
// ---------------------------------------------------------------------------

const STATE_ORDER = { pending: 0, deferred: 1, done: 2 };
const esc = (s) => String(s ?? '').replace(/\|/g, '\\|');

export function render(items, warnings, audit, extra) {
  const L = [];
  L.push('# Ledger');
  L.push('');
  L.push('Gerado por `node tools/doccheck/doccheck.mjs`. Não edite à mão: a próxima execução reescreve.');
  L.push('');
  L.push('| Tipo | Total | Feito | Adiado | Pendente |');
  L.push('|---|---|---|---|---|');
  for (const k of KINDS) {
    const xs = items.filter((x) => x.kind === k);
    if (!xs.length) continue;
    const c = (s) => xs.filter((x) => x.state === s).length;
    L.push(`| ${k} | ${xs.length} | ${c('done')} | ${c('deferred')} | ${c('pending')} |`);
  }
  L.push('');
  for (const e of extra) L.push(`- ${e}`);
  L.push('');
  for (const w of warnings) L.push(`> **Aviso:** ${w}`);
  if (warnings.length) L.push('');
  for (const k of KINDS) {
    const xs = items.filter((x) => x.kind === k).sort((a, b) => STATE_ORDER[a.state] - STATE_ORDER[b.state] || a.name.localeCompare(b.name));
    if (!xs.length) continue;
    L.push(`## ${k}`);
    L.push('');
    L.push('| Item | Fonte | Estado | Onde / motivo |');
    L.push('|---|---|---|---|');
    for (const x of xs) L.push(`| \`${esc(x.name)}\` | ${esc(x.source)} | ${x.state} | ${esc(x.where)} |`);
    L.push('');
  }
  L.push('## Auditoria da prosa');
  L.push('');
  const n = (s) => audit.filter((r) => r.state === s && !r.id.startsWith('P')).length;
  L.push('| Aberto | Feito | Adiado | Doc |');
  L.push('|---|---|---|---|');
  L.push(`| ${n('OPEN')} | ${n('BUILT')} | ${n('DEFERRED')} | ${n('DOC')} |`);
  L.push('');
  return L.join('\n');
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

export function run(root, { out = 'build/ledger.md', write = true } = {}) {
  const warnings = [];
  const warn = (w) => warnings.push(w);
  const inv = inventory(root, warn);
  const code = scanCode(root);
  const deferred = readDeferred(read(root, 'tools/doccheck/deferred.txt'), warn);
  const items = resolveAll(inv, code, deferred, warn);
  const linkStats = checkLinks(root, warn);
  const cited = checkIds(root, inv.defs, warn);
  const audit = readAudit(read(root, DOCS.audit), warn);
  auditProblems(audit, items, warn);
  const catalogued = new Set(items.filter((x) => x.kind === 'test').flatMap((x) => x.tests));
  const extra = [
    `testes no código: ${code.tests.length}; no catálogo (13): ${catalogued.size}; fora dele: ${code.tests.length - [...new Set(code.tests.map((t) => t.name))].filter((n) => catalogued.has(n)).length}`,
    `links relativos conferidos: ${linkStats.total}`,
    `IDs citados conferidos: ${cited}`,
  ];
  const ledger = render(items, warnings, audit, extra);
  if (write) {
    mkdirSync(dirname(join(root, out)), { recursive: true });
    writeFileSync(join(root, out), ledger);
  }
  return { items, warnings, audit, code, inv, extra };
}

function main(argv) {
  const args = new Set(argv.filter((a) => a.startsWith('-')));
  const rootArg = argv.find((a, i) => argv[i - 1] === '-root');
  const here = dirname(fileURLToPath(import.meta.url));
  const root = rootArg ? resolve(rootArg) : resolve(here, '../..');
  const { items, warnings, audit, extra } = run(root);
  const pad = (s, n) => String(s).padEnd(n);
  let pending = 0;
  for (const k of KINDS) {
    const xs = items.filter((x) => x.kind === k);
    if (!xs.length) continue;
    const p = xs.filter((x) => x.state === 'pending').length;
    pending += p;
    console.log(`${pad(k, 9)} ${String(xs.length).padStart(5)} total, ${xs.filter((x) => x.state === 'done').length} feitos, ${xs.filter((x) => x.state === 'deferred').length} adiados, ${p} pendentes`);
  }
  for (const e of extra) console.log(e);
  const open = audit.filter((r) => r.state === 'OPEN' && !r.id.startsWith('P')).length;
  console.log(`auditoria ${audit.length} linhas, ${open} abertas`);
  console.log(`avisos    ${warnings.length}`);
  if (args.has('-v')) for (const w of warnings) console.log(`  ${w}`);
  if (args.has('-strict') && (pending > 0 || warnings.length > 0)) process.exit(1);
  if (args.has('-closed') && open > 0) process.exit(1);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2));
