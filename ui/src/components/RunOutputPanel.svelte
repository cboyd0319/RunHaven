<script lang="ts">
  import { FileText } from "@lucide/svelte";
  import type { LogSnapshotResponse } from "../commands/runhaven";

  export let logAcknowledged: boolean;
  export let logLoading: boolean;
  export let logError: string;
  export let logSnapshot: LogSnapshotResponse | null;
  export let onLoad: () => void;
</script>

<section class="panel run-output-panel" aria-live="polite">
  <h2>Run output</h2>
  <p class="sensitive-note">Raw output can include secrets or workspace content.</p>
  <div class="log-actions">
    <label class="choice">
      <input type="checkbox" bind:checked={logAcknowledged} />
      <span>Show raw container output for this run.</span>
    </label>
    <button
      class="secondary"
      type="button"
      disabled={!logAcknowledged || logLoading}
      on:click={onLoad}
    >
      <FileText size={18} />
      <span>{logLoading ? "Loading..." : "View latest output"}</span>
    </button>
  </div>
  {#if logError}
    <p class="notice">{logError}</p>
  {/if}
  {#if logSnapshot}
    <div class="log-meta">
      <span>{logSnapshot.returnedLines} lines returned</span>
      <span>{logSnapshot.requestedLines} lines requested</span>
      {#if logSnapshot.truncated}
        <span>truncated</span>
      {/if}
    </div>
    <pre class="log-output">{logSnapshot.text || "No output returned."}</pre>
  {/if}
</section>
