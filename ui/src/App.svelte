<script>
  import Home from "./views/Home.svelte";
  import Dictionary from "./views/Dictionary.svelte";
  import Journal from "./views/Journal.svelte";
  import Stats from "./views/Stats.svelte";
  import Settings from "./views/Settings.svelte";

  let { view = "index" } = $props();
</script>

{#if view === "dictionary"}
  <Dictionary />
{:else if view === "journal"}
  <Journal />
{:else if view === "stats"}
  <Stats />
{:else if view === "settings"}
  <Settings />
{:else}
  <Home />
{/if}
