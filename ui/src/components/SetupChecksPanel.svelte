<script lang="ts">
  import { ShieldCheck } from "@lucide/svelte";
  import StatusPill from "./StatusPill.svelte";
  import Metric from "./Metric.svelte";
  import type { DashboardStatus } from "../commands/runhaven";

  export let loading: boolean;
  export let dashboard: DashboardStatus | null;
</script>

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
