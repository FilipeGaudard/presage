import type { CoachNote, Point, Series } from "../lib/types";

interface Props {
  series: Series;
  duration: number;
}

const L = 44; // espaço do eixo à esquerda
const R = 10;
const T = 10;
const B = 22;

function relogio(segundos: number) {
  const s = Math.max(0, Math.round(segundos));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

const BR = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 0 });

/**
 * Curva de uma grandeza que gira em torno do zero.
 *
 * O eixo é simétrico de propósito: 3.000 à frente e 3.000 atrás ocupam a mesma
 * altura, então a forma da curva diz o que aconteceu sem precisar ler número.
 * Verde acima da linha, coral abaixo — um gradiente com corte seco no zero, em
 * vez de dois caminhos recortados.
 */
function Curva({
  pontos,
  mortes,
  rotulo,
  formata,
  altura = 132,
  largura = 560,
}: {
  pontos: Point[];
  mortes?: number[];
  rotulo: string;
  formata: (v: number) => string;
  altura?: number;
  largura?: number;
}) {
  if (pontos.length < 2) return null;

  const t0 = pontos[0].t;
  const t1 = pontos[pontos.length - 1].t;
  const span = Math.max(1, t1 - t0);
  const pico = Math.max(1, ...pontos.map((p) => Math.abs(p.v)));

  const x = (t: number) => L + ((t - t0) / span) * (largura - L - R);
  const y = (v: number) => T + (1 - (v + pico) / (2 * pico)) * (altura - T - B);
  const zero = y(0);

  const linha = pontos.map((p) => `${x(p.t).toFixed(1)},${y(p.v).toFixed(1)}`).join(" ");
  const area = `M${x(t0)},${zero} L${linha.replace(/ /g, " L")} L${x(t1)},${zero} Z`;

  // O corte do gradiente acompanha a altura do zero no gráfico.
  const corte = ((zero - T) / (altura - T - B)) * 100;
  const id = `g-${rotulo.replace(/\W/g, "")}`;

  return (
    <figure className="grafico">
      <figcaption>{rotulo}</figcaption>
      <svg viewBox={`0 0 ${largura} ${altura}`} role="img" aria-label={rotulo}>
        <defs>
          <linearGradient id={id} x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="var(--mint)" stopOpacity="0.32" />
            <stop offset={`${corte}%`} stopColor="var(--mint)" stopOpacity="0.32" />
            <stop offset={`${corte}%`} stopColor="var(--coral)" stopOpacity="0.32" />
            <stop offset="100%" stopColor="var(--coral)" stopOpacity="0.32" />
          </linearGradient>
        </defs>

        <path d={area} fill={`url(#${id})`} />
        <line x1={L} y1={zero} x2={largura - R} y2={zero} className="eixo-zero" />
        <polyline points={linha} className="curva" />

        <text x={L - 7} y={y(pico) + 4} className="marca" textAnchor="end">
          {formata(pico)}
        </text>
        <text x={L - 7} y={zero + 4} className="marca" textAnchor="end">
          0
        </text>
        <text x={L - 7} y={y(-pico) + 4} className="marca" textAnchor="end">
          {formata(-pico)}
        </text>

        {mortes?.map((t, i) => (
          <line
            key={`${t}-${i}`}
            x1={x(t)}
            y1={altura - B + 2}
            x2={x(t)}
            y2={altura - B + 9}
            className="marca-morte"
          >
            <title>Morte aos {relogio(t)}</title>
          </line>
        ))}

        <text x={L} y={altura - 6} className="marca">
          {relogio(t0)}
        </text>
        <text x={largura - R} y={altura - 6} className="marca" textAnchor="end">
          {relogio(t1)}
        </text>
      </svg>
    </figure>
  );
}

export default function GraficoPartida({ series, duration }: Props) {
  if (!series || series.goldLead.length < 2) return null;

  const fim = series.goldLead[series.goldLead.length - 1].v;
  const pico = series.goldLead.reduce((a, b) => (b.v > a.v ? b : a));

  return (
    <div className="graficos">
      <Curva
        pontos={series.goldLead}
        mortes={series.deaths}
        rotulo="Ouro em itens, seu time menos o deles"
        formata={(v) => BR.format(v)}
      />

      <p className="grafico-leitura">
        {pico.v >= 1000 && fim < pico.v - 1500 ? (
          <>
            Pico de <b className="good">{BR.format(pico.v)}</b> aos{" "}
            {relogio(pico.t)}, fim em <b className={fim >= 0 ? "good" : "bad"}>{BR.format(fim)}</b>.
            As marcas embaixo são as suas mortes.
          </>
        ) : (
          <>
            Fechou em <b className={fim >= 0 ? "good" : "bad"}>{BR.format(fim)}</b> depois de{" "}
            {relogio(duration)}. As marcas embaixo são as suas mortes.
          </>
        )}
      </p>

      {series.csDiff.length >= 2 && (
        <Curva
          pontos={series.csDiff}
          rotulo="Tropas, você menos o seu oponente"
          formata={(v) => BR.format(v)}
          altura={104}
        />
      )}

      {series.levelDiff.length >= 2 && (
        <Curva
          pontos={series.levelDiff}
          rotulo="Nível, você menos o seu oponente"
          formata={(v) => v.toFixed(0)}
          altura={92}
        />
      )}

      <p className="faint">
        As três curvas saem da mesma gravação que gerou as observações acima:
        um quadro por segundo, reamostrado a cada 30 para caber na tela. O ouro
        em itens é soma exata dos preços — não é estimativa.
      </p>
    </div>
  );
}

export type { CoachNote };
