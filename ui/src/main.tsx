import { createRoot } from "react-dom/client"

import App from "@/app/App"
import { installSystemAppearance } from "@/app/systemAppearance"
import { installDiagnosticLogging } from "@/commands/logging"
import "./index.css"

installSystemAppearance()

// Dev/test log trail: forward console output, window errors, and stray promise
// rejections into the local app log file. The browser preview installs nothing.
if (import.meta.env.DEV) {
  installDiagnosticLogging()
}

createRoot(document.getElementById("app") as HTMLElement).render(<App />)
