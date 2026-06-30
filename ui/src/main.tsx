import { createRoot } from "react-dom/client"

import App from "@/app/App"
import "./index.css"

createRoot(document.getElementById("app") as HTMLElement).render(<App />)
