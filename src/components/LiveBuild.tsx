import { itemIcon, type StaticData } from "../lib/ddragon";
import type { BoughtReport, BuildContext, ItemSuggestion, LiveGame } from "../lib/types";

const BR = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 0 });
const PCT = new Intl.NumberFormat("pt-BR", { style: "percent", maximumFractionDigits: 1 });
const ROTA: Record<string, string> = { Top: "topo", Jungle: "selva", Mid: "meio", Bot: "atirador", Support: "suporte" };

interface Props {
  game: LiveGame;
  context: BuildContext | null;
  suggestions: ItemSuggestion[];
  staticData: StaticData | null;
  /** Por que os itens não vieram (`EngineStatus.itemsError`). */
  itemsError?: string | null;
  /** O que se compra com o seu campeão, segundo a base (`bought_items`). */
  bought?: BoughtReport | null;
  /** O patch da base (`EngineStatus.globalPatch`). */
  patch?: string | null;
}

export default function LiveBuild({ game, context, suggestions, staticData, itemsError = null, bought = null, patch = null }: Props) {
  if (!context) {
    // `build_suggestions` só volta nulo sem partida ou sem o jogador local (A32).
    return (
      <div className="empty">
        <h2>Build adaptativa</h2>
        <p>A build precisa do seu campeão, e a partida ainda não disse qual é (espectador, ou o jogo acabou de abrir).</p>
      </div>
    );
  }

  const me = game.players.find((player) => player.isLocal);
  const theirs = game.players.filter((p) => me && p.team !== me.team);
  const physical = Math.round(context.incomingPhysicalShare * 100);

  return (
    <>
      <div className="reco-head">
        <div>
          <h2>Build adaptativa</h2>
          <span className="muted">reage aos itens que eles têm agora</span>
        </div>
      </div>

      {itemsError && (
        <div className="banner banner-warn">
          Não consegui os dados de item: {itemsError}. A build fica sem sugestão até o Presage abrir de novo com
          internet.
        </div>
      )}

      {theirs.length === 0 ? (
        <div className="section">
          <h3>Dano que vem contra você</h3>
          {/* O treino: sem outro lado, a divisão do dano e a vida efetiva contra ele não existem. */}
          <p className="muted">Sem inimigos na partida: nada vem contra você ainda.</p>
        </div>
      ) : (
      <div className="section">
        <h3>Dano que vem contra você</h3>
        <div className="split">
          <div className="split-bar">
            <div className="split-phys" style={{ width: `${physical}%` }} />
          </div>
          <div className="split-legend">
            <span className="split-key">{physical}% físico</span>
            <span className="muted">{100 - physical}% mágico</span>
          </div>
        </div>

        <div className="facts">
          {/* O ouro mora no rastreador ao lado, com os dez jogadores. */}
          <Fact label="Sua vida efetiva" value={Math.round(context.yourEffectiveHp).toString()} />
          {context.enemyArmorPenPercent > 0 && (
            <Fact
              label="Pen. de armadura deles"
              value={`${Math.round(context.enemyArmorPenPercent * 100)}%`}
              bad
            />
          )}
          {context.enemyLethality > 0 && (
            <Fact label="Letalidade deles" value={Math.round(context.enemyLethality).toString()} bad />
          )}
        </div>
      </div>
      )}

      {bought && bought.items.length > 0 && (
        <div className="section">
          <h3>O que se compra com {me?.championKey ?? "o seu campeão"} no {ROTA[bought.role] ?? bought.role}</h3>
          <p className="muted">
            Base de partidas{patch ? ` do patch ${patch}` : ""}: {BR.format(bought.games)} partidas dele na rota,
            vence {PCT.format(bought.winrate)}. Do mais comprado ao menos; a vitória é de quem terminou com o item, e
            quem ganha termina com mais itens, então serve para comparar um com outro.
          </p>
          <ol className="build-list">
            {bought.items.map((item, index) => (
              <li className="build-item" key={item.itemId}>
                <span className="reco-rank">{index + 1}</span>
                <div className="cell-portrait build-portrait">
                  {staticData ? (
                    <img src={itemIcon(staticData.version, item.itemId)} alt={item.name} loading="lazy" />
                  ) : (
                    <span>?</span>
                  )}
                </div>
                <div className="reco-body">
                  <div className="reco-title">
                    <span className="reco-name">{item.name}</span>
                    <span className="muted">{Math.round(item.cost)} ouro</span>
                  </div>
                </div>
                <div className="reco-score">
                  <span className="score-value">{Math.round(item.share * 100)}% compram</span>
                  <span className="score-conf">
                    vence {PCT.format(item.winrate)} · n={BR.format(item.games)}
                  </span>
                </div>
              </li>
            ))}
          </ol>
        </div>
      )}

      <div className="section">
        <h3>Próximo item, pela conta de atributos</h3>
        <p className="muted">
          {bought && bought.items.length > 0
            ? "A conta de atributos: vida efetiva e dano contra o que eles têm agora, por ouro. Não usa a base."
            : "Sem base de partidas para este campeão: a lista é a conta de atributos (vida efetiva e dano contra o que eles têm agora, por ouro), não o que os jogadores compram."}
        </p>
        {suggestions.length === 0 ? (
          <p className="muted">Sem dados de item carregados.</p>
        ) : (
          <ol className="build-list">
            {suggestions.map((item, index) => (
              <li className="build-item" key={item.itemId}>
                <span className="reco-rank">{index + 1}</span>
                <div className="cell-portrait build-portrait">
                  {staticData ? (
                    <img src={itemIcon(staticData.version, item.itemId)} alt={item.name} loading="lazy" />
                  ) : (
                    <span>?</span>
                  )}
                </div>
                <div className="reco-body">
                  <div className="reco-title">
                    <span className="reco-name">{item.name}</span>
                    <span className="muted">{Math.round(item.cost)} ouro</span>
                  </div>
                  <ul className="detail-reasons">
                    {item.reasons.map((reason) => (
                      <li key={reason}>{reason}</li>
                    ))}
                  </ul>
                </div>
                <div className="reco-score">
                  <span className="score-value">{item.valuePer1000g.toFixed(1)}</span>
                  <span className="score-conf">poder por 1000g</span>
                </div>
              </li>
            ))}
          </ol>
        )}
      </div>

      <div className="section">
        <h3>Inimigos</h3>
        <div className="enemy-list">
          {theirs.map((player) => (
            <div className="enemy-row" key={player.displayName}>
              <span className="enemy-name">
                {player.championKey}
                <span className="muted"> nv {player.level}</span>
              </span>
              <div className="enemy-items">
                {player.items.length === 0 && <span className="muted">sem itens</span>}
                {player.items.map((id, slot) => (
                  <div className="ban-icon" key={`${id}-${slot}`}>
                    {staticData && <img src={itemIcon(staticData.version, id)} alt="" />}
                  </div>
                ))}
              </div>
              {player.isDead && (
                <span className="delta-weak">morto {Math.ceil(player.respawnTimer)}s</span>
              )}
            </div>
          ))}
        </div>
      </div>
    </>
  );
}

function Fact({
  label,
  value,
  good,
  bad,
}: {
  label: string;
  value: string;
  good?: boolean;
  bad?: boolean;
}) {
  return (
    <div className="fact">
      <span className="fact-label">{label}</span>
      <span className={`fact-value ${good ? "delta-strong" : ""} ${bad ? "delta-weak" : ""}`}>
        {value}
      </span>
    </div>
  );
}
