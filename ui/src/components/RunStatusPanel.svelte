<script lang="ts">
  import Metric from "./Metric.svelte";
  import type { RunStatusResponse } from "../commands/runhaven";

  export let runStatus: RunStatusResponse | null;
  export let runStatusLoading: boolean;
  export let runStatusError: string;

  function formatMemory(bytes: number | null): string {
    if (bytes === null) {
      return "unknown";
    }
    return `${Math.round(bytes / 1024 ** 2)} MiB`;
  }
</script>

<section class="panel run-status-panel" aria-live="polite">
  <h2>Run status</h2>
  {#if runStatusLoading}
    <p class="muted">Refreshing status...</p>
  {:else if runStatus}
    <dl class="plan-grid">
      <Metric label="Marker status" value={runStatus.run.status} />
      <Metric label="Container state" value={runStatus.container.state} />
      <Metric label="Image" value={runStatus.container.image ?? "-"} />
      <Metric label="Started" value={runStatus.container.startedAt ?? "-"} />
      <Metric label="CPU" value={runStatus.container.resources.cpus ?? "unknown"} />
      <Metric label="Memory" value={formatMemory(runStatus.container.resources.memoryBytes)} />
    </dl>
    {#if runStatus.container.networks.length > 0}
      <div class="status-list">
        {#each runStatus.container.networks as network}
          <p>
            {network.network ?? "network"}
            {#if network.ipv4Address}
              <span>ipv4={network.ipv4Address}</span>
            {/if}
            {#if network.hostname}
              <span>host={network.hostname}</span>
            {/if}
          </p>
        {/each}
      </div>
    {/if}
  {:else if runStatusError}
    <p class="notice">{runStatusError}</p>
  {/if}
</section>
