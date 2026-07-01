import { useState } from "react"

import { getAuthLog, getAuthStatus, getEgressLog } from "@/commands/runhaven"

import { errorMessage, type DiagnosticsState } from "./types"

export function useDiagnosticsState() {
  const [data, setData] = useState<DiagnosticsState | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")

  async function load() {
    setLoading(true)
    setError("")
    try {
      const [authStatus, authLog, egressLog] = await Promise.all([
        getAuthStatus(),
        getAuthLog(),
        getEgressLog(),
      ])
      setData({ authStatus, authLog, egressLog })
    } catch (cause) {
      setData(null)
      setError(errorMessage(cause))
    } finally {
      setLoading(false)
    }
  }

  return { data, loading, error, load }
}
