<script lang="ts">
  import { Play } from "@lucide/svelte";
  import Metric from "./Metric.svelte";
  import type { RunPlanResponse } from "../commands/runhaven";

  export let plan: RunPlanResponse;
  export let launchConfirmation: boolean;
  export let confirmedWarnings: string[];
  export let launchReady: boolean;
  export let launching: boolean;
  export let setupOk: boolean;
  export let launchImageReady: boolean;
  export let onConfirmWarning: (code: string, checked: boolean) => void;
  export let onStart: () => void;

  function warningConfirmed(code: string): boolean {
    return confirmedWarnings.includes(code);
  }
</script>

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
              on:change={(event) => onConfirmWarning(warning.code, event.currentTarget.checked)}
              aria-label={`Confirm ${warning.code} warning`}
            />
            <span>{warning.message}</span>
          </label>
        {/each}
      </div>
    {/if}

    {#if !setupOk}
      <p class="muted">Fix setup blockers before launching a run.</p>
    {/if}
    {#if !launchImageReady}
      <p class="muted">Fix image readiness before launching a run.</p>
    {/if}

    <button class="primary launch-button" type="button" disabled={!launchReady || launching} on:click={onStart}>
      <Play size={18} />
      <span>{launching ? "Launching..." : "Launch run"}</span>
    </button>
  </div>
</section>
