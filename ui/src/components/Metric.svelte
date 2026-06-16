<script lang="ts">
  export let label = "";
  export let value = "";
</script>

<div>
  <dt>{label}</dt>
  <dd>{value}</dd>
</div>

<style>
  div {
    min-width: 0;
    padding: 0.85rem 0;
    border-top: 1px solid #d9e0e8;
  }

  dt {
    margin: 0 0 0.28rem;
    color: #5b6878;
    font-size: 0.78rem;
    font-weight: 700;
    text-transform: uppercase;
  }

  dd {
    margin: 0;
    color: #16202c;
    font-size: 1rem;
    font-weight: 800;
    overflow-wrap: anywhere;
  }
</style>
