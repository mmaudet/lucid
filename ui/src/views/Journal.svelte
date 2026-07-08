<script>
  import { onMount } from "svelte";
  import { journalList, journalClear, dictAddTerm } from "../lib.js";

  let rows = $state([]);
  let err = $state(null);
  let q = $state("");
  let confirmClear = $state(false);

  // Modale « ajouter au dictionnaire »
  let adding = $state(null); // { before, after }
  let canonical = $state("");
  let alias = $state("");
  let addErr = $state(null);
  let addOk = $state(false);

  async function refresh() {
    try {
      rows = await journalList(200);
      err = null;
    } catch (e) {
      err = String(e);
    }
  }
  onMount(refresh);

  let filtered = $derived(
    q.trim()
      ? rows.filter((r) => `${r.input ?? ""} ${r.output ?? ""}`.toLowerCase().includes(q.toLowerCase()))
      : rows,
  );

  function openAdd(r) {
    adding = { before: r.input ?? "", after: r.output ?? "" };
    canonical = "";
    alias = "";
    addErr = null;
    addOk = false;
  }

  async function submitAdd() {
    addErr = null;
    if (!canonical.trim()) {
      addErr = "Indiquez la graphie canonique.";
      return;
    }
    try {
      await dictAddTerm(canonical.trim(), alias.trim() || null);
      addOk = true;
      setTimeout(() => (adding = null), 900);
    } catch (e) {
      addErr = String(e).replace(/^Error:\s*/, "");
    }
  }

  async function doClear() {
    try {
      await journalClear();
      confirmClear = false;
      await refresh();
    } catch (e) {
      err = String(e);
    }
  }

  function fmt(ms) {
    return new Date(ms).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "medium" });
  }
</script>

<div class="page">
  <h1>Journal des corrections</h1>
  <div class="toolbar">
    <input style="max-width:280px" placeholder="Rechercher…" bind:value={q} />
    <span class="muted">{filtered.length} / {rows.length}</span>
    <span class="spacer"></span>
    <button onclick={refresh}>Rafraîchir</button>
    <button class="danger" onclick={() => (confirmClear = true)}>Vider le journal</button>
  </div>
  {#if err}<p class="err">{err}</p>{/if}
  <table>
    <thead>
      <tr>
        <th style="width:132px">Quand</th>
        <th style="width:82px">Statut</th>
        <th>Avant</th>
        <th>Après</th>
        <th style="width:56px">ms</th>
        <th style="width:64px"></th>
      </tr>
    </thead>
    <tbody>
      {#each filtered as r (r.id)}
        <tr>
          <td class="muted" style="font-size:12px">{fmt(r.ts_ms)}</td>
          <td><span class="pill {r.status}">{r.status}</span></td>
          <td class="muted">{r.input ?? "—"}</td>
          <td>{r.output ?? "—"}</td>
          <td class="muted" style="font-variant-numeric:tabular-nums">{r.latency_ms}</td>
          <td><button class="small" onclick={() => openAdd(r)}>+ dico</button></td>
        </tr>
      {/each}
      {#if filtered.length === 0}
        <tr><td colspan="6" class="muted" style="text-align:center;padding:24px">Aucune entrée.</td></tr>
      {/if}
    </tbody>
  </table>
</div>

{#if adding}
  <div class="modal-backdrop" onclick={() => (adding = null)} role="presentation">
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog">
      <h2>Ajouter au dictionnaire</h2>
      <p class="muted" style="font-size:12.5px; line-height:1.5">
        Avant : <span class="mono">{adding.before}</span><br />
        Après : <span class="mono">{adding.after}</span>
      </p>
      <div class="field">
        <label>Graphie canonique (la bonne orthographe)</label>
        <input bind:value={canonical} placeholder="Michel-Marie Maudet" />
      </div>
      <div class="field">
        <label>Variante à corriger (la faute exacte — optionnel)</label>
        <input bind:value={alias} placeholder="michel marie mode" />
      </div>
      {#if addErr}<p class="err" style="font-size:12.5px">{addErr}</p>{/if}
      {#if addOk}<p class="muted">Ajouté ✓</p>{/if}
      <div class="toolbar" style="margin-top:6px">
        <span class="spacer"></span>
        <button onclick={() => (adding = null)}>Annuler</button>
        <button class="primary" onclick={submitAdd}>Ajouter</button>
      </div>
    </div>
  </div>
{/if}

{#if confirmClear}
  <div class="modal-backdrop" onclick={() => (confirmClear = false)} role="presentation">
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog">
      <h2>Vider le journal ?</h2>
      <p class="muted">Toutes les entrées seront définitivement supprimées.</p>
      <div class="toolbar" style="margin-top:6px">
        <span class="spacer"></span>
        <button onclick={() => (confirmClear = false)}>Annuler</button>
        <button class="danger" onclick={doClear}>Vider</button>
      </div>
    </div>
  </div>
{/if}
