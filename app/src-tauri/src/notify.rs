//! OS notifications for what needs attention while the window is not in front (design decision
//! 11): only when the user did not turn them off, and never while the window has focus.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

/// Shows a notification unless notifications are off or the window has focus. A click brings the
/// window to the front; that handler is registered once at start.
pub fn notify<R: Runtime>(app: &AppHandle<R>, enabled: bool, title: &str, body: &str) {
    if !enabled || window_focused(app) {
        return;
    }
    let _ = app.notification().builder().title(title).body(body).show();
}

/// Whether the main window exists and has focus; a closed window has none.
fn window_focused<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.get_webview_window("main").and_then(|window| window.is_focused().ok()).unwrap_or(false)
}
