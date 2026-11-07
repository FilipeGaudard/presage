import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import LiveBuild from "./LiveBuild";
import type { BuildContext, LiveGame } from "../lib/types";

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
});
