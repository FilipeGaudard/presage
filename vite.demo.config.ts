import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Ver a nota em vite.config.ts: o BOM do PowerShell quebra o JSON.parse.
const versao = JSON.parse(
  readFileSync(resolve(process.cwd(), "package.json"), "utf8").replace(/^﻿/, "")
).version as string;

// Build dedicada da página de demonstração.
//
// A build normal divide o código em vários módulos, e o Chrome recusa carregar
// módulo externo a partir de `file://` — por isso a página abria em branco ao
// dar dois cliques. Com uma entrada só e `inlineDynamicImports`, tudo cai num
// arquivo; `scripts/standalone.mjs` depois costura esse arquivo dentro do HTML.
export default defineConfig({
  plugins: [react()],
  base: "./",
  define: {
    __APP_VERSION__: JSON.stringify(versao),
  },
  build: {
    target: "chrome105",
    outDir: "dist-demo",
    emptyOutDir: true,
    cssCodeSplit: false,
    sourcemap: false,
    rollupOptions: {
      input: resolve(process.cwd(), "demo.html"),
      output: {
        inlineDynamicImports: true,
        entryFileNames: "demo.js",
        assetFileNames: "demo[extname]",
      },
    },
  },
});
