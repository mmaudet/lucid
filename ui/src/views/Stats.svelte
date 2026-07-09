<script>
  import { onMount } from "svelte";
  import { statsSummary } from "../lib.js";

  let s = $state(null);
  let err = $state(null);

  onMount(async () => {
    try {
      s = await statsSummary();
    } catch (e) {
      err = String(e);
    }
  });

  let modif = $derived(s && s.total ? Math.round((100 * s.corrected) / s.total) : 0);
  let maxDay = $derived(s && s.by_day.length ? Math.max(...s.by_day.map((d) => d.count)) : 1);
  let maxTerm = $derived(s && s.top_terms.length ? Math.max(...s.top_terms.map((t) => t.count)) : 1);
</script>

<div class="page">
  <h1>Statistiques</h1>
  {#if err}<p class="err">{err}</p>{/if}
  {#if s}
    <div class="tiles">
      <div class="tile"><div class="n">{s.total}</div><div class="l">corrections</div></div>
      <div class="tile"><div class="n">{modif}%</div><div class="l">modifiées</div></div>
      <div class="tile"><div class="n">{Math.round(s.avg_latency_ms)}<span style="font-size:14px"> ms</span></div><div class="l">latence moyenne</div></div>
      <div class="tile"><div class="n">{s.failsafe}</div><div class="l">fail-safe</div></div>
    </div>
    <p class="muted" style="font-size:12px; margin-top:10px; max-width:640px">
      <b>fail-safe</b> : corrections où le modèle n'a pas produit de sortie fiable (vide, aberrante,
      ou backend injoignable). Dans ce cas Lucid renvoie le texte d'entrée <b>inchangé</b> — pour ne
      jamais dégrader la dictée. Un taux élevé signale un modèle instable ou un backend indisponible.
    </p>

    <h2>Corrections par jour</h2>
    <div class="card" style="padding:12px 16px">
      {#each s.by_day as d}
        <div class="barrow">
          <span class="lbl">{d.day}</span>
          <span class="bar" style="width:{(100 * d.count) / maxDay}%"></span>
          <span class="muted">{d.count}</span>
        </div>
      {/each}
      {#if s.by_day.length === 0}<span class="muted">Pas encore de données.</span>{/if}
    </div>

    <h2>Termes les plus corrigés</h2>
    <div class="card" style="padding:12px 16px">
      {#each s.top_terms as t}
        <div class="barrow">
          <span class="lbl">{t.canonical}</span>
          <span class="bar" style="width:{(100 * t.count) / maxTerm}%;background:#34c759"></span>
          <span class="muted">{t.count}</span>
        </div>
      {/each}
      {#if s.top_terms.length === 0}<span class="muted">Aucun terme du dictionnaire détecté pour l'instant.</span>{/if}
    </div>
  {:else}
    <p>Chargement…</p>
  {/if}
</div>
