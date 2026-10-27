import type { LiveSnapshot } from "../lib/types";

function clock(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

function countdown(seconds: number): string {
  if (seconds <= 0) return "disponível";
  return clock(seconds);
}

export default function LiveOverview({ snapshot }: { snapshot: LiveSnapshot }) {
  const { game, timeline, priority } = snapshot;
  const me = game.players.find((player) => player.isLocal);

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
        </div>
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
