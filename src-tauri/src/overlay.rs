//! Overlay sobre o jogo.
//!
//! Janela externa, transparente e sempre no topo. **Não há injeção em nada**:
//! o Vanguard bloqueia hook de render, e um overlay que precise disso está
//! morto antes de nascer. O custo é que o jogo precisa rodar em janela sem
//! bordas — em tela cheia exclusiva nenhuma janela externa aparece por cima.
//!
//! Por padrão o overlay é atravessável pelo mouse, senão ele comeria cliques no
//! meio da partida. `Ctrl+Shift+O` torna a janela clicável quando você quiser
//! mexer nela.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

pub const LABEL: &str = "overlay";

/// Falso = o mouse atravessa a janela, que é o estado de jogo.
static INTERACTIVE: AtomicBool = AtomicBool::new(false);

pub fn toggle_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyP)
}

pub fn interact_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyO)
}

/// Deixa a janela atravessável antes de qualquer exibição.
pub fn prepare(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.set_ignore_cursor_events(true);
    }
}

pub fn toggle_visible(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };

    match window.is_visible() {
        Ok(true) => {
            let _ = window.hide();
        }
        _ => {
            let _ = window.show();
            // Mostrar não pode roubar o foco do jogo.
            let _ = window.set_always_on_top(true);
        }
    }
}

pub fn toggle_interactive(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };

    let interactive = !INTERACTIVE.load(Ordering::Relaxed);
    INTERACTIVE.store(interactive, Ordering::Relaxed);
    let _ = window.set_ignore_cursor_events(!interactive);

    tracing::debug!(interactive, "overlay: modo de clique alternado");
}

pub fn on_shortcut(app: &AppHandle, shortcut: &Shortcut) {
    if shortcut == &toggle_shortcut() {
        toggle_visible(app);
    } else if shortcut == &interact_shortcut() {
        toggle_interactive(app);
    }
}
