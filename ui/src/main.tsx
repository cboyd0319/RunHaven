import { createRoot } from "react-dom/client"

import App from "@/app/App"
import SessionMonitorApp from "@/app/SessionMonitorApp"
import { isSessionMonitorWindow } from "@/app/windowMode"
import { installSystemAppearance } from "@/app/systemAppearance"
import { installDiagnosticLogging } from "@/commands/logging"
import "./index.css"

installSystemAppearance()

// Dev/test log trail: forward console output, window errors, and stray
// promise rejections into the local app log file so every run leaves a
// reviewable trace. Main window only: the `logging` capability does not
// reach monitor windows, and the browser preview (no Tauri runtime)
// installs nothing.
if (import.meta.env.DEV && !isSessionMonitorWindow()) {
  installDiagnosticLogging()
}

// One bundle, two window roles, decided once at boot: the main window runs
// the full app; a per-session monitor window (opened by Rust with the
// constant ?monitor=1 flag) runs only the read-only session monitor.
createRoot(document.getElementById("app") as HTMLElement).render(
  isSessionMonitorWindow() ? <SessionMonitorApp /> : <App />,
)
