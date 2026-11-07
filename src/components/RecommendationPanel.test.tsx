import { afterEach, describe, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import RecommendationPanel from "./RecommendationPanel";
import type { StaticData } from "../lib/ddragon";
import type { DraftState, MinimaxPick, ScoredChampion } from "../lib/types";

afterEach(cleanup);

const ids = Array.from({ length: 16 }, (_, i) => i + 1);

const staticData: StaticData = {
  version: "16.21.1",
  champions: new Map(ids.map((id) => [id, { id, key: `C${id}`, name: `Campeão ${id}`, iconUrl: "" }])),
  allIds: ids,
};

const draft: DraftState = {
  phase: "BAN_PICK",
  timeLeftMs: 25000,
  totalTimeMs: 30000,
  localCellId: 0,
  localPosition: "middle",
  myTeam: [],
  theirTeam: [],
  myBans: [],
  theirBans: [],
  currentAction: "pick",
  isLocalTurn: true,
  unavailable: [],
};

function candidato(championId: number): ScoredChampion {
  return { championId, score: 60 - championId, delta: 1, winrate: 0.5, confidence: 8, sample: 0, inPool: false, reasons: [] };
}

function arvore(championId: number, exposure: number): MinimaxPick {
  return { championId, greedy: 0.5, afterResponse: 0.5 - exposure, exposure, bestResponse: null, inPool: false };
}

describe("RecommendationPanel", () => {
  it("TF-DRAFT-12: os menos expostos saem dos candidatos da tela, e o 15º abre com a resposta deles", () => {
    // Dezesseis candidatos: a tela mostra os quinze primeiros. O 16º é o menos
    // exposto de todos, mas não está na tela, então não entra nos menos
    // expostos (A21: o minimax agora traz todo candidato).
    const minimax = ids.map((id) => arvore(id, id === 16 ? 0.001 : 0.002 * id));
    render(
      <RecommendationPanel
        recommendations={ids.map(candidato)}
        minimax={minimax}
        bans={[]}
        staticData={staticData}
        draft={draft}
        engine={null}
      />
    );

    const secao = screen.getByText("Menos expostos ao contra-pick").closest(".section") as HTMLElement;
    const nomes = within(secao)
      .getAllByRole("button")
      .map((b) => b.getAttribute("title"));
    expect(nomes).toEqual(["Campeão 1", "Campeão 2", "Campeão 3", "Campeão 4", "Campeão 5"]);

    // O último da tela tem a árvore dele.
    const geral = screen.getByText("Melhores no geral").closest(".section") as HTMLElement;
    fireEvent.click(within(geral).getByTitle("Campeão 15"));
    expect(screen.getByText(/Melhor resposta deles custa 3\.0pp/)).toBeTruthy();
  });
});
