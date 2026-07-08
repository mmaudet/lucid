<script>
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { appBuild } from "./lib.js";
  import Home from "./views/Home.svelte";
  import Dictionary from "./views/Dictionary.svelte";
  import Journal from "./views/Journal.svelte";
  import Stats from "./views/Stats.svelte";
  import Settings from "./views/Settings.svelte";

  const ICONS = {
    home: "M3 11l9-8 9 8M5 10v10h14V10",
    dictionary: "M5 4h11a3 3 0 013 3v13H8a3 3 0 01-3-3zM5 4v13",
    journal: "M4 6h16M4 12h16M4 18h10",
    stats: "M5 20V10M12 20V4M19 20v-7",
    settings: "M12 15a3 3 0 100-6 3 3 0 000 6zM19 12l1.5 1-1 2-1.8-.5-1.4 1.2-.3 1.8h-2l-.3-1.8-1.4-1.2-1.8.5-1-2L6 12l-1.5-1 1-2 1.8.5 1.4-1.2.3-1.8h2l.3 1.8 1.4 1.2 1.8-.5 1 2z",
  };
  const SECTIONS = [
    { id: "home", label: "Accueil" },
    { id: "dictionary", label: "Dictionnaire" },
    { id: "journal", label: "Journal" },
    { id: "stats", label: "Statistiques" },
    { id: "settings", label: "Réglages" },
  ];

  function fromHash() {
    const h = location.hash.replace("#", "");
    return SECTIONS.some((s) => s.id === h) ? h : "home";
  }

  let section = $state(fromHash());
  let build = $state(null);

  onMount(async () => {
    try {
      build = await appBuild();
    } catch {}
    const un = await listen("navigate", (e) => {
      if (e.payload) section = String(e.payload);
    });
    return un;
  });
</script>

<div class="app">
  <aside class="sidebar">
    <div class="brand">
      <span class="badge">L</span>
      <span class="name">Lucid</span>
    </div>
    <nav>
      {#each SECTIONS as s}
        <button class="navitem" class:active={section === s.id} onclick={() => (section = s.id)}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d={ICONS[s.id]} /></svg>
          {s.label}
        </button>
      {/each}
    </nav>
    <div class="build">
      {#if build}v{build.version} · build {build.build}{/if}
    </div>
  </aside>

  <main class="content">
    {#if section === "dictionary"}
      <Dictionary />
    {:else if section === "journal"}
      <Journal />
    {:else if section === "stats"}
      <Stats />
    {:else if section === "settings"}
      <Settings />
    {:else}
      <Home />
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }
  .sidebar {
    width: 208px;
    flex-shrink: 0;
    background: #ececee;
    border-right: 1px solid #dcdcdf;
    display: flex;
    flex-direction: column;
    padding: 14px 12px 12px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 4px 6px 16px;
  }
  .badge {
    width: 26px;
    height: 26px;
    border-radius: 7px;
    background: #1d1d1f;
    color: #fff;
    display: grid;
    place-items: center;
    font-weight: 800;
    font-size: 16px;
    line-height: 1;
  }
  .name {
    font-weight: 700;
    font-size: 16px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }
  .navitem {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    border: none;
    background: transparent;
    border-radius: 7px;
    padding: 7px 9px;
    font-size: 13.5px;
    color: #333;
    cursor: pointer;
    text-align: left;
  }
  .navitem svg {
    width: 18px;
    height: 18px;
    opacity: 0.75;
  }
  .navitem:hover {
    background: #e0e0e3;
  }
  .navitem.active {
    background: #fff;
    color: #1d1d1f;
    font-weight: 600;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.06);
  }
  .build {
    padding: 8px 8px 2px;
    font-size: 11px;
    color: #8a8a8e;
    font-variant-numeric: tabular-nums;
  }
  .content {
    flex: 1;
    overflow-y: auto;
  }
  @media (prefers-color-scheme: dark) {
    .sidebar {
      background: #232325;
      border-color: #3a3a3c;
    }
    .badge {
      background: #fff;
      color: #000;
    }
    .navitem {
      color: #d0d0d3;
    }
    .navitem:hover {
      background: #303032;
    }
    .navitem.active {
      background: #3a3a3c;
      color: #fff;
    }
  }
</style>
