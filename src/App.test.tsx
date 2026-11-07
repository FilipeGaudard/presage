import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import App from "./App";
import type { DraftSlot, DraftState, EngineStatus, Pool, RunePlan, ScoredChampion } from "./lib/types";

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

function draft(timeLeftMs: number, over: Partial<DraftState> = {}): DraftState {
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
    ...over,
  };
}

const plano: RunePlan = {
  keystones: [
    { key: "Electrocute", id: 8112, name: "Eletrocutar", icon: "", treeName: "Dominação", score: 1, reasons: ["explosão curta"] },
  ],
  secondary: null,
  matchup: "Ahri contra Zed",
  fromData: false,
};

/** Quantas vezes cada comando foi chamado, e com quê. */
function contador() {
  const chamadas: Array<{ cmd: string; args: Record<string, unknown> }> = [];
  const conta = (cmd: string, pred: (a: Record<string, unknown>) => boolean = () => true) =>
    chamadas.filter((c) => c.cmd === cmd && pred(c.args)).length;
  const registra = (cmd: string, resposta: Resposta): Resposta => (args) => {
    chamadas.push({ cmd, args });
    return resposta(args);
  };
  return { conta, registra };
}

async function assentar() {
  await act(async () => {
    await new Promise((r) => setTimeout(r, 0));
  });
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

  it("TF-ROTAS-3: escolher a rota refaz rotas e runas; sem draft e sem partida, a runa some", async () => {
    const c = contador();
    casca({
      get_snapshot: () => [
        { connected: true, phase: "ChampSelect", summoner: null, detail: null },
        draft(25000, { localPosition: "", myTeam: [slot({ cellId: 0, isLocal: true, isActing: true })] }),
      ],
      engine_status: () => engine,
      lane_analysis: c.registra("lane_analysis", () => null),
      rune_plan: c.registra("rune_plan", () => plano),
      set_draft_role: () => null,
    });
    render(<App />);
    await assentar();
    const antes = { rotas: c.conta("lane_analysis"), runas: c.conta("rune_plan") };
    expect(antes.runas).toBeGreaterThan(0);

    // Em blind a rota é escolhida na tela; a análise depende dela (A11).
    fireEvent.click(screen.getByRole("button", { name: "MID" }));
    await waitFor(() => expect(c.conta("lane_analysis")).toBeGreaterThan(antes.rotas));
    expect(c.conta("rune_plan")).toBeGreaterThan(antes.runas);

    fireEvent.click(screen.getByRole("button", { name: "Rotas" }));
    await screen.findByText(/Ahri contra Zed/);

    // O champ select acabou e não há partida: a runa sai junto com as rotas.
    await act(async () => {
      await emit("presage://draft", null);
    });
    await waitFor(() => expect(screen.queryByText(/Ahri contra Zed/)).toBeNull());
  });

  it("TF-DRAFT-10: com um candidato aberto, marcar a rota de um inimigo refaz a prévia", async () => {
    const c = contador();
    casca({
      get_snapshot: () => [
        { connected: true, phase: "ChampSelect", summoner: null, detail: null },
        draft(25000, { theirTeam: [slot({ cellId: 5, championId: 238, locked: true })], unavailable: [238] }),
      ],
      engine_status: () => engine,
      rank_draft: () => [candidato(103)],
      lane_analysis: c.registra("lane_analysis", () => null),
      set_enemy_role: () => null,
    });
    render(<App />);
    fireEvent.click(await screen.findByTitle("#103"));
    const comCandidato = (a: Record<string, unknown>) => a.candidate === 103;
    await waitFor(() => expect(c.conta("lane_analysis", comCandidato)).toBe(1));

    // O inimigo vira MID à mão: a prévia muda de oponente (A11).
    fireEvent.change(screen.getByTitle(/Rota presumida pelo Presage/), { target: { value: "Mid" } });
    await waitFor(() => expect(c.conta("lane_analysis", comCandidato)).toBe(2));
  });
});
