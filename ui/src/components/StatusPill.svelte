<script lang="ts">
  export let ok = false;
  export let label = "";
</script>

<span class:ok class:warn={!ok}>{label}</span>

<style>
  span {
    display: inline-flex;
    align-items: center;
    min-height: 1.75rem;
    padding: 0 0.65rem;
    border: 1px solid #c9d3df;
    border-radius: 999px;
    font-size: 0.82rem;
    font-weight: 700;
    line-height: 1;
    white-space: nowrap;
  }

  .ok {
    color: #0f6848;
    background: #e7f6ee;
    border-color: #9fd4bd;
  }

  .warn {
    color: #8a4f00;
    background: #fff2db;
    border-color: #e5bd73;
  }
</style>
