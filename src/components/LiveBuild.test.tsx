import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import LiveBuild from "./LiveBuild";
import type { BoughtReport, BuildContext, ItemSuggestion, LiveGame } from "../lib/types";

afterEach(cleanup);

// Uma partida em que o Live Client ainda não disse quem é você.
const semVoce: LiveGame = { gameTime: 30, players: [] };

const contexto = { incomingPhysicalShare: 0.6 } as BuildContext;

describe("LiveBuild", () => {
  it("TF-PARTIDA-5: sem o seu campeão a build diz isso, e item que não veio diz por quê (A32)", () => {
    // `build_suggestions` só devolve nulo sem partida ou sem jogador local: não
    // é carga de item, e a tela não pode dizer "Carregando" para sempre.
    const { unmount } = render(<LiveBuild game={semVoce} context={null} suggestions={[]} staticData={null} />);
    expect(screen.queryByText(/Carregando os dados de item/)).toBeNull();
    expect(screen.getByText(/seu campeão/)).toBeTruthy();
    unmount();

    render(
      <LiveBuild
        game={semVoce}
        context={contexto}
        suggestions={[]}
        staticData={null}
        itemsError="sem rede e sem cache"
      />
    );
    expect(screen.getByText(/Não consegui os dados de item/)).toBeTruthy();
    expect(screen.getByText(/sem rede e sem cache/)).toBeTruthy();
  });

  it("TF-PARTIDA-6: com base, a build mostra o que se compra com o campeão; sem ela, diz que a lista é conta (R-10, R-21)", () => {
    const conta: ItemSuggestion[] = [
      { itemId: 3031, name: "Gume do Infinito", cost: 3450, valuePer1000g: 12.3, ehpGain: 0, offenseGainPercent: 4, reasons: [] },
    ];
    const comprado: BoughtReport = {
      championId: 42,
      role: "Mid",
      games: 1240,
      winrate: 0.51,
      items: [
        { itemId: 3078, name: "Força da Trindade", cost: 3333, games: 868, share: 0.7, winrate: 0.534 },
        { itemId: 6676, name: "O Coletor", cost: 3200, games: 310, share: 0.25, winrate: 0.528 },
      ],
    };
    const { unmount } = render(
      <LiveBuild game={semVoce} context={contexto} suggestions={conta} staticData={null} bought={comprado} patch="do patch 16.22" />
    );
    const secao = screen.getByText(/O que se compra/).closest(".section") as HTMLElement;
    expect(secao.textContent).toMatch(/1\.240 partidas/);
    expect(secao.textContent).toMatch(/patch 16\.22/);
    expect(secao.textContent).toMatch(/Força da Trindade/);
    expect(secao.textContent).toMatch(/70% compram/);
    expect(secao.textContent).toMatch(/vence 53,4%/);
    // A conta de atributos continua, e diz que é conta.
    expect(screen.getByText(/A conta de atributos: vida efetiva/)).toBeTruthy();
    unmount();

    render(<LiveBuild game={semVoce} context={contexto} suggestions={conta} staticData={null} bought={null} patch={null} />);
    expect(screen.queryByText(/O que se compra/)).toBeNull();
    expect(screen.getByText(/Sem base de partidas para este campeão/)).toBeTruthy();
  });

  it("TF-PARTIDA-7: no treino, sem inimigo, a build não inventa a divisão do dano", () => {
    const so_voce: LiveGame = {
      gameTime: 120,
      players: [
        {
          championKey: "Corki", championId: 42, displayName: "voce#BR1", team: "ORDER", position: "", level: 3,
          items: [], goldInvested: 0, isDead: false, respawnTimer: 0, isLocal: true,
          scores: { kills: 0, deaths: 0, assists: 0, creepScore: 10, wardScore: 0 },
        },
      ],
    };
    render(<LiveBuild game={so_voce} context={{ ...contexto, yourEffectiveHp: 793 } as BuildContext} suggestions={[]} staticData={null} />);
    expect(screen.getByText(/Sem inimigos na partida/)).toBeTruthy();
    expect(screen.queryByText(/50% físico/)).toBeNull();
  });
});
