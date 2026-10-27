// Monta um MSVC + Windows SDK portáteis, sem administrador.
//
// A Microsoft publica um manifesto com todos os pacotes do Visual Studio e as
// URLs de download. Os pacotes são .vsix (que são zips) e .msi. Dá para baixar
// e extrair à mão, sem passar pelo instalador — que é justamente a parte que
// exige elevação.
//
// Uso:
//   node scripts/msvc-portatil.mjs --plano   só lista o que seria baixado
//   node scripts/msvc-portatil.mjs           baixa e extrai

import { mkdirSync, writeFileSync, existsSync, readFileSync, rmSync, renameSync, readdirSync, statSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { resolve, join } from "node:path";
import { homedir } from "node:os";

const CANAL = "https://aka.ms/vs/17/release/channel";
const DESTINO = resolve(homedir(), "tools", "msvc");
const CACHE = resolve(homedir(), "tools", "msvc-cache");
const APENAS_PLANO = process.argv.includes("--plano");

const log = (...a) => console.log(...a);

async function json(url) {
  const r = await fetch(url);
  if (!r.ok) throw new Error(`${url} respondeu ${r.status}`);
  return r.json();
}

mkdirSync(CACHE, { recursive: true });
const manifestoCache = join(CACHE, "vs-manifest.json");

let pacotes;
if (existsSync(manifestoCache)) {
  log("usando o manifesto já baixado...");
  pacotes = JSON.parse(readFileSync(manifestoCache, "utf8"));
} else {
  log("lendo o manifesto do canal...");
  const canal = await json(CANAL);
  const ponteiro = canal.channelItems.find(
    (i) => i.id === "Microsoft.VisualStudio.Manifests.VisualStudio"
  );
  if (!ponteiro) throw new Error("não achei o ponteiro para o manifesto do VS");

  log("lendo o manifesto do VS (é grande, pode demorar)...");
  const vs = await json(ponteiro.payloads[0].url);
  pacotes = vs.packages;
  writeFileSync(manifestoCache, JSON.stringify(pacotes));
}
log(`  ${pacotes.length} pacotes no manifesto`);

if (process.argv.includes("--sonda")) {
  const amostra = pacotes
    .map((p) => p.id)
    .filter((id) => /^microsoft\.vc\..*tools\.hostx64\.targetx64/i.test(id))
    .slice(0, 10);
  log("amostra de ids de ferramentas:");
  for (const id of amostra) log(`  ${id}`);
  process.exit(0);
}

// A versão sai dos próprios pacotes de ferramentas, não de adivinhação sobre
// o id do componente: o formato da versão já mudou de três para quatro partes.
const versoesMsvc = [
  ...new Set(
    pacotes
      .map((p) => p.id)
      .map((id) => /^microsoft\.vc\.([\d.]+)\.tools\.hostx64\.targetx64\.base$/i.exec(id)?.[1])
      .filter(Boolean)
  ),
].sort(compararVersoes);

function compararVersoes(a, b) {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const d = (pa[i] ?? 0) - (pb[i] ?? 0);
    if (d) return d;
  }
  return 0;
}

const versoesSdk = [
  ...new Set(
    pacotes
      .map((p) => p.id.toLowerCase())
      .filter((id) => /^microsoft\.visualstudio\.component\.windows\d+sdk\.\d+$/.test(id))
      .map((id) => id.split(".").pop())
  ),
].sort((a, b) => Number(a) - Number(b));

const msvc = versoesMsvc.at(-1);
const sdk = versoesSdk.at(-1);
log(`MSVC escolhido: ${msvc}`);
log(`Windows SDK escolhido: 10.0.${sdk}.0`);

// O mesmo id aparece várias vezes, um por idioma. Só queremos o neutro.
const achar = (id) =>
  pacotes.filter(
    (p) =>
      p.id.toLowerCase() === id.toLowerCase() &&
      (!p.language || /^(neutral|en-us)$/i.test(p.language))
  );

// O pacote x86 da CRT carrega as libs comuns de link (OLDNAMES,
// legacy_stdio_definitions) que o x64 não traz — dá para notar pela diferença
// de tamanho entre os dois. Sem ele o link do Tauri falha no último passo.
const idsMsvc = [
  `Microsoft.VC.${msvc}.Tools.HostX64.TargetX64.base`,
  `Microsoft.VC.${msvc}.Tools.HostX64.TargetX64.Res.base`,
  `Microsoft.VC.${msvc}.CRT.Headers.base`,
  `Microsoft.VC.${msvc}.CRT.x64.Desktop.base`,
  `Microsoft.VC.${msvc}.CRT.x86.Desktop.base`,
  `Microsoft.VC.${msvc}.CRT.Source.base`,
];

const pacoteSdk = pacotes.find(
  (p) => p.id.toLowerCase() === `win11sdk_10.0.${sdk}` || p.id.toLowerCase() === `win10sdk_10.0.${sdk}`
);

const alvos = [];
for (const id of idsMsvc) {
  const achados = achar(id);
  if (!achados.length) {
    log(`  AVISO: pacote ausente -> ${id}`);
    continue;
  }
  for (const p of achados) for (const pay of p.payloads ?? []) alvos.push({ tipo: "vsix", ...pay });
}

if (!pacoteSdk) throw new Error(`não achei o pacote do SDK 10.0.${sdk}`);
// O SDK vem como .msi + .cab, e o nome do MSI engana: "Desktop Headers x64"
// é a arquitetura alvo, não a do instalador — todos terminam em `-x86_en-us`.
// Pegar o x86 por engano rende um SDK sem Windows.h nem kernel32.lib.
// Os headers do SDK são repartidos entre vários MSIs de um jeito que o nome
// não revela: a variante x64 traz `um`, a x86 traz parte de `shared`, e o
// winapifamily.h — que todo header do Windows inclui — vem de outro pacote
// ainda. Em vez de adivinhar, pegamos todos os de header, lib e CRT. São
// alguns megabytes a mais e uma classe inteira de erro a menos.
// Lista de exclusão em vez de inclusão. Descobrir qual MSI traz qual arquivo
// é tentativa e erro — o rc.exe, por exemplo, não está no pacote chamado
// "Desktop Tools". Sai mais barato pegar tudo menos o que claramente não
// serve para compilar.
const SDK_FORA = /(certification kit|intellisense|extension sdk|verifier|remote|arm64|wdk|eula|redistributab)/i;

const querSdk = (nome) =>
  /\.cab$/i.test(nome) || (/\.msi$/i.test(nome) && !SDK_FORA.test(nome));

for (const pay of pacoteSdk.payloads ?? []) {
  const nome = pay.fileName.split("\\").pop();
  if (querSdk(nome)) alvos.push({ tipo: "sdk", ...pay, fileName: nome });
}

const totalMb = alvos.reduce((s, a) => s + (a.size ?? 0), 0) / 1024 / 1024;
log(`\n${alvos.length} arquivos, ${totalMb.toFixed(0)} MB no total`);

if (APENAS_PLANO) {
  for (const a of alvos.slice(0, 12)) {
    log(`  ${a.tipo.padEnd(4)} ${((a.size ?? 0) / 1024 / 1024).toFixed(1).padStart(7)} MB  ${a.fileName}`);
  }
  if (alvos.length > 12) log(`  ... mais ${alvos.length - 12}`);
  log("\nplano apenas; rode sem --plano para baixar");
  process.exit(0);
}

mkdirSync(CACHE, { recursive: true });
mkdirSync(DESTINO, { recursive: true });

let n = 0;
for (const alvo of alvos) {
  n++;
  const nome = alvo.fileName.split("\\").pop();
  const destino = join(CACHE, nome);
  if (existsSync(destino) && statSync(destino).size === alvo.size) continue;

  process.stdout.write(`[${n}/${alvos.length}] ${nome} ... `);
  const r = await fetch(alvo.url);
  if (!r.ok) throw new Error(`${nome} respondeu ${r.status}`);
  writeFileSync(destino, Buffer.from(await r.arrayBuffer()));
  process.stdout.write("ok\n");
}

// Extrai direto no destino, acumulando. Apagar e recriar a pasta dava EPERM
// sempre que algum processo tinha um arquivo dela aberto, e perder 700 MB de
// extração por causa disso não vale.
log("\nextraindo...");
const bruto = DESTINO;
mkdirSync(bruto, { recursive: true });

for (const alvo of alvos.filter((a) => a.tipo === "vsix")) {
  const nome = alvo.fileName.split("\\").pop();
  // O tar do Windows 10+ abre zip, então não precisamos de dependência.
  execFileSync("tar", ["-xf", join(CACHE, nome), "-C", bruto], { stdio: "ignore" });
}

for (const alvo of alvos.filter((a) => a.tipo === "sdk" && a.fileName.endsWith(".msi"))) {
  // /a é instalação administrativa: extrai os arquivos sem mexer no registro
  // e sem pedir elevação.
  execFileSync("msiexec", ["/a", join(CACHE, alvo.fileName), "/quiet", `TARGETDIR=${bruto}`], {
    stdio: "ignore",
  });
}

log("organizando...");
// Os .vsix despejam tudo sob `Contents`. Precisa ser fusão recursiva: mover
// só o que não existe deixa, por exemplo, Contents\VC\...\bin perdido quando
// VC já foi criado por outro pacote — e aí o link.exe some.
const conteudo = join(bruto, "Contents");
if (existsSync(conteudo)) {
  // robocopy usa 0-7 para sucesso e execFileSync trata qualquer código não
  // zero como exceção, então o status é conferido à mão.
  try {
    execFileSync("robocopy", [conteudo, bruto, "/E", "/MOVE", "/NFL", "/NDL", "/NJH", "/NJS"], {
      stdio: "ignore",
    });
  } catch (err) {
    if ((err.status ?? 16) > 7) throw err;
  }
  rmSync(conteudo, { recursive: true, force: true });
}

const vcRoot = join(DESTINO, "VC", "Tools", "MSVC");
const sdkRoot = join(DESTINO, "Windows Kits", "10");
log(`\nMSVC em: ${vcRoot} (${existsSync(vcRoot)})`);
log(`SDK em:  ${sdkRoot} (${existsSync(sdkRoot)})`);
writeFileSync(join(DESTINO, "versoes.json"), JSON.stringify({ msvc, sdk }, null, 2));
log("\npronto");
