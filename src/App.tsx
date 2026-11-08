import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke, type InvokeArgs } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { loadStaticData, type StaticData } from "./lib/ddragon";
import type {
  BanSuggestion,
  BuildContext,
  CoachReport,
  ConnStatus,
  DraftPrefs,
  DraftState,
  EngineStatus,
  GoldReport,
  BoughtReport,
  ItemSuggestion,
  LaneProfile,
  LaneReport,
  LiveSnapshot,
  MinimaxPick,
  OverlayPrefs,
  Pool,
  PoolPlan,
  Role,
  RunePlan,
  ScoredChampion,
} from "./lib/types";
import { ROLE_TO_LCU } from "./lib/types";
import Atualizacao from "./components/Atualizacao";
import ConnectionStatus from "./components/ConnectionStatus";
import DraftBoard from "./components/DraftBoard";
import GoldTracker from "./components/GoldTracker";
import LaneAnalysis from "./components/LaneAnalysis";
import LiveBuild from "./components/LiveBuild";
import Logo from "./components/Logo";
import LiveOverview from "./components/LiveOverview";
import OverlayPrefsPanel from "./components/OverlayPrefsPanel";
import PerfilDeRota from "./components/PerfilDeRota";
import PoolBuilder from "./components/PoolBuilder";
import PostGame from "./components/PostGame";
import RecommendationPanel from "./components/RecommendationPanel";
import Runas from "./components/Runas";
import SimuladorDraft, { draftVazio } from "./components/SimuladorDraft";

const EVENT_STATUS = "presage://status";
const EVENT_DRAFT = "presage://draft";
const EVENT_ENGINE = "presage://engine";
const EVENT_LIVE = "presage://live";
const EVENT_REVIEW = "presage://review";

type View = "draft" | "simulador" | "lanes" | "live" | "coach" | "pool" | "overlay";

/** O que uma chamada à casca devolveu: o valor, ou que ela falhou. */
type Resultado<T> = { ok: true; valor: T } | { ok: false };

/** Uma chamada que falhou: o que o app tentava fazer, e o que a casca disse. */
interface Falha {
  rotulo: string;
  detalhe: string;
}

function textoDoErro(err: unknown): string {
  if (typeof err === "string") return err;
  if (err instanceof Error) return err.message;
  return String(err);
}

export default function App() {
  const [view, setView] = useState<View>("draft");
  const [status, setStatus] = useState<ConnStatus | null>(null);
  const [draft, setDraft] = useState<DraftState | null>(null);
  const [engine, setEngine] = useState<EngineStatus | null>(null);
  const [pool, setPool] = useState<Pool>({ entries: [] });
  const [recommendations, setRecommendations] = useState<ScoredChampion[]>([]);
  const [minimax, setMinimax] = useState<MinimaxPick[]>([]);
  const [bans, setBans] = useState<BanSuggestion[]>([]);
  const [live, setLive] = useState<LiveSnapshot | null>(null);
  const [buildContext, setBuildContext] = useState<BuildContext | null>(null);
  const [suggestions, setSuggestions] = useState<ItemSuggestion[]>([]);
  const [bought, setBought] = useState<BoughtReport | null>(null);
  const [lanes, setLanes] = useState<LaneReport | null>(null);
  const [goldReport, setGoldReport] = useState<GoldReport | null>(null);
  const [review, setReview] = useState<CoachReport | null>(null);
  const [draftRole, setDraftRole] = useState<Role | null>(null);
  const [laneProfile, setLaneProfile] = useState<LaneProfile | null>(null);
  const [runas, setRunas] = useState<RunePlan | null>(null);
  const [overlayPrefs, setOverlayPrefs] = useState<OverlayPrefs>({ hidden: [] });
  const [draftPrefs, setDraftPrefs] = useState<DraftPrefs>({ showAllChampions: false });
  const [autostart, setAutostart] = useState(false);
  // A rota que você marcou em cada inimigo, por campeão.
  const [enemyRoles, setEnemyRoles] = useState<Record<number, Role>>({});
  // O simulador: o que está na tela e o que o backend já guardou. As
  // recomendações só são pedidas depois que ele guardou.
  const [simDraft, setSimDraft] = useState<DraftState>(draftVazio);
  const [simAplicado, setSimAplicado] = useState<DraftState | null>(null);
  const [staticData, setStaticData] = useState<StaticData | null>(null);
  const [staticError, setStaticError] = useState<string | null>(null);

  const [falhas, setFalhas] = useState<Falha[]>([]);

  // Toda chamada à casca passa por aqui (GC-1). A falha vira um aviso que diz o
  // que o app tentava fazer, e some quando a mesma chamada volta a dar certo.
  // Nada é engolido: quem chama decide o que mostrar no lugar, e um dado que não
  // veio nunca fica na tela como se fosse o de agora.
  const chamar = useCallback(
    async <T,>(rotulo: string, comando: string, args?: InvokeArgs): Promise<Resultado<T>> => {
      try {
        const valor = await invoke<T>(comando, args);
        setFalhas((xs) => (xs.some((x) => x.rotulo === rotulo) ? xs.filter((x) => x.rotulo !== rotulo) : xs));
        return { ok: true, valor };
      } catch (err) {
        const detalhe = textoDoErro(err);
        setFalhas((xs) => [...xs.filter((x) => x.rotulo !== rotulo), { rotulo, detalhe }]);
        return { ok: false };
      }
    },
    []
  );

  const clockAnchor = useRef<{ at: number; leftMs: number } | null>(null);
  const [tick, setTick] = useState(0);

  useEffect(() => {
    loadStaticData()
      .then(setStaticData)
      .catch((err: unknown) => setStaticError(String(err)));
  }, []);

  useEffect(() => {
    let disposed = false;

    chamar<[ConnStatus | null, DraftState | null]>("ler o cliente", "get_snapshot").then((r) => {
      if (disposed || !r.ok) return;
      const [snapStatus, snapDraft] = r.valor;
      if (snapStatus) setStatus(snapStatus);
      if (snapDraft) {
        setDraft(snapDraft);
        clockAnchor.current = { at: Date.now(), leftMs: snapDraft.timeLeftMs };
      }
    });

    // Leitura que falha deixa o valor inicial e o aviso: nunca um valor inventado.
    const ler = <T,>(rotulo: string, comando: string, aplicar: (valor: T) => void) =>
      chamar<T>(rotulo, comando).then((r) => {
        if (!disposed && r.ok) aplicar(r.valor);
      });
    ler<EngineStatus>("ler o estado do motor", "engine_status", setEngine);
    ler<Pool>("ler a pool", "get_pool", setPool);
    ler<LiveSnapshot | null>("ler a partida", "get_live", setLive);
    ler<CoachReport | null>("ler o pós-jogo", "last_review", setReview);
    ler<LaneProfile | null>("ler o perfil de rota", "lane_profile", setLaneProfile);
    ler<OverlayPrefs>("ler as preferências do overlay", "get_overlay_prefs", setOverlayPrefs);
    ler<DraftPrefs>("ler as preferências do draft", "get_draft_prefs", setDraftPrefs);
    ler<boolean>("ler o início com o Windows", "get_autostart", setAutostart);
    // As rotas marcadas nos inimigos vivem na casca: depois de recarregar a tela
    // no meio do champ select, ela as pede de volta em vez de perdê-las (A2).
    ler<[number, Role][]>("ler as rotas marcadas", "get_enemy_roles", (pares) =>
      setEnemyRoles(Object.fromEntries(pares))
    );

    const unlisteners = [
      listen<ConnStatus>(EVENT_STATUS, (event) => setStatus(event.payload)),
      // O histórico pessoal termina de atualizar emitindo este evento, e é
      // aí que o perfil de rota muda.
      listen<EngineStatus>(EVENT_ENGINE, (event) => {
        setEngine(event.payload);
        ler<LaneProfile | null>("ler o perfil de rota", "lane_profile", setLaneProfile);
      }),
      listen<LiveSnapshot | null>(EVENT_LIVE, (event) => setLive(event.payload)),
      // A partida acabou e a leitura ficou pronta: traz a aba para a frente,
      // porque é exatamente a hora em que ela interessa.
      listen<CoachReport>(EVENT_REVIEW, (event) => {
        setReview(event.payload);
        setView("coach");
      }),
      listen<DraftState | null>(EVENT_DRAFT, (event) => {
        setDraft(event.payload);
        // O backend esquece as marcas quando o champ select acaba.
        if (!event.payload) setEnemyRoles({});
        clockAnchor.current = event.payload
          ? { at: Date.now(), leftMs: event.payload.timeLeftMs }
          : null;
      }),
    ];

    const ticker = window.setInterval(() => setTick((n) => n + 1), 100);

    return () => {
      disposed = true;
      window.clearInterval(ticker);
      // Desligar um ouvinte que nunca ligou não tem o que avisar.
      unlisteners.forEach((p) => p.then((un) => un()).catch(() => void 0));
    };
  }, [chamar]);

  // O simulador manda o draft dele para o backend enquanto a aba está aberta;
  // fora dela, o motor volta a ler o champ select do cliente.
  useEffect(() => {
    if (view !== "simulador") {
      setSimAplicado(null);
      void chamar("devolver o draft ao cliente", "set_simulated_draft", { draft: null });
      return;
    }
    let vivo = true;
    chamar("aplicar o draft do simulador", "set_simulated_draft", { draft: simDraft }).then((r) => {
      if (vivo && r.ok) setSimAplicado(simDraft);
    });
    return () => {
      vivo = false;
    };
  }, [view, simDraft, chamar]);

  const activeDraft = view === "simulador" ? simAplicado : draft;

  useEffect(() => {
    if (!activeDraft || !engine?.ready) {
      setRecommendations([]);
      setMinimax([]);
      setBans([]);
      return;
    }
    let cancelled = false;

    Promise.all([
      chamar<ScoredChampion[]>("ler o ranking", "rank_draft"),
      chamar<MinimaxPick[]>("ler a resposta adversária", "minimax_draft"),
      chamar<BanSuggestion[]>("ler a sugestão de ban", "ban_suggestions"),
    ]).then(([ranked, tree, banList]) => {
      if (cancelled) return;
      // Cada lista que falhou sai da tela: a da leitura anterior não é a de agora.
      setRecommendations(ranked.ok ? ranked.valor : []);
      setMinimax(tree.ok ? tree.valor : []);
      setBans(banList.ok ? banList.valor : []);
    });

    return () => {
      cancelled = true;
    };
  }, [activeDraft, engine?.ready, pool, draftRole, enemyRoles, draftPrefs, chamar]);

  useEffect(() => {
    if (!live) {
      setBuildContext(null);
      setSuggestions([]);
      setBought(null);
      setGoldReport(null);
      return;
    }
    let cancelled = false;

    Promise.all([
      chamar<[BuildContext, ItemSuggestion[]] | null>("ler a build", "build_suggestions", { limit: 8 }),
      chamar<GoldReport | null>("ler o ouro", "gold_report"),
      chamar<BoughtReport | null>("ler a build da base", "bought_items", { limit: 6 }),
    ]).then(([build, gold, base]) => {
      if (cancelled) return;
      if (!build.ok) {
        setBuildContext(null);
        setSuggestions([]);
      } else if (build.valor) {
        setBuildContext(build.valor[0]);
        setSuggestions(build.valor[1]);
      }
      setGoldReport(gold.ok ? gold.valor : null);
      setBought(base.ok ? base.valor : null);
    });

    return () => {
      cancelled = true;
    };
  }, [live, chamar]);

  // A análise de rota serve os dois momentos: no draft é projeção, na partida
  // sai dos níveis e itens reais. O comando decide sozinho qual fonte usar.
  useEffect(() => {
    if (!engine?.ready || (!activeDraft && !live)) {
      // Sem draft e sem partida, rotas e runas saem juntas (A11).
      setLanes(null);
      setRunas(null);
      return;
    }
    let cancelled = false;
    Promise.all([
      chamar<LaneReport | null>("ler a análise de rota", "lane_analysis", { candidate: null }),
      chamar<RunePlan | null>("ler as runas", "rune_plan"),
    ]).then(([report, plano]) => {
      if (cancelled) return;
      setLanes(report.ok ? report.valor : null);
      setRunas(plano.ok ? plano.valor : null);
    });
    return () => {
      cancelled = true;
    };
    // A casca resolve a sua rota pela escolhida na tela e pela pool: as duas
    // mudam a análise e a runa tanto quanto o draft (A11).
  }, [activeDraft, live, engine?.ready, enemyRoles, draftRole, pool, chamar]);

  // O construtor pergunta o que falta só quando chega no último passo.
  // Quem chama precisa saber que falhou, para não mostrar "nada a recomendar"
  // no lugar de um erro: aqui a falha volta como promessa rejeitada.
  const pedirSugestoes = useCallback(
    (role: Role | null) =>
      chamar<PoolPlan>("calcular a sugestão da pool", "pool_suggestions", {
        role: role && role !== "Unknown" ? ROLE_TO_LCU[role] : null,
        limit: 6,
      }).then((r) => {
        if (!r.ok) throw new Error("pool_suggestions falhou");
        return r.valor;
      }),
    [chamar]
  );

  // A prévia de um candidato: a análise do draft com ele no seu lugar.
  // Ela depende das rotas marcadas nos inimigos: marcar uma troca a função, e o
  // painel refaz a prévia aberta com o oponente novo (A11).
  const previa = useCallback(
    (championId: number) =>
      chamar<LaneReport | null>("calcular a prévia", "lane_analysis", { candidate: championId }).then((r) => {
        if (!r.ok) throw new Error("lane_analysis falhou");
        return r.valor;
      }),
    [chamar, enemyRoles]
  );

  // A rota só muda na tela depois que o backend guardou: a lista que volta já
  // é a da rota nova.
  const escolherRota = useCallback((proxima: Role) => {
    if (proxima === "Unknown") return;
    chamar("escolher a rota", "set_draft_role", { role: ROLE_TO_LCU[proxima] }).then((r) => {
      if (r.ok) setDraftRole(proxima);
    });
  }, [chamar]);

  // Corrigir a rota de um inimigo refaz ranking, prévia e análise de rota.
  const marcarRota = useCallback((championId: number, role: Role | null) => {
    const lcu = role && role !== "Unknown" ? ROLE_TO_LCU[role] : null;
    chamar("marcar a rota do inimigo", "set_enemy_role", { championId, role: lcu }).then((r) => {
      if (!r.ok) return;
      setEnemyRoles((atual) => {
        const proximo = { ...atual };
        if (role && role !== "Unknown") proximo[championId] = role;
        else delete proximo[championId];
        return proximo;
      });
    });
  }, [chamar]);

  // A rota que o motor presumiu para cada inimigo, lida da análise do draft.
  const presumidas = useMemo(() => {
    const mapa: Record<number, Role> = {};
    if (lanes?.source === "draft") {
      for (const lane of lanes.lanes) if (lane.enemy > 0) mapa[lane.enemy] = lane.role;
    }
    return mapa;
  }, [lanes]);

  // As escritas otimistas mudam a tela na hora; se a casca recusar, a tela
  // volta ao valor de antes, que é o que a casca e o disco ainda têm. Só volta
  // se a tela ainda mostra o que esta escrita pôs lá: uma escrita mais nova
  // que deu certo não é desfeita.
  function saveDraftPrefs(next: DraftPrefs) {
    const anterior = draftPrefs;
    setDraftPrefs(next);
    chamar("gravar as preferências do draft", "set_draft_prefs", { prefs: next }).then((r) => {
      if (!r.ok) setDraftPrefs((atual) => (atual === next ? anterior : atual));
    });
  }

  function saveAutostart(next: boolean) {
    chamar("mudar o início com o Windows", "set_autostart", { enabled: next }).then((r) => {
      if (r.ok) setAutostart(next);
    });
  }

  const secondsLeft = useMemo(() => {
    const anchor = clockAnchor.current;
    if (!anchor) return null;
    return Math.max(0, (anchor.leftMs - (Date.now() - anchor.at)) / 1000);
  }, [draft, tick]);

  function savePool(next: Pool) {
    const anterior = pool;
    setPool(next);
    chamar("gravar a pool", "set_pool", { pool: next }).then((r) => {
      if (!r.ok) setPool((atual) => (atual === next ? anterior : atual));
    });
  }

  function saveOverlayPrefs(next: OverlayPrefs) {
    // Otimista: o clique responde na hora e o backend confirma depois. Travar
    // um checkbox esperando ida e volta ao disco seria lento à toa.
    const anterior = overlayPrefs;
    setOverlayPrefs(next);
    chamar("gravar as preferências do overlay", "set_overlay_prefs", { prefs: next }).then((r) => {
      if (!r.ok) setOverlayPrefs((atual) => (atual === next ? anterior : atual));
    });
  }

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <Logo size={22} />
          <span className="brand-mark">PRESAGE</span>
          <span className="brand-tag">
            {engine?.ready ? `patch ${engine.version}` : "draft engine"} · v
            {__APP_VERSION__}
          </span>
        </div>
        <nav className="nav">
          <button
            type="button"
            className={view === "draft" ? "on" : ""}
            onClick={() => setView("draft")}
          >
            Draft
          </button>
          <button
            type="button"
            className={view === "simulador" ? "on" : ""}
            onClick={() => setView("simulador")}
          >
            Simulador
          </button>
          <button
            type="button"
            className={view === "lanes" ? "on" : ""}
            onClick={() => setView("lanes")}
          >
            Rotas
          </button>
          <button
            type="button"
            className={view === "live" ? "on" : ""}
            onClick={() => setView("live")}
          >
            Partida
            {live && <span className="nav-dot" />}
          </button>
          <button
            type="button"
            className={view === "coach" ? "on" : ""}
            onClick={() => setView("coach")}
          >
            Pós-jogo
          </button>
          <button
            type="button"
            className={view === "pool" ? "on" : ""}
            onClick={() => setView("pool")}
          >
            Pool
          </button>
          <button
            type="button"
            className={view === "overlay" ? "on" : ""}
            onClick={() => setView("overlay")}
          >
            Ajustes
          </button>
        </nav>
        <ConnectionStatus status={status} />
      </header>

      <Atualizacao />

      {falhas.map((falha) => (
        <div className="banner banner-warn banner-falha" role="alert" key={falha.rotulo}>
          <span>
            Não consegui {falha.rotulo}: {falha.detalhe}
          </span>
          <button
            type="button"
            className="banner-fechar"
            title="Fechar"
            onClick={() => setFalhas((xs) => xs.filter((x) => x.rotulo !== falha.rotulo))}
          >
            ×
          </button>
        </div>
      ))}

      {staticError && (
        <div className="banner banner-warn">
          Não consegui carregar os dados do Data Dragon ({staticError}). Nomes e
          ícones ficam indisponíveis.
        </div>
      )}

      {view === "pool" ? (
        <main className="layout layout-single">
          <section className="panel">
            <PoolBuilder
              pool={pool}
              staticData={staticData}
              onChange={savePool}
              onSuggest={pedirSugestoes}
            />
          </section>
        </main>
      ) : view === "overlay" ? (
        <main className="layout layout-single">
          <section className="panel">
            <OverlayPrefsPanel prefs={overlayPrefs} onChange={saveOverlayPrefs} />
            <div className="section">
              <h3>App</h3>
              <label className="reco-toggle">
                <input
                  type="checkbox"
                  checked={autostart}
                  onChange={(e) => saveAutostart(e.target.checked)}
                />
                Iniciar o Presage com o Windows
              </label>
              <p className="faint">
                Desligado até você ligar. O ícone na bandeja traz a janela de volta.
              </p>
            </div>
          </section>
        </main>
      ) : view === "simulador" ? (
        <main className="layout">
          <section className="panel panel-draft">
            <SimuladorDraft
              value={simDraft}
              staticData={staticData}
              onChange={setSimDraft}
              live={draft}
            />
          </section>
          <section className="panel panel-reco">
            <RecommendationPanel
              recommendations={recommendations}
              minimax={minimax}
              bans={bans}
              staticData={staticData}
              draft={simAplicado}
              engine={engine}
              onPreview={previa}
            />
          </section>
        </main>
      ) : view === "lanes" ? (
        <main className="layout layout-single">
          <section className="panel">
            <LaneAnalysis report={lanes} staticData={staticData} />
            <Runas plan={runas} patch={engine?.version ?? null} />
          </section>
        </main>
      ) : view === "coach" ? (
        <main className="layout layout-single">
          {/* Uma seção só: duas dividiriam a tela ao meio e cortariam o gráfico
              do pós-jogo, cada metade com a própria rolagem. */}
          <section className="panel">
            <PostGame report={review} />
            <PerfilDeRota profile={laneProfile} staticData={staticData} />
          </section>
        </main>
      ) : view === "live" ? (
        live ? (
          <main className="layout">
            <section className="panel">
              <LiveOverview snapshot={live} staticData={staticData} />
              <GoldTracker report={goldReport} staticData={staticData} />
            </section>
            <section className="panel">
              <LiveBuild
                game={live.game}
                context={buildContext}
                suggestions={suggestions}
                staticData={staticData}
                itemsError={engine?.itemsError ?? null}
                bought={bought}
                patch={engine?.globalPatch ?? null}
              />
            </section>
          </main>
        ) : (
          <main className="layout layout-single">
            <section className="panel">
              <div className="empty">
                <h2>Sem partida em andamento</h2>
                <p>
                  {status?.connected
                    ? "Entre numa partida e os timers e a build adaptativa ligam sozinhos."
                    : "Abra o cliente do League of Legends."}
                </p>
              </div>
            </section>
          </main>
        )
      ) : (
        <main className="layout">
          <section className="panel panel-draft">
            <DraftBoard
              draft={draft}
              staticData={staticData}
              secondsLeft={secondsLeft}
              connected={status?.connected ?? false}
              phase={status?.phase ?? "Disconnected"}
              inferred={presumidas}
              marked={enemyRoles}
              onEnemyRole={marcarRota}
            />
          </section>

          <section className="panel panel-reco">
            <RecommendationPanel
              recommendations={recommendations}
              minimax={minimax}
              bans={bans}
              staticData={staticData}
              draft={draft}
              engine={engine}
              role={draftRole}
              onRole={escolherRota}
              onPreview={previa}
              showAll={draftPrefs.showAllChampions}
              onShowAll={(showAllChampions) => saveDraftPrefs({ ...draftPrefs, showAllChampions })}
            />
          </section>
        </main>
      )}
    </div>
  );
}
