// Instala o Git no perfil do usuário, sem administrador.
//
// Mesma ideia do Node e do MSVC portáteis: o instalador oficial do Git pede
// elevação, e a TI não libera. O projeto `git-for-windows` publica o **MinGit**
// — um zip com o `git.exe` e o mínimo para ele funcionar, feito exatamente para
// ser embutido em outro programa.
//
// Zip em vez do instalador autoextraível: não executa nada baixado, só
// descompacta. Num ambiente onde o antivírus já reagiu a binário novo, isso
// não é preciosismo.
//
// Uso:  node scripts/git-portatil.mjs

import { mkdirSync, writeFileSync, existsSync, readdirSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { resolve, join } from "node:path";
import { homedir } from "node:os";

const DESTINO = resolve(homedir(), "tools", "git");
const CACHE = resolve(homedir(), "tools", "git-cache");

const gitExe = join(DESTINO, "cmd", "git.exe");
if (existsSync(gitExe)) {
  console.log(`git já está em ${gitExe}`);
  process.exit(0);
}

console.log("procurando a versão mais recente do MinGit...");

const release = await fetch(
  "https://api.github.com/repos/git-for-windows/git/releases/latest",
  { headers: { "User-Agent": "presage-setup", Accept: "application/vnd.github+json" } }
).then((r) => {
  if (!r.ok) throw new Error(`a API do GitHub respondeu ${r.status}`);
  return r.json();
});

// `MinGit-2.47.1-64-bit.zip`. O `busybox` é uma variante menor que não traz
// tudo que precisamos, então fica de fora.
const asset = (release.assets ?? []).find(
  (a) =>
    /^MinGit-.*-64-bit\.zip$/i.test(a.name) && !/busybox/i.test(a.name)
);

if (!asset) {
  console.error(
    `não achei o MinGit em ${release.tag_name}. Assets disponíveis:\n  ` +
      (release.assets ?? []).map((a) => a.name).join("\n  ")
  );
  process.exit(1);
}

console.log(`baixando ${asset.name} (${Math.round(asset.size / 1024 / 1024)} MB)...`);

mkdirSync(CACHE, { recursive: true });
const zip = join(CACHE, asset.name);

if (!existsSync(zip)) {
  const resposta = await fetch(asset.browser_download_url);
  if (!resposta.ok) throw new Error(`download respondeu ${resposta.status}`);
  writeFileSync(zip, Buffer.from(await resposta.arrayBuffer()));
}

console.log("extraindo...");
mkdirSync(DESTINO, { recursive: true });
// O tar do Windows 10+ abre zip, então não precisamos de dependência.
execFileSync("tar", ["-xf", zip, "-C", DESTINO], { stdio: "ignore" });

if (!existsSync(gitExe)) {
  console.error(
    `extraiu, mas não achei ${gitExe}. Conteúdo:\n  ` +
      readdirSync(DESTINO).join("\n  ")
  );
  process.exit(1);
}

const versao = execFileSync(gitExe, ["--version"], { encoding: "utf8" }).trim();
console.log(`\n${versao}`);
console.log(`instalado em ${DESTINO}`);
console.log(`\nPara usar nesta sessão do PowerShell:`);
console.log(`  $env:Path = "${join(DESTINO, "cmd")};$env:Path"`);
