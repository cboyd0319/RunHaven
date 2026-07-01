import { useEffect, useState } from "react"

export function useSessionInputState(selectedSessionId: string | null) {
  const [draft, setDraft] = useState("")
  const [sending, setSending] = useState(false)
  const [error, setError] = useState("")
  const [acceptedMessage, setAcceptedMessage] = useState("")

  useEffect(() => {
    setDraft("")
    setSending(false)
    setError("")
    setAcceptedMessage("")
  }, [selectedSessionId])

  function changeDraft(value: string) {
    setDraft(value)
    setError("")
    setAcceptedMessage("")
  }

  return {
    draft,
    setDraft,
    sending,
    setSending,
    error,
    setError,
    acceptedMessage,
    setAcceptedMessage,
    changeDraft,
  }
}
