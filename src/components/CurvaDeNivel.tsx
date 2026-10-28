import type { LevelOdds } from "../lib/types";

interface Props {
  curve: LevelOdds[];
  /** O ponto de agora — na partida, os níveis reais. */
  marker?: { level: number; odds: number; label: string } | null;
}

const W = 320;
const H = 96;
const ESQ = 34;
const DIR = 8;
const TOPO = 8;
const BASE = 20;
const CHAVE = [3, 6, 11, 16];

/** A chance no 1v1 por atributos, do nível 1 ao 18, com os dois nivelados.
 *  A escala acompanha os dados: entre um nível e o outro a conta quase não
 *  muda, e o que vale ler é a forma — quem cresce mais —, não um pico. */
export default function CurvaDeNivel({ curve, marker = null }: Props) {
  if (curve.length === 0) return null;

  const desvio = Math.max(
    0.04,
    ...curve.map((p) => Math.abs(p.odds - 0.5)),
    marker ? Math.abs(marker.odds - 0.5) : 0
  );
  const x = (nivel: number) => ESQ + ((nivel - 1) / 17) * (W - ESQ - DIR);
  const y = (odds: number) => TOPO + ((0.5 + desvio - odds) / (2 * desvio)) * (H - TOPO - BASE);
  const linha = curve.map((p) => `${x(p.level).toFixed(1)},${y(p.odds).toFixed(1)}`).join(" ");
  const tom = (odds: number) => (odds >= 0.53 ? "good" : odds <= 0.47 ? "bad" : "even");

  return (
    <svg className="curva-nivel" viewBox={`0 0 ${W} ${H}`} role="img" aria-label="Chance por nível">
      <line className="curva-meio" x1={ESQ} x2={W - DIR} y1={y(0.5)} y2={y(0.5)} />
      <text className="curva-eixo" x={ESQ - 6} y={y(0.5) + 3} textAnchor="end">
        50%
      </text>
      <text className="curva-eixo" x={ESQ - 6} y={TOPO + 6} textAnchor="end">
        {Math.round((0.5 + desvio) * 100)}%
      </text>
      <text className="curva-eixo" x={ESQ - 6} y={H - BASE} textAnchor="end">
        {Math.round((0.5 - desvio) * 100)}%
      </text>

      {CHAVE.map((nivel) => (
        <g key={nivel}>
          <line className="curva-chave" x1={x(nivel)} x2={x(nivel)} y1={TOPO} y2={H - BASE} />
          <text className="curva-eixo" x={x(nivel)} y={H - 6} textAnchor="middle">
            {nivel}
          </text>
        </g>
      ))}
      <text className="curva-eixo" x={x(1)} y={H - 6} textAnchor="middle">
        1
      </text>
      <text className="curva-eixo" x={x(18)} y={H - 6} textAnchor="middle">
        18
      </text>

      <polyline className="curva-linha" points={linha} />
      {curve
        .filter((p) => CHAVE.includes(p.level))
        .map((p) => (
          <circle key={p.level} className={`curva-ponto ${tom(p.odds)}`} cx={x(p.level)} cy={y(p.odds)} r={3} />
        ))}

      {marker && (
        <g>
          <circle className="curva-agora" cx={x(marker.level)} cy={y(marker.odds)} r={5} />
          <text className="curva-agora-txt" x={x(marker.level)} y={y(marker.odds) - 9} textAnchor="middle">
            {marker.label}
          </text>
        </g>
      )}
    </svg>
  );
}

/** No 6, no 11 e no 16 o nível a mais é a suprema, que a conta não vê. */
const SEM_SUPREMA = [6, 11, 16];

/** Você um nível atrás, nivelado e um à frente, nos níveis que mudam a rota. */
export function NiveisDeVantagem({ gaps }: { gaps: LevelOdds[] }) {
  if (gaps.length === 0) return null;
  const niveis = [...new Set(gaps.map((g) => g.level))];
  const em = (level: number, gap: number) => gaps.find((g) => g.level === level && g.gap === gap);
  const pct = (g?: LevelOdds) => (g ? `${Math.round(g.odds * 100)}%` : "—");
  const tom = (g?: LevelOdds) => (!g ? "" : g.odds >= 0.53 ? "good" : g.odds <= 0.47 ? "bad" : "even");

  return (
    <table className="vantagem-nivel">
      <thead>
        <tr>
          <th>nível dele</th>
          <th>você 1 atrás</th>
          <th>nivelado</th>
          <th>você 1 à frente</th>
        </tr>
      </thead>
      <tbody>
        {niveis.map((level) => (
          <tr key={level}>
            <td>
              {level}
              {SEM_SUPREMA.includes(level) && <small> atributos, sem a suprema</small>}
            </td>
            <td className={tom(em(level, -1))}>{pct(em(level, -1))}</td>
            <td className={tom(em(level, 0))}>{pct(em(level, 0))}</td>
            <td className={tom(em(level, 1))}>{pct(em(level, 1))}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
