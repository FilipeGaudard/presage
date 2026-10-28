import { useMemo } from "react";
import type { StaticData } from "../lib/ddragon";
import { POSITION_LABEL, ROLES, ROLE_TO_LCU, type DraftSlot, type DraftState } from "../lib/types";

interface Props {
  value: DraftState;
  staticData: StaticData | null;
  onChange: (draft: DraftState) => void;
  /** Copiar o champ select de agora, quando existe. */
  live?: DraftState | null;
}

const POSICOES = ["top", "jungle", "middle", "bottom", "utility"];

function vaga(cellId: number, position: string, isLocal = false): DraftSlot {
  return {
    cellId,
    position,
    championId: 0,
    intentId: 0,
    spell1: 0,
    spell2: 0,
    isLocal,
    isActing: false,
    locked: false,
  };
}

/** Um draft vazio: você no meio, o inimigo sem rota (como na ranqueada). */
export function draftVazio(): DraftState {
  return {
    phase: "SIMULADO",
    timeLeftMs: 0,
    totalTimeMs: 0,
    localCellId: 2,
    localPosition: "middle",
    myTeam: POSICOES.map((p, i) => vaga(i, p, p === "middle")),
    theirTeam: POSICOES.map((_, i) => vaga(i + 5, "")),
    myBans: [],
    theirBans: [],
    currentAction: "pick",
    isLocalTurn: true,
    unavailable: [],
  };
}

/** Monte os dois times à mão e veja o que o motor recomenda, sem champ select.
 *  O motor é o mesmo do draft de verdade: ranking, resposta adversária, bans e
 *  a prévia de cada candidato. */
export default function SimuladorDraft({ value, staticData, onChange, live }: Props) {
  const campeoes = useMemo(
    () =>
      staticData
        ? [...staticData.champions.values()].sort((a, b) => a.name.localeCompare(b.name))
        : [],
    [staticData]
  );
  const porNome = useMemo(
    () => new Map(campeoes.map((c) => [c.name.toLowerCase(), c.id])),
    [campeoes]
  );
  const nome = (id: number) => (id ? staticData?.champions.get(id)?.name ?? `#${id}` : "");

  function trocar(lado: "myTeam" | "theirTeam", cellId: number, mudanca: Partial<DraftSlot>) {
    const slots = value[lado].map((s) => (s.cellId === cellId ? { ...s, ...mudanca } : s));
    onChange({ ...value, [lado]: slots });
  }

  function campeao(lado: "myTeam" | "theirTeam", cellId: number, texto: string) {
    const id = porNome.get(texto.trim().toLowerCase()) ?? 0;
    if (texto.trim() && !id) return;
    trocar(lado, cellId, { championId: id, locked: id > 0 });
  }

  // A sua rota é a da sua vaga: mudar uma muda a outra.
  function rota(cellId: number, position: string) {
    const myTeam = value.myTeam.map((s) => (s.cellId === cellId ? { ...s, position } : s));
    const local = myTeam.find((s) => s.isLocal);
    onChange({ ...value, myTeam, localPosition: local?.position ?? "" });
  }

  function serVoce(cellId: number) {
    const myTeam = value.myTeam.map((s) => ({ ...s, isLocal: s.cellId === cellId }));
    const local = myTeam.find((s) => s.isLocal);
    onChange({ ...value, myTeam, localCellId: cellId, localPosition: local?.position ?? "" });
  }

  function ban(texto: string) {
    const id = porNome.get(texto.trim().toLowerCase());
    if (!id || value.myBans.includes(id) || value.myBans.length >= 10) return;
    onChange({ ...value, myBans: [...value.myBans, id] });
  }

  return (
    <>
      <div className="draft-head">
        <div>
          <h2>Simulador de draft</h2>
          <span className="muted">
            Monte os times; a recomendação ao lado é a mesma do champ select.
          </span>
        </div>
        <div className="builder-actions">
          {live && (
            <button type="button" className="ghost" onClick={() => onChange({ ...live, phase: "SIMULADO" })}>
              Copiar o champ select
            </button>
          )}
          <button type="button" className="ghost" onClick={() => onChange(draftVazio())}>
            Limpar
          </button>
        </div>
      </div>

      <datalist id="sim-campeoes">
        {campeoes.map((c) => (
          <option key={c.id} value={c.name} />
        ))}
      </datalist>

      <div className="reco-role">
        <span className="muted">Fase:</span>
        <div className="role-tabs">
          {(["pick", "ban"] as const).map((fase) => (
            <button
              type="button"
              key={fase}
              className={value.currentAction === fase ? "on" : ""}
              onClick={() => onChange({ ...value, currentAction: fase })}
            >
              {fase === "pick" ? "ESCOLHA" : "BAN"}
            </button>
          ))}
        </div>
      </div>

      <div className="teams">
        <div className="team team-ally">
          <h3>Seu time</h3>
          {value.myTeam.map((slot) => (
            <div className={`slot ${slot.isLocal ? "slot-local" : ""}`} key={slot.cellId}>
              <div className="slot-info">
                <input
                  className="sim-input"
                  list="sim-campeoes"
                  placeholder={slot.isLocal ? "você (vazio = candidatos)" : "campeão"}
                  defaultValue={nome(slot.championId)}
                  key={`${slot.cellId}-${slot.championId}`}
                  onBlur={(e) => campeao("myTeam", slot.cellId, e.target.value)}
                />
                <span className="slot-meta">
                  <select
                    className="slot-role"
                    value={slot.position}
                    onChange={(e) => rota(slot.cellId, e.target.value)}
                  >
                    {POSICOES.map((p) => (
                      <option key={p} value={p}>
                        {POSITION_LABEL[p]}
                      </option>
                    ))}
                  </select>{" "}
                  <label>
                    <input type="radio" checked={slot.isLocal} onChange={() => serVoce(slot.cellId)} /> você
                  </label>
                </span>
              </div>
            </div>
          ))}
        </div>

        <div className="team team-enemy">
          <h3>Inimigos</h3>
          {value.theirTeam.map((slot) => (
            <div className="slot" key={slot.cellId}>
              <div className="slot-info">
                <input
                  className="sim-input"
                  list="sim-campeoes"
                  placeholder="campeão"
                  defaultValue={nome(slot.championId)}
                  key={`${slot.cellId}-${slot.championId}`}
                  onBlur={(e) => campeao("theirTeam", slot.cellId, e.target.value)}
                />
                <select
                  className={`slot-role ${slot.position ? "slot-role-marked" : ""}`}
                  value={slot.position}
                  title="Sem rota, o motor presume como na ranqueada"
                  onChange={(e) => trocar("theirTeam", slot.cellId, { position: e.target.value })}
                >
                  <option value="">rota presumida</option>
                  {ROLES.map((r) => (
                    <option key={r.id} value={ROLE_TO_LCU[r.id as Exclude<typeof r.id, "Unknown">]}>
                      {r.label}
                    </option>
                  ))}
                </select>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="bans">
        <div className="ban-row">
          <span className="ban-label">Bans</span>
          <div className="ban-icons">
            {value.myBans.map((id) => (
              <button
                type="button"
                className="ban-icon"
                key={id}
                title={`${nome(id)} — clique para tirar`}
                onClick={() => onChange({ ...value, myBans: value.myBans.filter((b) => b !== id) })}
              >
                {staticData?.champions.get(id) ? (
                  <img src={staticData.champions.get(id)!.iconUrl} alt={nome(id)} />
                ) : (
                  <span>?</span>
                )}
              </button>
            ))}
            <input
              className="sim-input"
              list="sim-campeoes"
              placeholder="banir…"
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  ban(e.currentTarget.value);
                  e.currentTarget.value = "";
                }
              }}
            />
          </div>
        </div>
      </div>
    </>
  );
}
