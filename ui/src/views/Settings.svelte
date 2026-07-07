<script>
  import { onMount } from "svelte";
  import { configGet, configSave } from "../lib.js";

  let cfg = $state(null);
  let err = $state(null);
  let saved = $state(false);

  onMount(async () => {
    try {
      cfg = await configGet();
    } catch (e) {
      err = String(e);
    }
  });

  async function save() {
    try {
      await configSave(cfg);
      saved = true;
      setTimeout(() => (saved = false), 1800);
      err = null;
    } catch (e) {
      err = String(e);
    }
  }
</script>

<div class="page">
  <h1>Réglages</h1>
  {#if err}<p class="err">{err}</p>{/if}
  {#if cfg}
    <h2>Serveur</h2>
    <div class="grid2">
      <div class="field"><label>Hôte</label><input bind:value={cfg.server.host} /></div>
      <div class="field"><label>Port</label><input type="number" bind:value={cfg.server.port} /></div>
    </div>
    <div class="field">
      <label>Token bearer (vide = auth désactivée)</label>
      <input class="mono" bind:value={cfg.server.bearer_token} />
    </div>

    <h2>Backend</h2>
    <div class="grid2">
      <div class="field">
        <label>Type</label>
        <select bind:value={cfg.backend.kind}>
          <option value="llamacpp">llama.cpp</option>
          <option value="ollama">Ollama</option>
        </select>
      </div>
      <div class="field"><label>Modèle</label><input bind:value={cfg.backend.model} /></div>
    </div>
    <div class="field"><label>URL de base</label><input class="mono" bind:value={cfg.backend.base_url} /></div>

    <h2>Correction</h2>
    <div class="grid2">
      <div class="field">
        <label>Mode de prompt</label>
        <select bind:value={cfg.correction.prompt_mode}>
          <option value="override">override</option>
          <option value="prepend">prepend</option>
          <option value="passthrough">passthrough</option>
        </select>
      </div>
      <div class="field"><label>Température</label><input type="number" step="0.05" bind:value={cfg.correction.temperature} /></div>
    </div>

    <h2>Journal</h2>
    <div class="grid2">
      <div class="field">
        <label>Journalisation</label>
        <select bind:value={cfg.journal.enabled}>
          <option value={true}>active</option>
          <option value={false}>désactivée</option>
        </select>
      </div>
      <div class="field">
        <label>Texte des dictées</label>
        <select bind:value={cfg.journal.store_text}>
          <option value={true}>stocké</option>
          <option value={false}>métadonnées seules</option>
        </select>
      </div>
    </div>
    <div class="field" style="max-width:220px">
      <label>Rétention (jours, 0 = illimité)</label>
      <input type="number" bind:value={cfg.journal.retention_days} />
    </div>

    <div class="toolbar" style="margin-top:8px">
      <span class="muted">L'enregistrement redémarre le service.</span>
      <span class="spacer"></span>
      {#if saved}<span class="muted">enregistré ✓</span>{/if}
      <button class="primary" onclick={save}>Enregistrer</button>
    </div>
  {:else}
    <p>Chargement…</p>
  {/if}
</div>
