import type { StaticData } from "../lib/ddragon";
import type { ChampionLane, LaneMetric, LaneProfile, LaneReading, MatchupLane } from "../lib/types";

interface Props {
  profile: LaneProfile | null;
  staticData: StaticData | null;
}

const METRICAS: { id: LaneMetric; rotulo: string; casas: number; sufixo: string }[] = [
  { id: "cs10", rotulo: "Tropas aos 10", casas: 0, sufixo: "" },
  { id: "killParticipation", rotulo: "Part. em abates", casas: 0, sufixo: "pp" },
  { id: "damageShare", rotulo: "Fatia de dano", casas: 0, sufixo: "pp" },
  { id: "visionPerMinute", rotulo: "Visão/min", casas: 1, sufixo: "" },
  { id: "deaths", rotulo: "Mortes", casas: 1, sufixo: "" },
];

/** O intervalo de 95% não cruza o zero — a mesma regra de `Reading::is_clear`. */
function clara(leitura: LaneReading) {
  return Math.abs(leitura.value) > 1.96 * leitura.sd;
}

/** Só ganha cor o que sai do ruído. Mortes é a única em que menos é melhor. */
function tom(leitura: LaneReading) {
  if (!clara(leitura)) return "even";
  const bom = leitura.value > 0 !== (leitura.metric === "deaths");
  return bom ? "good" : "bad";
}

function numero(valor: number, casas: number) {
  const texto = Math.abs(valor).toFixed(casas);
  if (Number(texto) === 0) return texto;
  return `${valor > 0 ? "+" : "−"}${texto}`;
}

function Face({ id, staticData }: { id: number; staticData: StaticData | null }) {
  const champion = staticData?.champions.get(id);
  return (
    <span className="face sm">
      {champion ? <img src={champion.iconUrl} alt={champion.name} loading="lazy" /> : null}
    </span>
  );
}

function Celula({ leitura, casas, sufixo }: { leitura?: LaneReading; casas: number; sufixo: string }) {
  if (!leitura) return <td className="even">—</td>;
  return (
    <td
      className={tom(leitura)}
      title={`média crua ${numero(leitura.raw, casas)}${sufixo} em ${leitura.games} partidas`}
    >
      <b>
        {numero(leitura.value, casas)}
        {sufixo}
      </b>
      <small>±{(1.96 * leitura.sd).toFixed(casas)}</small>
    </td>
  );
}

/** Um confronto resumido na leitura que mais se destaca do ruído. */
function Confronto({ confronto, staticData }: { confronto: MatchupLane; staticData: StaticData | null }) {
  const z = (r: LaneReading) => Math.abs(r.value) / Math.max(r.sd, 1e-9);
  const destaque = [...confronto.readings].sort((a, b) => z(b) - z(a))[0];
  const metrica = destaque && METRICAS.find((m) => m.id === destaque.metric);
  if (!destaque || !metrica) return null;

  return (
    <span className="perfil-confronto">
      <small>vs</small>
      <Face id={confronto.opponent} staticData={staticData} />
      <b className={tom(destaque)}>
        {numero(destaque.value, metrica.casas)}
        {metrica.sufixo}
      </b>
      {metrica.rotulo.toLowerCase()}
      <small>n={confronto.games}</small>
    </span>
  );
}

function Linha({ campeao, staticData }: { campeao: ChampionLane; staticData: StaticData | null }) {
  const nome = staticData?.champions.get(campeao.champion)?.name ?? `#${campeao.champion}`;
  const leituras = new Map(campeao.readings.map((r) => [r.metric, r]));
  // Uma partida só contra alguém é anedota, mesmo encolhida.
  const confrontos = campeao.matchups.filter((m) => m.games >= 2).slice(0, 4);

  return (
    <>
      <tr>
        <td>
          <span className="perfil-nome">
            <Face id={campeao.champion} staticData={staticData} />
            {nome}
          </span>
        </td>
        <td>{campeao.games}</td>
        {METRICAS.map((m) => (
          <Celula key={m.id} leitura={leituras.get(m.id)} casas={m.casas} sufixo={m.sufixo} />
        ))}
      </tr>
      {confrontos.length > 0 && (
        <tr className="perfil-confrontos">
          <td colSpan={2 + METRICAS.length}>
            {confrontos.map((m) => (
              <Confronto key={m.opponent} confronto={m} staticData={staticData} />
            ))}
          </td>
        </tr>
      )}
    </>
  );
}

export default function PerfilDeRota({ profile, staticData }: Props) {
  if (!profile || profile.overall.length === 0) {
    return (
      <div className="empty">
        <h2>Sem histórico de rota ainda</h2>
        <p>
          Com a chave da Riot definida, o Presage lê suas ranqueadas e compara
          cada partida com o seu oponente direto.
        </p>
      </div>
    );
  }

  const geral = new Map(profile.overall.map((r) => [r.metric, r]));

  return (
    <div className="perfil">
      <div className="panel-head">
        <div>
          <h2>Sua rota contra o oponente direto</h2>
          <span className="muted">
            {profile.games} partidas · você menos quem jogou a mesma rota do outro lado
          </span>
        </div>
      </div>

      <table className="perfil-tabela">
        <thead>
          <tr>
            <th>Campeão</th>
            <th>Partidas</th>
            {METRICAS.map((m) => (
              <th key={m.id}>{m.rotulo}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          <tr className="perfil-geral">
            <td>Você, no geral</td>
            <td>{profile.games}</td>
            {METRICAS.map((m) => (
              <Celula key={m.id} leitura={geral.get(m.id)} casas={m.casas} sufixo={m.sufixo} />
            ))}
          </tr>
          {profile.champions.map((c) => (
            <Linha key={c.champion} campeao={c} staticData={staticData} />
          ))}
        </tbody>
      </table>

      <p className="faint">
        Cada campeão é puxado para a sua média, e cada confronto para o campeão,
        tanto mais quanto menor a amostra. Só ganha cor o que sai do ruído
        (intervalo de 95%). O número mistura você e o campeão: quem perde tropa
        na rota por desenho aparece aqui como se fosse seu — separar os dois
        exige a base de partidas.
      </p>
    </div>
  );
}
