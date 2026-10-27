import type { ConnStatus } from "../lib/types";

const PHASE_LABEL: Record<string, string> = {
  None: "Cliente aberto",
  Lobby: "No lobby",
  Matchmaking: "Na fila",
  ReadyCheck: "Partida encontrada",
  ChampSelect: "Seleção de campeões",
  InProgress: "Em partida",
  WaitingForStats: "Finalizando",
  PreEndOfGame: "Fim de jogo",
  EndOfGame: "Fim de jogo",
  Reconnect: "Reconectando",
  Disconnected: "Cliente fechado",
};

export default function ConnectionStatus({ status }: { status: ConnStatus | null }) {
  const connected = status?.connected ?? false;
  const phase = status?.phase ?? "Disconnected";

  return (
    <div className="conn" title={status?.detail ?? undefined}>
      <span className={`dot ${connected ? "dot-on" : "dot-off"}`} />
      <span className="conn-phase">{PHASE_LABEL[phase] ?? phase}</span>
      {status?.summoner && <span className="conn-summoner">{status.summoner}</span>}
    </div>
  );
}
