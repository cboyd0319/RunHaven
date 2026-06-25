<script lang="ts">
  import { onMount } from "svelte";
  import { ClipboardCheck, FolderOpen, RefreshCw } from "@lucide/svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import SetupChecksPanel from "../components/SetupChecksPanel.svelte";
  import PlanReviewPanel from "../components/PlanReviewPanel.svelte";
  import LastLaunchPanel from "../components/LastLaunchPanel.svelte";
  import RunStatusPanel from "../components/RunStatusPanel.svelte";
  import RunOutputPanel from "../components/RunOutputPanel.svelte";
  import DiagnosticsPanel from "../components/DiagnosticsPanel.svelte";
  import {
    chooseProjectFolder,
    defaultRunPlanRequest,
    getDashboardStatus,
    getImageStatus,
    getLogSnapshot,
    getRunStatus,
    isLaunchReady,
    launchRun,
    planRun,
    secureNetworkDefault,
    stopRun,
    killRun,
    repairRun,
    type DashboardStatus,
    type ImageStatusResponse,
    type LogSnapshotResponse,
    type LaunchRunResponse,
    type RunStatusResponse,
    type RunPlanRequest,
    type RunPlanResponse
  } from "../commands/runhaven";

  let dashboard: DashboardStatus | null = null;
  let request: RunPlanRequest = defaultRunPlanRequest(undefined);
  let plan: RunPlanResponse | null = null;
  let loading = true;
  let planning = false;
  let launching = false;
  let imageLoading = false;
  let imageStatus: ImageStatusResponse | null = null;
  let imageError = "";
  let launchConfirmation = false;
  let confirmedWarnings: string[] = [];
  let launchMessage = "";
  let lastLaunch: LaunchRunResponse | null = null;
  let runStatus: RunStatusResponse | null = null;
  let runStatusLoading = false;
  let runStatusError = "";
  let logSnapshot: LogSnapshotResponse | null = null;
  let logAcknowledged = false;
  let logLoading = false;
  let logError = "";
  let controlBusy = false;
  let controlError = "";
  let controlMessage = "";
  let error = "";

  $: selectedAgent = dashboard?.agents.find((agent) => agent.name === request.agent);
  $: launchImageReady = request.image ? true : imageStatus?.image.ready === true;
  $: launchReady =
    Boolean(dashboard?.setup.ok) &&
    isLaunchReady(plan, launchConfirmation, new Set(confirmedWarnings), launchImageReady);

  onMount(() => {
    void refresh();
  });

  async function refresh() {
    loading = true;
    error = "";
    try {
      dashboard = await getDashboardStatus();
      const agent = dashboard.agents.find((item) => item.name === request.agent) ?? dashboard.agents[0];
      if (agent) {
        request = {
          ...defaultRunPlanRequest(agent),
          workspacePath: request.workspacePath,
          sessionName: request.sessionName
        };
        await loadImageStatus(agent.name);
      }
      if (lastLaunch) {
        await loadRunStatus(lastLaunch.runId);
      }
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }

  function changeAgent(agentName: string) {
    const agent = dashboard?.agents.find((item) => item.name === agentName);
    request = {
      ...request,
      agent: agentName,
      networkMode: secureNetworkDefault(agent)
    };
    invalidatePlan();
    void loadImageStatus(agentName);
  }

  async function loadImageStatus(agentName: string) {
    imageLoading = true;
    imageError = "";
    try {
      imageStatus = await getImageStatus(agentName);
    } catch (cause) {
      imageStatus = null;
      imageError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      imageLoading = false;
    }
  }

  async function loadRunStatus(runId: string) {
    runStatusLoading = true;
    runStatusError = "";
    try {
      runStatus = await getRunStatus(runId);
    } catch (cause) {
      runStatus = null;
      runStatusError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      runStatusLoading = false;
    }
  }

  async function loadLogSnapshot() {
    if (!lastLaunch) {
      return;
    }
    logLoading = true;
    logError = "";
    try {
      logSnapshot = await getLogSnapshot(lastLaunch.runId, {
        confirmSensitiveOutput: logAcknowledged
      });
    } catch (cause) {
      logSnapshot = null;
      logError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      logLoading = false;
    }
  }

  async function runControl(action: (runId: string) => Promise<string>) {
    if (!lastLaunch) {
      return;
    }
    const runId = lastLaunch.runId;
    controlBusy = true;
    controlError = "";
    controlMessage = "";
    try {
      controlMessage = await action(runId);
      await loadRunStatus(runId);
    } catch (cause) {
      controlError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      controlBusy = false;
    }
  }

  function stopActiveRun() {
    void runControl(async (runId) => {
      const result = await stopRun(runId);
      return `Stop requested for ${result.runId}.`;
    });
  }

  function killActiveRun() {
    void runControl(async (runId) => {
      const result = await killRun(runId);
      return `Hard stop requested for ${result.runId}.`;
    });
  }

  function repairActiveRun() {
    void runControl(async (runId) => {
      const result = await repairRun(runId);
      return result.markerRemoved
        ? `Stale marker cleared for ${result.runId}.`
        : `Repair for ${result.runId}: ${result.status}.`;
    });
  }

  async function pickFolder() {
    const selected = await chooseProjectFolder();
    if (selected) {
      request = { ...request, workspacePath: selected };
      invalidatePlan();
    }
  }

  async function reviewPlan() {
    planning = true;
    error = "";
    plan = null;
    resetLaunchConfirmation();
    try {
      plan = await planRun(request);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      planning = false;
    }
  }

  function updateNetworkMode(networkMode: RunPlanRequest["networkMode"]) {
    request = { ...request, networkMode };
    invalidatePlan();
  }

  function invalidatePlan() {
    plan = null;
    resetLaunchConfirmation();
  }

  function resetLaunchConfirmation() {
    launchConfirmation = false;
    confirmedWarnings = [];
    launchMessage = "";
  }

  function setWarningConfirmation(code: string, checked: boolean) {
    const next = new Set(confirmedWarnings);
    if (checked) {
      next.add(code);
    } else {
      next.delete(code);
    }
    confirmedWarnings = Array.from(next);
  }

  async function startRun() {
    if (!plan) {
      return;
    }
    launching = true;
    error = "";
    launchMessage = "";
    try {
      const started = await launchRun({
        plan: request,
        confirmLaunch: launchConfirmation,
        confirmedWarnings
      });
      lastLaunch = started;
      logSnapshot = null;
      logAcknowledged = false;
      logError = "";
      await loadRunStatus(started.runId);
      launchMessage = `Run started: ${started.runId}`;
      plan = null;
      launchConfirmation = false;
      confirmedWarnings = [];
      await refresh();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      launching = false;
    }
  }
</script>

<svelte:head>
  <title>RunHaven</title>
</svelte:head>

<main>
  <header class="topbar">
    <div>
      <p class="eyebrow">RunHaven</p>
      <h1>Agent runs</h1>
    </div>
    <button class="icon-button" type="button" on:click={refresh} aria-label="Refresh status">
      <RefreshCw size={18} />
    </button>
  </header>

  {#if error}
    <section class="notice" role="alert">{error}</section>
  {/if}

  {#if launchMessage}
    <section class="notice success" role="status">{launchMessage}</section>
  {/if}

  <section class="status-band">
    <div>
      <p class="eyebrow">Setup</p>
      <h2>{dashboard?.setup.ok ? "Ready" : "Needs attention"}</h2>
    </div>
    <StatusPill ok={Boolean(dashboard?.setup.ok)} label={dashboard?.setup.ok ? "Passing" : "Blocked"} />
  </section>

  <section class="grid">
    <SetupChecksPanel {loading} {dashboard} />

    <section class="panel launch-panel">
      <div class="section-heading">
        <ClipboardCheck size={20} />
        <h2>Launch plan</h2>
      </div>

      <form on:submit|preventDefault={reviewPlan}>
        <label>
          <span>Agent</span>
          <select bind:value={request.agent} on:change={(event) => changeAgent(event.currentTarget.value)}>
            {#each dashboard?.agents ?? [] as agent}
              <option value={agent.name}>{agent.name}</option>
            {/each}
          </select>
        </label>

        {#if selectedAgent}
          <p class="agent-detail">{selectedAgent.description}</p>
        {/if}

        <div class="readiness" aria-live="polite">
          {#if imageLoading}
            <p class="muted">Checking image...</p>
          {:else if imageStatus}
            <div class="readiness-row">
              <StatusPill
                ok={imageStatus.image.ready}
                label={imageStatus.image.ready ? "OK" : "Fix"}
              />
              <div>
                <h3>{imageStatus.image.ready ? "Image ready" : "Image needs rebuild"}</h3>
                <p>{imageStatus.image.image}</p>
                {#if imageStatus.image.fixCommand}
                  <p class="remedy">{imageStatus.image.fixCommand}</p>
                {/if}
              </div>
            </div>
            <div class="readiness-row builder-row">
              <StatusPill ok={imageStatus.builder.status !== "unavailable"} label="Info" />
              <div>
                <h3>Builder {imageStatus.builder.status}</h3>
                <p>{imageStatus.builder.detail}</p>
                {#if imageStatus.builder.cpus || imageStatus.builder.memory}
                  <p>{imageStatus.builder.cpus ?? "unknown"} CPUs, {imageStatus.builder.memory ?? "unknown memory"}</p>
                {/if}
                {#if imageStatus.builder.warning}
                  <p class="remedy">{imageStatus.builder.warning}</p>
                {/if}
              </div>
            </div>
          {:else if imageError}
            <div class="readiness-row">
              <StatusPill ok={false} label="Fix" />
              <div>
                <h3>Image status unavailable</h3>
                <p>{imageError}</p>
              </div>
            </div>
          {/if}
        </div>

        <label>
          <span>Project folder</span>
          <div class="folder-row">
            <input
              bind:value={request.workspacePath}
              on:input={invalidatePlan}
              placeholder="/Users/you/project"
              aria-label="Project folder path"
            />
            <button class="icon-button" type="button" on:click={pickFolder} aria-label="Choose project folder">
              <FolderOpen size={18} />
            </button>
          </div>
        </label>

        <fieldset>
          <legend>Network goal</legend>
          <label class="choice">
            <input
              type="radio"
              name="network"
              checked={request.networkMode === "provider"}
              on:change={() => updateNetworkMode("provider")}
            />
            <span>AI provider only</span>
          </label>
          <label class="choice">
            <input
              type="radio"
              name="network"
              checked={request.networkMode === "internal"}
              on:change={() => updateNetworkMode("internal")}
            />
            <span>Offline or local only</span>
          </label>
          <label class="choice">
            <input
              type="radio"
              name="network"
              checked={request.networkMode === "internet"}
              on:change={() => updateNetworkMode("internet")}
            />
            <span>Full internet</span>
          </label>
        </fieldset>

        <div class="inline-fields">
          <label>
            <span>CPU</span>
            <input bind:value={request.cpus} on:input={invalidatePlan} />
          </label>
          <label>
            <span>Memory</span>
            <input bind:value={request.memory} on:input={invalidatePlan} />
          </label>
        </div>

        <label class="choice">
          <input type="checkbox" bind:checked={request.readOnlyWorkspace} on:change={invalidatePlan} />
          <span>Read-only project folder</span>
        </label>

        <button class="primary" type="submit" disabled={!request.workspacePath || planning}>
          {planning ? "Reviewing..." : "Review plan"}
        </button>
      </form>
    </section>
  </section>

  {#if plan}
    <PlanReviewPanel
      {plan}
      bind:launchConfirmation
      {confirmedWarnings}
      {launchReady}
      {launching}
      setupOk={Boolean(dashboard?.setup.ok)}
      {launchImageReady}
      onConfirmWarning={setWarningConfirmation}
      onStart={startRun}
    />
  {/if}

  {#if lastLaunch}
    <LastLaunchPanel {lastLaunch} />
  {/if}

  {#if lastLaunch}
    <RunStatusPanel
      {runStatus}
      {runStatusLoading}
      {runStatusError}
      {controlBusy}
      {controlError}
      {controlMessage}
      onStop={stopActiveRun}
      onKill={killActiveRun}
      onRepair={repairActiveRun}
    />
  {/if}

  {#if lastLaunch}
    <RunOutputPanel bind:logAcknowledged {logLoading} {logError} {logSnapshot} onLoad={loadLogSnapshot} />
  {/if}

  <DiagnosticsPanel />
</main>
