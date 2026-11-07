import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import Runas from "./Runas";
import type { RunePlan } from "../lib/types";

afterEach(cleanup);

const plano: RunePlan = {
  keystones: [{ key: "Conqueror", id: 8010, name: "Conquistador", icon: "", treeName: "Precisão", score: 1.8, reasons: ["troca longa"] }],
  secondary: null,
  matchup: "Garen contra Malphite",
  fromData: false,
};

describe("Runas", () => {
  it("TF-ROTAS-2: runa sem base diz que é regra, e não fala em partidas (RNF-1)", () => {
    const { container } = render(<Runas plan={plano} patch="16.21" />);
    expect(screen.getByText(/Isto sai de regras sobre a mecânica do confronto/)).toBeTruthy();
    expect(container.textContent).not.toMatch(/Base de partidas|partidas do patch/);
  });
});
