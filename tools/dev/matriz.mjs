#!/usr/bin/env node
// tools/dev/matriz.mjs: reescreve a matriz de rastreabilidade dos requisitos
// (docs/produto/requisitos.md §4) a partir do que os catálogos já dizem: a
// coluna Atende do 13 (testes) e do 11 (funções). Um fato num lugar só (L-7):
// quem atende um requisito é o catálogo; a matriz é a visão invertida dele.
// Rodar depois de mexer no Atende de um catálogo; o doccheck confere o resultado.
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseMd, ticks } from '../doccheck/md.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const A = (p) => join(ROOT, 'docs/arquitetura', p);
const rd = (p) => readFileSync(p, 'utf8');

const reqPath = join(ROOT, 'docs/produto/requisitos.md');
const req = rd(reqPath);
const reqMd = parseMd(req);
const names = new Map();
for (const h of reqMd.headings) {
  const m = h.text.match(/^(R-\d+) — (.*)$/);
  if (m) names.set(m[1], m[2]);
}
for (const r of reqMd.rows) {
  const id = r.cells[0]?.match(/^((?:C|RNF)-\d+)$/)?.[1];
  if (!id || /matriz/i.test(r.h2)) continue;
  const bold = r.cells[1]?.match(/\*\*(.+?)\*\*/)?.[1] ?? r.cells[1];
  names.set(id, bold.replace(/\.$/, '').replace(/ —.*$/, '').slice(0, 70));
}
const C_NAMES = {
  'C-1': 'Nada de timer de ultimate inimiga',
  'C-2': 'Nada de leitura de memória do jogo',
  'C-3': 'Nada de injeção nem automação de input',
  'C-4': 'Nada que o jogador não saberia',
  'C-5': 'Destacar decisões, nunca tirá-las',
  'C-6': 'Aumentar a diversidade de decisões',
  'C-7': 'Nada de MMR, nota ou escada paralela',
  'C-8': 'Cobrar só aprovado e com camada grátis',
  'C-9': 'Nada de anúncio dentro do jogo',
  'C-10': 'Produto público só com chave de produção',
};
for (const [k, v] of Object.entries(C_NAMES)) names.set(k, v);
const ids = [...names.keys()].sort((a, b) => {
  const k = (x) => [x.startsWith('R-') ? 0 : x.startsWith('C-') ? 1 : 2, Number(x.split('-')[1])];
  const [a1, a2] = k(a);
  const [b1, b2] = k(b);
  return a1 - b1 || a2 - b2;
});

const tests = new Map(ids.map((i) => [i, []]));
const t13 = parseMd(rd(A('13-testes.md')));
for (const r of t13.rows) {
  const t = r.cells[0]?.match(/^(T-[A-Z]+-\d+)$/)?.[1];
  if (!t) continue;
  const at = r.cells[r.cells.length - 1] ?? '';
  for (const m of at.matchAll(/\b(R-\d+|C-\d+|RNF-\d+)\b/g)) tests.get(m[1])?.push(t);
}

// Os testes do front que existem (o ID num *.test.ts(x) de src/) também provam.
const escritos = new Set();
const varrer = (d) => readdirSync(d).forEach((f) => {
  const p = join(d, f);
  if (statSync(p).isDirectory()) varrer(p);
  else if (/\.(test|spec)\.tsx?$/.test(f)) for (const m of rd(p).matchAll(/\bTF-[A-Z]+-\d+\b/g)) escritos.add(m[0]);
});
varrer(join(ROOT, 'src'));
const t06 = parseMd(rd(join(ROOT, 'docs/frontend/06-testes.md')));
for (const r of t06.rows) {
  const t = r.cells[0]?.match(/^(TF-[A-Z]+-\d+)$/)?.[1];
  if (!t || !escritos.has(t)) continue;
  for (const m of (r.cells[r.cells.length - 1] ?? '').matchAll(/\b(R-\d+|C-\d+|RNF-\d+)\b/g)) tests.get(m[1])?.push(t);
}

const funcs = new Map(ids.map((i) => [i, []]));
const c11 = parseMd(rd(A('11-catalogo-de-funcoes.md')));
const lines11 = rd(A('11-catalogo-de-funcoes.md')).split('\n');
// "### N.k `nome`" seguido de "- **Atende:** …"
let cur = null;
for (const l of lines11) {
  const h = l.match(/^###\s+[\d.]+\s+`([^`]+)`/);
  if (h) { cur = h[1]; continue; }
  if (/^##\s/.test(l)) cur = null;
  const at = l.match(/^\s*-\s+\*\*Atende:\*\*\s*(.*)$/);
  if (at && cur) for (const m of at[1].matchAll(/\b(R-\d+|C-\d+|RNF-\d+)\b/g)) funcs.get(m[1])?.push(cur);
}
for (const r of c11.rows) {
  const ai = r.header.findIndex((x) => /atende/i.test(x));
  if (ai < 0) continue;
  const fn = ticks(r.cells[0] ?? '')[0];
  if (!fn) continue;
  for (const m of (r.cells[ai] ?? '').matchAll(/\b(R-\d+|C-\d+|RNF-\d+)\b/g)) funcs.get(m[1])?.push(fn.replace(/\(.*$/, ''));
}

const uniq = (xs) => [...new Set(xs)];
const order = (t) => t.replace(/\d+$/, (d) => d.padStart(4, '0'));
const rows = ids.map((id) => {
  const f = uniq(funcs.get(id));
  const fcell = f.length ? f.slice(0, 8).map((x) => `\`${x}\``).join(', ') + (f.length > 8 ? ` e mais ${f.length - 8}` : '') : '—';
  const t = uniq(tests.get(id)).sort((a, b) => order(a).localeCompare(order(b)));
  return `| ${id} — ${names.get(id)} | ${fcell} | ${t.join(', ') || '—'} |`;
});
const head = '| Requisito | Funções principais | Testes |\n|---|---|---|\n';
const i = req.indexOf(head);
if (i < 0) throw new Error('cabeçalho da matriz não encontrado');
const after = req.slice(i + head.length);
const end = after.search(/\n(?!\|)/);
const rest = end < 0 ? '' : after.slice(end);
writeFileSync(reqPath, req.slice(0, i) + head + rows.join('\n') + '\n' + rest.replace(/^\n+/, '\n'));
const empty = rows.filter((r) => r.endsWith('| — |')).map((r) => r.split(' ')[1]);
console.log('matriz:', rows.length, 'linhas; sem teste:', empty.join(', ') || 'nenhuma');
