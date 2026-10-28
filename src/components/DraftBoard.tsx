import type { StaticData } from "../lib/ddragon";
import { POSITION_LABEL, ROLES, type DraftSlot, type DraftState, type Role } from "../lib/types";

interface Props {
  draft: DraftState | null;
  staticData: StaticData | null;
  secondsLeft: number | null;
  connected: boolean;
  phase: string;
  /** A rota que o motor presumiu para cada inimigo, por campeão. */
  inferred?: Record<number, Role>;
  /** A rota que você marcou, por campeão. */
  marked?: Record<number, Role>;
  onEnemyRole?: (championId: number, role: Role | null) => void;
}

export default function DraftBoard({
  draft,
  staticData,
  secondsLeft,
  connected,
  phase,
  inferred,
  marked,
  onEnemyRole,
}: Props) {
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
        <TeamColumn
          title="Inimigos"
          side="enemy"
          slots={draft.theirTeam}
          staticData={staticData}
          inferred={inferred}
          marked={marked}
          onEnemyRole={onEnemyRole}
        />
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
  inferred,
  marked,
  onEnemyRole,
}: {
  title: string;
  side: "ally" | "enemy";
  slots: DraftSlot[];
  staticData: StaticData | null;
  inferred?: Record<number, Role>;
  marked?: Record<number, Role>;
  onEnemyRole?: (championId: number, role: Role | null) => void;
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
              {side === "enemy" && slot.championId > 0 && !slot.position && onEnemyRole ? (
                <RotaInimiga
                  championId={slot.championId}
                  presumida={inferred?.[slot.championId]}
                  marcada={marked?.[slot.championId]}
                  onChange={onEnemyRole}
                />
              ) : (
                <span className="slot-meta">
                  {POSITION_LABEL[slot.position] ?? slot.position.toUpperCase()}
                  {slot.isLocal && " · você"}
                  {isHover && " · hover"}
                </span>
              )}
            </div>
            {slot.isActing && <span className="slot-pulse" />}
          </div>
        );
      })}
    </div>
  );
}

/** A ranqueada não diz a rota do inimigo: o motor presume, e você corrige. */
function RotaInimiga({
  championId,
  presumida,
  marcada,
  onChange,
}: {
  championId: number;
  presumida?: Role;
  marcada?: Role;
  onChange: (championId: number, role: Role | null) => void;
}) {
  const rotulo = (role?: Role) => ROLES.find((r) => r.id === role)?.label ?? "?";
  return (
    <select
      className={`slot-role ${marcada ? "slot-role-marked" : ""}`}
      value={marcada ?? ""}
      title={marcada ? "Rota marcada por você" : "Rota presumida pelo Presage; corrija se estiver errada"}
      onChange={(e) => onChange(championId, (e.target.value || null) as Role | null)}
    >
      <option value="">{`${rotulo(presumida)} (presumida)`}</option>
      {ROLES.map((r) => (
        <option key={r.id} value={r.id}>
          {r.label}
        </option>
      ))}
    </select>
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
