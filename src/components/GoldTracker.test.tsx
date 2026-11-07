import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import GoldTracker from "./GoldTracker";
import type { GoldPlayer, GoldReport } from "../lib/types";

afterEach(cleanup);

function jogador(over: Partial<GoldPlayer>): GoldPlayer {
  return {
    championKey: "Ahri",
    championId: 103,
    displayName: "eu#BR1",
    team: "ORDER",
    position: "MIDDLE",
    isLocal: false,
    isAlly: true,
    invested: 3000,
    earned: 4200,
    unspent: null,
    goldPerMinute: 400,
    ...over,
  };
}

function relatorio(players: GoldPlayer[]): GoldReport {
  return {
    gameTime: 600,
    players,
    oursInvested: 3000,
    theirsInvested: 3000,
    investedLead: 0,
    oursEarned: 4200,
    theirsEarned: 4200,
    earnedLead: 0,
    notes: [],
  };
}

describe("GoldTracker", () => {
  it("TF-PARTIDA-3: o ouro em mão de um inimigo nunca aparece; o seu, sim", () => {
    const eu = jogador({ isLocal: true, unspent: 1800 });
    // Mesmo que a casca mandasse a estimativa de um inimigo, a tela não a desenha (C-4).
    const inimigo = jogador({
      championKey: "Zed",
      championId: 238,
      displayName: "inimigo#BR1",
      team: "CHAOS",
      isAlly: false,
      unspent: 1200,
    });
    render(<GoldTracker report={relatorio([eu, inimigo])} staticData={null} />);

    expect(screen.queryByText("+1.800")).not.toBeNull();
    expect(screen.queryByText("+1.200")).toBeNull();
    expect(screen.queryByText(/o dos outros nove é reconstruído/)).toBeNull();
  });
});
