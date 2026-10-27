// Servidor mínimo que registra cada requisição.
//
// Existe só para provar que o atualizador do app realmente sai perguntando no
// endereço configurado. O Vite serve os mesmos arquivos, mas não registra
// acesso a arquivo estático — e sem registro não há prova, só suposição.
//
// Uso: node scripts/servidor-teste.mjs

import { createServer } from "node:http";
import { readFileSync, statSync, createReadStream } from "node:fs";
import { resolve } from "node:path";

const raiz = process.cwd();
const PORTA = 1420;

const tipos = {
  ".json": "application/json",
  ".exe": "application/octet-stream",
};

createServer((req, res) => {
  const caminho = decodeURIComponent(req.url.split("?")[0]).replace(/^\/+/, "");
  const arquivo = resolve(raiz, caminho);
  const hora = new Date().toTimeString().slice(0, 8);

  // Não serve nada fora da raiz, mesmo sendo um servidor de teste.
  if (!arquivo.startsWith(raiz)) {
    console.log(`[${hora}] ${req.method} ${req.url} -> 403 (fora da raiz)`);
    res.writeHead(403).end();
    return;
  }

  let info;
  try {
    info = statSync(arquivo);
  } catch {
    console.log(`[${hora}] ${req.method} ${req.url} -> 404`);
    res.writeHead(404).end();
    return;
  }

  const ext = arquivo.slice(arquivo.lastIndexOf("."));
  console.log(
    `[${hora}] ${req.method} ${req.url} -> 200 (${info.size} bytes)` +
      `  agente: ${req.headers["user-agent"] ?? "-"}`
  );

  res.writeHead(200, {
    "Content-Type": tipos[ext] ?? "application/octet-stream",
    "Content-Length": info.size,
  });

  if (req.method === "HEAD") {
    res.end();
    return;
  }
  createReadStream(arquivo).pipe(res);
}).listen(PORTA, () => {
  console.log(`servidor de teste em http://localhost:${PORTA} (raiz: ${raiz})`);
  console.log("aguardando o app perguntar por /latest.json ...");
});
