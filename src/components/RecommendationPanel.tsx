import { useEffect, useState } from "react";
import type { StaticData } from "../lib/ddragon";
import CondicaoDeVitoria from "./CondicaoDeVitoria";
import CurvaDeNivel, { NiveisDeVantagem } from "./CurvaDeNivel";
import { SkirmishRow } from "./LaneAnalysis";
import {
  ROLES,
  type BanSuggestion,
  type DraftState,
  type EngineStatus,
  type LaneReport,
  type MinimaxPick,
  type Role,
  type ScoredChampion,
} from "../lib/types";

interface Props {
  recommendations: ScoredChampion[];
  minimax: MinimaxPick[];
  bans: BanSuggestion[];
  staticData: StaticData | null;
  draft: DraftState | null;
  engine: EngineStatus | null;
  /** A rota escolhida na tela, para quando o cliente não informa. */
  role?: Role | null;
  onRole?: (role: Role) => void;
  /** A análise do draft com este campeão no seu lugar. */
  onPreview?: (championId: number) => Promise<LaneReport | null>;
  /** Recomendar também quem a conta não pode escolher. */
  showAll?: boolean;
  onShowAll?: (showAll: boolean) => void;
}

const OVERALL_COUNT = 15;

function deltaClass(delta: number): string {
  if (delta >= 2) return "delta-strong";
  if (delta >= 1) return "delta-ok";
  return "delta-weak";
}

export default function RecommendationPanel({
  recommendations,
  minimax,
  bans,
  staticData,
  draft,
  engine,
  role = null,
  onRole,
  onPreview,
  showAll,
  onShowAll,
}: Props) {
  const [selected, setSelected] = useState<number | null>(null);
  const [previa, setPrevia] = useState<LaneReport | null>(null);
  const [calculando, setCalculando] = useState(false);

  // A prévia se refaz quando o draft muda: um inimigo novo travado muda a rota
  // e as escaramuças do candidato.
  useEffect(() => {
    if (selected === null || !onPreview) {
      setPrevia(null);
      return;
    }
    let vivo = true;
    setCalculando(true);
    onPreview(selected)
      .then((r) => vivo && setPrevia(r))
      .catch(() => vivo && setPrevia(null))
      .finally(() => vivo && setCalculando(false));
    return () => {
      vivo = false;
    };
  }, [selected, draft, onPreview]);

  // Candidato que saiu da lista (banido, escolhido por outro) sai da seleção.
  useEffect(() => {
    if (
      selected !== null &&
      !recommendations.some((r) => r.championId === selected) &&
      !minimax.some((m) => m.championId === selected)
    ) {
      setSelected(null);
    }
  }, [recommendations, minimax, selected]);

  if (!draft) {
    return (
      <div className="empty">
        <h2>Recomendações</h2>
        <p>O ranking aparece aqui durante a seleção de campeões.</p>
      </div>
    );
  }

  const banning = draft.currentAction === "ban";
  const pool = recommendations.filter((item) => item.inPool);
  const overall = recommendations.filter((item) => !item.inPool).slice(0, OVERALL_COUNT);
  const detail = recommendations.find((item) => item.championId === selected) ?? null;
  const tree = minimax.find((pick) => pick.championId === selected) ?? null;
  const safest = [...minimax].sort((a, b) => a.exposure - b.exposure).slice(0, 5);

  return (
    <>
      <div className="reco-head">
        <div>
          <h2>Recomendações</h2>
          <span className="muted">
            {draft.currentAction === "ban" ? "melhores bans" : "melhores picks"} ·{" "}
            {recommendations.length} candidatos
          </span>
        </div>
        {onShowAll && (
          <label className="reco-toggle" title="Desligado, a lista só traz quem a sua conta pode escolher nesta fila, quando o cliente informa">
            <input
              type="checkbox"
              checked={!!showAll}
              onChange={(e) => onShowAll(e.target.checked)}
            />
            incluir quem você não tem
          </label>
        )}
      </div>

      {/* Na ranqueada o cliente diz a rota. Em blind e normal não diz, e sem
          rota a lista viraria o jogo inteiro. */}
      {!draft.localPosition && onRole && (
        <div className="reco-role">
          <span className="muted">Sua rota:</span>
          <div className="role-tabs">
            {ROLES.map((item) => (
              <button
                type="button"
                key={item.id}
                className={role === item.id ? "on" : ""}
                onClick={() => onRole(item.id)}
              >
                {item.label}
              </button>
            ))}
          </div>
          {!role && <span className="faint">sem rota, a lista inclui todos</span>}
        </div>
      )}

      {!engine?.ready && (
        <div className="banner banner-warn">Carregando dados de campeões…</div>
      )}

      {engine?.ready && !engine.hasGlobal && (
        <div className="banner banner-mock">
          Sem base estatística: o ranking sai da composição dos times
          {engine.hasPersonal
            ? ` e do seu histórico (${engine.personalGames} partidas${
                engine.account ? `, ${engine.account}` : ""
              })`
            : ""}
          . Winrate base, matchup e sinergia precisam do pipeline de partidas —
          nenhum dos três sai de uma conta só.
        </div>
      )}

      {/* Base de patch velho recomenda item que mudou de preço. Vale avisar:
          o número continua saindo de partidas reais, só que de outro jogo. */}
      {engine?.ready && engine.hasGlobal && engine.globalPatch && (
        <p
          className={
            engine.version.startsWith(engine.globalPatch) ? "faint" : "warn-line"
          }
        >
          {engine.version.startsWith(engine.globalPatch)
            ? `Base do patch ${engine.globalPatch}, ${engine.globalMatches.toLocaleString(
                "pt-BR"
              )} partidas.`
            : `A base é do patch ${engine.globalPatch} e o jogo está no ${engine.version}. Os números são de outro patch.`}
        </p>
      )}

      {banning && (
        <div className="section">
          <h3>Bans</h3>
          {bans.length === 0 ? (
            <p className="muted">
              Nada travado do lado deles ainda e nenhum ban na sua lista (aba
              Pool). Sem pick e sem base estatística não há o que ranquear — e
              ordenar ruído seria pior que não dizer nada.
            </p>
          ) : (
            <div className="grid">
              {bans.slice(0, 10).map((ban) => {
                const champion = staticData?.champions.get(ban.championId);
                return (
                  <div
                    className={`cell cell-static ${ban.fromPool ? "cell-pool" : ""}`}
                    key={ban.championId}
                    title={ban.reason}
                  >
                    <div className="cell-portrait">
                      {champion ? (
                        <img src={champion.iconUrl} alt={champion.name} loading="lazy" />
                      ) : (
                        <span>?</span>
                      )}
                    </div>
                    <span className="cell-name">{champion?.name ?? `#${ban.championId}`}</span>
                    {ban.fromPool && <span className="cell-tag">sua lista</span>}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}

      {!banning && safest.length > 0 && (
        <div className="section">
          <h3>Menos expostos ao contra-pick</h3>
          <div className="grid">
            {safest.map((pick) => {
              const champion = staticData?.champions.get(pick.championId);
              return (
                <button
                  type="button"
                  key={pick.championId}
                  className={`cell ${selected === pick.championId ? "cell-on" : ""}`}
                  onClick={() =>
                    setSelected(selected === pick.championId ? null : pick.championId)
                  }
                  title={champion?.name}
                >
                  {pick.inPool && <span className="cell-mark">✳</span>}
                  <div className="cell-portrait">
                    {champion ? (
                      <img src={champion.iconUrl} alt={champion.name} loading="lazy" />
                    ) : (
                      <span>?</span>
                    )}
                  </div>
                  <span className="cell-delta delta-ok">
                    −{(pick.exposure * 100).toFixed(1)}
                  </span>
                </button>
              );
            })}
          </div>
        </div>
      )}

      <Section
        title="Sua pool"
        empty="Nenhum campeão da sua pool está disponível. Monte uma no Construtor de Pool."
        items={pool}
        staticData={staticData}
        selected={selected}
        onSelect={setSelected}
        marked
      />

      <Section
        title="Melhores no geral"
        empty="Sem candidatos."
        items={overall}
        staticData={staticData}
        selected={selected}
        onSelect={setSelected}
      />

      {detail && (
        <div className="detail">
          <div className="detail-head">
            <strong>
              {staticData?.champions.get(detail.championId)?.name ?? `#${detail.championId}`}
            </strong>
            <span className="muted">
              {detail.score.toFixed(1)}% ±{detail.confidence.toFixed(1)}
              {detail.sample > 0 && ` · n=${detail.sample}`}
            </span>
          </div>
          <ul className="detail-reasons">
            {detail.reasons.map((reason) => (
              <li key={reason}>{reason}</li>
            ))}
            {tree && (
              <li>
                Melhor resposta deles custa {(tree.exposure * 100).toFixed(1)}pp
                {tree.bestResponse !== null &&
                  ` (${
                    staticData?.champions.get(tree.bestResponse)?.name ??
                    `#${tree.bestResponse}`
                  })`}
              </li>
            )}
          </ul>
          {onPreview && (
            <Previa
              previa={previa}
              calculando={calculando}
              champion={detail.championId}
              staticData={staticData}
            />
          )}
        </div>
      )}
    </>
  );
}

/** A rota, as escaramuças e a condição de vitória com o candidato no seu lugar —
 *  a mesma conta de quando ele estiver travado. */
function Previa({
  previa,
  calculando,
  champion,
  staticData,
}: {
  previa: LaneReport | null;
  calculando: boolean;
  champion: number;
  staticData: StaticData | null;
}) {
  if (calculando && !previa) return <p className="muted">Calculando a prévia…</p>;
  if (!previa) {
    return (
      <p className="muted previa-vazia">
        A prévia de rota aparece quando o outro time travar alguém.
      </p>
    );
  }

  const rota = previa.lanes.find((lane) => lane.ally === champion) ?? null;
  const brigas = previa.skirmishes.filter((s) => s.ours.includes(champion));

  return (
    <div className="previa">
      {rota ? (
        <>
          <h3 className="section-h">
            Rota contra {rota.enemyName}: o 1v1 de atributos, nível a nível
          </h3>
          <CurvaDeNivel curve={rota.curve} />
          <NiveisDeVantagem gaps={rota.gaps} />
        </>
      ) : (
        <p className="muted">O oponente da sua rota ainda não travou.</p>
      )}

      {brigas.length > 0 && (
        <>
          <h3 className="section-h">2v2 e 3v3 com ele</h3>
          <div className="fight-list">
            {brigas.map((fight) => (
              <SkirmishRow key={fight.label} fight={fight} staticData={staticData} />
            ))}
          </div>
        </>
      )}

      {previa.team && (
        <div className="time-chance">
          <h3 className="section-h">Chance do time com ele, pela base de partidas</h3>
          <p>
            <b className={previa.team.win >= 0.52 ? "good" : previa.team.win <= 0.48 ? "bad" : "even"}>
              {Math.round(previa.team.win * 100)}%
            </b>{" "}
            <span className="muted">±{previa.team.confidence.toFixed(1)}</span>
          </p>
          {previa.team.terms.length > 0 && (
            <ul className="detail-reasons">
              {previa.team.terms.map((t) => (
                <li key={t.label}>
                  {t.label}: {t.shift > 0 ? "+" : ""}
                  {t.shift.toFixed(1)}pp <span className="muted">(n={t.games})</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

      <CondicaoDeVitoria condition={previa.winCondition} />

      {previa.rolesAssumed && (
        <p className="faint">
          As rotas deles foram inferidas pelo que cada campeão costuma jogar.
        </p>
      )}
    </div>
  );
}

function Section({
  title,
  empty,
  items,
  staticData,
  selected,
  onSelect,
  marked = false,
}: {
  title: string;
  empty: string;
  items: ScoredChampion[];
  staticData: StaticData | null;
  selected: number | null;
  onSelect: (id: number | null) => void;
  marked?: boolean;
}) {
  return (
    <div className="section">
      <h3>{title}</h3>
      {items.length === 0 ? (
        <p className="muted">{empty}</p>
      ) : (
        <div className="grid">
          {items.map((item) => {
            const champion = staticData?.champions.get(item.championId);
            const isSelected = selected === item.championId;
            return (
              <button
                type="button"
                key={item.championId}
                className={`cell ${isSelected ? "cell-on" : ""}`}
                onClick={() => onSelect(isSelected ? null : item.championId)}
                title={champion?.name ?? `#${item.championId}`}
              >
                {marked && <span className="cell-mark">✳</span>}
                <div className="cell-portrait">
                  {champion ? (
                    <img src={champion.iconUrl} alt={champion.name} loading="lazy" />
                  ) : (
                    <span>?</span>
                  )}
                </div>
                <span className={`cell-delta ${deltaClass(item.delta)}`}>
                  {item.delta.toFixed(1)}
                </span>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
