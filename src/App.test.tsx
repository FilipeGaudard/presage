import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import App from "./App";
import type { DraftSlot, DraftState, EngineStatus, Pool, ScoredChampion } from "./lib/types";

// A casca do front (frontend 06 §1): `App` montado com a IPC do Tauri simulada.
// Cada teste diz o que cada comando responde; o que ele não conhece responde
// `null`, como um comando de leitura sem dado.

type Resposta = (args: Record<string, unknown>) => unknown;

// O que a casca sempre devolve, mesmo sem dado: os comandos que não são `Option`.
const PADRAO: Record<string, Resposta> = {
  get_snapshot: () => [null, null],
  get_pool: () => ({ entries: [] }),
  get_overlay_prefs: () => ({ hidden: [] }),
  get_draft_prefs: () => ({ showAllChampions: false }),
  get_autostart: () => false,
  rank_draft: () => [],
  minimax_draft: () => [],
  ban_suggestions: () => [],
};

function casca(respostas: Record<string, Resposta>) {
  const todas = { ...PADRAO, ...respostas };
  mockIPC(
    (cmd, args) => {
      const r = todas[cmd];
      return r ? r((args ?? {}) as Record<string, unknown>) : null;
    },
    { shouldMockEvents: true }
  );
}

const engine: EngineStatus = {
  ready: true,
  version: "16.21.1",
  champions: 170,
  hasGlobal: false,
  hasPersonal: false,
  personalGames: 0,
  account: null,
  globalPatch: null,
  globalMatches: 0,
};

function slot(over: Partial<DraftSlot>): DraftSlot {
  return {
    cellId: 0,
    position: "",
    championId: 0,
    intentId: 0,
    spell1: 0,
    spell2: 0,
    isLocal: false,
    isActing: false,
    locked: false,
    ...over,
  };
}

function draft(timeLeftMs: number): DraftState {
  return {
    phase: "BAN_PICK",
    timeLeftMs,
    totalTimeMs: 30000,
    localCellId: 0,
    localPosition: "middle",
    myTeam: [slot({ cellId: 0, position: "middle", isLocal: true, isActing: true })],
    theirTeam: [slot({ cellId: 5 })],
    myBans: [],
    theirBans: [],
    currentAction: "pick",
    isLocalTurn: true,
    unavailable: [],
  };
}

function candidato(championId: number): ScoredChampion {
  return {
    championId,
    score: 51,
    delta: 1,
    winrate: 0.51,
    confidence: 8,
    sample: 0,
    inPool: false,
    reasons: ["Equilibra o dano do time"],
  };
}

beforeEach(() => {
  // O Data Dragon fica fora: sem rede no teste, a faixa de erro dele aparece.
  vi.stubGlobal("fetch", vi.fn(() => Promise.reject(new Error("sem rede no teste"))));
});

afterEach(() => {
  cleanup();
  clearMocks();
  vi.unstubAllGlobals();
});

describe("App e a casca", () => {
  it("TF-POOL-4: um set_pool que rejeita mostra o erro e devolve a tela à pool que a casca tem", async () => {
    const pool: Pool = { entries: [{ championId: 103, role: "Mid", tier: "core" }], bans: [1, 2] };
    casca({
      get_pool: () => pool,
      set_pool: () => Promise.reject("disco cheio"),
    });
    render(<App />);
    // As leituras iniciais voltam antes do clique, como no app de verdade.
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });
    fireEvent.click(screen.getByRole("button", { name: "Pool" }));
    await screen.findByText("2. #2");

    // Subir o segundo ban: a casca recusa a gravação.
    const subir = screen.getAllByTitle("Subir");
    await act(async () => {
      fireEvent.click(subir[1]);
    });

    await screen.findByText(/Não consegui gravar a pool/);
    expect(screen.getByText(/disco cheio/)).toBeTruthy();
    // A tela volta ao que a casca tem: o ban 1 continua em primeiro.
    expect(screen.getByText("1. #1")).toBeTruthy();
    expect(screen.getByText("2. #2")).toBeTruthy();
  });

  it("TF-CASCA-4: um rank_draft que rejeita mostra um aviso que nomeia o ranking, e a lista velha sai da tela", async () => {
    let chamadas = 0;
    casca({
      get_snapshot: () => [{ connected: true, phase: "ChampSelect", summoner: null, detail: null }, draft(25000)],
      engine_status: () => engine,
      rank_draft: () => {
        chamadas += 1;
        return chamadas === 1 ? [candidato(103)] : Promise.reject("motor ocupado");
      },
      minimax_draft: () => [],
      ban_suggestions: () => [],
    });
    render(<App />);
    await screen.findByText(/1 candidatos/);

    // O champ select muda; a nova leitura do ranking falha.
    await act(async () => {
      await emit("presage://draft", draft(20000));
    });

    await screen.findByText(/Não consegui ler o ranking/);
    expect(screen.getByText(/motor ocupado/)).toBeTruthy();
    await waitFor(() => expect(screen.queryByText(/1 candidatos/)).toBeNull());
  });
});
