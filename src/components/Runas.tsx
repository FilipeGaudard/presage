import type { RunePlan } from "../lib/types";

interface Props {
  plan: RunePlan | null;
  /** Versão do Data Dragon, para montar a URL do ícone. */
  patch: string | null;
}

const CDN = "https://ddragon.leagueoflegends.com/cdn/img/";

export default function Runas({ plan, patch }: Props) {
  if (!plan || plan.keystones.length === 0) return null;

  const principal = plan.keystones[0];
  const alternativas = plan.keystones.slice(1);

  return (
    <div className="runas">
      <div className="panel-head">
        <div>
          <h3 className="section-h">Runa principal · {plan.matchup}</h3>
        </div>
      </div>

      <div className="runa-destaque">
        <img src={CDN + principal.icon} alt="" className="runa-icone" />
        <div>
          <b>{principal.name}</b>
          <small>{principal.treeName}</small>
          <ul className="runa-porque">
            {principal.reasons.map((r) => (
              <li key={r}>{r}</li>
            ))}
          </ul>
        </div>
      </div>

      {alternativas.length > 0 && (
        <div className="runa-alt">
          <p className="label">Também servem</p>
          {alternativas.map((r) => (
            <div className="runa-linha" key={r.key}>
              <img src={CDN + r.icon} alt="" />
              <span>
                {r.name}
                <small>{r.reasons[0]}</small>
              </span>
            </div>
          ))}
        </div>
      )}

      {plan.secondary && (
        <div className="runa-alt">
          <p className="label">Árvore secundária</p>
          <div className="runa-linha">
            <img src={CDN + plan.secondary.icon} alt="" />
            <span>
              {plan.secondary.name}
              <small>{plan.secondary.reasons[0]}</small>
            </span>
          </div>
        </div>
      )}

      {/* A distinção que mais importa nesta tela. */}
      <p className="faint">
        {plan.fromData
          ? `Base de partidas do patch ${patch ?? "atual"}.`
          : "Isto sai de regras sobre a mecânica do confronto — quem alcança quem, quem aguenta troca longa, quem morre de explosão. Não sai de winrate: essa base ainda não existe. Uma regra acerta o caso comum e erra o campeão excêntrico."}
      </p>
    </div>
  );
}
