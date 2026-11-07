// node --test tools/doccheck
// The scanners on fixtures, the whole check on a tiny repository built in a
// temp dir, and minimum counts on the real one, so a parser change that
// silently loses items fails here instead of passing the gate.

import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { strip, scanRust, modulePath, wireName } from './rust.mjs';
import { scanTs } from './ts.mjs';
import { splitRow, slug, anchors, parseMd } from './md.mjs';
import { run } from './doccheck.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');

test('strip: comentários, strings cruas, char e lifetime', () => {
  const src = [
    '/* a /* b */ { */ fn a<\'x>(s: &\'x str) -> char { \'{\' }',
    'const S: &str = r#"tem "aspas" e { chave"#;',
    'let c = \'\\\'\'; // }',
    'let e = "é \\" {";',
  ].join('\n');
  const { code, strings, comments } = strip(src);
  assert.equal(code.split('\n').length, 4, 'as linhas ficam no lugar');
  assert.ok(!code.includes('chave'));
  assert.equal((code.match(/[{}]/g) || []).length, 2, 'só as chaves de verdade');
  assert.deepEqual(strings.map((s) => s.value), ['tem "aspas" e { chave', 'é " {']);
  assert.equal(comments.length, 2);
  assert.ok(code.includes("<'x>"), 'lifetime não vira char');
});

test('scanRust: itens, métodos, campos serde, testes, comandos', () => {
  const src = `
//! módulo
use std::fmt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scored {
    pub champion_id: i64,
    #[serde(rename = "trait")]
    pub kind: Kind,
    #[serde(skip)]
    pub cache: Vec<u8>,
    pub map: HashMap<String, Vec<(u8, u8)>>,
}

#[serde(rename_all = "lowercase")]
pub enum Tier { Core, Situational, Extra { x: u8 }, Tupla(u8) }

pub struct Tupla(pub u8);
pub struct Unidade;
pub const LIMITE: [u8; 2] = [1, 2];
pub type Mapa = HashMap<u8, u8>;
pub trait Store { fn get(&self, k: u8) -> u8; }

impl Scored {
    pub fn new(x: [u8; 4]) -> Self { let y = Self { champion_id: 0 }; y }
    fn privado(&self) {}
}
impl<'a> Store for Leitura<'a> { fn get(&self, k: u8) -> u8 { k } }

pub(crate) fn interna() {}
pub fn livre() -> u8 { if true { 1 } else { 2 } }

#[tauri::command]
async fn get_snapshot() -> u8 { 0 }

fn main() { tauri::Builder::default().invoke_handler(tauri::generate_handler![get_snapshot, crate::outro]); }

#[cfg(test)]
mod tests {
    pub fn ajudante() {}
    #[test]
    fn popularidade_nao_vira_matchup() {}
}
`;
  const r = scanRust(src, { file: 'f.rs', module: 'core::engine' });
  const names = r.items.map((i) => `${i.kind}:${i.name}:${i.isPub ? 'pub' : '-'}`);
  for (const n of ['struct:Scored:pub', 'enum:Tier:pub', 'struct:Tupla:pub', 'struct:Unidade:pub', 'const:LIMITE:pub', 'type:Mapa:pub', 'trait:Store:pub', 'method:Store::get:pub', 'method:Scored::new:pub', 'method:Scored::privado:-', 'method:Leitura::get:pub', 'fn:interna:-', 'fn:livre:pub', 'fn:get_snapshot:-', 'fn:main:-']) {
    assert.ok(names.includes(n), `falta ${n} em ${names.join(' ')}`);
  }
  assert.ok(!names.some((n) => n.includes('ajudante')), 'nada de cfg(test) na API');
  const scored = r.items.find((i) => i.name === 'Scored');
  assert.deepEqual(scored.fields.map((f) => f.name), ['champion_id', 'kind', 'cache', 'map']);
  assert.deepEqual(scored.fields.map((f) => f.wire), ['championId', 'trait', null, 'map']);
  const tier = r.items.find((i) => i.name === 'Tier');
  assert.deepEqual(tier.variants.map((v) => v.wire), ['core', 'situational', 'extra', 'tupla']);
  assert.deepEqual(tier.variants.map((v) => v.unit), [true, true, false, false]);
  assert.equal(r.items.find((i) => i.name === 'Tupla').fields, null);
  assert.equal(r.items.find((i) => i.name === 'Leitura::get').trait, 'Store');
  assert.deepEqual(r.tests.map((t) => t.name), ['popularidade_nao_vira_matchup']);
  assert.deepEqual(r.commands.map((c) => c.name), ['get_snapshot']);
  assert.deepEqual(r.handlers, ['get_snapshot', 'outro']);
});

test('modulePath e wireName', () => {
  assert.equal(modulePath('core', 'lib.rs'), 'core');
  assert.equal(modulePath('core', 'engine/mod.rs'), 'core::engine');
  assert.equal(modulePath('core', 'engine/minimax.rs'), 'core::engine::minimax');
  assert.equal(modulePath('casca', 'lcu/client.rs'), 'casca::lcu::client');
  assert.equal(wireName('in_pool', 'camelCase'), 'inPool');
  assert.equal(wireName('NextItem', 'camelCase', true), 'nextItem');
  assert.equal(wireName('Situational', 'lowercase', true), 'situational');
});

test('scanTs: interface, extends, união, invoke', () => {
  const src = `
export interface Base { id: number }
export interface ScoredChampion extends Base {
  championId: number;
  reasons: string[];
  onPick?: (id: number) => void;
  nested: { a: number; b: string };
  "quoted": boolean,
}
export type Tier = "core" | "situational" | "extra";
export type Obj = { x: number };
const a = await invoke<[A, B[]] | null>("build_suggestions", { limit: 8 });
invoke("set_pool", { pool });
const s = 'presage://live';
`;
  const r = scanTs(src);
  const sc = r.interfaces.find((i) => i.name === 'ScoredChampion');
  assert.deepEqual(sc.fields, ['championId', 'reasons', 'onPick', 'nested', 'quoted']);
  assert.deepEqual(sc.extends, ['Base']);
  assert.deepEqual(r.unions.find((u) => u.name === 'Tier').values, ['core', 'situational', 'extra']);
  assert.deepEqual(r.interfaces.find((i) => i.name === 'Obj').fields, ['x']);
  assert.deepEqual(r.invokes.map((i) => i.name), ['build_suggestions', 'set_pool']);
  assert.ok(r.strings.some((s) => s.value === 'presage://live'));
});

test('md: célula com pipe em código, slug com acento e repetido', () => {
  assert.deepEqual(splitRow('| `a|b` | x \\| y | z |'), ['`a|b`', 'x | y', 'z']);
  assert.equal(slug('2. De onde vem cada número'), '2-de-onde-vem-cada-número');
  assert.equal(slug('`rank()` — o ranking'), 'rank--o-ranking');
  const a = anchors(parseMd('## Uso\n## Uso\n```\n## dentro\n```\n').headings);
  assert.deepEqual([...a], ['uso', 'uso-1']);
});

function fixture() {
  const dir = mkdtempSync(join(tmpdir(), 'doccheck-'));
  const put = (rel, text) => {
    mkdirSync(dirname(join(dir, rel)), { recursive: true });
    writeFileSync(join(dir, rel), text);
  };
  put('core/src/lib.rs', 'pub mod engine;\n');
  put('core/src/engine/mod.rs', [
    '#[serde(rename_all = "camelCase")]',
    'pub struct Scored { pub champion_id: i64, pub in_pool: bool }',
    'pub fn rank(x: u8) -> Vec<Scored> { vec![] }',
    'pub fn sem_doc() {}',
    '#[cfg(test)] mod tests { #[test] fn popularidade_nao_vira_matchup() {} }',
  ].join('\n'));
  put('src-tauri/src/lib.rs', [
    'const EVENT_DRAFT: &str = "presage://draft";',
    'const POOL: &str = "pool.json";',
    '#[tauri::command] fn rank_draft() {}',
    '#[tauri::command] fn get_enemy_roles() {}',
    'fn run() { generate_handler![rank_draft, get_enemy_roles]; }',
    '// TODO(fase 1): websocket',
  ].join('\n'));
  put('tools/ingest/src/main.rs', 'fn main() { match a { "--regiao" => {} _ => {} } }\n');
  put('src/App.tsx', 'invoke("rank_draft"); listen("presage://draft");\nexport interface ScoredChampion { championId: number; inPool: boolean }\n');
  put('docs/arquitetura/10-modelo-de-dados.md', [
    '# 10', '', '## 1. `core::engine`', '', '### 1.1 `Scored` ↔ `ScoredChampion`', '', '```rust',
    '#[serde(rename_all = "camelCase")]', 'pub struct Scored {', '    pub champion_id: i64, // id', '    pub in_pool: bool,', '}', '```', '',
  ].join('\n'));
  put('docs/arquitetura/11-catalogo-de-funcoes.md', [
    '# 11', '', '## 1. `core::engine`', '', '```rust', 'pub fn rank(x: u8) -> Vec<Scored>;', 'pub fn inexistente();', '```', '',
    'Ver [o modelo](10-modelo-de-dados.md#11-scored--scoredchampion) e [nada](nada.md).',
  ].join('\n'));
  put('docs/arquitetura/12-contrato-da-casca.md', [
    '# 12', '', '## 1. Comandos', '', '| Comando | Devolve |', '|---|---|', '| `rank_draft` | x |', '| `get_enemy_roles` | x |', '',
    '## 2. Eventos', '', '| Evento | Quando |', '|---|---|', '| `presage://draft` | x |', '',
    '## 3. Disco', '', '| Arquivo | O quê |', '|---|---|', '| `pool.json` | a pool |', '',
  ].join('\n'));
  put('docs/arquitetura/05-coletor.md', '# 05\n\n| Opção | Faz |\n|---|---|\n| `--regiao` | x |\n');
  put('docs/arquitetura/13-testes.md', '# 13\n\n## 2. Catálogo\n\n| ID | Teste | Tipo | Prova | Atende |\n|---|---|---|---|---|\n| T-DRAFT-1 | `popularidade_nao_vira_matchup` | unidade | x | R-1 |\n| T-DRAFT-2 | `nao_existe` | unidade | x | R-1 |\n');
  put('docs/produto/requisitos.md', '# R\n\n### R-1 — Ranking\n\n| ID | Regra |\n|---|---|\n| C-1 | x |\n\n## 4. Matriz de rastreabilidade\n\n| Requisito | Funções | Testes |\n|---|---|---|\n| R-1 | `rank` | T-DRAFT-1 |\n| C-1 | — | T-DRAFT-9 |\n\nCita R-7 por engano.\n');
  put('tools/doccheck/deferred.txt', 'command get_enemy_roles | A2: registrado e nunca chamado\ntodo src-tauri/src/lib.rs TODO(fase 1) | P-9\nfunc core::nada | sobra\n');
  return dir;
}

test('a checagem inteira sobre um repositório mínimo', () => {
  const dir = fixture();
  try {
    const { items, warnings } = run(dir, { write: false });
    const st = (kind, name) => items.find((x) => x.kind === kind && x.name === name)?.state;
    assert.equal(st('type', 'core::engine::Scored'), 'done');
    assert.equal(st('func', 'core::engine::rank'), 'done');
    assert.equal(st('func', 'core::engine::inexistente'), 'pending');
    assert.equal(st('wire', 'Scored ↔ ScoredChampion'), 'done');
    assert.equal(st('command', 'rank_draft'), 'done');
    assert.equal(st('command', 'get_enemy_roles'), 'deferred', 'nunca chamado, mas adiado com motivo');
    assert.equal(st('event', 'presage://draft'), 'done');
    assert.equal(st('file', 'pool.json'), 'done');
    assert.equal(st('flag', '--regiao'), 'done');
    assert.equal(st('test', 'T-DRAFT-1'), 'done');
    assert.equal(st('test', 'T-DRAFT-2'), 'pending');
    assert.equal(st('trace', 'R-1'), 'done');
    assert.equal(st('trace', 'C-1'), 'pending', 'T-ID fora do catálogo');
    assert.equal(st('undoc', 'core::engine::sem_doc'), 'pending');
    assert.equal(st('todo', 'src-tauri/src/lib.rs TODO(fase 1)'), 'deferred');
    assert.ok(warnings.some((w) => w.includes('func core::nada')), 'adiamento que não casa vira aviso');
    assert.ok(warnings.some((w) => w.includes('link quebrado nada.md')));
    assert.ok(!warnings.some((w) => w.includes('#11-scored')), 'a âncora com ↔ resolve');
    assert.ok(warnings.some((w) => w.includes('R-7 não está definido')));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('um campo a mais no TS quebra o wire', () => {
  const dir = fixture();
  try {
    writeFileSync(join(dir, 'src/App.tsx'), 'invoke("rank_draft"); listen("presage://draft");\nexport interface ScoredChampion { championId: number; inPool: boolean; extra: number }\n');
    const { items } = run(dir, { write: false });
    const w = items.find((x) => x.kind === 'wire');
    assert.equal(w.state, 'pending');
    assert.match(w.where, /Rust ≠ TS: só no segundo: extra/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('o repositório real: o scanner não perde itens', () => {
  const { code } = run(ROOT, { write: false });
  assert.ok(code.tests.length >= 460, `testes: ${code.tests.length}`);
  assert.equal(new Set(code.commands.map((c) => c.name)).size, 26);
  assert.deepEqual([...new Set(code.handlers)].sort(), [...new Set(code.commands.map((c) => c.name))].sort());
  const corePub = code.items.filter((x) => x.crate === 'core' && x.isPub);
  assert.ok(corePub.filter((x) => ['struct', 'enum', 'trait', 'type'].includes(x.kind)).length >= 100);
  assert.ok(corePub.filter((x) => x.kind === 'fn').length >= 70);
  assert.ok(code.ts.interfaces.length >= 60);
  assert.ok(new Set(code.ts.invokes.map((x) => x.name)).size >= 25);
});
