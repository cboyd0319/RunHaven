<script lang="ts">
  import Metric from "./Metric.svelte";
  import type { LaunchRunResponse } from "../commands/runhaven";

  export let lastLaunch: LaunchRunResponse;
</script>

<section class="panel last-launch-panel">
  <h2>Last launch</h2>
  <dl class="plan-grid">
    <Metric label="Run" value={lastLaunch.snapshot.runId} />
    <Metric label="Status" value={lastLaunch.snapshot.status} />
    <Metric label="Agent" value={lastLaunch.snapshot.profile} />
    <Metric label="Project" value={lastLaunch.snapshot.workspace} />
    <Metric label="State volume" value={lastLaunch.snapshot.stateVolume} />
    <Metric label="Network" value={lastLaunch.snapshot.networkMode} />
    <Metric label="Container" value={lastLaunch.snapshot.containerName} />
  </dl>
</section>
