import { useEffect, useState } from "react"

import {
  getImageStatus,
  getNetworkPrunePreview,
  type AgentProfile,
} from "@/commands/runhaven"

// One agent's image readiness, reduced to the secret-free facts the read-only
// Diagnostics matrix shows: which agent, whether the image is ready, and the
// backend state word. No digests, paths, or fix commands cross into the view.
export type ImageReadinessRow = {
  agent: string
  available: boolean
  ready: boolean
  state: string
}

export type DiagnosticsReadModels = {
  images: ImageReadinessRow[]
  staleNetworks: number | null
  loading: boolean
  reload: () => Promise<void>
}

// Read-only support data for the Diagnostics view, built entirely from existing
// commands: per-agent image status and the stale-network prune preview count.
// It never mutates anything (rebuild and prune stay in their own views). Image
// calls use allSettled so one agent's failure does not blank the whole matrix,
// and the prune preview falls back to an unknown count on failure.
export function useDiagnosticsReadModels(
  agents: AgentProfile[],
): DiagnosticsReadModels {
  const [images, setImages] = useState<ImageReadinessRow[]>([])
  const [staleNetworks, setStaleNetworks] = useState<number | null>(null)
  const [loading, setLoading] = useState(false)
  // Key the entry load on the set of agent names, not the agents array
  // identity, so a quiet background poll that returns an equal dashboard never
  // reloads and blinks these panels.
  const agentNames = agents.map((agent) => agent.name).join(",")

  async function reload(): Promise<void> {
    setLoading(true)
    try {
      const [imageResults, prune] = await Promise.all([
        Promise.allSettled(agents.map((agent) => getImageStatus(agent.name))),
        getNetworkPrunePreview()
          .then((preview) => preview.count)
          .catch(() => null),
      ])
      setImages(
        imageResults.map((result, index) => {
          const name = agents[index]?.name ?? ""
          if (result.status === "fulfilled") {
            return {
              agent: name,
              available: true,
              ready: result.value.image.ready,
              state: result.value.image.status,
            }
          }
          return { agent: name, available: false, ready: false, state: "" }
        }),
      )
      setStaleNetworks(prune)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    void reload()
    // reload reads the current agents through agentNames; re-run only when the
    // agent set changes or the view remounts on destination re-entry.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [agentNames])

  return { images, staleNetworks, loading, reload }
}
