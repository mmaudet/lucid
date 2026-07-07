<script>
  import { onMount } from "svelte";
  import { serverStatus, endpointInfo } from "./lib.js";

  let { view = "index" } = $props();
  let status = $state(null);
  let info = $state(null);
  let err = $state(null);

  const titles = {
    index: "Lucid",
    dictionary: "Dictionnaire",
    journal: "Journal",
    stats: "Statistiques",
    settings: "Réglages",
  };
  const soon = { dictionary: "M4", journal: "M5", stats: "M6", settings: "M7" };

  onMount(async () => {
    try {
      status = await serverStatus();
      info = await endpointInfo();
    } catch (e) {
      err = String(e);
    }
  });
</script>

<main>
  <h1>{titles[view] ?? "Lucid"}</h1>
  {#if view !== "index"}
    <p class="soon">Interface complète à venir ({soon[view] ?? ""}).</p>
  {/if}

  {#if err}
    <p class="err">Erreur : {err}</p>
  {:else if status}
    <div class="card">
      <div class="row">
        <span class="k">Service</span>
        <span class="v">{status.running ? "en marche" : "arrêté"}</span>
      </div>
      <div class="row">
        <span class="k">Backend</span>
        <span class="v">{status.backend_reachable ? "joignable" : "injoignable"}</span>
      </div>
      {#if info}
        <div class="row">
          <span class="k">Endpoint</span><span class="v mono">{info.base_url}</span>
        </div>
        <div class="row"><span class="k">Modèle</span><span class="v mono">{info.model}</span></div>
      {/if}
    </div>
  {:else}
    <p>Chargement…</p>
  {/if}
</main>

<style>
  :global(body) {
    font-family: -apple-system, system-ui, sans-serif;
    margin: 0;
    background: #f6f6f7;
    color: #1a1a1a;
  }
  main {
    padding: 24px 28px;
  }
  h1 {
    font-size: 20px;
    margin: 0 0 12px;
  }
  .soon {
    color: #888;
    margin: 0 0 16px;
  }
  .card {
    background: #fff;
    border: 1px solid #e3e3e6;
    border-radius: 10px;
    padding: 6px 16px;
    max-width: 520px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    padding: 9px 0;
    border-bottom: 1px solid #f0f0f2;
  }
  .row:last-child {
    border-bottom: none;
  }
  .k {
    color: #777;
  }
  .v {
    font-weight: 600;
  }
  .mono {
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-weight: 500;
    font-size: 13px;
  }
  .err {
    color: #c0392b;
  }
  @media (prefers-color-scheme: dark) {
    :global(body) {
      background: #1e1e20;
      color: #eee;
    }
    .card {
      background: #29292c;
      border-color: #3a3a3e;
    }
    .row {
      border-color: #333;
    }
    .k {
      color: #aaa;
    }
  }
</style>
