<script lang="ts">
  import { onMount } from "svelte";
  import { ClipboardCheck, FolderOpen, Play, RefreshCw, ShieldCheck } from "@lucide/svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import Metric from "../components/Metric.svelte";
  import {
    chooseProjectFolder,
    defaultRunPlanRequest,
    getDashboardStatus,
    getImageStatus,
    isLaunchReady,
    launchRun,
    planRun,
    secureNetworkDefault,
    type AgentProfile,
    type DashboardStatus,
    type ImageStatusResponse,
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

  function warningConfirmed(code: string): boolean {
    return confirmedWarnings.includes(code);
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
    <section class="panel setup-panel">
      <div class="section-heading">
        <ShieldCheck size={20} />
        <h2>Setup checks</h2>
      </div>

      {#if loading}
        <p class="muted">Loading status...</p>
      {:else}
        <dl class="metrics">
          <Metric label="Blockers" value={String(dashboard?.setup.blockerCount ?? 0)} />
          <Metric label="Active runs" value={String(dashboard?.activeRuns.length ?? 0)} />
          <Metric label="Recent runs" value={String(dashboard?.recentRuns.length ?? 0)} />
        </dl>

        <div class="check-list">
          {#each dashboard?.setup.checks ?? [] as check}
            <article>
              <StatusPill ok={check.ok} label={check.ok ? "OK" : "Fix"} />
              <div>
                <h3>{check.name}</h3>
                <p>{check.detail}</p>
                {#if !check.ok}
                  <p class="remedy">{check.remedy}</p>
                {/if}
              </div>
            </article>
          {/each}
        </div>
      {/if}
    </section>

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
    <section class="panel plan-panel">
      <h2>Plan review</h2>
      <dl class="plan-grid">
        <Metric label="Agent" value={plan.profile} />
        <Metric label="Project" value={plan.workspace} />
        <Metric label="Agent memory" value={plan.stateVolume} />
        <Metric label="Network" value={plan.networkMode} />
        <Metric label="Image" value={plan.image} />
        <Metric label="Preflight steps" value={String(plan.preflightCount)} />
      </dl>

      {#if plan.workspaceScopeNote}
        <p class="notice">{plan.workspaceScopeNote}</p>
      {/if}

      <p class="egress">{plan.egressSummary}</p>

      {#if plan.warnings.length > 0}
        <div class="warning-list">
          {#each plan.warnings as warning}
            <p>{warning.message}</p>
          {/each}
        </div>
      {/if}

      <div class="launch-confirmation">
        <label class="choice">
          <input type="checkbox" bind:checked={launchConfirmation} />
          <span>I reviewed this plan and want to start this run.</span>
        </label>

        {#if plan.warnings.length > 0}
          <div class="warning-confirmations">
            {#each plan.warnings as warning}
              <label class="choice warning-choice">
                <input
                  type="checkbox"
                  checked={warningConfirmed(warning.code)}
                  on:change={(event) => setWarningConfirmation(warning.code, event.currentTarget.checked)}
                  aria-label={`Confirm ${warning.code} warning`}
                />
                <span>{warning.message}</span>
              </label>
            {/each}
          </div>
        {/if}

        {#if dashboard && !dashboard.setup.ok}
          <p class="muted">Fix setup blockers before launching a run.</p>
        {/if}
        {#if !launchImageReady}
          <p class="muted">Fix image readiness before launching a run.</p>
        {/if}

        <button class="primary launch-button" type="button" disabled={!launchReady || launching} on:click={startRun}>
          <Play size={18} />
          <span>{launching ? "Launching..." : "Launch run"}</span>
        </button>
      </div>
    </section>
  {/if}
</main>
