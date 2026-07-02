import { createRoot } from "react-dom/client"

import App from "@/app/App"
import SessionMonitorApp from "@/app/SessionMonitorApp"
import { isSessionMonitorWindow } from "@/app/windowMode"
import "./index.css"

// One bundle, two window roles, decided once at boot: the main window runs
// the full app; a per-session monitor window (opened by Rust with the
// constant ?monitor=1 flag) runs only the read-only session monitor.
createRoot(document.getElementById("app") as HTMLElement).render(
  isSessionMonitorWindow() ? <SessionMonitorApp /> : <App />,
)
