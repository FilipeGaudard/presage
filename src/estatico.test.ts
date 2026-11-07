import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

// Testes estáticos do front (frontend 06 §1): leem o código, não o montam. São
// as regras que nenhuma tela mostra, mas que uma mudança distraída quebra.

const RAIZ = join(__dirname, "..");
const SRC = join(RAIZ, "src");

function arquivos(dir: string): string[] {
  return readdirSync(dir).flatMap((f) => {
    const p = join(dir, f);
    return statSync(p).isDirectory() ? arquivos(p) : /\.(ts|tsx)$/.test(f) && !/\.test\./.test(f) ? [p] : [];
  });
}

const fontes = arquivos(SRC).map((p) => ({ nome: relative(SRC, p), texto: readFileSync(p, "utf8") }));

describe("o código do front", () => {
  it("TF-CASCA-5: só App.tsx e overlay.tsx falam com a casca, e nada injeta HTML cru", () => {
    const comApi = fontes.filter((f) => /from "@tauri-apps\/api/.test(f.texto)).map((f) => f.nome).sort();
    expect(comApi).toEqual(["App.tsx", "overlay.tsx"]);
    // O plugin do atualizador é a exceção conhecida (frontend 01).
    const comPlugin = fontes.filter((f) => /from "@tauri-apps\/plugin-/.test(f.texto)).map((f) => f.nome);
    expect(comPlugin).toEqual([join("components", "Atualizacao.tsx")]);
    for (const f of fontes) {
      for (const proibido of ["dangerouslySetInnerHTML", "innerHTML", "eval(", "localStorage", "<a href"]) {
        expect(f.texto.includes(proibido), `${f.nome} usa ${proibido}`).toBe(false);
      }
    }
  });

  it("TF-OVERLAY-6: nenhum anúncio, no overlay nem em lugar nenhum (C-9)", () => {
    const pacote = JSON.parse(readFileSync(join(RAIZ, "package.json"), "utf8"));
    const deps = Object.keys({ ...pacote.dependencies, ...pacote.devDependencies });
    for (const d of deps) {
      expect(/\b(ads?|adsense|admob|adnxs|overwolf|ad-?sdk|banner-?ads?)\b/i.test(d), `dependência de anúncio: ${d}`).toBe(false);
    }
    const paginas = ["index.html", "overlay.html", "demo.html"].map((f) => readFileSync(join(RAIZ, f), "utf8"));
    for (const texto of [...paginas, ...fontes.map((f) => f.texto)]) {
      expect(texto).not.toMatch(/<iframe|adsbygoogle|googletagservices|doubleclick|<script[^>]+src="https?:/i);
    }
  });

  it("TF-CASCA-7: a cor da marca nunca carrega significado de dado (RNF-12)", () => {
    const css = readFileSync(join(SRC, "styles.css"), "utf8") + readFileSync(join(SRC, "overlay.css"), "utf8");
    // Cada regra cujo seletor é um caminho de cor para número (good, bad, even,
    // warn) não pode usar o violeta da marca.
    const regras = [...css.matchAll(/([^{}]+)\{([^}]*)\}/g)];
    const dePorDado = regras.filter(([, seletor]) => /\.(good|bad|even|warn-line|win|loss)\b/.test(seletor));
    expect(dePorDado.length).toBeGreaterThan(5);
    for (const [, seletor, corpo] of dePorDado) {
      expect(/--halo|#7b6cff|123,\s*108,\s*255/i.test(corpo), `${seletor.trim()} usa a cor da marca`).toBe(false);
    }
  });
});
