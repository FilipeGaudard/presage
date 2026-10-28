import type { StaticData } from "../lib/ddragon";
import { ROLES, type Buff, type LiveSnapshot, type Rival } from "../lib/types";

function clock(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

function countdown(seconds: number): string {
  if (seconds <= 0) return "disponível";
  return clock(seconds);
}

export default function LiveOverview({
  snapshot,
  staticData = null,
}: {
  snapshot: LiveSnapshot;
  staticData?: StaticData | null;
}) {
  const { game, timeline, priority } = snapshot;
  const me = game.players.find((player) => player.isLocal);
  const respawns = timeline.respawns ?? [];
  const buffs = snapshot.buffs ?? [];

  return (
    <>
      <div className="draft-head">
        <div>
          <h2>Partida</h2>
          <span className="muted">
            {me ? `${me.championKey} · nível ${me.level}` : "jogador não identificado"}
          </span>
        </div>
        <div className="clock">{clock(timeline.gameTime)}</div>
      </div>

      {priority.focus && <div className="banner banner-turn">{priority.focus}</div>}

      {snapshot.rival && <SuaRota rival={snapshot.rival} staticData={staticData} />}

      <div className="section">
        <h3>Objetivos</h3>
        <div className="timers">
          {timeline.upcoming.length === 0 && <p className="muted">Nenhum objetivo na fila.</p>}
          {timeline.upcoming.map((spawn) => (
            <div className="timer" key={spawn.name}>
              <span className="timer-name">{spawn.name}</span>
              <span
                className={`timer-value ${spawn.secondsLeft <= 60 ? "split-key" : ""}`}
              >
                {countdown(spawn.secondsLeft)}
              </span>
              {!spawn.observed && <span className="timer-note">estimado</span>}
            </div>
          ))}
          {respawns.map((r) => (
            <div className="timer" key={`${r.name}-${r.at}`}>
              <span className="timer-name">{r.name}</span>
              <span className="timer-value">{clock(r.secondsLeft)}</span>
              <span className={`timer-note ${r.ours ? "delta-weak" : "delta-strong"}`}>
                {r.ours ? "seu, caído" : "deles, caído"}
              </span>
            </div>
          ))}
        </div>
        {buffs.map((buff) => (
          <BuffLinha key={`${buff.name}-${buff.started}`} buff={buff} staticData={staticData} />
        ))}
        <div className="facts">
          <div className="fact">
            <span className="fact-label">Dragões</span>
            <span className="fact-value">
              {timeline.dragons.ally} <span className="muted">–</span> {timeline.dragons.enemy}
            </span>
          </div>
          <div className="fact">
            <span className="fact-label">Torres</span>
            <span className="fact-value">
              {timeline.turrets.ally} <span className="muted">–</span> {timeline.turrets.enemy}
            </span>
          </div>
          <div className="fact">
            <span className="fact-label">Abates</span>
            <span className="fact-value">
              {timeline.kills.ally} <span className="muted">–</span> {timeline.kills.enemy}
            </span>
          </div>
          {timeline.soul && (
            <div className="fact">
              <span className="fact-label">Alma</span>
              <span className={`fact-value ${timeline.soul === "ally" ? "delta-strong" : "delta-weak"}`}>
                {timeline.soul === "ally" ? "sua" : "deles"}
              </span>
            </div>
          )}
        </div>
      </div>

      <div className="section">
        <h3>Prioridade agora</h3>
        {priority.lanes.length === 0 ? (
          <p className="muted">Sem dados de rota.</p>
        ) : (
          <div className="lanes">
            {priority.lanes.map((lane) => (
              <div className="lane" key={lane.lane}>
                <div className="lane-head">
                  <span className="lane-name">{lane.lane}</span>
                  <span className="lane-score">{lane.score.toFixed(0)}</span>
                </div>
                <div className="lane-bar">
                  <div className="lane-fill" style={{ width: `${lane.score}%` }} />
                </div>
                <ul className="detail-reasons">
                  {lane.reasons.map((reason) => (
                    <li key={reason}>{reason}</li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        )}
        <p className="muted footnote">
          Ranking de valor, não de posição. A API da Riot não entrega onde os
          campeões estão, e o Presage não tenta adivinhar.
        </p>
      </div>
    </>
  );
}

const METRICA: Record<"farm" | "vision", string> = {
  farm: "tropas/min",
  vision: "visão/min",
};

const ESTADO: Record<Rival["standing"], { texto: string; classe: string }> = {
  ahead: { texto: "à frente", classe: "delta-strong" },
  even: { texto: "igual", classe: "delta-ok" },
  behind: { texto: "atrás", classe: "delta-weak" },
};

/** Você contra o oponente direto, no que a sua rota cobra: o suporte é medido
 *  em visão, nunca em tropas. */
function SuaRota({ rival, staticData }: { rival: Rival; staticData: StaticData | null }) {
  const oponente = staticData?.champions.get(rival.opponentId)?.name ?? rival.opponentKey;
  const rota = ROLES.find((r) => r.id === rival.role)?.label ?? "";
  const estado = ESTADO[rival.standing];
  const sinal = rival.goldDiff > 0 ? "+" : "";
  return (
    <div className="section">
      <h3>
        Sua rota <span className="muted">({rota}) contra {oponente}</span>
      </h3>
      <div className="facts">
        {rival.metric && (
          <div className="fact">
            <span className="fact-label">{METRICA[rival.metric]}</span>
            <span className="fact-value">
              {rival.minePerMin.toFixed(1)} <span className="muted">–</span>{" "}
              {rival.theirsPerMin.toFixed(1)}
            </span>
          </div>
        )}
        <div className="fact">
          <span className="fact-label">ouro em itens</span>
          <span className={`fact-value ${estado.classe}`}>
            {sinal}
            {Math.round(rival.goldDiff).toLocaleString("pt-BR")} · {estado.texto}
          </span>
        </div>
      </div>
      <p className="faint">
        Ouro em itens é exato. "À frente" a partir de 300 ou de 10% do que ele
        investiu, o maior — regra, não estatística.
      </p>
    </div>
  );
}

/** Buff de Barão ou Ancião: quanto falta e quem ainda tem. */
function BuffLinha({ buff, staticData }: { buff: Buff; staticData: StaticData | null }) {
  return (
    <div className="buff">
      <span className="timer-name">
        {buff.name} <span className="muted">({buff.ours ? "seu time" : "deles"})</span>
      </span>
      <span className="timer-value">{clock(buff.secondsLeft)}</span>
      {buff.holders === null ? (
        <span className="timer-note">quem tem: abate antes de o app olhar</span>
      ) : (
        <span className="buff-holders">
          {buff.holders.map((id) => {
            const champion = staticData?.champions.get(id);
            return champion ? (
              <img key={id} src={champion.iconUrl} alt={champion.name} title={champion.name} />
            ) : (
              <span key={id}>#{id}</span>
            );
          })}
        </span>
      )}
    </div>
  );
}
