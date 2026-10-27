import type { StaticData } from "../lib/ddragon";
import type { GoldReport } from "../lib/types";
import { POSITION_LABEL } from "../lib/types";

interface Props {
  report: GoldReport | null;
  staticData: StaticData | null;
  /** Versão curta, para a sobreposição por cima do jogo. */
  compact?: boolean;
}

const BR = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 0 });

export default function GoldTracker({ report, staticData, compact = false }: Props) {
  if (!report) return null;

  const lead = report.investedLead;
  const ahead = lead >= 0;

  const lado = (ally: boolean) =>
    report.players
      .filter((player) => player.isAlly === ally)
      .sort((a, b) => b.invested - a.invested);

  return (
    <div className={`gold ${compact ? "gold-compact" : ""}`}>
      <div className="gold-head">
        <span className="label">Ouro em itens</span>
        <b className={ahead ? "good" : "bad"}>
          {ahead ? "+" : "−"}
          {BR.format(Math.abs(lead))}
        </b>
      </div>

      <div className="gold-bar">
        <span
          className="ours"
          style={{
            width: `${
              (report.oursInvested / Math.max(1, report.oursInvested + report.theirsInvested)) * 100
            }%`,
          }}
        />
      </div>
      <div className="gold-legend">
        <span className="good">{BR.format(report.oursInvested)}</span>
        <span className="bad">{BR.format(report.theirsInvested)}</span>
      </div>

      {!compact && (
        <div className="gold-grid">
          {[true, false].map((ally) => (
            <div key={String(ally)}>
              <p className="label">{ally ? "Seu time" : "Inimigos"}</p>
              {lado(ally).map((player) => {
                const champion = staticData?.champions.get(player.championId);
                return (
                  <div
                    className={`gold-row ${player.isLocal ? "mine" : ""}`}
                    key={player.displayName + player.championKey}
                  >
                    <span className="face sm">
                      {champion && <img src={champion.iconUrl} alt={champion.name} />}
                    </span>
                    <span className="gold-name">
                      {champion?.name ?? player.championKey}
                      <small>{POSITION_LABEL[player.position.toLowerCase()] ?? "—"}</small>
                    </span>
                    <span className="gold-num">{BR.format(player.invested)}</span>
                    {player.unspent >= 800 && (
                      <span className="gold-idle" title="Estimativa do ouro em mão">
                        +{BR.format(player.unspent)}
                      </span>
                    )}
                  </div>
                );
              })}
            </div>
          ))}
        </div>
      )}

      {report.notes.length > 0 && (
        <ul className="gold-notes">
          {report.notes.map((note) => (
            <li key={note}>{note}</li>
          ))}
        </ul>
      )}

      {!compact && (
        <p className="faint">
          A coluna em negrito é exata: sai do preço de cada item no inventário. O
          valor claro ao lado é estimativa do ouro em mão — a API só entrega o
          seu, então o dos outros nove é reconstruído a partir de tempo, tropas e
          abates.
        </p>
      )}
    </div>
  );
}
