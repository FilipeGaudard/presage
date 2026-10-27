// Gera o `latest.json` que o app consulta para saber se há versão nova.
//
// É o contrato do atualizador: o app baixa este arquivo, compara a versão com
// a dele, e se for maior baixa a URL indicada. A assinatura vem do `.sig` que
// o `tauri build` produz quando `createUpdaterArtifacts` está ligado — sem ela
// o app recusa o download, que é o ponto: ninguém consegue empurrar um
// executável para a base instalada sem a chave privada.
//
// Uso:
//   node scripts/manifesto.mjs --url https://presage.gg --notas "o que mudou"

import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { resolve, join } from "node:path";

const raiz = process.cwd();

function arg(nome, padrao) {
  const i = process.argv.indexOf(`--${nome}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : padrao;
}

// Onde o `tauri build` deixou o instalador. No GitHub Actions é `target/` na
// raiz; nesta máquina é a pasta curta que o linker exige. Fixar o caminho
// fazia o script só funcionar num dos dois.
const BUNDLE = arg(
  "bundle",
  join(
    process.env.CARGO_TARGET_DIR ?? resolve(raiz, "target"),
    "release",
    "bundle",
    "nsis"
  )
);

const pacote = JSON.parse(readFileSync(resolve(raiz, "package.json"), "utf8"));
const versao = pacote.version;

// A URL base é de onde o app vai baixar. Sem ela o manifesto fica apontando
// para lugar nenhum, então é melhor falhar aqui que publicar quebrado.
const base = arg("url", "").replace(/\/+$/, "");
if (!base) {
  console.error(
    "falta --url. Exemplo:\n" +
      '  node scripts/manifesto.mjs --url https://presage.gg --notas "..."'
  );
  process.exit(1);
}

let arquivos;
try {
  arquivos = readdirSync(BUNDLE);
} catch {
  console.error(`não achei ${BUNDLE} — rode antes: npm run tauri build`);
  process.exit(1);
}

const instalador = arquivos.find(
  (nome) => nome.startsWith(`Presage_${versao}_`) && nome.endsWith("-setup.exe")
);
if (!instalador) {
  console.error(
    `não achei o instalador da versão ${versao} em ${BUNDLE}.\n` +
      `A versão do package.json tem de bater com a do tauri.conf.json.`
  );
  process.exit(1);
}

const assinatura = `${instalador}.sig`;
if (!arquivos.includes(assinatura)) {
  console.error(
    `o instalador existe mas não está assinado (falta ${assinatura}).\n` +
      "Rode o build com TAURI_SIGNING_PRIVATE_KEY apontando para a chave."
  );
  process.exit(1);
}

const manifesto = {
  version: versao,
  notes: arg("notas", `Presage ${versao}`),
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      // A URL carrega a versão: assim o arquivo é imutável e nenhum cache
      // serve um binário velho para quem está atualizando. No GitHub isso quer
      // dizer apontar para a tag, não para `latest` — `latest` deixa de servir
      // este arquivo no dia em que sair a versão seguinte.
      url: `${base}/${instalador}`,
      signature: readFileSync(join(BUNDLE, assinatura), "utf8").trim(),
    },
  },
};

const saida = resolve(raiz, "latest.json");
writeFileSync(saida, `${JSON.stringify(manifesto, null, 2)}\n`, "utf8");

console.log(`latest.json gerado para a versão ${versao}`);
console.log(`  instalador: ${manifesto.platforms["windows-x86_64"].url}`);
console.log(
  "\nAnexe os dois à release. O app não lê pela URL da tag — ele lê por\n" +
    "`/releases/latest/download/latest.json`, que o GitHub resolve sempre\n" +
    "para a release mais recente. Por isso a release precisa sair publicada:\n" +
    "um rascunho não é servido por esse endereço, e ninguém atualiza."
);
