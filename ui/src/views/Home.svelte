<script>
  import { onMount } from "svelte";
  import { serverStatus, endpointInfo, startServer, stopServer } from "../lib.js";

  let status = $state(null);
  let info = $state(null);
  let err = $state(null);

  async function refresh() {
    try {
      status = await serverStatus();
      info = await endpointInfo();
      err = null;
    } catch (e) {
      err = String(e);
    }
  }
  onMount(refresh);

  async function toggle() {
    try {
      if (status?.running) await stopServer();
      else await startServer();
      await refresh();
    } catch (e) {
      err = String(e);
    }
  }
</script>

<div class="page">
  <h1>Lucid</h1>
  <p class="sub">Correcteur de dictée FR, 100 % local.</p>
  {#if err}<p class="err">{err}</p>{/if}
  {#if status}
    <div class="card">
      <div class="row"><span class="k">Service</span><span class="v">{status.running ? "en marche" : "arrêté"}</span></div>
      <div class="row"><span class="k">Backend</span><span class="v">{status.backend_reachable ? "joignable" : "injoignable"}</span></div>
      <div class="row"><span class="k">Modèle</span><span class="v">{status.model ?? "—"}</span></div>
      {#if info}
        <div class="row"><span class="k">Endpoint</span><span class="v mono">{info.base_url}</span></div>
      {/if}
    </div>
    {#if info}
      <p class="muted" style="margin-top:12px; max-width:560px">
        Dans VoiceInk / Handy / FluidVoice : <b>Base URL</b> = l'endpoint ci-dessus ·
        <b>Model</b> = <code>{info.model}</code> · <b>API Key</b> = le token bearer (menu → Copier le token).
      </p>
    {/if}
    <div class="toolbar" style="margin-top:16px">
      <button class="primary" onclick={toggle}>{status.running ? "Arrêter le service" : "Démarrer le service"}</button>
    </div>
  {:else}
    <p>Chargement…</p>
  {/if}
</div>
