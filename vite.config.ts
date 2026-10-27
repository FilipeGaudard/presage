import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// A versão sai do package.json, não de uma string na interface. Já deixamos a
// barra de cima dizer 0.1.0 depois de subir para 0.2.0, e versão que mente é
// pior que versão nenhuma.
// O `replace` tira um BOM, se houver. O PowerShell 5.1 grava UTF-8 *com* BOM
// quando se pede `-Encoding utf8`, e `JSON.parse` engasga nele — já derrubou
// o build uma vez depois de um bump de versão.
export const versao = JSON.parse(
  readFileSync(resolve(process.cwd(), "package.json"), "utf8").replace(/^﻿/, "")
).version as string;

// Porta fixa: o Tauri aponta devUrl para 1420 em tauri.conf.json.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  define: {
    __APP_VERSION__: JSON.stringify(versao),
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // src-tauri é compilado pelo cargo, o vite não precisa observar.
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: "chrome105",
    sourcemap: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        overlay: resolve(__dirname, "overlay.html"),
        // Página de demonstração: renderiza os mesmos componentes com dados de
        // exemplo, para olhar a interface sem o cliente da Riot aberto.
        demo: resolve(__dirname, "demo.html"),
      },
    },
  },
});
