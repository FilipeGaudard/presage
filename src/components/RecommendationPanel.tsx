import { useState } from "react";
import type { StaticData } from "../lib/ddragon";
import type {
  BanSuggestion,
  DraftState,
  EngineStatus,
  MinimaxPick,
  ScoredChampion,
} from "../lib/types";

interface Props {
  recommendations: ScoredChampion[];
  minimax: MinimaxPick[];
  bans: BanSuggestion[];
  staticData: StaticData | null;
  draft: DraftState | null;
  engine: EngineStatus | null;
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
}: Props) {
  const [selected, setSelected] = useState<number | null>(null);

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
      </div>

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
              Nada travado do lado deles ainda. Sem pick e sem base estatística
              não há o que ranquear — e ordenar ruído seria pior que não dizer
              nada.
            </p>
          ) : (
            <div className="grid">
              {bans.slice(0, 10).map((ban) => {
                const champion = staticData?.champions.get(ban.championId);
                return (
                  <div className="cell cell-static" key={ban.championId} title={ban.reason}>
                    <div className="cell-portrait">
                      {champion ? (
                        <img src={champion.iconUrl} alt={champion.name} loading="lazy" />
                      ) : (
                        <span>?</span>
                      )}
                    </div>
                    <span className="cell-name">{champion?.name ?? `#${ban.championId}`}</span>
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
        </div>
      )}
    </>
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
