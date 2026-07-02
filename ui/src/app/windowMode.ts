// Which window this WebView is running in, decided once at boot from the
// URL Rust used to create the window.
//
// The main window loads plain `index.html`. A per-session monitor window
// loads `index.html?monitor=1`: a constant flag with no user data, set only
// by the Rust `open_session_window` command. In a monitor window the app
// renders the read-only session monitor and never mounts the main app, the
// sidebar, settings, menus, or anything that writes shared UI state.
//
// The monitor window learns which session it shows from Rust
// (`get_session_window_target`, keyed by the window's own label), never from
// the URL. The `session` query parameter below exists only for the browser
// preview and tests, where there is no Tauri runtime and no window label.

// Opaque-token shape shared with uiStatePersistence: letters, digits, dash,
// and underscore only, bounded length. Anything path- or secret-shaped is
// rejected outright.
export const OPAQUE_TOKEN = /^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/

export function isSessionMonitorWindow(): boolean {
  try {
    return new URLSearchParams(window.location.search).has("monitor")
  } catch {
    return false
  }
}

// Browser-preview fallback for the monitored session id (?session=<id>).
// Under Tauri this value is never consulted; Rust answers from the window
// label instead. Only opaque-token ids are accepted so a hand-crafted URL
// cannot inject markup- or path-shaped text into the monitor view.
export function monitorPreviewSessionId(): string | null {
  try {
    const id = new URLSearchParams(window.location.search).get("session")
    return id && OPAQUE_TOKEN.test(id) ? id : null
  } catch {
    return null
  }
}
