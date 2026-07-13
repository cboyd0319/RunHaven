import { useEffect, useState } from "react"

export function useRunControlState(controlTargetId: string | null) {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState("")
  const [message, setMessage] = useState("")
  useEffect(() => {
    setError("")
    setMessage("")
  }, [controlTargetId])

  return {
    busy,
    setBusy,
    error,
    setError,
    message,
    setMessage,
  }
}
