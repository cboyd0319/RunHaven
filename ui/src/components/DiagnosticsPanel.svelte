<script lang="ts">
  import Metric from "./Metric.svelte";
  import {
    getAuthLog,
    getAuthStatus,
    getEgressLog,
    type AuthLogEntry,
    type AuthStatusResponse,
    type EgressLogEntry
  } from "../commands/runhaven";

  let authStatus: AuthStatusResponse | null = null;
  let egressEntries: EgressLogEntry[] = [];
  let authEntries: AuthLogEntry[] = [];
  let loading = false;
  let loaded = false;
  let error = "";

  async function loadDiagnostics() {
    loading = true;
    error = "";
    try {
      const [status, egress, auth] = await Promise.all([getAuthStatus(), getEgressLog(), getAuthLog()]);
      authStatus = status;
      egressEntries = egress.entries;
      authEntries = auth.entries;
      loaded = true;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }
</script>

<section class="panel diagnostics-panel" aria-live="polite">
  <h2>Diagnostics</h2>
  <p class="muted">Secret-free provider egress and auth broker activity for RunHaven runs.</p>
  <button class="secondary" type="button" disabled={loading} on:click={loadDiagnostics}>
    <span>{loading ? "Loading..." : "Load diagnostics"}</span>
  </button>

  {#if error}
    <p class="notice" role="alert">{error}</p>
  {/if}

  {#if authStatus}
    <h3>Auth broker status</h3>
    <dl class="plan-grid">
      <Metric label="Status" value={authStatus.status} />
      <Metric label="Runtime" value={authStatus.runtime} />
    </dl>
    <div class="status-list">
      {#each authStatus.profiles as profile}
        <p>{profile.name}: {profile.status}</p>
      {/each}
    </div>
  {/if}

  {#if loaded}
    <h3>Provider egress decisions</h3>
    {#if egressEntries.length === 0}
      <p class="muted">No provider egress decisions recorded.</p>
    {:else}
      <div class="status-list">
        {#each egressEntries as entry}
          <p>{entry.decision} {entry.host}:{entry.port} ({entry.reason}) count={entry.count}</p>
        {/each}
      </div>
    {/if}

    <h3>Auth broker decisions</h3>
    {#if authEntries.length === 0}
      <p class="muted">No auth broker decisions recorded.</p>
    {:else}
      <div class="status-list">
        {#each authEntries as entry}
          <p>{entry.decision} {entry.method} {entry.path} ({entry.reason})</p>
        {/each}
      </div>
    {/if}
  {/if}
</section>
