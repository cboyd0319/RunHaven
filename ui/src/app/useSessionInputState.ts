import { useMemo, useState } from "react"

export function useSessionInputState(selectedSessionId: string | null) {
  const [records, setRecords] = useState<SessionInputRecords>({})
  const current = useMemo(
    () => selectedSessionInputState(records, selectedSessionId),
    [records, selectedSessionId],
  )

  function changeDraft(value: string) {
    if (!selectedSessionId) {
      return
    }
    setRecords((currentRecords) =>
      updateSessionInputState(currentRecords, selectedSessionId, {
        draft: value,
        error: "",
        acceptedMessage: "",
      }),
    )
  }

  function setDraftFor(sessionId: string, draft: string) {
    setRecords((currentRecords) =>
      updateSessionInputState(currentRecords, sessionId, { draft }),
    )
  }

  function setSendingFor(sessionId: string, sending: boolean) {
    setRecords((currentRecords) =>
      updateSessionInputState(currentRecords, sessionId, { sending }),
    )
  }

  function setErrorFor(sessionId: string, error: string) {
    setRecords((currentRecords) =>
      updateSessionInputState(currentRecords, sessionId, { error }),
    )
  }

  function setAcceptedMessageFor(sessionId: string, acceptedMessage: string) {
    setRecords((currentRecords) =>
      updateSessionInputState(currentRecords, sessionId, { acceptedMessage }),
    )
  }

  return {
    draft: current.draft,
    sending: current.sending,
    error: current.error,
    acceptedMessage: current.acceptedMessage,
    setDraftFor,
    setSendingFor,
    setErrorFor,
    setAcceptedMessageFor,
    changeDraft,
  }
}

export type SessionInputState = {
  draft: string
  sending: boolean
  error: string
  acceptedMessage: string
}

type SessionInputRecords = Record<string, SessionInputState>

const EMPTY_SESSION_INPUT_STATE: SessionInputState = {
  draft: "",
  sending: false,
  error: "",
  acceptedMessage: "",
}

export function selectedSessionInputState(
  records: SessionInputRecords,
  sessionId: string | null,
): SessionInputState {
  if (!sessionId) {
    return EMPTY_SESSION_INPUT_STATE
  }
  return records[sessionId] ?? EMPTY_SESSION_INPUT_STATE
}

export function updateSessionInputState(
  records: SessionInputRecords,
  sessionId: string,
  patch: Partial<SessionInputState>,
): SessionInputRecords {
  const current = records[sessionId] ?? EMPTY_SESSION_INPUT_STATE
  const next = { ...current, ...patch }
  if (
    current.draft === next.draft &&
    current.sending === next.sending &&
    current.error === next.error &&
    current.acceptedMessage === next.acceptedMessage
  ) {
    return records
  }
  return {
    ...records,
    [sessionId]: next,
  }
}
