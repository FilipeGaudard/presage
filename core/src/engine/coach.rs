//! Leitura da partida depois que ela acaba.
//!
//! Não depende da chave da Riot. Enquanto o jogo roda, o Presage grava um
//! quadro por segundo da Live Client Data API; no fim, estas funções leem a
//! gravação e dizem o que foi bem e o que foi mal.
//!
//! Cada observação precisa ser verificável pela própria pessoa: um número que
//! ela poderia ter conferido na aba de pontuação. Nada de "jogue mais seguro".
//!
//! A rota decide o que se cobra (`cobranca`). Farm contra o oponente direto é
//! trabalho do topo, da selva, do meio e do atirador; do suporte, não — cobrar
//! tropas de um suporte contra o outro é mandar fazer o trabalho de outra pessoa.
//! O suporte é cobrado pela visão contra o outro suporte, e a participação em
//! abates pesa diferente em cada rota.

use serde::{Deserialize, Serialize};

use super::champion::Role;
use super::gold::earned_at;
use super::items::ItemIndex;
use crate::live::{LiveGame, LivePlayer};

/// A partir deste custo o item conta como compra grande.
const LEGENDARY_COST: f64 = 2500.0;

/// Janela para considerar que as mortes vieram em sequência.
const CLUSTER_WINDOW: f64 = 240.0;
const CLUSTER_DEATHS: usize = 3;

/// Acima disto, andar com ouro no bolso vira observação.
const IDLE_GOLD: f64 = 1300.0;

/// Partida curta demais não rende leitura nenhuma.
const MIN_DURATION: f64 = 300.0;

/// O que cada rota tem como trabalho, e portanto o que o relatório cobra dela.
///
/// Escrito à mão: regra, não estatística. Os limiares de participação seguem o
/// que cada rota costuma ter — o suporte e a selva vivem de entrar nas brigas, o
/// topo passa boa parte do jogo sozinho na rota.
struct Cobranca {
    /// Farm contra o oponente direto é trabalho da rota.
    farm: bool,
    /// Visão contra o oponente direto é trabalho da rota.
    vision: bool,
    /// Participação abaixo disto é ponto fraco; acima de `kp_good`, forte.
    kp_bad: f64,
    kp_good: f64,
}

fn cobranca(role: Role) -> Cobranca {
    let (farm, vision, kp_bad, kp_good) = match role {
        Role::Support => (false, true, 0.50, 0.75),
        Role::Jungle => (true, false, 0.45, 0.70),
        Role::Top => (true, false, 0.30, 0.60),
        Role::Mid | Role::Bot => (true, false, 0.40, 0.65),
        // Sem rota (ARAM, partida sem posição) não há oponente direto.
        Role::Unknown => (false, false, 0.40, 0.65),
    };
    Cobranca {
        farm,
        vision,
        kp_bad,
        kp_good,
    }
}

fn sem_rota() -> Role {
    Role::Unknown
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Recording {
    /// Um quadro por amostragem, em ordem de tempo.
    pub frames: Vec<LiveGame>,
    /// Preenchido quando o resultado é conhecido. A leitura não depende disso.
    pub won: Option<bool>,
}

impl Recording {
    pub fn push(&mut self, game: LiveGame) {
        // O relógio pode repetir entre duas leituras. Quadro repetido só
        // engorda o arquivo e distorce a conta de tempo morto.
        if let Some(last) = self.frames.last() {
            if game.game_time <= last.game_time {
                return;
            }
        }
        self.frames.push(game);
    }

    pub fn duration(&self) -> f64 {
        match (self.frames.first(), self.frames.last()) {
            (Some(a), Some(b)) => b.game_time - a.game_time,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    Good,
    Bad,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub verdict: Verdict,
    pub title: String,
    pub detail: String,
    /// Quanto a observação pesa, de 0 a 1. Serve para ordenar.
    pub weight: f64,
}

/// Um ponto de uma curva: instante da partida e valor.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub t: f64,
    pub v: f64,
}

/// As curvas que a gravação permite desenhar.
///
/// Reamostradas a cada [`BUCKET`] segundos. Um quadro por segundo numa partida
/// de 40 minutos dá 2400 pontos, e nenhum gráfico de 600 pixels mostra 2400
/// pontos — só engorda o arquivo e a tela.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    /// Ouro em itens do seu time menos o deles. Exato: sai dos preços.
    pub gold_lead: Vec<Point>,
    /// Suas tropas menos as do oponente direto.
    pub cs_diff: Vec<Point>,
    /// Seu placar de visão menos o do oponente direto. É a curva do suporte.
    #[serde(default)]
    pub vision_diff: Vec<Point>,
    pub level_diff: Vec<Point>,
    /// Instantes em que você morreu, para marcar na linha do tempo.
    pub deaths: Vec<f64>,
    /// E em que seu time abateu alguém.
    pub team_kills: Vec<f64>,
}

impl Series {
    pub fn is_empty(&self) -> bool {
        self.gold_lead.is_empty()
    }
}

/// Distância entre pontos das curvas, em segundos.
const BUCKET: f64 = 30.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub champion: String,
    /// A rota da partida. Decide o que foi cobrado e qual curva aparece.
    #[serde(default = "sem_rota")]
    pub role: Role,
    pub duration: f64,
    pub won: Option<bool>,
    pub good: Vec<Note>,
    pub bad: Vec<Note>,
    pub summary: String,
    /// As curvas por trás das observações. Cada frase do relatório deve poder
    /// ser conferida olhando o gráfico.
    #[serde(default)]
    pub series: Series,
}

fn good(title: impl Into<String>, detail: impl Into<String>, weight: f64) -> Note {
    Note {
        verdict: Verdict::Good,
        title: title.into(),
        detail: detail.into(),
        weight,
    }
}

fn bad(title: impl Into<String>, detail: impl Into<String>, weight: f64) -> Note {
    Note {
        verdict: Verdict::Bad,
        title: title.into(),
        detail: detail.into(),
        weight,
    }
}

fn clock(seconds: f64) -> String {
    let s = seconds.max(0.0).round() as i64;
    format!("{}:{:02}", s / 60, s % 60)
}

fn me_in(frame: &LiveGame) -> Option<&LivePlayer> {
    frame.players.iter().find(|p| p.is_local)
}

/// O adversário direto: mesmo posto, time contrário.
fn opponent_in<'a>(frame: &'a LiveGame, me: &LivePlayer) -> Option<&'a LivePlayer> {
    if me.position.is_empty() {
        return None;
    }
    frame
        .players
        .iter()
        .find(|p| p.team != me.team && p.position == me.position)
}

/// Lê a gravação inteira e devolve o que dá para afirmar.
///
/// Sem jogador local, sem quadros ou com partida curta demais devolve um
/// relatório vazio em vez de inventar observação.
pub fn review(recording: &Recording, items: &ItemIndex) -> Report {
    let duration = recording.duration();
    let last = recording.frames.last();
    let me = last.and_then(me_in);

    let role = me.map_or(Role::Unknown, |m| Role::from_lcu(&m.position));
    let mut report = Report {
        champion: me.map(|m| m.champion_key.clone()).unwrap_or_default(),
        role,
        duration,
        won: recording.won,
        good: Vec::new(),
        bad: Vec::new(),
        summary: String::new(),
        series: Series::default(),
    };

    let (Some(last), Some(me)) = (last, me) else {
        report.summary = "Sem gravação desta partida.".into();
        return report;
    };

    if duration < MIN_DURATION {
        report.summary = format!(
            "Partida de {} — curta demais para tirar conclusão.",
            clock(duration)
        );
        return report;
    }

    report.series = series(recording);

    let minutes = duration / 60.0;
    let mut notes = Vec::new();

    let regra = cobranca(role);
    if regra.farm {
        notes.extend(farm(last, me, minutes));
    }
    if regra.vision {
        notes.extend(vision(last, me, minutes));
    }
    notes.extend(participation(last, me, &regra));
    notes.extend(deaths(recording, me));
    notes.extend(death_clusters(recording));
    notes.extend(first_legendary(recording, items));
    notes.extend(idle_gold(recording));
    notes.extend(lead_swing(recording));
    notes.extend(levels(last, me));

    for note in notes {
        match note.verdict {
            Verdict::Good => report.good.push(note),
            Verdict::Bad => report.bad.push(note),
        }
    }

    report.good.sort_by(|a, b| b.weight.total_cmp(&a.weight));
    report.bad.sort_by(|a, b| b.weight.total_cmp(&a.weight));

    report.summary = format!(
        "{} de {}{} · {} a corrigir, {} a manter.",
        clock(duration),
        if report.champion.is_empty() {
            "partida".into()
        } else {
            report.champion.clone()
        },
        if role == Role::Unknown {
            String::new()
        } else {
            format!(" ({})", role.short())
        },
        report.bad.len(),
        report.good.len()
    );

    report
}

/// As curvas da partida, reamostradas.
///
/// Toda frase do relatório tem de poder ser conferida aqui. "Quatro mortes
/// entre 21:40 e 24:10" vira quatro marcas na linha do tempo, e a vantagem que
/// sumiu vira uma curva que sobe e desce — que é mais convincente que a frase.
fn series(recording: &Recording) -> Series {
    let mut out = Series::default();
    let mut proximo = f64::NEG_INFINITY;
    let mut mortes_antes = 0;
    let mut abates_antes = 0;

    for frame in &recording.frames {
        let Some(me) = me_in(frame) else { continue };
        let t = frame.game_time;

        // Eventos entram com o instante exato; só as curvas são reamostradas.
        if me.scores.deaths > mortes_antes {
            for _ in mortes_antes..me.scores.deaths {
                out.deaths.push(t);
            }
            mortes_antes = me.scores.deaths;
        }

        let abates: i64 = frame
            .players
            .iter()
            .filter(|p| p.team == me.team)
            .map(|p| p.scores.kills)
            .sum();
        if abates > abates_antes {
            for _ in abates_antes..abates {
                out.team_kills.push(t);
            }
            abates_antes = abates;
        }

        if t < proximo {
            continue;
        }
        proximo = t + BUCKET;

        let (nosso, deles) = frame.gold_invested();
        out.gold_lead.push(Point {
            t,
            v: nosso - deles,
        });

        if let Some(rival) = opponent_in(frame, me) {
            out.cs_diff.push(Point {
                t,
                v: (me.scores.creep_score - rival.scores.creep_score) as f64,
            });
            out.vision_diff.push(Point {
                t,
                v: me.scores.ward_score - rival.scores.ward_score,
            });
            out.level_diff.push(Point {
                t,
                v: f64::from(me.level) - f64::from(rival.level),
            });
        }
    }

    out
}

/// Tropas por minuto contra o adversário direto.
fn farm(last: &LiveGame, me: &LivePlayer, minutes: f64) -> Vec<Note> {
    let meu = me.scores.creep_score as f64 / minutes;

    let Some(rival) = opponent_in(last, me) else {
        return vec![];
    };
    let dele = rival.scores.creep_score as f64 / minutes;
    let diff_cs = me.scores.creep_score - rival.scores.creep_score;
    let ouro = (diff_cs as f64 * 21.0).abs();

    if dele - meu >= 0.8 {
        return vec![bad(
            format!("{:.1} tropas por minuto contra {:.1}", meu, dele),
            format!(
                "{} terminou com {} tropas a mais. São ~{:.0} de ouro que você não pegou.",
                rival.champion_key,
                -diff_cs,
                ouro
            ),
            ((dele - meu) / 3.0).min(1.0),
        )];
    }

    if meu - dele >= 0.5 {
        return vec![good(
            format!("{:.1} tropas por minuto contra {:.1}", meu, dele),
            format!(
                "Você fechou {} tropas a mais que {}, ~{:.0} de ouro.",
                diff_cs, rival.champion_key, ouro
            ),
            ((meu - dele) / 3.0).min(1.0),
        )];
    }

    vec![]
}

/// Placar de visão por minuto contra o oponente direto. Para o suporte, é o
/// que o farm é para as outras rotas.
fn vision(last: &LiveGame, me: &LivePlayer, minutes: f64) -> Vec<Note> {
    let Some(rival) = opponent_in(last, me) else {
        return vec![];
    };
    let meu = me.scores.ward_score / minutes;
    let dele = rival.scores.ward_score / minutes;
    let diferenca = (me.scores.ward_score - rival.scores.ward_score).abs();

    if dele - meu >= 0.4 {
        return vec![bad(
            format!("{meu:.1} de visão por minuto contra {dele:.1}"),
            format!(
                "{} terminou com {diferenca:.0} pontos de visão a mais. Visão é o que deixa o time ver a briga antes dela acontecer.",
                rival.champion_key
            ),
            ((dele - meu) / 1.5).min(1.0),
        )];
    }
    if meu - dele >= 0.3 {
        return vec![good(
            format!("{meu:.1} de visão por minuto contra {dele:.1}"),
            format!(
                "Você fechou {diferenca:.0} pontos de visão a mais que {}.",
                rival.champion_key
            ),
            ((meu - dele) / 1.5).min(1.0),
        )];
    }
    vec![]
}

/// Fatia dos abates do time em que você entrou, medida contra o que a rota
/// costuma ter.
fn participation(last: &LiveGame, me: &LivePlayer, regra: &Cobranca) -> Vec<Note> {
    let total: i64 = last
        .players
        .iter()
        .filter(|p| p.team == me.team)
        .map(|p| p.scores.kills)
        .sum();

    if total < 5 {
        return vec![];
    }

    let minha = me.scores.kills + me.scores.assists;
    let share = minha as f64 / total as f64;

    if share >= regra.kp_good {
        return vec![good(
            format!("{:.0}% dos abates do time", share * 100.0),
            format!("Você entrou em {minha} dos {total} abates."),
            share,
        )];
    }
    if share <= regra.kp_bad {
        return vec![bad(
            format!("{:.0}% dos abates do time", share * 100.0),
            format!(
                "Seu time matou {total} vezes e você estava em {minha}. O jogo aconteceu sem você."
            ),
            1.0 - share,
        )];
    }
    vec![]
}

/// Mortes e tempo passado esperando para voltar.
fn deaths(recording: &Recording, me: &LivePlayer) -> Vec<Note> {
    let mortes = me.scores.deaths;
    if mortes == 0 {
        return vec![good(
            "Nenhuma morte",
            "Partida inteira sem entregar abate.",
            1.0,
        )];
    }

    // Integra os trechos em que você estava morto, usando o intervalo real
    // entre os quadros em vez de supor a taxa de amostragem.
    let mut morto = 0.0;
    for par in recording.frames.windows(2) {
        let dt = par[1].game_time - par[0].game_time;
        if dt <= 0.0 || dt > 10.0 {
            continue; // buraco na gravação: não dá para contar o que não vimos
        }
        if me_in(&par[0]).is_some_and(|p| p.is_dead) {
            morto += dt;
        }
    }

    if morto <= 0.0 {
        return vec![];
    }

    let fatia = morto / recording.duration().max(1.0);
    if fatia >= 0.12 {
        return vec![bad(
            format!("{} esperando para voltar", clock(morto)),
            format!(
                "{:.0}% da partida, em {mortes} mortes. É o tempo em que o mapa jogou com um a menos.",
                fatia * 100.0
            ),
            fatia.min(1.0),
        )];
    }
    vec![]
}

/// Mortes amontoadas numa janela curta.
fn death_clusters(recording: &Recording) -> Vec<Note> {
    let mut momentos = Vec::new();
    let mut anterior = 0;

    for frame in &recording.frames {
        let Some(me) = me_in(frame) else { continue };
        if me.scores.deaths > anterior {
            for _ in anterior..me.scores.deaths {
                momentos.push(frame.game_time);
            }
            anterior = me.scores.deaths;
        }
    }

    if momentos.len() < CLUSTER_DEATHS {
        return vec![];
    }

    // Maior aglomerado dentro da janela.
    let mut melhor = (0usize, 0.0, 0.0);
    for (i, &inicio) in momentos.iter().enumerate() {
        let fim = inicio + CLUSTER_WINDOW;
        let quantas = momentos[i..].iter().take_while(|&&t| t <= fim).count();
        if quantas > melhor.0 {
            let ultima = momentos[i + quantas - 1];
            melhor = (quantas, inicio, ultima);
        }
    }

    if melhor.0 < CLUSTER_DEATHS {
        return vec![];
    }

    vec![bad(
        format!(
            "{} mortes entre {} e {}",
            melhor.0,
            clock(melhor.1),
            clock(melhor.2)
        ),
        format!(
            "{} das suas {} mortes saíram nessa janela. Foi ali que a partida virou contra você.",
            melhor.0,
            momentos.len()
        ),
        (melhor.0 as f64 / momentos.len() as f64).min(1.0),
    )]
}

/// Quando saiu o primeiro item grande.
fn first_legendary(recording: &Recording, items: &ItemIndex) -> Vec<Note> {
    let quando = recording.frames.iter().find_map(|frame| {
        let me = me_in(frame)?;
        me.items
            .iter()
            .any(|id| items.get(*id).is_some_and(|item| item.cost >= LEGENDARY_COST))
            .then_some(frame.game_time)
    });

    let Some(t) = quando else {
        // Sem índice de itens carregado não dá para afirmar nada; calar é
        // melhor que afirmar que você não comprou.
        return vec![];
    };

    if t >= 900.0 {
        return vec![bad(
            format!("Primeiro item grande aos {}", clock(t)),
            "Depois dos 15 minutos o jogo já está decidindo brigas. Vale rever a rota de ouro até lá.",
            ((t - 900.0) / 600.0).min(1.0),
        )];
    }
    if t <= 660.0 {
        return vec![good(
            format!("Primeiro item grande aos {}", clock(t)),
            "Chegou cedo no primeiro pico de poder.",
            (660.0 - t) / 400.0,
        )];
    }
    vec![]
}

/// Tempo andando com ouro suficiente para comprar.
fn idle_gold(recording: &Recording) -> Vec<Note> {
    let mut parado = 0.0;
    let mut pico: f64 = 0.0;

    for par in recording.frames.windows(2) {
        let dt = par[1].game_time - par[0].game_time;
        if dt <= 0.0 || dt > 10.0 {
            continue;
        }
        let Some(me) = me_in(&par[0]) else { continue };
        if me.is_dead {
            continue; // morto não compra
        }

        let sobra = earned_at(
            par[0].game_time,
            me.scores.creep_score,
            me.scores.kills,
            me.scores.assists,
            me.gold_invested,
        ) - me.gold_invested;

        if sobra >= IDLE_GOLD {
            parado += dt;
            pico = pico.max(sobra);
        }
    }

    if parado >= 120.0 {
        return vec![bad(
            format!("{} com ouro parado", clock(parado)),
            format!(
                "Chegou a ~{:.0} sem gastar. É estimativa, não medida — a API só entrega o ouro em mão do jogador local.",
                pico
            ),
            (parado / 600.0).min(1.0),
        )];
    }
    vec![]
}

/// Vantagem de ouro do time que existiu e sumiu.
fn lead_swing(recording: &Recording) -> Vec<Note> {
    let mut pico = (0.0f64, 0.0f64); // (vantagem, instante)
    let mut fim = 0.0;

    for frame in &recording.frames {
        if me_in(frame).is_none() {
            continue;
        }
        let (nosso, deles) = frame.gold_invested();
        let lead = nosso - deles;
        fim = lead;
        if lead > pico.0 {
            pico = (lead, frame.game_time);
        }
    }

    if pico.0 >= 3000.0 && fim <= pico.0 - 3000.0 {
        return vec![bad(
            format!("{:.0} de vantagem aos {}", pico.0, clock(pico.1)),
            format!(
                "A partida terminou com {:.0}. A vantagem existiu e foi devolvida.",
                fim
            ),
            ((pico.0 - fim) / 8000.0).min(1.0),
        )];
    }
    if fim >= 3000.0 {
        return vec![good(
            format!("{:.0} de vantagem em itens no fim", fim),
            "Seu time fechou a partida mais rico do que começou a briga.",
            (fim / 8000.0).min(1.0),
        )];
    }
    vec![]
}

/// Nível contra o adversário direto no fim.
fn levels(last: &LiveGame, me: &LivePlayer) -> Vec<Note> {
    let Some(rival) = opponent_in(last, me) else {
        return vec![];
    };
    let diff = me.level as i16 - rival.level as i16;

    if diff <= -2 {
        return vec![bad(
            format!("{} níveis atrás de {}", -diff, rival.champion_key),
            "Nível é vida, dano e habilidade. Dois de diferença decidem a briga sozinhos.",
            (-diff as f64 / 5.0).min(1.0),
        )];
    }
    if diff >= 2 {
        return vec![good(
            format!("{} níveis à frente de {}", diff, rival.champion_key),
            "Você fechou a partida com a vantagem de experiência.",
            (diff as f64 / 5.0).min(1.0),
        )];
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::items::{parse_stats, Item};
    use crate::live::RawScores;
    use crate::engine::champion::Role;
    use std::collections::HashMap;

    fn items() -> ItemIndex {
        let mut by_id = HashMap::new();
        by_id.insert(
            3089,
            Item {
                id: 3089,
                name: "Rabadon".into(),
                cost: 3600.0,
                purchasable: true,
                stats: parse_stats("<stats><attention>130</attention> Ability Power</stats>"),
            },
        );
        by_id.insert(
            1056,
            Item {
                id: 1056,
                name: "Anel Doran".into(),
                cost: 400.0,
                purchasable: true,
                stats: parse_stats("<stats><attention>15</attention> Ability Power</stats>"),
            },
        );
        ItemIndex {
            version: "test".into(),
            by_id,
        }
    }

    struct Quadro {
        t: f64,
        cs: i64,
        kills: i64,
        deaths: i64,
        assists: i64,
        level: u8,
        dead: bool,
        gold: f64,
        items: Vec<i64>,
        rival_cs: i64,
        rival_kills: i64,
        rival_level: u8,
        rival_gold: f64,
        /// A rota dos dois, como a Live Client escreve.
        position: &'static str,
        ward: f64,
        rival_ward: f64,
        /// Abates de um aliado, para a participação ter time.
        ally_kills: i64,
    }

    impl Default for Quadro {
        fn default() -> Self {
            Quadro {
                t: 0.0,
                cs: 0,
                kills: 0,
                deaths: 0,
                assists: 0,
                level: 1,
                dead: false,
                gold: 0.0,
                items: vec![],
                rival_cs: 0,
                rival_kills: 0,
                rival_level: 1,
                rival_gold: 0.0,
                position: "MIDDLE",
                ward: 0.0,
                rival_ward: 0.0,
                ally_kills: 0,
            }
        }
    }

    fn frame(q: Quadro) -> LiveGame {
        let position = q.position;
        let mk = |key: &str, team: &str, local: bool, level: u8, gold: f64, items: Vec<i64>, s: RawScores, dead: bool| {
            LivePlayer {
                champion_key: key.into(),
                champion_id: 1,
                display_name: key.into(),
                team: team.into(),
                position: position.into(),
                level,
                items,
                gold_invested: gold,
                is_dead: dead,
                respawn_timer: 0.0,
                is_local: local,
                scores: s,
            }
        };

        LiveGame {
            game_time: q.t,
            players: vec![
                mk(
                    "Ahri",
                    "ORDER",
                    true,
                    q.level,
                    q.gold,
                    q.items,
                    RawScores {
                        kills: q.kills,
                        deaths: q.deaths,
                        assists: q.assists,
                        creep_score: q.cs,
                        ward_score: q.ward,
                    },
                    q.dead,
                ),
                mk(
                    "Zed",
                    "CHAOS",
                    false,
                    q.rival_level,
                    q.rival_gold,
                    vec![],
                    RawScores {
                        kills: q.rival_kills,
                        deaths: 0,
                        assists: 0,
                        creep_score: q.rival_cs,
                        ward_score: q.rival_ward,
                    },
                    false,
                ),
                LivePlayer {
                    champion_key: "Aliado".into(),
                    champion_id: 2,
                    display_name: "Aliado".into(),
                    team: "ORDER".into(),
                    position: "JUNGLE".into(),
                    level: q.level,
                    items: vec![],
                    gold_invested: 0.0,
                    is_dead: false,
                    respawn_timer: 0.0,
                    is_local: false,
                    scores: RawScores {
                        kills: q.ally_kills,
                        deaths: 0,
                        assists: 0,
                        creep_score: 0,
                        ward_score: 0.0,
                    },
                },
            ],
        }
    }

    /// Gravação de 30 minutos, um quadro por segundo, com um molde por quadro.
    fn gravar(f: impl Fn(f64) -> Quadro, ate: f64) -> Recording {
        let mut r = Recording::default();
        let mut t = 0.0;
        while t <= ate {
            r.push(frame(f(t)));
            t += 1.0;
        }
        r
    }

    fn reta(t: f64) -> Quadro {
        Quadro {
            t,
            cs: (t / 8.0) as i64,
            rival_cs: (t / 8.0) as i64,
            level: ((t / 120.0) as u8 + 1).min(18),
            rival_level: ((t / 120.0) as u8 + 1).min(18),
            gold: t * 3.0,
            rival_gold: t * 3.0,
            ..Default::default()
        }
    }

    #[test]
    fn quadro_com_relogio_parado_ou_voltando_e_descartado() {
        let mut r = Recording::default();
        r.push(frame(Quadro { t: 100.0, ..Default::default() }));
        r.push(frame(Quadro { t: 100.0, ..Default::default() }));
        r.push(frame(Quadro { t: 90.0, ..Default::default() }));
        r.push(frame(Quadro { t: 101.0, ..Default::default() }));

        assert_eq!(r.frames.len(), 2);
    }

    #[test]
    fn sem_gravacao_nao_ha_relatorio() {
        let r = review(&Recording::default(), &items());
        assert!(r.good.is_empty() && r.bad.is_empty());
        assert!(r.summary.contains("Sem gravação"));
    }

    #[test]
    fn partida_curta_nao_rende_conclusao() {
        let r = review(&gravar(reta, 120.0), &items());
        assert!(r.good.is_empty() && r.bad.is_empty());
        assert!(r.summary.contains("curta"));
    }

    #[test]
    fn farm_atras_do_rival_vira_ponto_negativo() {
        let rec = gravar(
            |t| Quadro {
                cs: (t / 12.0) as i64,
                rival_cs: (t / 7.0) as i64,
                ..reta(t)
            },
            1800.0,
        );
        let r = review(&rec, &items());

        assert!(
            r.bad.iter().any(|n| n.title.contains("tropas por minuto")),
            "{:?}",
            r.bad
        );
    }

    /// 30 minutos de suporte: o outro suporte farmou mais e pôs mais visão.
    fn suporte(t: f64) -> Quadro {
        let m = t / 60.0;
        Quadro {
            t,
            position: "UTILITY",
            cs: (m * 1.0) as i64,
            rival_cs: (m * 2.5) as i64,
            ward: m * 1.2,
            rival_ward: m * 2.0,
            level: 1 + (m / 3.0) as u8,
            rival_level: 1 + (m / 3.0) as u8,
            ..Quadro::default()
        }
    }

    #[test]
    fn suporte_nao_e_cobrado_por_farm() {
        // O caso real: farm de suporte contra suporte não é o trabalho de
        // nenhum dos dois, e o relatório não pode mandar farmar.
        let r = review(&gravar(suporte, 1800.0), &items());
        let tudo: Vec<&str> = r.bad.iter().chain(r.good.iter()).map(|n| n.title.as_str()).collect();
        assert!(!tudo.iter().any(|t| t.contains("tropas")), "{tudo:?}");
    }

    #[test]
    fn suporte_e_cobrado_pela_visao() {
        let r = review(&gravar(suporte, 1800.0), &items());
        assert!(
            r.bad.iter().any(|n| n.title.contains("visão")),
            "o outro suporte pôs mais visão: {:?}",
            r.bad
        );
    }

    #[test]
    fn a_curva_do_suporte_e_a_de_visao() {
        let r = review(&gravar(suporte, 1800.0), &items());
        assert_eq!(r.role, Role::Support);
        assert!(r.series.vision_diff.len() >= 2);
        assert!(r.series.vision_diff.last().unwrap().v < 0.0);
    }

    #[test]
    fn participacao_cobra_mais_de_quem_vive_dela() {
        // 9 de 20 abates: pouco para suporte, normal para o meio.
        let molde = |pos: &'static str| {
            move |t: f64| Quadro {
                t,
                position: pos,
                kills: if t >= 900.0 { 4 } else { 0 },
                assists: if t >= 900.0 { 5 } else { 0 },
                ally_kills: if t >= 900.0 { 16 } else { 0 },
                ..reta(t)
            }
        };
        let sup = review(&gravar(molde("UTILITY"), 1800.0), &items());
        let meio = review(&gravar(molde("MIDDLE"), 1800.0), &items());
        assert!(sup.bad.iter().any(|n| n.title.contains("abates")), "{:?}", sup.bad);
        assert!(!meio.bad.iter().any(|n| n.title.contains("abates")), "{:?}", meio.bad);
    }

    #[test]
    fn leitura_salva_antes_da_rota_ainda_carrega() {
        // O app guarda a última leitura em disco; a de antes não tem rota nem
        // curva de visão.
        let antiga = r#"{"champion":"Ahri","duration":1800.0,"won":true,"good":[],"bad":[],
            "summary":"30:00 de Ahri","series":{"goldLead":[],"csDiff":[],"levelDiff":[],
            "deaths":[],"teamKills":[]}}"#;
        let r: Report = serde_json::from_str(antiga).unwrap();
        assert_eq!(r.role, Role::Unknown);
        assert!(r.series.vision_diff.is_empty());
    }

    #[test]
    fn farm_na_frente_vira_ponto_positivo() {
        let rec = gravar(
            |t| Quadro {
                cs: (t / 6.0) as i64,
                rival_cs: (t / 12.0) as i64,
                ..reta(t)
            },
            1800.0,
        );
        let r = review(&rec, &items());

        assert!(r.good.iter().any(|n| n.title.contains("tropas por minuto")));
    }

    #[test]
    fn tempo_morto_sai_do_intervalo_real_entre_quadros() {
        // Morto dos 600 aos 660: um minuto cravado.
        let rec = gravar(
            |t| Quadro {
                dead: (600.0..660.0).contains(&t),
                deaths: if t >= 600.0 { 6 } else { 0 },
                ..reta(t)
            },
            1800.0,
        );
        let r = review(&rec, &items());

        // 60s em 1800 é 3,3%: abaixo do limiar, então não vira observação.
        assert!(!r.bad.iter().any(|n| n.title.contains("esperando")));

        let pesado = gravar(
            |t| Quadro {
                dead: t > 600.0 && (t as i64 / 30) % 2 == 0,
                deaths: 8,
                ..reta(t)
            },
            1800.0,
        );
        assert!(review(&pesado, &items())
            .bad
            .iter()
            .any(|n| n.title.contains("esperando")));
    }

    #[test]
    fn partida_sem_morte_vira_elogio() {
        let r = review(&gravar(reta, 1800.0), &items());
        assert!(r.good.iter().any(|n| n.title == "Nenhuma morte"));
    }

    #[test]
    fn mortes_em_sequencia_viram_uma_janela() {
        // Quatro mortes entre 14:00 e 16:00, uma solta aos 5:00.
        let rec = gravar(
            |t| Quadro {
                deaths: match t {
                    t if t >= 960.0 => 5,
                    t if t >= 900.0 => 4,
                    t if t >= 870.0 => 3,
                    t if t >= 840.0 => 2,
                    t if t >= 300.0 => 1,
                    _ => 0,
                },
                ..reta(t)
            },
            1800.0,
        );
        let r = review(&rec, &items());

        let janela = r
            .bad
            .iter()
            .find(|n| n.title.contains("mortes entre"))
            .expect("a janela deveria aparecer");
        assert!(janela.title.starts_with("4 mortes"), "{}", janela.title);
    }

    #[test]
    fn mortes_espalhadas_nao_viram_janela() {
        let rec = gravar(
            |t| Quadro {
                deaths: (t / 400.0) as i64,
                ..reta(t)
            },
            1800.0,
        );
        let r = review(&rec, &items());
        assert!(!r.bad.iter().any(|n| n.title.contains("mortes entre")));
    }

    #[test]
    fn item_grande_tardio_e_apontado() {
        let rec = gravar(
            |t| Quadro {
                items: if t >= 1200.0 { vec![3089] } else { vec![1056] },
                ..reta(t)
            },
            1800.0,
        );
        let r = review(&rec, &items());

        assert!(
            r.bad.iter().any(|n| n.title.contains("Primeiro item grande")),
            "{:?}",
            r.bad
        );
    }

    #[test]
    fn item_barato_nao_conta_como_item_grande() {
        let rec = gravar(
            |t| Quadro {
                items: vec![1056],
                ..reta(t)
            },
            1800.0,
        );
        let r = review(&rec, &items());

        assert!(!r.good.iter().any(|n| n.title.contains("item grande")));
        assert!(!r.bad.iter().any(|n| n.title.contains("item grande")));
    }

    #[test]
    fn vantagem_que_sumiu_e_apontada() {
        let rec = gravar(
            |t| Quadro {
                gold: if t <= 1200.0 { t * 6.0 } else { 7200.0 },
                rival_gold: if t <= 1200.0 { t * 1.0 } else { 1200.0 + (t - 1200.0) * 10.0 },
                ..reta(t)
            },
            1800.0,
        );
        let r = review(&rec, &items());

        assert!(
            r.bad.iter().any(|n| n.title.contains("de vantagem aos")),
            "{:?}",
            r.bad
        );
    }

    #[test]
    fn nivel_atras_do_rival_e_apontado() {
        let rec = gravar(
            |t| Quadro {
                level: ((t / 200.0) as u8 + 1).min(18),
                rival_level: ((t / 100.0) as u8 + 1).min(18),
                ..reta(t)
            },
            1800.0,
        );
        let r = review(&rec, &items());

        assert!(r.bad.iter().any(|n| n.title.contains("níveis atrás")));
    }

    #[test]
    fn as_observacoes_saem_da_mais_pesada_para_a_menos() {
        let rec = gravar(
            |t| Quadro {
                cs: (t / 15.0) as i64,
                rival_cs: (t / 6.0) as i64,
                deaths: (t / 150.0) as i64,
                dead: (t as i64 / 20) % 3 == 0,
                level: 10,
                rival_level: 16,
                ..reta(t)
            },
            1800.0,
        );
        let r = review(&rec, &items());

        assert!(r.bad.len() >= 2);
        for par in r.bad.windows(2) {
            assert!(par[0].weight >= par[1].weight);
        }
    }

    #[test]
    fn buraco_grande_na_gravacao_nao_conta_como_tempo_morto() {
        // Alt-tab, cliente travado, o que for: entre dois quadros passaram
        // cinco minutos. Não vimos nada ali, então não contamos nada.
        let mut rec = Recording::default();
        rec.push(frame(Quadro { t: 0.0, dead: true, deaths: 1, ..reta(0.0) }));
        rec.push(frame(Quadro { t: 300.0, dead: true, deaths: 1, ..reta(300.0) }));
        rec.push(frame(Quadro { t: 1800.0, deaths: 1, ..reta(1800.0) }));

        let r = review(&rec, &items());
        assert!(!r.bad.iter().any(|n| n.title.contains("esperando")));
    }

    #[test]
    fn as_curvas_saem_reamostradas_e_nao_quadro_a_quadro() {
        let rec = gravar(reta, 1800.0);
        let r = review(&rec, &items());

        assert_eq!(rec.frames.len(), 1801, "a gravação é por segundo");
        // 1800s a cada 30s dá 61 pontos, não 1801.
        assert_eq!(r.series.gold_lead.len(), 61, "{}", r.series.gold_lead.len());
        assert!(r.series.cs_diff.len() == r.series.gold_lead.len());
    }

    #[test]
    fn a_curva_de_ouro_acompanha_a_vantagem_real() {
        let rec = gravar(
            |t| Quadro {
                gold: t * 6.0,
                rival_gold: t * 1.0,
                ..reta(t)
            },
            1800.0,
        );
        let s = review(&rec, &items()).series;

        assert!(s.gold_lead[0].v < s.gold_lead.last().unwrap().v, "deveria subir");
        // Aos 1800s: 10800 contra 1800.
        assert!((s.gold_lead.last().unwrap().v - 9000.0).abs() < 1.0);
    }

    #[test]
    fn as_mortes_entram_com_o_instante_exato_nao_reamostradas() {
        let rec = gravar(
            |t| Quadro {
                deaths: if t >= 907.0 { 1 } else { 0 },
                ..reta(t)
            },
            1800.0,
        );
        let s = review(&rec, &items()).series;

        assert_eq!(s.deaths.len(), 1);
        assert_eq!(
            s.deaths[0], 907.0,
            "marcar a morte no múltiplo de 30 mais próximo moveria o evento"
        );
    }

    #[test]
    fn a_curva_de_tropas_mede_a_diferenca_contra_o_oponente() {
        let rec = gravar(
            |t| Quadro {
                cs: (t / 6.0) as i64,
                rival_cs: (t / 12.0) as i64,
                ..reta(t)
            },
            1800.0,
        );
        let s = review(&rec, &items()).series;

        assert!(s.cs_diff.last().unwrap().v > 100.0);
        assert!(s.cs_diff[0].v.abs() < 5.0, "começa empatado");
    }

    #[test]
    fn partida_curta_nao_rende_curva_nenhuma() {
        let r = review(&gravar(reta, 120.0), &items());
        assert!(r.series.is_empty());
    }

    #[test]
    fn os_abates_do_time_contam_uma_vez_cada() {
        let rec = gravar(
            |t| Quadro {
                kills: (t / 300.0) as i64,
                ..reta(t)
            },
            1800.0,
        );
        let s = review(&rec, &items()).series;
        assert_eq!(s.team_kills.len(), 6, "{:?}", s.team_kills);
    }

    #[test]
    fn participacao_baixa_nos_abates_e_apontada() {
        let mut rec = gravar(reta, 1800.0);
        // O time mata muito, você entra pouco.
        if let Some(last) = rec.frames.last_mut() {
            last.players.push(LivePlayer {
                champion_key: "Ashe".into(),
                champion_id: 2,
                display_name: "Ashe".into(),
                team: "ORDER".into(),
                position: "BOTTOM".into(),
                level: 16,
                items: vec![],
                gold_invested: 0.0,
                is_dead: false,
                respawn_timer: 0.0,
                is_local: false,
                scores: RawScores {
                    kills: 20,
                    deaths: 2,
                    assists: 1,
                    creep_score: 200,
                    ward_score: 0.0,
                },
            });
        }
        let r = review(&rec, &items());

        assert!(
            r.bad.iter().any(|n| n.title.contains("dos abates do time")),
            "{:?}",
            r.bad
        );
    }
}
