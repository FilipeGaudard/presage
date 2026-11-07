// Os comandos do app, cada um com a sua permissão `allow-…` (A52). Com o
// manifesto, uma janela só alcança o comando que a capability dela libera: a
// principal libera todos, o overlay só os que ele chama. A lista é a de
// `generate_handler!` em `src/lib.rs`, e o teste de arquitetura confere as duas.
const COMANDOS: &[&str] = &[
    "get_snapshot",
    "engine_status",
    "lane_profile",
    "rank_draft",
    "set_draft_role",
    "set_enemy_role",
    "get_enemy_roles",
    "set_simulated_draft",
    "get_draft_prefs",
    "set_draft_prefs",
    "minimax_draft",
    "ban_suggestions",
    "get_pool",
    "set_pool",
    "get_live",
    "build_suggestions",
    "lane_analysis",
    "skirmish_focus",
    "gold_report",
    "last_review",
    "pool_suggestions",
    "rune_plan",
    "get_overlay_prefs",
    "set_overlay_prefs",
    "get_autostart",
    "set_autostart",
];

fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(COMANDOS)))
        .expect("tauri-build");
}
