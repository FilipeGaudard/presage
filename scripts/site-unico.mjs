// Gera o site inteiro num arquivo só.
//
// A pasta `site/` tem quatro arquivos, o que é certo para hospedar. Mas quem
// manda a página para um amigo manda um arquivo, e aí o HTML chega sem o CSS
// e sem o JS — texto preto no branco. Este script costura tudo num
// `Presage-site.html` que abre com dois cliques em qualquer lugar.

import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const root = process.cwd();
const dir = resolve(root, "site");
const out = resolve(root, "Presage-site.html");

const css = readFileSync(resolve(dir, "styles.css"), "utf8");
// Um `</script>` dentro de uma string do JS fecharia a tag antes da hora.
const js = readFileSync(resolve(dir, "app.js"), "utf8").replace(/<\/script/gi, "<\\/script");
const favicon = readFileSync(resolve(dir, "favicon.svg"), "utf8");
const faviconUri = `data:image/svg+xml;base64,${Buffer.from(favicon).toString("base64")}`;

const feito = { css: false, js: false };
let html = readFileSync(resolve(dir, "index.html"), "utf8");

// Sempre função no lugar de string: `$&` e companhia são expandidos num
// replacement literal, e CSS e JS estão cheios de cifrão.
html = html.replace(/<link rel="stylesheet"[^>]*>/, () => {
  feito.css = true;
  return `<style>\n${css}\n</style>`;
});

html = html.replace(/<script[^>]*src="app\.js"[^>]*><\/script>/, () => {
  feito.js = true;
  return `<script>\n${js}\n</script>`;
});

html = html.replace(/href="\.?\/?favicon\.svg"/, () => `href="${faviconUri}"`);

// Hospedada, a interface se chama `demo.html` ao lado do index. Solta, ela é
// o `Presage-demo.html` que vai na mesma mensagem.
html = html.replace(/href="demo\.html"/, () => 'href="Presage-demo.html"');

// O botão de baixar aponta para o instalador ao lado da página. Hospedado,
// isso funciona. Num arquivo solto que chegou por mensagem, só funciona se o
// .exe estiver na mesma pasta — então a letra miúda avisa, em vez de deixar um
// botão que não faz nada sem explicar por quê.
html = html.replace(
  /<p class="fine">([^<]*)<\/p>/,
  (_todo, miudo) =>
    `<p class="fine">${miudo}<br />` +
    "O botão procura o instalador na mesma pasta desta página.</p>"
);

if (!feito.css || !feito.js) {
  console.error(`não consegui embutir: css=${feito.css}, js=${feito.js}`);
  process.exit(1);
}

writeFileSync(out, html, "utf8");
console.log(`Presage-site.html gerado (${Math.round(html.length / 1024)} KB)`);
