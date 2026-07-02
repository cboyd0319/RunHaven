import { useEffect, useRef } from "react"

import type { ManagedSessionSummary } from "@/commands/runhaven"
import { setAttentionBadge } from "@/commands/runhaven"

import { sessionAttention } from "./sessionAttention"

// Highest count shown on the Dock; mirrors MAX_ATTENTION_BADGE_COUNT in
// src-tauri/src/commands/attention_badge.rs, which rejects anything larger.
export const MAX_ATTENTION_BADGE_COUNT = 99

// The Dock badge is the count of sessions needing you, nothing else (spec
// section 4). The needs-you set is exactly the sidebar's attention rules
// (sessionAttention), so the Dock number always matches the visible row
// badges and clears as each session is attended.
export function attentionBadgeCount(
  sessions: ManagedSessionSummary[],
): number {
  return sessions.filter((session) => sessionAttention(session) !== null)
    .length
}

// `enabled` is the Settings toggle: turning the badge off reports 0 once,
// which clears the Dock number, and stays quiet until it is turned back on.
export function useAttentionBadge(
  sessions: ManagedSessionSummary[],
  enabled: boolean,
) {
  const count = enabled
    ? Math.min(attentionBadgeCount(sessions), MAX_ATTENTION_BADGE_COUNT)
    : 0
  const lastSent = useRef<number | null>(null)
  useEffect(() => {
    if (lastSent.current === count) {
      return
    }
    lastSent.current = count
    // Best effort: a failed badge update never blocks the UI. Rust clears the
    // badge at startup, so a missed update can only leave a stale number
    // until the count next changes.
    void setAttentionBadge(count).catch(() => undefined)
  }, [count])
}
