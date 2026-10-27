// Lista os .msi oferecidos pelo pacote do Windows SDK, para escolher
// corretamente quais extrair.

import { readFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { homedir } from "node:os";

const cache = resolve(homedir(), "tools", "msvc-cache", "vs-manifest.json");
const pacotes = JSON.parse(readFileSync(cache, "utf8"));

const sdk = pacotes.find((p) => /^win11sdk_10\.0\.26100$/i.test(p.id));
if (!sdk) {
  console.error("pacote do SDK não encontrado");
  process.exit(1);
}

const msis = (sdk.payloads ?? [])
  .map((p) => p.fileName.split("\\").pop())
  .filter((n) => n.toLowerCase().endsWith(".msi"))
  .sort();

console.log(`${msis.length} MSIs no pacote:\n`);
for (const m of msis) console.log("  " + m);
