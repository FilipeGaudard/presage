import { useEffect, useMemo, useState } from "react";
import type { StaticData } from "../lib/ddragon";
import {
  CORE_SIZE,
  POOL_SIZE,
  ROLES,
  TIERS,
  type Pool,
  type PoolPlan,
  type Role,
  type Tier,
} from "../lib/types";

interface Props {
  pool: Pool;
  staticData: StaticData | null;
  onChange: (pool: Pool) => void;
  /** O retrato dos core e os candidatos do próximo lugar. Na demonstração vem
   *  uma lista fixa. */
  onSuggest?: (role: Role | null) => Promise<PoolPlan>;
}

type Passo = "rota" | "core" | "completar" | "revisao";

/** A ordem em que a pool aparece na trilha: core, depois os que cobrem. */
const ORDEM: Record<Tier, number> = { core: 0, situational: 1, extra: 2 };

export default function PoolBuilder({ pool, staticData, onChange, onSuggest }: Props) {
  const inicial = pool.entries[0]?.role ?? null;
  const [role, setRole] = useState<Role | null>(inicial);
  const [passo, setPasso] = useState<Passo>(inicial ? "revisao" : "rota");
  const [search, setSearch] = useState("");
  const [plano, setPlano] = useState<PoolPlan | null>(null);

  // Cada lugar é escolhido pelo que os anteriores ainda não cobrem, então o
  // plano se refaz a cada campeão que entra ou sai.
  useEffect(() => {
    if (passo !== "completar" || !onSuggest) return;
    let vivo = true;
    setPlano(null);
    onSuggest(role)
      .then((resposta) => vivo && setPlano(resposta))
      .catch(() => vivo && setPlano({ analysis: { strengths: [], weaknesses: [] }, candidates: [] }));
    return () => {
      vivo = false;
    };
  }, [passo, role, onSuggest, pool]);

  const entries = pool.entries
    .filter((entry) => entry.role === role)
    .sort((a, b) => ORDEM[a.tier] - ORDEM[b.tier]);
  const chosen = new Set(entries.map((entry) => entry.championId));
  const core = entries.filter((entry) => entry.tier === "core");
  const rotulo = ROLES.find((item) => item.id === role)?.label ?? "";

  const catalog = useMemo(() => {
    if (!staticData) return [];
    const term = search.trim().toLowerCase();
    return [...staticData.champions.values()]
      .filter((champion) => !term || champion.name.toLowerCase().includes(term))
      .sort((a, b) => a.name.localeCompare(b.name));
  }, [staticData, search]);

  function add(championId: number, tier: Tier) {
    if (chosen.has(championId)) return;
    onChange({ entries: [...pool.entries, { championId, role, tier }] });
  }

  function retier(championId: number, tier: Tier) {
    onChange({
      entries: pool.entries.map((entry) =>
        entry.championId === championId && entry.role === role ? { ...entry, tier } : entry
      ),
    });
  }

  function remove(championId: number) {
    onChange({
      entries: pool.entries.filter(
        (entry) => !(entry.championId === championId && entry.role === role)
      ),
    });
  }

  function escolherRota(proxima: Role) {
    setRole(proxima);
    setSearch("");
    const jaTem = pool.entries.some((entry) => entry.role === proxima);
    setPasso(jaTem ? "revisao" : "core");
  }

  // ---- passo 1: a rota ----

  if (passo === "rota") {
    return (
      <div className="builder">
        <div className="wizard-head">
          <h2>Em que rota você joga?</h2>
          <p className="muted">
            A pool é por rota. Dá para montar as outras depois.
          </p>
        </div>
        <div className="role-cards">
          {ROLES.map((item) => {
            const quantos = pool.entries.filter((entry) => entry.role === item.id).length;
            return (
              <button type="button" key={item.id} onClick={() => escolherRota(item.id)}>
                <b>{item.label}</b>
                <small>{quantos > 0 ? `${quantos} escolhidos` : "vazia"}</small>
              </button>
            );
          })}
        </div>
      </div>
    );
  }

  // ---- passo 2: os três que você escolhe ----

  if (passo === "core") {
    const feitos = core.length;
    const cheio = feitos >= CORE_SIZE;

    return (
      <div className="builder">
        <div className="wizard-head">
          <p className="eyebrow">{rotulo} · passo 2 de 3</p>
          <h2>Escolha seus {CORE_SIZE} campeões</h2>
          <p className="muted">
            Os que você mais joga. A pool vai ser montada em volta deles.
          </p>
        </div>

        <div className="wizard-track">
          {Array.from({ length: CORE_SIZE }, (_, i) => {
            const entry = core[i];
            const champion = entry && staticData?.champions.get(entry.championId);
            return (
              <div className={`wizard-slot ${entry ? "on" : ""}`} key={i}>
                {champion ? (
                  <>
                    <img src={champion.iconUrl} alt={champion.name} />
                    <button
                      type="button"
                      className="remove"
                      onClick={() => remove(entry.championId)}
                      title={`Tirar ${champion.name}`}
                    >
                      ×
                    </button>
                  </>
                ) : (
                  <span>{i + 1}</span>
                )}
              </div>
            );
          })}
          <span className="counter">
            {feitos} de {CORE_SIZE}
          </span>
        </div>

        <input
          className="search"
          type="search"
          placeholder="Buscar campeão…"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          autoFocus
        />

        <div className="grid grid-catalog">
          {catalog.map((champion) => {
            const pego = chosen.has(champion.id);
            return (
              <button
                type="button"
                key={champion.id}
                className={`cell ${pego ? "cell-taken" : ""}`}
                onClick={() => add(champion.id, "core")}
                title={cheio && !pego ? `Já escolheu ${CORE_SIZE}` : champion.name}
                disabled={pego || cheio}
              >
                <div className="cell-portrait">
                  <img src={champion.iconUrl} alt={champion.name} loading="lazy" />
                </div>
                <span className="cell-name">{champion.name}</span>
              </button>
            );
          })}
        </div>

        <div className="wizard-nav">
          <button
            type="button"
            className="ghost"
            onClick={() => {
              setSearch("");
              setPasso("rota");
            }}
          >
            Voltar
          </button>
          {/* A análise sai dos três: com menos, o retrato seria de outra pool. */}
          <button
            type="button"
            className="primary"
            disabled={!cheio}
            onClick={() => {
              setSearch("");
              setPasso("completar");
            }}
          >
            {cheio ? "Analisar os três" : `Faltam ${CORE_SIZE - feitos}`}
          </button>
        </div>
      </div>
    );
  }

  // ---- passo 3: os cinco que cobrem o que os três não cobrem ----

  if (passo === "completar") {
    const total = entries.length;
    const completa = total >= POOL_SIZE;
    const candidatos = plano?.candidates.filter((c) => !chosen.has(c.championId)) ?? null;

    return (
      <div className="builder">
        <div className="wizard-head">
          <p className="eyebrow">{rotulo} · passo 3 de 3</p>
          <h2>
            {completa ? "Pool completa" : `Completar a pool: lugar ${total + 1} de ${POOL_SIZE}`}
          </h2>
          <p className="muted">
            Cada lugar sai do que os anteriores ainda não cobrem, e a lista se refaz
            a cada escolha.
          </p>
        </div>

        <div className="wizard-track">
          {Array.from({ length: POOL_SIZE }, (_, i) => {
            const entry = entries[i];
            const champion = entry && staticData?.champions.get(entry.championId);
            return (
              <div key={i} className="wizard-slot-wrap">
                {i === CORE_SIZE && <span className="wizard-sep" />}
                <div
                  className={`wizard-slot ${entry ? "on" : ""} ${entry?.tier === "core" ? "core" : ""}`}
                  title={entry?.tier === "core" ? "Escolhido por você" : undefined}
                >
                  {champion ? (
                    <>
                      <img src={champion.iconUrl} alt={champion.name} />
                      {entry.tier !== "core" && (
                        <button
                          type="button"
                          className="remove"
                          onClick={() => remove(entry.championId)}
                          title={`Tirar ${champion.name}`}
                        >
                          ×
                        </button>
                      )}
                    </>
                  ) : (
                    <span>{i + 1}</span>
                  )}
                </div>
              </div>
            );
          })}
          <span className="counter">
            {Math.min(total, POOL_SIZE)} de {POOL_SIZE}
          </span>
        </div>

        {plano && (plano.analysis.strengths.length > 0 || plano.analysis.weaknesses.length > 0) && (
          <div className="notes retrato">
            <div className="notes-col">
              <h3 className="section-h">Os seus três: pontos fortes</h3>
              {plano.analysis.strengths.length === 0 ? (
                <p className="muted">Nada que a rota valorize.</p>
              ) : (
                plano.analysis.strengths.map((f) => (
                  <div className="note-card good" key={f.text}>
                    <p>{f.text}</p>
                  </div>
                ))
              )}
            </div>
            <div className="notes-col">
              <h3 className="section-h">Pontos fracos</h3>
              {plano.analysis.weaknesses.length === 0 ? (
                <p className="muted">Nenhum buraco no que a rota cobra.</p>
              ) : (
                plano.analysis.weaknesses.map((f) => (
                  <div className="note-card bad" key={f.text}>
                    <p>{f.text}</p>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {completa ? (
          <p className="muted">Oito na pool. Dá para trocar qualquer um na revisão.</p>
        ) : candidatos === null ? (
          <p className="muted">Calculando…</p>
        ) : candidatos.length === 0 ? (
          <p className="muted">Nada a recomendar para este lugar.</p>
        ) : (
          <>
            <h3 className="section-h">Para o lugar {total + 1}</h3>
            <div className="grid grid-catalog grid-sugestao">
              {candidatos.map((c) => {
                const champion = staticData?.champions.get(c.championId);
                return (
                  <button
                    type="button"
                    key={c.championId}
                    className="cell cell-sugestao"
                    onClick={() => add(c.championId, "situational")}
                    title={c.reasons.join(" · ")}
                  >
                    <div className="cell-portrait">
                      {champion && (
                        <img src={champion.iconUrl} alt={champion.name} loading="lazy" />
                      )}
                    </div>
                    <span className="cell-name">{champion?.name ?? c.name}</span>
                    <span className="cell-why">{c.reasons[0]}</span>
                  </button>
                );
              })}
            </div>
          </>
        )}

        {/* O número por trás da ordem não é winrate, e dizer isso é mais
            importante que o número. */}
        <p className="faint">
          Esta lista sai da composição da sua pool — tipo de dano, linha de frente,
          alcance, estilo — e da matemática de resistências do jogo. O que conta em
          cada rota são pesos escritos à mão: regra, não estatística. Contra quem os
          seus três perdem só a base de partidas vai dizer, e ela ainda não existe.
        </p>

        <div className="wizard-nav">
          <button type="button" className="ghost" onClick={() => setPasso("core")}>
            Voltar
          </button>
          <button type="button" className="primary" onClick={() => setPasso("revisao")}>
            {completa ? "Ver a pool" : `Fechar com ${total}`}
          </button>
        </div>
      </div>
    );
  }

  // ---- revisão, sem limite ----

  return (
    <div className="builder">
      <div className="builder-head">
        <div>
          <h2>Sua pool</h2>
          <span className="muted">Aqui não há limite. Ajuste o que quiser.</span>
        </div>
        <div className="builder-actions">
          {core.length > 0 && entries.length < POOL_SIZE && (
            <button type="button" className="ghost" onClick={() => setPasso("completar")}>
              Completar até {POOL_SIZE}
            </button>
          )}
          <button type="button" className="ghost" onClick={() => setPasso("rota")}>
            Outra rota
          </button>
        </div>
      </div>

      <div className="role-tabs">
        {ROLES.map((item) => (
          <button
            type="button"
            key={item.id}
            className={role === item.id ? "on" : ""}
            onClick={() => setRole(item.id)}
          >
            {item.label}
          </button>
        ))}
      </div>

      {TIERS.map((tier) => {
        const group = entries.filter((entry) => entry.tier === tier.id);
        return (
          <div className="section" key={tier.id}>
            <h3>
              {tier.label} <span className="muted">({group.length})</span>
            </h3>
            {group.length === 0 ? (
              <p className="muted">Vazio.</p>
            ) : (
              <div className="grid grid-pool">
                {group.map((entry) => {
                  const champion = staticData?.champions.get(entry.championId);
                  return (
                    <div className="cell cell-static" key={entry.championId}>
                      <div className="cell-portrait">
                        {champion ? (
                          <img src={champion.iconUrl} alt={champion.name} loading="lazy" />
                        ) : (
                          <span>?</span>
                        )}
                      </div>
                      <span className="cell-name">{champion?.name ?? `#${entry.championId}`}</span>
                      <div className="cell-actions">
                        {TIERS.map((option) => (
                          <button
                            type="button"
                            key={option.id}
                            className={entry.tier === option.id ? "on" : ""}
                            onClick={() => retier(entry.championId, option.id)}
                            title={option.label}
                          >
                            {option.label[0]}
                          </button>
                        ))}
                        <button
                          type="button"
                          className="remove"
                          onClick={() => remove(entry.championId)}
                          title="Remover"
                        >
                          ×
                        </button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        );
      })}

      <div className="section">
        <h3>Adicionar</h3>
        <input
          className="search"
          type="search"
          placeholder="Buscar campeão…"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
        <div className="grid grid-catalog">
          {catalog.map((champion) => (
            <button
              type="button"
              key={champion.id}
              className={`cell ${chosen.has(champion.id) ? "cell-taken" : ""}`}
              onClick={() => add(champion.id, "extra")}
              title={champion.name}
              disabled={chosen.has(champion.id)}
            >
              <div className="cell-portrait">
                <img src={champion.iconUrl} alt={champion.name} loading="lazy" />
              </div>
              <span className="cell-name">{champion.name}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
