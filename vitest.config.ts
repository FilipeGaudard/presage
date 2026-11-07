import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Os testes do front (frontend 06). Os componentes são puros — recebem tudo por
// props —, então o teste os monta com dados no formato da casca, sem Tauri.
// O ID da linha do catálogo vai no título do teste: é o marcador que o
// doccheck procura (`TF-ÁREA-n`).
export default defineConfig({
  plugins: [react()],
  define: {
    __APP_VERSION__: JSON.stringify("0.0.0-teste"),
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
  },
});
