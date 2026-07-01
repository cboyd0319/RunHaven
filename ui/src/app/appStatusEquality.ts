import type {
  AgentProfile,
  BuilderStatus,
  CheckStatus,
  DashboardStatus,
  ImageStatusResponse,
  ProfileImageStatus,
  RunStatusContainer,
  RunStatusNetwork,
  RunStatusResources,
  RunStatusResponse,
  RunStatusRun,
  RunSummary,
  SetupStatus,
} from "@/commands/runhaven"

// Compile-time drift guards. Each checklist lists every key of an equality
// helper's subject type, so adding a field to the type fails `tsc` here until
// the matching `same*` helper compares the new field (or deliberately lists
// it). No runtime behavior; the consts are never read.
const dashboardStatusChecked = {
  setup: true,
  agents: true,
  activeRuns: true,
  recentRuns: true,
  warnings: true,
} satisfies Record<keyof DashboardStatus, true>
void dashboardStatusChecked

const setupStatusChecked = {
  ok: true,
  checks: true,
  blockerCount: true,
} satisfies Record<keyof SetupStatus, true>
void setupStatusChecked

const checkStatusChecked = {
  name: true,
  ok: true,
  detail: true,
  remedy: true,
} satisfies Record<keyof CheckStatus, true>
void checkStatusChecked

const agentProfileChecked = {
  name: true,
  description: true,
  image: true,
  defaultCommand: true,
  providerHosts: true,
  signIn: true,
  broker: true,
  supportsInAppSignIn: true,
} satisfies Record<keyof AgentProfile, true>
void agentProfileChecked

const runSummaryChecked = {
  runId: true,
  profile: true,
  workspace: true,
  network: true,
  status: true,
  timestamp: true,
  authScope: true,
  session: true,
} satisfies Record<keyof RunSummary, true>
void runSummaryChecked

const imageStatusResponseChecked = {
  agent: true,
  image: true,
  builder: true,
} satisfies Record<keyof ImageStatusResponse, true>
void imageStatusResponseChecked

const profileImageStatusChecked = {
  agent: true,
  image: true,
  status: true,
  ready: true,
  expectedSourceDigest: true,
  localSourceDigest: true,
  fixCommand: true,
} satisfies Record<keyof ProfileImageStatus, true>
void profileImageStatusChecked

const builderStatusChecked = {
  status: true,
  detail: true,
  image: true,
  cpus: true,
  memory: true,
  rosetta: true,
  startedDate: true,
  ipv4Address: true,
  warning: true,
} satisfies Record<keyof BuilderStatus, true>
void builderStatusChecked

const runStatusResponseChecked = {
  run: true,
  container: true,
} satisfies Record<keyof RunStatusResponse, true>
void runStatusResponseChecked

const runStatusRunChecked = {
  runId: true,
  profile: true,
  workspace: true,
  networkMode: true,
  status: true,
  timestamp: true,
  authScope: true,
  session: true,
  containerName: true,
} satisfies Record<keyof RunStatusRun, true>
void runStatusRunChecked

const runStatusContainerChecked = {
  state: true,
  image: true,
  startedAt: true,
  resources: true,
  networks: true,
} satisfies Record<keyof RunStatusContainer, true>
void runStatusContainerChecked

const runStatusResourcesChecked = {
  cpus: true,
  memoryBytes: true,
} satisfies Record<keyof RunStatusResources, true>
void runStatusResourcesChecked

const runStatusNetworkChecked = {
  network: true,
  hostname: true,
  ipv4Address: true,
  ipv4Gateway: true,
  ipv6Address: true,
} satisfies Record<keyof RunStatusNetwork, true>
void runStatusNetworkChecked

export function sameDashboardStatus(
  current: DashboardStatus,
  next: DashboardStatus,
): boolean {
  return (
    current.setup.ok === next.setup.ok &&
    current.setup.blockerCount === next.setup.blockerCount &&
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
        agent.supportsInAppSignIn === nextAgent.supportsInAppSignIn &&
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
    sameRunStatusContainer(current.container, next.container)
  )
}

// Shared by run status and managed-session status, which carry the same
// container payload.
export function sameRunStatusContainer(
  current: RunStatusContainer,
  next: RunStatusContainer,
): boolean {
  return (
    current.state === next.state &&
    current.image === next.image &&
    current.startedAt === next.startedAt &&
    current.resources.cpus === next.resources.cpus &&
    current.resources.memoryBytes === next.resources.memoryBytes &&
    current.networks.length === next.networks.length &&
    current.networks.every((network, index) => {
      const nextNetwork = next.networks[index]
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
