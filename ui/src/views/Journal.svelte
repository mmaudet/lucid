<script>
  import { onMount } from "svelte";
  import { journalList, journalClear, dictAddTerm } from "../lib.js";

  let rows = $state([]);
  let err = $state(null);
  let q = $state("");

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

  async function clearAll() {
    if (confirm("Vider tout le journal ?")) {
      await journalClear();
      await refresh();
    }
  }

  async function addToDict(r) {
    const canonical = prompt("Terme canonique à ajouter au dictionnaire :", r.output ?? "");
    if (!canonical) return;
    const alias = prompt("Variante à corriger (optionnel) :", r.input ?? "");
    try {
      await dictAddTerm(canonical, alias || null);
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
    <button class="danger" onclick={clearAll}>Vider le journal</button>
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
          <td><button class="small" onclick={() => addToDict(r)}>+ dico</button></td>
        </tr>
      {/each}
      {#if filtered.length === 0}
        <tr><td colspan="6" class="muted" style="text-align:center;padding:24px">Aucune entrée.</td></tr>
      {/if}
    </tbody>
  </table>
</div>
