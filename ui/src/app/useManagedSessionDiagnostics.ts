import { useEffect, useState } from "react"

import {
  getManagedSessionDiagnostics,
  type ManagedSessionDiagnosticsResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

export function useManagedSessionDiagnostics(sessionId: string | null) {
  const [data, setData] = useState<ManagedSessionDiagnosticsResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")

  useEffect(() => {
    setData(null)
    setError("")
    setLoading(false)
  }, [sessionId])

  async function load() {
    if (!sessionId) {
      return
    }
    setLoading(true)
    setError("")
    try {
      setData(await getManagedSessionDiagnostics(sessionId))
    } catch (cause) {
      setData(null)
      setError(errorMessage(cause))
    } finally {
      setLoading(false)
    }
  }

  return { data, loading, error, load }
}
