import type { StaticData } from "../lib/ddragon";
import { POSITION_LABEL, type DraftSlot, type DraftState } from "../lib/types";

interface Props {
  draft: DraftState | null;
  staticData: StaticData | null;
  secondsLeft: number | null;
  connected: boolean;
  phase: string;
}

export default function DraftBoard({ draft, staticData, secondsLeft, connected, phase }: Props) {
  if (!draft) {
    return (
      <div className="empty">
        <h2>Sem draft ativo</h2>
        <p>{connected ? `Cliente conectado (${phase}).` : "Abra o League of Legends."}</p>
      </div>
    );
  }

  const actionLabel =
    draft.currentAction === "ban" ? "BANINDO" : draft.currentAction === "pick" ? "ESCOLHENDO" : "—";

  return (
    <>
      <div className="draft-head">
        <div>
          <h2>Seleção de campeões</h2>
          <span className="muted">
            {draft.phase} · {actionLabel}
            {draft.localPosition &&
              ` · você: ${POSITION_LABEL[draft.localPosition] ?? draft.localPosition}`}
          </span>
        </div>
        <div className={`clock ${draft.isLocalTurn ? "clock-urgent" : ""}`}>
          {secondsLeft === null ? "--" : Math.ceil(secondsLeft)}
          <span className="clock-unit">s</span>
        </div>
      </div>

      {draft.isLocalTurn && (
        <div className="banner banner-turn">
          É a sua vez de {draft.currentAction === "ban" ? "banir" : "escolher"}.
        </div>
      )}

      <div className="teams">
        <TeamColumn title="Seu time" side="ally" slots={draft.myTeam} staticData={staticData} />
        <TeamColumn title="Inimigos" side="enemy" slots={draft.theirTeam} staticData={staticData} />
      </div>

      <div className="bans">
        <BanRow label="Bans aliados" ids={draft.myBans} staticData={staticData} />
        <BanRow label="Bans inimigos" ids={draft.theirBans} staticData={staticData} />
      </div>
    </>
  );
}

function TeamColumn({
  title,
  side,
  slots,
  staticData,
}: {
  title: string;
  side: "ally" | "enemy";
  slots: DraftSlot[];
  staticData: StaticData | null;
}) {
  return (
    <div className={`team team-${side}`}>
      <h3>{title}</h3>
      {slots.length === 0 && <p className="muted">—</p>}
      {slots.map((slot) => {
        const shownId = slot.championId || slot.intentId;
        const champion = shownId ? staticData?.champions.get(shownId) : undefined;
        const isHover = !slot.championId && !!slot.intentId;

        return (
          <div
            key={slot.cellId}
            className={[
              "slot",
              slot.isLocal ? "slot-local" : "",
              slot.isActing ? "slot-acting" : "",
              isHover ? "slot-hover" : "",
            ]
              .filter(Boolean)
              .join(" ")}
          >
            <div className="slot-portrait">
              {champion ? (
                <img src={champion.iconUrl} alt={champion.name} loading="lazy" />
              ) : (
                <span className="slot-placeholder">?</span>
              )}
            </div>
            <div className="slot-info">
              <span className="slot-name">
                {champion?.name ?? (shownId ? `#${shownId}` : "Aguardando")}
              </span>
              <span className="slot-meta">
                {POSITION_LABEL[slot.position] ?? slot.position.toUpperCase()}
                {slot.isLocal && " · você"}
                {isHover && " · hover"}
              </span>
            </div>
            {slot.isActing && <span className="slot-pulse" />}
          </div>
        );
      })}
    </div>
  );
}

function BanRow({
  label,
  ids,
  staticData,
}: {
  label: string;
  ids: number[];
  staticData: StaticData | null;
}) {
  return (
    <div className="ban-row">
      <span className="ban-label">{label}</span>
      <div className="ban-icons">
        {ids.length === 0 && <span className="muted">nenhum</span>}
        {ids.map((id, index) => {
          const champion = staticData?.champions.get(id);
          return (
            <div className="ban-icon" key={`${id}-${index}`} title={champion?.name ?? `#${id}`}>
              {champion ? <img src={champion.iconUrl} alt={champion.name} /> : <span>?</span>}
            </div>
          );
        })}
      </div>
    </div>
  );
}
