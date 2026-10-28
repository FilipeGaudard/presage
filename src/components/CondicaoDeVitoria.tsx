import type { WinCondition, WinPoint } from "../lib/types";

interface Props {
  condition: WinCondition;
}

/** De onde cada frase sai, ao lado dela: regra de composição ou conta de
 *  atributos. Nenhuma das duas é winrate, e a diferença tem de estar na tela. */
function Linha({ point }: { point: WinPoint }) {
  return (
    <li>
      <span className={`fonte fonte-${point.source}`}>{point.source}</span>
      {point.text}
    </li>
  );
}

export default function CondicaoDeVitoria({ condition }: Props) {
  const nossos = [...condition.facts, ...condition.edges].filter((p) => p.side === "nosso");
  const deles = [...condition.facts, ...condition.edges].filter((p) => p.side === "deles");

  return (
    <div className="wincon">
      <h3 className="section-h">Condição de vitória</h3>
      {condition.phase && <p className="wincon-fase">{condition.phase}</p>}

      {nossos.length === 0 && deles.length === 0 ? (
        <p className="muted">
          Nada a ler ainda: as composições estão incompletas e nenhum duelo pende
          para um lado.
        </p>
      ) : (
        <div className="wincon-lados">
          <div>
            <h4>Vocês</h4>
            {nossos.length === 0 ? (
              <p className="muted">Nada que se destaque.</p>
            ) : (
              <ul>
                {nossos.map((p) => (
                  <Linha key={p.text} point={p} />
                ))}
              </ul>
            )}
          </div>
          <div>
            <h4>Eles</h4>
            {deles.length === 0 ? (
              <p className="muted">Nada que se destaque.</p>
            ) : (
              <ul>
                {deles.map((p) => (
                  <Linha key={p.text} point={p} />
                ))}
              </ul>
            )}
          </div>
        </div>
      )}

      <p className="faint">{condition.caveat}</p>
    </div>
  );
}
