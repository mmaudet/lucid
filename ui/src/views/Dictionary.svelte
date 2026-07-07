<script>
  import { onMount } from "svelte";
  import { dictList, dictSave } from "../lib.js";

  // aliases stocké en chaîne (séparée par des virgules) pour l'édition.
  let terms = $state([]);
  let err = $state(null);
  let saved = $state(false);

  onMount(async () => {
    try {
      const d = await dictList();
      terms = (d.terms ?? []).map((t) => ({
        canonical: t.canonical,
        aliases: (t.aliases ?? []).join(", "),
      }));
    } catch (e) {
      err = String(e);
    }
  });

  function addRow() {
    terms = [...terms, { canonical: "", aliases: "" }];
  }
  function removeRow(i) {
    terms = terms.filter((_, j) => j !== i);
  }

  async function save() {
    const clean = terms
      .filter((t) => t.canonical.trim())
      .map((t) => ({
        canonical: t.canonical.trim(),
        aliases: t.aliases.split(",").map((a) => a.trim()).filter(Boolean),
      }));
    try {
      await dictSave({ terms: clean });
      saved = true;
      setTimeout(() => (saved = false), 1500);
      err = null;
    } catch (e) {
      err = String(e);
    }
  }
</script>

<div class="page">
  <h1>Dictionnaire</h1>
  <p class="sub">Graphies exactes réinjectées dans la correction (noms propres, patronymes, sigles, produits). Appliqué à chaud.</p>
  {#if err}<p class="err">{err}</p>{/if}
  <table>
    <thead>
      <tr><th style="width:38%">Graphie canonique</th><th>Variantes (séparées par des virgules)</th><th style="width:40px"></th></tr>
    </thead>
    <tbody>
      {#each terms as t, i (i)}
        <tr>
          <td><input bind:value={t.canonical} placeholder="LINAGORA" /></td>
          <td><input bind:value={t.aliases} placeholder="linagora, lina gora" /></td>
          <td><button class="small danger" onclick={() => removeRow(i)}>✕</button></td>
        </tr>
      {/each}
      {#if terms.length === 0}
        <tr><td colspan="3" class="muted" style="text-align:center;padding:22px">Dictionnaire vide.</td></tr>
      {/if}
    </tbody>
  </table>
  <div class="toolbar" style="margin-top:14px">
    <button onclick={addRow}>+ Ajouter un terme</button>
    <span class="spacer"></span>
    {#if saved}<span class="muted">enregistré ✓</span>{/if}
    <button class="primary" onclick={save}>Enregistrer</button>
  </div>
</div>
