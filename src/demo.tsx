// Demonstração da interface com dados de exemplo.
//
// Os componentes são puramente visuais — nenhum deles fala com o Tauri — então
// esta página renderiza exatamente o que o app mostra, sem precisar do cliente
// da Riot nem do backend compilado.

import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";

import { loadStaticData, type StaticData } from "./lib/ddragon";
import ConnectionStatus from "./components/ConnectionStatus";
import DraftBoard from "./components/DraftBoard";
import GoldTracker from "./components/GoldTracker";
import LaneAnalysis from "./components/LaneAnalysis";
import LiveBuild from "./components/LiveBuild";
import LiveOverview from "./components/LiveOverview";
import Logo from "./components/Logo";
import OverlayPrefsPanel from "./components/OverlayPrefsPanel";
import PerfilDeRota from "./components/PerfilDeRota";
import PoolBuilder from "./components/PoolBuilder";
import PostGame from "./components/PostGame";
import Runas from "./components/Runas";
import RecommendationPanel from "./components/RecommendationPanel";
import SimuladorDraft, { draftVazio } from "./components/SimuladorDraft";
import type {
  BanSuggestion,
  BuildContext,
  CoachReport,
  ConnStatus,
  DraftSlot,
  DraftState,
  EngineStatus,
  GoldReport,
  ItemSuggestion,
  LaneProfile,
  LaneReading,
  LaneReport,
  LivePlayer,
  LiveSnapshot,
  MinimaxPick,
  OverlayPrefs,
  Pool,
  PoolPlan,
  Role,
  RunePlan,
  ScoredChampion,
} from "./lib/types";
import "./styles.css";

const AHRI = 103;
const LEE_SIN = 64;
const YASUO = 157;
const ZED = 238;
const MALPHITE = 54;
const LUX = 99;
const ASHE = 22;
const THRESH = 412;
const GAREN = 86;
const ORIANNA = 61;
const SYNDRA = 134;
const VIEGO = 234;

function slot(
  cellId: number,
  position: string,
  championId: number,
  extra: Partial<DraftSlot> = {}
): DraftSlot {
  return {
    cellId,
    position,
    championId,
    intentId: 0,
    spell1: 4,
    spell2: 14,
    isLocal: false,
    isActing: false,
    locked: championId > 0,
    ...extra,
  };
}

const status: ConnStatus = {
  connected: true,
  phase: "ChampSelect",
  summoner: "crit is back#split",
  detail: null,
};

const engine: EngineStatus = {
  ready: true,
  version: "16.21.1",
  champions: 171,
  // A demonstração mostra o app com a base já carregada. O aviso de "sem base
  // estatística" continua no componente e aparece no app real quando é o caso.
  hasGlobal: true,
  hasPersonal: true,
  personalGames: 47,
  account: "crit is back#split",
  globalPatch: "16.21",
  globalMatches: 1_240_000,
};

const draft: DraftState = {
  phase: "BAN_PICK",
  timeLeftMs: 21_000,
  totalTimeMs: 30_000,
  localCellId: 0,
  localPosition: "middle",
  myTeam: [
    slot(0, "top", GAREN),
    slot(1, "jungle", LEE_SIN),
    slot(2, "middle", 0, { isLocal: true, isActing: true, intentId: AHRI }),
    slot(3, "bottom", ASHE),
    slot(4, "utility", THRESH),
  ],
  // A ranqueada não informa a rota do inimigo: o motor presume, você corrige.
  theirTeam: [
    slot(5, "", MALPHITE),
    slot(6, "", VIEGO),
    slot(7, "", ZED),
    slot(8, "", 0),
    slot(9, "", 0),
  ],
  myBans: [YASUO, 157],
  theirBans: [LUX, 7],
  currentAction: "pick",
  isLocalTurn: true,
  unavailable: [GAREN, LEE_SIN, ASHE, THRESH, MALPHITE, VIEGO, ZED, YASUO, LUX, 7],
};

const recommendations: ScoredChampion[] = [
  {
    championId: AHRI,
    score: 53.4,
    delta: 3.4,
    winrate: 0.534,
    confidence: 4.1,
    sample: 3120,
    inPool: true,
    reasons: ["Vence Zed na rota (+2.8pp, n=1840)", "Seus jogos: 18V-9D (+2.1pp)"],
  },
  {
    championId: ORIANNA,
    score: 52.1,
    delta: 2.1,
    winrate: 0.521,
    confidence: 4.4,
    sample: 2410,
    inPool: true,
    reasons: ["Equilibra o dano: 78% → 62% físico"],
  },
  {
    championId: SYNDRA,
    score: 50.9,
    delta: 0.9,
    winrate: 0.509,
    confidence: 5.2,
    sample: 1180,
    inPool: true,
    reasons: ["Waveclear seguro contra a pressão deles"],
  },
  {
    championId: 1,
    score: 54.2,
    delta: 4.2,
    winrate: 0.542,
    confidence: 3.8,
    sample: 5300,
    inPool: false,
    reasons: ["Equilibra o dano: 78% → 60% físico", "Vence Zed na rota (+3.1pp, n=2200)"],
  },
  {
    championId: 45,
    score: 53.0,
    delta: 3.0,
    winrate: 0.53,
    confidence: 4.0,
    sample: 4100,
    inPool: false,
    reasons: ["Escala melhor que a comp inimiga pós-25min"],
  },
  {
    championId: 142,
    score: 51.5,
    delta: 1.5,
    winrate: 0.515,
    confidence: 4.9,
    sample: 1600,
    inPool: false,
    reasons: ["Dá o engage que falta na sua comp"],
  },
  {
    championId: 517,
    score: 50.4,
    delta: 0.4,
    winrate: 0.504,
    confidence: 5.5,
    sample: 980,
    inPool: false,
    reasons: ["Comp inimiga é 100% AD — seu dano misto resolve"],
  },
];

const minimax: MinimaxPick[] = [
  {
    championId: ORIANNA,
    greedy: 0.531,
    afterResponse: 0.524,
    exposure: 0.007,
    bestResponse: 238,
    inPool: true,
  },
  {
    championId: 1,
    greedy: 0.548,
    afterResponse: 0.529,
    exposure: 0.019,
    bestResponse: 54,
    inPool: false,
  },
  {
    championId: AHRI,
    greedy: 0.54,
    afterResponse: 0.518,
    exposure: 0.022,
    bestResponse: 55,
    inPool: true,
  },
  {
    championId: SYNDRA,
    greedy: 0.522,
    afterResponse: 0.494,
    exposure: 0.028,
    bestResponse: 105,
    inPool: true,
  },
  {
    championId: 142,
    greedy: 0.535,
    afterResponse: 0.498,
    exposure: 0.037,
    bestResponse: 64,
    inPool: false,
  },
];

const bans: BanSuggestion[] = [
  { championId: 517, threat: 0, reason: "Da sua lista de bans", fromPool: true },
  { championId: 55, threat: 0.031, reason: "Completa o que falta na composição deles" },
  { championId: 105, threat: 0.027, reason: "Completa o que falta na composição deles" },
  { championId: 245, threat: 0.024, reason: "Completa o que falta na composição deles" },
];

function livePlayer(
  key: string,
  championId: number,
  team: string,
  position: string,
  extra: Partial<LivePlayer> = {}
): LivePlayer {
  return {
    championKey: key,
    championId,
    displayName: `${key}#BR1`,
    team,
    position,
    level: 13,
    items: [],
    goldInvested: 0,
    isDead: false,
    respawnTimer: 0,
    isLocal: false,
    scores: { kills: 2, deaths: 1, assists: 4, creepScore: 142, wardScore: 12.4 },
    ...extra,
  };
}

const live: LiveSnapshot = {
  game: {
    gameTime: 1147,
    players: [
      livePlayer("Garen", GAREN, "ORDER", "TOP", {
        items: [3047, 3075],
        goldInvested: 4100,
      }),
      livePlayer("LeeSin", LEE_SIN, "ORDER", "JUNGLE", {
        items: [3071, 3047],
        goldInvested: 4100,
      }),
      livePlayer("Ahri", AHRI, "ORDER", "MIDDLE", {
        items: [3157, 3020],
        goldInvested: 4150,
        isLocal: true,
        scores: { kills: 5, deaths: 2, assists: 6, creepScore: 168, wardScore: 9.1 },
      }),
      livePlayer("Ashe", ASHE, "ORDER", "BOTTOM", {
        items: [3031, 3006],
        goldInvested: 4650,
      }),
      livePlayer("Thresh", THRESH, "ORDER", "UTILITY", {
        items: [3190],
        goldInvested: 2200,
      }),
      livePlayer("Malphite", MALPHITE, "CHAOS", "TOP", {
        items: [3068, 3047],
        goldInvested: 4400,
      }),
      livePlayer("Viego", VIEGO, "CHAOS", "JUNGLE", {
        items: [6672, 3006],
        goldInvested: 4400,
        isDead: true,
        respawnTimer: 23,
      }),
      livePlayer("Zed", ZED, "CHAOS", "MIDDLE", {
        items: [3142, 3036],
        goldInvested: 6100,
      }),
      livePlayer("Jhin", 202, "CHAOS", "BOTTOM", {
        items: [3031, 3036],
        goldInvested: 6750,
      }),
      livePlayer("Lulu", 117, "CHAOS", "UTILITY", {
        items: [3853],
        goldInvested: 500,
      }),
    ],
  },
  timeline: {
    gameTime: 1147,
    dragons: { ally: 2, enemy: 1 },
    dragonTypes: ["Fire", "Ocean", "Cloud"],
    soul: null,
    turrets: { ally: 3, enemy: 1 },
    kills: { ally: 11, enemy: 8 },
    upcoming: [
      { name: "Dragão", at: 1190, secondsLeft: 43, observed: true },
      { name: "Barão", at: 1460, secondsLeft: 313, observed: true },
    ],
    respawns: [{ name: "Inibidor do meio", ours: false, at: 1290, secondsLeft: 143 }],
  },
  buffs: [
    {
      name: "Barão",
      ours: true,
      started: 1100,
      until: 1280,
      secondsLeft: 133,
      holders: [AHRI, GAREN, ASHE],
    },
  ],
  rival: {
    role: "Mid",
    opponentId: ZED,
    opponentKey: "Zed",
    metric: "farm",
    minePerMin: 7.4,
    theirsPerMin: 8.1,
    goldDiff: -1950,
    standing: "behind",
  },
  priority: {
    lanes: [
      {
        lane: "BOT",
        score: 78.4,
        skirmishOdds: 0.61,
        reasons: ["3v2 a seu favor em 61%", "Jungler inimigo morto por 23s", "Dragão em 43s"],
      },
      {
        lane: "TOP",
        score: 61.2,
        skirmishOdds: 0.49,
        reasons: ["2v1 a seu favor em 49%", "Jungler inimigo morto por 23s"],
      },
      {
        lane: "MID",
        score: 54.8,
        skirmishOdds: 0.43,
        reasons: ["1v1 a seu favor em 43%", "Jungler inimigo morto por 23s"],
      },
    ],
    focus: "Dragão em 43s",
  },
};

const buildContext: BuildContext = {
  incomingPhysicalShare: 0.81,
  enemyArmorPenPercent: 0.35,
  enemyLethality: 18,
  enemyMagicPenPercent: 0,
  yourEffectiveHp: 3240,
};

const suggestions: ItemSuggestion[] = [
  {
    itemId: 3157,
    name: "Ampulheta de Zhonya",
    cost: 3250,
    valuePer1000g: 6.8,
    ehpGain: 980,
    offenseGainPercent: 11.2,
    reasons: ["81% do dano deles é físico", "Contra 18 de letalidade, vida rende mais que armadura"],
  },
  {
    itemId: 3116,
    name: "Rylai",
    cost: 2600,
    valuePer1000g: 5.4,
    ehpGain: 1120,
    offenseGainPercent: 4.1,
    reasons: ["Eles têm 35% de penetração de armadura — armadura rende menos"],
  },
  {
    itemId: 3135,
    name: "Cajado do Vazio",
    cost: 3000,
    valuePer1000g: 4.9,
    ehpGain: 0,
    offenseGainPercent: 22.7,
    reasons: ["Fura a resistência mágica deles"],
  },
  {
    itemId: 3089,
    name: "Capuz da Morte de Rabadon",
    cost: 3600,
    valuePer1000g: 3.6,
    ehpGain: 0,
    offenseGainPercent: 18.4,
    reasons: [],
  },
];

// A curva de uma rota na demonstração: começa em `cedo` no nível 1 e chega em
// `tarde` no 18, como a conta de atributos costuma fazer — devagar, sem pico.
function rotaDemo(cedo: number, tarde: number, meu = 6, dele = 6) {
  const em = (nivel: number) => cedo + ((tarde - cedo) * (nivel - 1)) / 17;
  const curve = Array.from({ length: 18 }, (_, i) => ({ level: i + 1, gap: 0, odds: em(i + 1) }));
  const gaps = [3, 6, 11, 16].flatMap((level) =>
    [1, 0, -1].map((gap) => ({ level, gap, odds: Math.min(0.95, Math.max(0.05, em(level) + gap * 0.06)) }))
  );
  return { curve, gaps, duel: em(dele) + (meu - dele) * 0.06, myLevel: meu, theirLevel: dele };
}

const laneReport: LaneReport = {
  source: "partida",
  rolesAssumed: false,
  team: null,
  winCondition: {
    facts: [
      {
        side: "deles",
        source: "regra",
        text: "Eles: 72% do dano é físico — armadura responde a quase tudo",
      },
    ],
    edges: [
      {
        side: "nosso",
        source: "atributos",
        text: "SUP: Thresh ganha o 1v1 de Lulu na conta de atributos (62%)",
      },
      {
        side: "nosso",
        source: "atributos",
        text: "JG+TOP x JG+TOP (gank ou invade no topo): vocês levam na conta de atributos (56%)",
      },
      {
        side: "deles",
        source: "atributos",
        text: "TOP: Malphite ganha o 1v1 de Garen na conta de atributos (55%)",
      },
    ],
    phase: null,
    caveat:
      "Composição por regra e duelos pela conta de atributos e itens, sem habilidades: quem escala pela suprema ou por acúmulo não aparece aqui. A fase do jogo só entra com a base de partidas.",
  },
  lanes: [
    {
      role: "Top",
      lane: "TOP",
      ally: GAREN,
      enemy: MALPHITE,
      allyName: "Garen",
      enemyName: "Malphite",
      winProbability: 0.43,
      margin: -7,
      sample: 2410,
      fromData: true,
      byLevel: [{ level: 3, gap: 0, odds: 0.49 }, { level: 6, gap: 0, odds: 0.52 }, { level: 11, gap: 0, odds: 0.47 }, { level: 16, gap: 0, odds: 0.44 }],
      ...rotaDemo(0.49, 0.44),
      reasons: [
        "Perde para Malphite na rota (−3.1pp, n=2410)",
        "Perde o 1v1 com os itens de agora (45%)",
      ],
    },
    {
      role: "Jungle",
      lane: "JG",
      ally: LEE_SIN,
      enemy: VIEGO,
      allyName: "Lee Sin",
      enemyName: "Viego",
      winProbability: 0.56,
      margin: 6,
      sample: 1920,
      fromData: true,
      byLevel: [{ level: 3, gap: 0, odds: 0.49 }, { level: 6, gap: 0, odds: 0.52 }, { level: 11, gap: 0, odds: 0.47 }, { level: 16, gap: 0, odds: 0.44 }],
      ...rotaDemo(0.49, 0.44),
      reasons: ["Vence Viego na rota (+2.4pp, n=1920)", "Ganha o 1v1 com os itens de agora (58%)"],
    },
    {
      role: "Mid",
      lane: "MID",
      ally: AHRI,
      enemy: ZED,
      allyName: "Ahri",
      enemyName: "Zed",
      winProbability: 0.47,
      margin: -3,
      sample: 1840,
      fromData: true,
      byLevel: [{ level: 3, gap: 0, odds: 0.49 }, { level: 6, gap: 0, odds: 0.52 }, { level: 11, gap: 0, odds: 0.47 }, { level: 16, gap: 0, odds: 0.44 }],
      ...rotaDemo(0.49, 0.44),
      reasons: [
        "Vence Zed na rota (+2.8pp, n=1840)",
        "Perde o 1v1 com os itens de agora (41%)",
        "Ahri tem alcance; Zed precisa chegar perto",
      ],
    },
    {
      role: "Bot",
      lane: "ADC",
      ally: ASHE,
      enemy: 202,
      allyName: "Ashe",
      enemyName: "Jhin",
      winProbability: 0.41,
      margin: -9,
      sample: 3100,
      fromData: true,
      byLevel: [{ level: 3, gap: 0, odds: 0.49 }, { level: 6, gap: 0, odds: 0.52 }, { level: 11, gap: 0, odds: 0.47 }, { level: 16, gap: 0, odds: 0.44 }],
      ...rotaDemo(0.49, 0.44),
      reasons: ["Perde para Jhin na rota (−2.2pp, n=3100)", "Perde o 1v1 com os itens de agora (38%)"],
    },
    {
      role: "Support",
      lane: "SUP",
      ally: THRESH,
      enemy: 117,
      allyName: "Thresh",
      enemyName: "Lulu",
      winProbability: 0.62,
      margin: 12,
      sample: 1450,
      fromData: true,
      byLevel: [{ level: 3, gap: 0, odds: 0.49 }, { level: 6, gap: 0, odds: 0.52 }, { level: 11, gap: 0, odds: 0.47 }, { level: 16, gap: 0, odds: 0.44 }],
      ...rotaDemo(0.49, 0.44),
      reasons: ["Vence Lulu na rota (+4.0pp, n=1450)", "Ganha o 1v1 com os itens de agora (66%)"],
    },
  ],
  skirmishes: [
    {
      key: "jg_top",
      label: "JG+TOP x JG+TOP",
      detail: "gank ou invade no topo",
      size: 2,
      ours: [LEE_SIN, GAREN],
      theirs: [VIEGO, MALPHITE],
      odds: 0.48,
      byLevel: [{ level: 3, gap: 0, odds: 0.46 }, { level: 6, gap: 0, odds: 0.5 }, { level: 11, gap: 0, odds: 0.53 }, { level: 16, gap: 0, odds: 0.56 }],
      reasons: ["Quem mais pesa do seu lado: Garen", "Do lado deles: Malphite"],
    },
    {
      key: "jg_mid",
      label: "JG+MID x JG+MID",
      detail: "gank ou briga no meio",
      size: 2,
      ours: [LEE_SIN, AHRI],
      theirs: [VIEGO, ZED],
      odds: 0.46,
      byLevel: [{ level: 3, gap: 0, odds: 0.46 }, { level: 6, gap: 0, odds: 0.5 }, { level: 11, gap: 0, odds: 0.53 }, { level: 16, gap: 0, odds: 0.56 }],
      reasons: ["Do lado deles: Zed", "Eles têm frontline no grupo e vocês não"],
    },
    {
      key: "bot",
      label: "ADC+SUP x ADC+SUP",
      detail: "rota inferior, sem jungle",
      size: 2,
      ours: [ASHE, THRESH],
      theirs: [202, 117],
      odds: 0.53,
      byLevel: [{ level: 3, gap: 0, odds: 0.46 }, { level: 6, gap: 0, odds: 0.5 }, { level: 11, gap: 0, odds: 0.53 }, { level: 16, gap: 0, odds: 0.56 }],
      reasons: ["Quem mais pesa do seu lado: Thresh", "Do lado deles: Jhin"],
    },
    {
      key: "dragao",
      label: "JG+ADC+SUP x JG+ADC+SUP",
      detail: "briga no dragão",
      size: 3,
      ours: [LEE_SIN, ASHE, THRESH],
      theirs: [VIEGO, 202, 117],
      odds: 0.61,
      byLevel: [{ level: 3, gap: 0, odds: 0.46 }, { level: 6, gap: 0, odds: 0.5 }, { level: 11, gap: 0, odds: 0.53 }, { level: 16, gap: 0, odds: 0.56 }],
      reasons: ["Quem mais pesa do seu lado: Thresh", "Vocês têm frontline e eles não"],
    },
    {
      key: "arauto",
      label: "JG+TOP+MID x JG+TOP+MID",
      detail: "briga no arauto",
      size: 3,
      ours: [LEE_SIN, GAREN, AHRI],
      theirs: [VIEGO, MALPHITE, ZED],
      odds: 0.45,
      byLevel: [{ level: 3, gap: 0, odds: 0.46 }, { level: 6, gap: 0, odds: 0.5 }, { level: 11, gap: 0, odds: 0.53 }, { level: 16, gap: 0, odds: 0.56 }],
      reasons: ["Quem mais pesa do seu lado: Garen", "Do lado deles: Malphite"],
    },
  ],
};

// A prévia de um candidato na demonstração: o mesmo relatório, com ele no
// lugar da Ahri no meio. O app real refaz a conta inteira para cada um.
function previaDemo(championId: number): LaneReport {
  const troca = (id: number) => (id === AHRI ? championId : id);
  return {
    ...laneReport,
    source: "draft",
    rolesAssumed: true,
    lanes: laneReport.lanes.map((lane) =>
      lane.ally === AHRI ? { ...lane, ally: championId, allyName: "Candidato" } : lane
    ),
    skirmishes: laneReport.skirmishes.map((s) => ({ ...s, ours: s.ours.map(troca) })),
  };
}

const goldReport: GoldReport = {
  gameTime: 1147,
  oursInvested: 19_200,
  theirsInvested: 22_150,
  investedLead: -2950,
  oursEarned: 26_400,
  theirsEarned: 29_100,
  earnedLead: -2700,
  notes: [
    "Seu time está 2950 atrás em itens",
    "Você tem ~1800 de ouro para gastar",
    "O mais adiantado deles é Jhin (6750 em itens)",
  ],
  players: live.game.players.map((player, i) => {
    // Cada um com a própria sobra, senão a coluna vira uma coluna de "+900"
    // que não parece dado de ninguém.
    const unspent = player.isLocal ? 1800 : [320, 1150, 0, 640, 2210, 80, 470, 1340, 0][i % 9];
    return {
      championKey: player.championKey,
      championId: player.championId,
      displayName: player.displayName,
      team: player.team,
      position: player.position,
      isLocal: player.isLocal,
      isAlly: player.team === "ORDER",
      invested: player.goldInvested,
      earned: player.goldInvested + unspent,
      unspent,
      goldPerMinute: (player.goldInvested + unspent) / (1147 / 60),
    };
  }),
};

// As curvas da demonstração contam a mesma história das observações abaixo:
// vantagem construída até os 21 minutos, as quatro mortes em sequência, e o
// jogo virando a partir dali.
const MORTES = [612, 1300, 1340, 1402, 1450, 1612, 1880];

function curvaDemo(ate: number, f: (t: number) => number) {
  const pontos = [];
  for (let t = 0; t <= ate; t += 30) pontos.push({ t, v: f(t) });
  return pontos;
}

const serieDemo = {
  goldLead: curvaDemo(2184, (t) =>
    t <= 1300 ? t * 2.6 : 3380 - (t - 1300) * 5.2
  ),
  csDiff: curvaDemo(2184, (t) => (t <= 900 ? t * 0.012 : 11 - (t - 900) * 0.055)),
  levelDiff: curvaDemo(2184, (t) =>
    t <= 1300 ? Math.min(1, t / 900) : Math.max(-2, 1 - (t - 1300) / 420)
  ),
  deaths: MORTES,
  teamKills: [430, 690, 905, 1120, 1270, 1500, 1750, 1990],
};

const review: CoachReport = {
  champion: "Ahri",
  role: "Mid",
  duration: 2184,
  won: false,
  summary: "36:24 de Ahri · 3 a corrigir, 2 a manter.",
  series: serieDemo,
  bad: [
    {
      verdict: "bad",
      title: "4 mortes entre 21:40 e 24:10",
      detail:
        "4 das suas 7 mortes saíram nessa janela. Foi ali que a partida virou contra você.",
      weight: 0.57,
    },
    {
      verdict: "bad",
      title: "5,4 tropas por minuto contra 7,1",
      detail: "Zed terminou com 62 tropas a mais. São ~1302 de ouro que você não pegou.",
      weight: 0.38,
    },
    {
      verdict: "bad",
      title: "3:10 com ouro parado",
      detail:
        "Chegou a ~2400 sem gastar. É estimativa, não medida — a API só entrega o ouro em mão do jogador local.",
      weight: 0.31,
    },
  ],
  good: [
    {
      verdict: "good",
      title: "71% dos abates do time",
      detail: "Você entrou em 17 dos 24 abates.",
      weight: 0.71,
    },
    {
      verdict: "good",
      title: "Primeiro item grande aos 10:50",
      detail: "Chegou cedo no primeiro pico de poder.",
      weight: 0.4,
    },
  ],
};

// Pool vazia de propósito: assim quem abre a demonstração cai no construtor
// guiado — rota, dois core, três situacionais — que é o que a aba tem de
// interessante para mostrar. Com a pool pronta ela abriria direto na revisão.
const initialPool: Pool = { entries: [] };

// Na demonstração não há backend, então o passo de completar recebe um plano
// fixo — o retrato de três lutadores de dano físico no topo e uma lista de quem
// cobre o que eles não cobrem, com as mesmas frases que o motor produz. O
// construtor tira da lista quem já entrou.
const planoDemo: PoolPlan = {
  analysis: {
    strengths: [{ trait: "fisico", text: "3 de 3 com dano físico" }],
    weaknesses: [
      { trait: "magico", text: "nenhum com dano mágico: o inimigo empilha armadura" },
      { trait: "frontline", text: "nenhum segura a linha de frente" },
      {
        trait: "alcance",
        text: "nenhum ataca à distância: quem tem alcance troca de graça na rota",
      },
      {
        trait: "lutador",
        text: "os 3 são lutadores: um estilo só, e a mesma resposta serve contra todos",
      },
    ],
  },
  candidates: [
    {
      championId: MALPHITE,
      name: "Malphite",
      score: 0.71,
      fills: ["magico", "frontline", "tanque"],
      reasons: [
        "é a linha de frente que a pool não tem",
        "traz o dano mágico que falta",
        "traz um estilo que a pool não tem: tanque",
      ],
    },
    {
      championId: 79,
      name: "Gragas",
      score: 0.66,
      fills: ["magico", "frontline"],
      reasons: ["é a linha de frente que a pool não tem", "traz o dano mágico que falta"],
    },
    {
      championId: 516,
      name: "Ornn",
      score: 0.63,
      fills: ["frontline", "robusto", "tanque"],
      reasons: [
        "é a linha de frente que a pool não tem",
        "aguenta dano físico e mágico parecido",
        "traz um estilo que a pool não tem: tanque",
      ],
    },
    {
      championId: 85,
      name: "Kennen",
      score: 0.52,
      fills: ["magico", "alcance", "mago"],
      reasons: ["traz o dano mágico que falta", "é a opção à distância que falta"],
    },
    {
      championId: 82,
      name: "Mordekaiser",
      score: 0.41,
      fills: ["magico"],
      reasons: ["traz o dano mágico que falta"],
    },
    {
      championId: 14,
      name: "Sion",
      score: 0.38,
      fills: ["frontline"],
      reasons: ["é a linha de frente que a pool não tem"],
    },
    {
      championId: 17,
      name: "Teemo",
      score: 0.35,
      fills: ["magico", "alcance"],
      reasons: ["traz o dano mágico que falta", "é a opção à distância que falta"],
    },
    {
      championId: 517,
      name: "Sylas",
      score: 0.31,
      fills: ["magico"],
      reasons: ["traz o dano mágico que falta"],
    },
  ],
};

const runasDemo: RunePlan = {
  matchup: "Ahri contra Zed",
  fromData: false,
  keystones: [
    {
      key: "PhaseRush",
      id: 8230,
      name: "Correria Fásica",
      icon: "perk-images/Styles/Sorcery/PhaseRush/PhaseRush.png",
      treeName: "Feitiçaria",
      score: 0.95,
      reasons: ["contra quem explode, sair da briga vale mais que somar dano"],
    },
    {
      key: "ArcaneComet",
      id: 8229,
      name: "Cometa Arcano",
      icon: "perk-images/Styles/Sorcery/ArcaneComet/ArcaneComet.png",
      treeName: "Feitiçaria",
      score: 0.9,
      reasons: ["mago contra corpo a corpo: dá para castigar sem entrar no alcance dele"],
    },
  ],
  secondary: {
    key: "BonePlating",
    id: 8473,
    name: "Revestimento Ósseo",
    icon: "perk-images/Styles/Resolve/BonePlating/BonePlating.png",
    treeName: "Determinação",
    score: 1,
    reasons: ["contra sequência curta de dano, ela corta justamente a parte que te mata"],
  },
};

type View = "draft" | "simulador" | "lanes" | "live" | "coach" | "pool" | "overlay";

// Perfil de rota com números do tamanho que 40 partidas produzem: a média
// geral já sai do ruído em tropa, os campeões menos jogados ainda não.
const leitura = (
  metric: LaneReading["metric"],
  games: number,
  value: number,
  sd: number,
  raw = value
): LaneReading => ({ metric, games, raw, value, sd });

const perfilDemo: LaneProfile = {
  games: 40,
  overall: [
    leitura("cs10", 34, 3.1, 1.4),
    leitura("killParticipation", 40, 2.0, 1.6),
    leitura("damageShare", 40, 1.8, 0.9),
    leitura("visionPerMinute", 40, -0.2, 0.06),
    leitura("deaths", 40, 0.6, 0.4),
  ],
  champions: [
    {
      champion: AHRI,
      games: 22,
      readings: [
        leitura("cs10", 22, 6.2, 1.9, 6.9),
        leitura("killParticipation", 22, 3.1, 2.0, 3.8),
        leitura("damageShare", 22, 2.4, 1.1, 2.7),
        leitura("visionPerMinute", 22, -0.2, 0.08),
        leitura("deaths", 22, -0.3, 0.5, -0.5),
      ],
      matchups: [
        { opponent: ZED, games: 4, readings: [leitura("cs10", 4, -4.0, 3.5, -9.5)] },
        { opponent: YASUO, games: 3, readings: [leitura("deaths", 3, 1.9, 0.8, 2.7)] },
        { opponent: 105, games: 1, readings: [leitura("cs10", 1, 5.0, 4.5, 14.0)] },
      ],
    },
    {
      champion: 61,
      games: 11,
      readings: [
        leitura("cs10", 11, 1.0, 2.2, 0.2),
        leitura("killParticipation", 11, 1.0, 2.0),
        leitura("damageShare", 11, 1.5, 1.2),
        leitura("visionPerMinute", 11, -0.2, 0.08),
        leitura("deaths", 11, 1.4, 0.6, 1.8),
      ],
      matchups: [{ opponent: 84, games: 2, readings: [leitura("cs10", 2, -3.0, 3.9, -11.0)] }],
    },
    {
      champion: 112,
      games: 7,
      readings: [
        leitura("cs10", 7, -2.0, 2.6, -4.1),
        leitura("killParticipation", 7, 1.5, 2.1),
        leitura("damageShare", 7, 1.2, 1.3),
        leitura("visionPerMinute", 7, -0.2, 0.09),
        leitura("deaths", 7, 0.8, 0.6, 1.1),
      ],
      matchups: [],
    },
  ],
};

const ABAS: { id: View; label: string }[] = [
  { id: "draft", label: "Draft" },
  { id: "simulador", label: "Simulador" },
  { id: "lanes", label: "Rotas" },
  { id: "live", label: "Partida" },
  { id: "coach", label: "Pós-jogo" },
  { id: "pool", label: "Pool" },
  { id: "overlay", label: "Ajustes" },
];

function Demo() {
  const [view, setView] = useState<View>("draft");
  const [staticData, setStaticData] = useState<StaticData | null>(null);
  const [pool, setPool] = useState<Pool>(initialPool);
  const [overlayPrefs, setOverlayPrefs] = useState<OverlayPrefs>({ hidden: [] });
  const [seconds, setSeconds] = useState(21);
  const [marcadas, setMarcadas] = useState<Record<number, Role>>({});
  const [todos, setTodos] = useState(false);
  const [simulado, setSimulado] = useState<DraftState>(draftVazio);

  useEffect(() => {
    loadStaticData().then(setStaticData).catch(() => void 0);
    const ticker = window.setInterval(
      () => setSeconds((value) => (value <= 0 ? 30 : value - 1)),
      1000
    );
    return () => window.clearInterval(ticker);
  }, []);

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <Logo size={22} />
          <span className="brand-mark">PRESAGE</span>
          <span className="brand-tag">patch {engine.version} · demonstração</span>
        </div>
        <nav className="nav">
          {ABAS.map((aba) => (
            <button
              type="button"
              key={aba.id}
              className={view === aba.id ? "on" : ""}
              onClick={() => setView(aba.id)}
            >
              {aba.label}
            </button>
          ))}
        </nav>
        <ConnectionStatus status={status} />
      </header>

      <div className="banner banner-warn">
        Dados de exemplo. Esta página existe para olhar a interface sem o cliente
        da Riot aberto — o app real preenche tudo isto sozinho.
      </div>

      {view === "pool" ? (
        <main className="layout layout-single">
          <section className="panel">
            <PoolBuilder
              pool={pool}
              staticData={staticData}
              onChange={setPool}
              onSuggest={async () => planoDemo}
            />
          </section>
        </main>
      ) : view === "overlay" ? (
        <main className="layout layout-single">
          <section className="panel">
            <OverlayPrefsPanel prefs={overlayPrefs} onChange={setOverlayPrefs} />
          </section>
        </main>
      ) : view === "simulador" ? (
        <main className="layout">
          <section className="panel panel-draft">
            <SimuladorDraft value={simulado} staticData={staticData} onChange={setSimulado} live={draft} />
          </section>
          <section className="panel panel-reco">
            <RecommendationPanel
              recommendations={recommendations}
              minimax={minimax}
              bans={bans}
              staticData={staticData}
              draft={simulado}
              engine={engine}
              onPreview={async (id) => previaDemo(id)}
            />
          </section>
        </main>
      ) : view === "lanes" ? (
        <main className="layout layout-single">
          <section className="panel">
            <LaneAnalysis report={laneReport} staticData={staticData} />
            <Runas plan={runasDemo} patch={engine.version} />
          </section>
        </main>
      ) : view === "coach" ? (
        <main className="layout layout-single">
          {/* Uma seção só: duas dividiriam a tela ao meio e cortariam o gráfico
              do pós-jogo, cada metade com a própria rolagem. */}
          <section className="panel">
            <PostGame report={review} />
            <PerfilDeRota profile={perfilDemo} staticData={staticData} />
          </section>
        </main>
      ) : view === "live" ? (
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
            />
          </section>
        </main>
      ) : (
        <main className="layout">
          <section className="panel panel-draft">
            <DraftBoard
              draft={draft}
              staticData={staticData}
              secondsLeft={seconds}
              connected
              phase="ChampSelect"
              inferred={{ [MALPHITE]: "Top", [VIEGO]: "Jungle", [ZED]: "Mid" }}
              marked={marcadas}
              onEnemyRole={(id, role) =>
                setMarcadas((atual) => {
                  const proximo = { ...atual };
                  if (role) proximo[id] = role;
                  else delete proximo[id];
                  return proximo;
                })
              }
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
              onPreview={async (id) => previaDemo(id)}
              showAll={todos}
              onShowAll={setTodos}
            />
          </section>
        </main>
      )}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Demo />
  </React.StrictMode>
);
