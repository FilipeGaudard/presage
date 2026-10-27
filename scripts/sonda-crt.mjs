// Lista os pacotes da CRT do MSVC disponíveis no manifesto, para achar o que
// traz as libs que faltam (OLDNAMES, legacy_stdio_definitions e companhia).

import { readFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { homedir } from "node:os";

const cache = resolve(homedir(), "tools", "msvc-cache", "vs-manifest.json");
const pacotes = JSON.parse(readFileSync(cache, "utf8"));

const versao = JSON.parse(
  readFileSync(resolve(homedir(), "tools", "msvc", "versoes.json"), "utf8")
).msvc;

const alvo = pacotes
  .filter((p) => new RegExp(`^microsoft\\.vc\\.${versao.replace(/\./g, "\\.")}\\.`, "i").test(p.id))
  .filter((p) => !p.language || /^(neutral|en-us)$/i.test(p.language))
  .map((p) => ({
    id: p.id,
    mb: ((p.payloads ?? []).reduce((s, x) => s + (x.size ?? 0), 0) / 1024 / 1024).toFixed(1),
  }))
  .sort((a, b) => a.id.localeCompare(b.id));

console.log(`MSVC ${versao} — ${alvo.length} pacotes:\n`);
for (const p of alvo) console.log(`  ${p.mb.padStart(7)} MB  ${p.id}`);
