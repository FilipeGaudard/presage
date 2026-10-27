// Costura o CSS e o JS da build de demonstração dentro do próprio HTML, para
// o arquivo abrir com dois cliques em qualquer máquina, sem servidor e sem
// nada instalado.

import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { resolve } from "node:path";

const root = process.cwd();
const from = resolve(root, "dist-demo");
const out = resolve(root, "Presage-demo.html");

for (const file of ["demo.html", "demo.js", "demo.css"]) {
  if (!existsSync(resolve(from, file))) {
    console.error(`faltou ${file} em dist-demo — rode a build antes`);
    process.exit(1);
  }
}

const css = readFileSync(resolve(from, "demo.css"), "utf8");

// Uma ocorrência de </script> dentro de uma string do bundle encerraria a tag
// antes da hora e levaria o resto da página junto.
const js = readFileSync(resolve(from, "demo.js"), "utf8").replace(/<\/script/gi, "<\\/script");

const favicon = readFileSync(resolve(root, "public", "favicon.svg"), "utf8");
const faviconUri = `data:image/svg+xml;base64,${Buffer.from(favicon).toString("base64")}`;

// Sempre com função no lugar de string: num replacement literal, `$&`, `$'` e
// `` $` `` são expandidos, e um bundle minificado está cheio de cifrão.
let replaced = { script: false, style: false };
let html = readFileSync(resolve(from, "demo.html"), "utf8");

html = html.replace(/<script type="module"[^>]*src="[^"]*"><\/script>/, () => {
  replaced.script = true;
  return `<script type="module">\n${js}\n</script>`;
});

html = html.replace(/<link rel="stylesheet"[^>]*>/, () => {
  replaced.style = true;
  return `<style>\n${css}\n</style>`;
});

html = html
  .replace(/<link rel="modulepreload"[^>]*>\s*/g, "")
  .replace(/href="\.?\/?favicon\.svg"/, () => `href="${faviconUri}"`);

if (!replaced.script || !replaced.style) {
  console.error(
    `não consegui embutir: script=${replaced.script}, estilo=${replaced.style}. Confira o formato da build.`
  );
  process.exit(1);
}

writeFileSync(out, html, "utf8");
console.log(`Presage-demo.html gerado (${Math.round(html.length / 1024)} KB)`);
