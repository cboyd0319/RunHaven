import type {
  DashboardStatus,
  ImageStatusResponse,
  RunStatusResponse,
} from "@/commands/runhaven"

export function sameDashboardStatus(
  current: DashboardStatus,
  next: DashboardStatus,
): boolean {
  return (
    current.setup.ok === next.setup.ok &&
    current.setup.blockerCount === next.setup.blockerCount &&
    current.setup.sshAvailable === next.setup.sshAvailable &&
    current.setup.checks.length === next.setup.checks.length &&
    current.setup.checks.every((check, index) => {
      const nextCheck = next.setup.checks[index]
      return (
        nextCheck !== undefined &&
        check.name === nextCheck.name &&
        check.ok === nextCheck.ok &&
        check.detail === nextCheck.detail &&
        check.remedy === nextCheck.remedy
      )
    }) &&
    sameAgents(current.agents, next.agents) &&
    sameRunSummaries(current.activeRuns, next.activeRuns) &&
    sameRunSummaries(current.recentRuns, next.recentRuns) &&
    current.warnings.length === next.warnings.length &&
    current.warnings.every((warning, index) => warning === next.warnings[index])
  )
}

function sameAgents(
  current: DashboardStatus["agents"],
  next: DashboardStatus["agents"],
): boolean {
  return (
    current.length === next.length &&
    current.every((agent, index) => {
      const nextAgent = next[index]
      return (
        nextAgent !== undefined &&
        agent.name === nextAgent.name &&
        agent.description === nextAgent.description &&
        agent.image === nextAgent.image &&
        agent.signIn === nextAgent.signIn &&
        agent.broker === nextAgent.broker &&
        agent.defaultCommand.length === nextAgent.defaultCommand.length &&
        agent.defaultCommand.every(
          (value, valueIndex) => value === nextAgent.defaultCommand[valueIndex],
        ) &&
        agent.providerHosts.length === nextAgent.providerHosts.length &&
        agent.providerHosts.every(
          (value, valueIndex) => value === nextAgent.providerHosts[valueIndex],
        )
      )
    })
  )
}

function sameRunSummaries(
  current: DashboardStatus["activeRuns"],
  next: DashboardStatus["activeRuns"],
): boolean {
  return (
    current.length === next.length &&
    current.every((run, index) => {
      const nextRun = next[index]
      return (
        nextRun !== undefined &&
        run.runId === nextRun.runId &&
        run.profile === nextRun.profile &&
        run.workspace === nextRun.workspace &&
        run.network === nextRun.network &&
        run.status === nextRun.status &&
        run.timestamp === nextRun.timestamp &&
        run.authScope === nextRun.authScope &&
        run.session === nextRun.session
      )
    })
  )
}

export function sameImageStatus(
  current: ImageStatusResponse,
  next: ImageStatusResponse,
): boolean {
  return (
    current.agent === next.agent &&
    current.image.agent === next.image.agent &&
    current.image.image === next.image.image &&
    current.image.status === next.image.status &&
    current.image.ready === next.image.ready &&
    current.image.expectedSourceDigest === next.image.expectedSourceDigest &&
    current.image.localSourceDigest === next.image.localSourceDigest &&
    current.image.fixCommand === next.image.fixCommand &&
    current.builder.status === next.builder.status &&
    current.builder.detail === next.builder.detail &&
    current.builder.image === next.builder.image &&
    current.builder.cpus === next.builder.cpus &&
    current.builder.memory === next.builder.memory &&
    current.builder.rosetta === next.builder.rosetta &&
    current.builder.startedDate === next.builder.startedDate &&
    current.builder.ipv4Address === next.builder.ipv4Address &&
    current.builder.warning === next.builder.warning
  )
}

export function sameRunStatus(
  current: RunStatusResponse,
  next: RunStatusResponse,
): boolean {
  return (
    current.run.runId === next.run.runId &&
    current.run.profile === next.run.profile &&
    current.run.workspace === next.run.workspace &&
    current.run.networkMode === next.run.networkMode &&
    current.run.status === next.run.status &&
    current.run.timestamp === next.run.timestamp &&
    current.run.authScope === next.run.authScope &&
    current.run.session === next.run.session &&
    current.run.containerName === next.run.containerName &&
    current.container.state === next.container.state &&
    current.container.image === next.container.image &&
    current.container.startedAt === next.container.startedAt &&
    current.container.resources.cpus === next.container.resources.cpus &&
    current.container.resources.memoryBytes ===
      next.container.resources.memoryBytes &&
    current.container.networks.length === next.container.networks.length &&
    current.container.networks.every((network, index) => {
      const nextNetwork = next.container.networks[index]
      return (
        nextNetwork !== undefined &&
        network.network === nextNetwork.network &&
        network.hostname === nextNetwork.hostname &&
        network.ipv4Address === nextNetwork.ipv4Address &&
        network.ipv4Gateway === nextNetwork.ipv4Gateway &&
        network.ipv6Address === nextNetwork.ipv6Address
      )
    })
  )
}
