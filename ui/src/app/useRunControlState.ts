import { useEffect, useState } from "react"

import type { RunControlAction } from "./types"

const emptyConfirmations: Record<RunControlAction, boolean> = {
  stop: false,
  kill: false,
  repair: false,
}

export function useRunControlState(controlTargetId: string | null) {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState("")
  const [message, setMessage] = useState("")
  const [confirmations, setConfirmations] =
    useState<Record<RunControlAction, boolean>>(emptyConfirmations)

  useEffect(() => {
    setConfirmations(emptyConfirmations)
    setError("")
    setMessage("")
  }, [controlTargetId])

  function setConfirmation(action: RunControlAction, checked: boolean) {
    setConfirmations((current) => ({
      ...current,
      [action]: checked,
    }))
  }

  return {
    busy,
    setBusy,
    error,
    setError,
    message,
    setMessage,
    confirmations,
    setConfirmation,
  }
}
