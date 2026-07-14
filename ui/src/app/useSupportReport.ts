import { useRef, useState } from "react"

import {
  createSupportReport,
  hasTauriRuntime,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

export type SupportReportState = {
  busy: boolean
  message: string
  error: string
  create: () => Promise<void>
}

export function useSupportReport(): SupportReportState {
  const busyRef = useRef(false)
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState("")
  const [error, setError] = useState("")

  async function create() {
    if (busyRef.current) return
    setMessage("")
    setError("")
    if (!hasTauriRuntime()) {
      setError("Open the RunHaven desktop app or use runhaven support to create a report.")
      return
    }
    busyRef.current = true
    setBusy(true)
    try {
      const response = await createSupportReport()
      if (!response) return
      if (response.issueOpened && response.finderRevealed) {
        setMessage(
          "Support report created. Review the selected file in Finder before you drag it into the public GitHub form. GitHub uploads it when you attach it. You decide whether to submit the issue.",
        )
      } else if (response.finderRevealed) {
        setError(
          "The support report was created and selected in Finder, but GitHub did not open. Open the RunHaven support form in your browser, then attach the selected file.",
        )
      } else if (response.issueOpened) {
        setError(
          `The GitHub support form opened, but Finder could not select ${response.fileName}. Run runhaven support in Terminal to create a report with a visible path.`,
        )
      } else {
        setError(
          "The support report was created, but RunHaven could not open Finder or GitHub. Run runhaven support in Terminal to create a report with a visible path.",
        )
      }
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      busyRef.current = false
      setBusy(false)
    }
  }

  return { busy, message, error, create }
}
