#!/usr/bin/env python3
"""Confere que a referência do iTero cobre o bundle inteiro.

Lê o bundle extraído (o diretório com `dist/` e `dist-electron/`), tira dele as
listas que a documentação promete cobrir, e falha se algum item não aparece nos
`.md` desta pasta. Também confere o cruzamento: todo `I-n` do catálogo aparece
uma vez no cruzamento, com uma decisão entre copiar, mudar, rejeitar, adiar e
fora e uma nota, e as contagens por situação e por decisão batem com as tabelas.

Uso, sem executar nada do bundle:

    python3 -I docs/referencia/itero/conferir.py <diretório-extraído>
"""

import pathlib
import re
import sys

AQUI = pathlib.Path(__file__).resolve().parent


def ler(caminho):
    return caminho.read_text(encoding="utf-8", errors="replace")


def janelas(raiz):
    return sorted(p.name for p in (raiz / "dist/src/app").iterdir() if (p / "index.html").exists())


def rotas(raiz):
    desktop = next((raiz / "dist/assets").glob("desktop-*.js"))
    achadas = re.findall(r'=[A-Za-z0-9_$]+\("(/[^"]*)"\)\(\{component:', ler(desktop))
    # Grupos de rota sem segmento na URL: "/(auth)/login/" é servida em "/login/".
    return sorted({re.sub(r"/\([^)]*\)", "", r) for r in achadas})


def preload(raiz):
    return sorted(set(re.findall(r'invoke\("([a-z:-]+)"', ler(raiz / "dist-electron/preload.mjs"))))


def canais_principais(raiz):
    main = ler(raiz / "dist-electron/main.js")
    tratados = set(re.findall(r'Ce\.handle\(\s*"([^"]+)"', main))
    tratados |= set(re.findall(r'Ft\(\s*"(api:[^"]+)"', main))
    enviados = set(re.findall(r'webContents\.send\(\s*"([^"]+)"', main))
    return sorted(tratados), sorted(enviados)


def stores(raiz):
    main = ler(raiz / "dist-electron/main.js")
    nomes = set(re.findall(r'name: "([a-z-]+)-store"', main))
    nomes |= set(re.findall(r'\b(?:Po|Uo|ma|To|Wr) = "([a-z0-9-]+)"', main)) - {"settings-store"}
    # As dez stores da partida: identificador = "nome", passados ao fábrica Zr.
    ids = re.findall(r"Zr\(\s*([A-Za-z0-9_$]+),", main)
    for ident in ids:
        m = re.search(rf"\b{re.escape(ident)} = \"([a-z-]+)\"", main)
        if m:
            nomes.add(m.group(1))
    return sorted(n for n in nomes if not n.startswith("http"))


def recursos_gep(raiz):
    main = ler(raiz / "dist-electron/main.js")
    saida = set()
    for bloco in re.findall(r'_(?:FEATURES)", \[(.*?)\]', main, re.S):
        saida |= set(re.findall(r'"([a-z_]+)"', bloco))
    return sorted(saida)


def endpoints(raiz):
    textos = [ler(p) for p in (raiz / "dist/assets").glob("*.js")] + [ler(raiz / "dist-electron/main.js")]
    saida = set()
    for t in textos:
        saida |= set(re.findall(r"lake-[a-z0-9-]*[a-z0-9]", t))
    return sorted(saida)


def caminhos_lcu(raiz):
    main = ler(raiz / "dist-electron/main.js")
    achados = re.findall(r'["`](/lol-[a-z-]+/v1/[^"`$]*)', main)
    return sorted({a.rstrip("/") for a in achados})


def chaves(raiz, marcador):
    linhas = ler(raiz / "dist-electron/main.js").splitlines()
    inicio = next(i for i, l in enumerate(linhas) if marcador in l)
    saida = []
    for linha in linhas[inicio + 1 :]:
        if re.match(r"^\}", linha):
            break
        m = re.match(r"^  ([A-Za-z0-9_]+):", linha)
        if m:
            saida.append(m.group(1))
    return saida


def eventos_ao_vivo(raiz):
    main = ler(raiz / "dist-electron/main.js")
    casos = set(re.findall(r'case "((?:kill|death|assist|gold|level|game_info|events|live_client_data|jungle_camps|me)\.[a-z_0-9]+)"', main))
    casos |= set(re.findall(r'case "(GameEnd|DragonKill|BaronKill|InhibKilled|TurretKilled)"', main))
    return sorted(casos)


DECISOES = ("copiar", "mudar", "rejeitar", "adiar", "fora")


def tabelas(texto):
    """Cada tabela do markdown como (cabeçalho, linhas), com as células aparadas."""
    def celulas(linha):
        return [c.strip() for c in linha.strip().strip("|").split("|")]

    linhas = texto.splitlines()
    saida = []
    i = 0
    while i < len(linhas):
        if linhas[i].startswith("|") and i + 1 < len(linhas) and re.match(r"^\|[-| ]+\|$", linhas[i + 1]):
            cabecalho, corpo = celulas(linhas[i]), []
            i += 2
            while i < len(linhas) and linhas[i].startswith("|"):
                corpo.append(celulas(linhas[i]))
                i += 1
            saida.append((cabecalho, corpo))
        else:
            i += 1
    return saida


def principal(celula):
    """A primeira palavra da célula, antes do `·`: a situação ou a decisão."""
    return celula.replace("*", "").split(" · ")[0].strip()


def conferir_cruzamento(docs):
    catalogo = set(re.findall(r"^\| (I-\d+) \|", docs["05-funcionalidades.md"], re.M))
    cruz = docs["07-cruzamento.md"]
    erros = []
    linhas = []
    contagens = {}
    for cabecalho, corpo in tabelas(cruz):
        if cabecalho[:1] == ["Id"]:
            if "Decisão" not in cabecalho:
                erros.append(f"tabela sem a coluna Decisão: {' | '.join(cabecalho)}")
            linhas += [dict(zip(cabecalho, c)) for c in corpo]
        elif cabecalho[1:2] == ["Quantas"]:
            contagens[cabecalho[0]] = {c[0]: [int(n) for n in c[1:]] for c in corpo}

    vistos = [l.get("Id", "") for l in linhas]
    faltam = sorted(catalogo - set(vistos), key=lambda x: int(x[2:]))
    repetidos = sorted({i for i in vistos if vistos.count(i) > 1})
    if faltam:
        erros.append(f"no catálogo e fora do cruzamento: {', '.join(faltam)}")
    if repetidos:
        erros.append(f"repetidos no cruzamento: {', '.join(repetidos)}")

    situacoes, decisoes = {}, {}
    for l in linhas:
        s, d = principal(l.get("Presage", "")), principal(l.get("Decisão", ""))
        situacoes[s] = situacoes.get(s, 0) + 1
        if d not in DECISOES:
            erros.append(f"{l['Id']}: decisão '{d}' fora de {'/'.join(DECISOES)}")
        n = decisoes.setdefault(d, [0, 0])
        n[0] += 1
        n[1] += "a revisar" in l.get("Decisão", "")
        if not l.get("Nota", "").strip():
            erros.append(f"{l['Id']}: sem nota")

    for nome, achado in (("Situação", {s: [n] for s, n in situacoes.items()}), ("Decisão", decisoes)):
        tabela = contagens.get(nome)
        if tabela is None:
            erros.append(f"falta a tabela de contagem por {nome.lower()}")
            continue
        for chave in sorted(set(tabela) | set(achado)):
            if tabela.get(chave) != achado.get(chave):
                erros.append(f"contagem de '{chave}': tabela diz {tabela.get(chave)}, linhas dão {achado.get(chave)}")
    return len(catalogo), situacoes, decisoes, erros


def main():
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    raiz = pathlib.Path(sys.argv[1])
    docs = {p.name: ler(p) for p in AQUI.glob("*.md")}
    tudo = "\n".join(docs.values())

    listas = {
        "janelas": janelas(raiz),
        "rotas": rotas(raiz),
        "canais do preload": preload(raiz),
        "canais tratados no principal": canais_principais(raiz)[0],
        "canais enviados às janelas": canais_principais(raiz)[1],
        "stores": stores(raiz),
        "recursos do GEP": recursos_gep(raiz),
        "endpoints do backend": endpoints(raiz),
        "caminhos do LCU": caminhos_lcu(raiz),
        "configurações (settings)": chaves(raiz, "}, ts = {"),
        "chaves da store config": chaves(raiz, 'const To = "config", Gu = {'),
        "eventos ao vivo tratados": eventos_ao_vivo(raiz),
    }

    falhou = False
    for nome, itens in listas.items():
        faltando = [i for i in itens if f"`{i}`" not in tudo and i not in tudo]
        estado = "ok" if not faltando and itens else "FALTA"
        print(f"{estado:5} {nome}: {len(itens) - len(faltando)}/{len(itens)}")
        for i in faltando:
            print(f"        - {i}")
        falhou |= bool(faltando) or not itens

    total, situacoes, decisoes, erros = conferir_cruzamento(docs)
    por_decisao = {d: n for d, (n, _) in decisoes.items()}
    revisar = sum(r for _, r in decisoes.values())
    print(f"{'ok' if not erros else 'FALTA':5} cruzamento: {total} funcionalidades, {sum(situacoes.values())} linhas, {situacoes}")
    print(f"{'ok' if not erros else 'FALTA':5} decisões: {por_decisao}, {revisar} a revisar")
    for e in erros:
        print(f"        - {e}")
    falhou |= bool(erros)
    return 1 if falhou else 0


if __name__ == "__main__":
    sys.exit(main())
