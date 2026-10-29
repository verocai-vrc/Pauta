<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import { t } from "./lib/i18n";
  import { app, start, type View } from "./lib/store.svelte";
  import Setup from "./components/Setup.svelte";
  import TodayView from "./components/Today.svelte";
  import Boards from "./components/Boards.svelte";
  import Calendar from "./components/Calendar.svelte";
  import Rhythm from "./components/Rhythm.svelte";
  import Settings from "./components/Settings.svelte";
  import CardEditor from "./components/CardEditor.svelte";
  import EventEditor from "./components/EventEditor.svelte";
  import CommandPalette from "./components/CommandPalette.svelte";
  import TrashArchive from "./components/TrashArchive.svelte";
  import Toasts from "./components/Toasts.svelte";

  const views: { id: View; key: string; icon: string; shortcut: string }[] = [
    { id: "today", key: "nav.today", icon: "☀", shortcut: "Alt+1" },
    { id: "boards", key: "nav.boards", icon: "▦", shortcut: "Alt+2" },
    { id: "calendar", key: "nav.calendar", icon: "▤", shortcut: "Alt+3" },
    { id: "rhythm", key: "nav.rhythm", icon: "▥", shortcut: "Alt+4" },
    { id: "settings", key: "nav.settings", icon: "⚙", shortcut: "Alt+5" },
  ];

  async function applyTheme() {
    try {
      const p = await api.preferences();
      if (p.theme === "system") document.documentElement.removeAttribute("data-theme");
      else document.documentElement.dataset.theme = p.theme;
    } catch {
      /* preferences are optional */
    }
  }

  onMount(() => {
    start();
    applyTheme();
  });

  function onKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      app.paletteOpen = !app.paletteOpen;
    } else if (e.altKey && ["1", "2", "3", "4", "5"].includes(e.key)) {
      e.preventDefault();
      app.view = views[Number(e.key) - 1].id;
    } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "i") {
      e.preventDefault();
      api.openCapture();
    }
  }
</script>

<svelte:window onkeydown={onKey} />

{#if app.status && !app.status.root}
  <Setup />
{:else}
  <div class="shell">
    <nav class="sidebar" aria-label={t("nav.label")}>
      <div class="brand" aria-hidden="true">Pauta</div>
      <ul>
        {#each views as v (v.id)}
          <li>
            <button
              class="nav-item"
              class:active={app.view === v.id}
              aria-current={app.view === v.id ? "page" : undefined}
              title={`${t(v.key)} (${v.shortcut})`}
              onclick={() => (app.view = v.id)}
            >
              <span class="icon" aria-hidden="true">{v.icon}</span>
              <span>{t(v.key)}</span>
            </button>
          </li>
        {/each}
      </ul>
      <div class="spacer"></div>
      <button class="nav-item" onclick={() => api.openCapture()} title={t("nav.capture_hint")}>
        <span class="icon" aria-hidden="true">＋</span><span>{t("nav.capture")}</span>
      </button>
      <button class="nav-item" onclick={() => (app.paletteOpen = true)} title="Ctrl+K">
        <span class="icon" aria-hidden="true">⌕</span><span>{t("nav.search")}</span>
      </button>
    </nav>

    <main class="content">
      {#if app.loadError}
        <div class="load-error" role="alert">
          <strong>{t("app.load_error")}</strong>
          <p>{app.loadError}</p>
        </div>
      {/if}
      {#if app.index}
        {#if app.view === "today"}
          <TodayView />
        {:else if app.view === "boards"}
          <Boards />
        {:else if app.view === "calendar"}
          <Calendar />
        {:else if app.view === "rhythm"}
          <Rhythm />
        {:else}
          <Settings />
        {/if}
      {:else if !app.loadError}
        <p class="muted loading">{t("app.loading")}</p>
      {/if}
    </main>
  </div>

  {#if app.openCard}
    {#key app.openCard}
      <CardEditor id={app.openCard} />
    {/key}
  {/if}
  {#if app.openEvent}
    {#key app.openEvent}
      <EventEditor id={app.openEvent} />
    {/key}
  {/if}
  {#if app.paletteOpen}
    <CommandPalette />
  {/if}
  {#if app.trashOpen}
    <TrashArchive />
  {/if}
{/if}
<Toasts />

<style>
  .shell {
    display: grid;
    grid-template-columns: 200px 1fr;
    height: 100%;
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    padding: var(--sp-3) var(--sp-2);
    background: var(--surface);
    border-right: 1px solid var(--border);
  }

  .brand {
    font-weight: 700;
    font-size: var(--fs-lg);
    color: var(--accent);
    padding: var(--sp-1) var(--sp-2) var(--sp-3);
    letter-spacing: 0.02em;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .spacer {
    flex: 1;
  }

  .nav-item {
    display: flex;
    gap: var(--sp-2);
    align-items: center;
    width: 100%;
    border: none;
    background: transparent;
    text-align: left;
    padding: var(--sp-2);
  }

  .nav-item.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }

  .icon {
    width: 1.2em;
    text-align: center;
  }

  .content {
    overflow: auto;
    min-width: 0;
  }

  .loading {
    padding: var(--sp-5);
  }

  .load-error {
    margin: var(--sp-4);
    padding: var(--sp-3);
    border-radius: var(--radius-md);
    background: var(--danger-soft);
    color: var(--danger);
  }

  .load-error p {
    margin: var(--sp-1) 0 0;
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }

  @media (max-width: 860px) {
    .shell {
      grid-template-columns: 56px 1fr;
    }

    .nav-item span:not(.icon),
    .brand {
      display: none;
    }
  }
</style>
