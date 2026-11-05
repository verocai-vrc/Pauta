<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import { t } from "./lib/i18n";
  import { app, start, type View } from "./lib/store.svelte";
  import Icon, { type IconName } from "./components/Icon.svelte";
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

  const views: { id: View; key: string; icon: IconName; shortcut: string }[] = [
    { id: "today", key: "nav.today", icon: "sun", shortcut: "Alt+1" },
    { id: "boards", key: "nav.boards", icon: "columns-3", shortcut: "Alt+2" },
    { id: "calendar", key: "nav.calendar", icon: "calendar", shortcut: "Alt+3" },
    { id: "rhythm", key: "nav.rhythm", icon: "chart-column", shortcut: "Alt+4" },
    { id: "settings", key: "nav.settings", icon: "settings", shortcut: "Alt+5" },
  ];
  const SEARCH_SHORTCUT = "Ctrl+K";
  const CAPTURE_SHORTCUT = "Ctrl+Shift+I";

  /** The reading screens sit on ruled paper; the boards, the calendar and the editor do not. */
  const paperViews: View[] = ["today", "rhythm", "settings"];

  /** `aria-keyshortcuts` spells the modifier "Control". */
  const aria = (shortcut: string) => shortcut.replace("Ctrl", "Control");

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
      <div class="brand" aria-hidden="true">
        Pauta
        <small>{t("nav.tagline")}</small>
      </div>
      <ul>
        {#each views as v (v.id)}
          <li>
            <button
              class="nav-item"
              class:active={app.view === v.id}
              aria-current={app.view === v.id ? "page" : undefined}
              aria-keyshortcuts={aria(v.shortcut)}
              title={`${t(v.key)} (${v.shortcut})`}
              onclick={() => (app.view = v.id)}
            >
              <Icon name={v.icon} />
              <span class="label">{t(v.key)}</span>
              <kbd aria-hidden="true">{v.shortcut}</kbd>
            </button>
          </li>
        {/each}
      </ul>
      <div class="spacer"></div>
      <button
        class="nav-item"
        aria-keyshortcuts={aria(SEARCH_SHORTCUT)}
        onclick={() => (app.paletteOpen = true)}
        title={`${t("nav.search")} (${SEARCH_SHORTCUT})`}
      >
        <Icon name="search" />
        <span class="label">{t("nav.search")}</span>
        <kbd aria-hidden="true">{SEARCH_SHORTCUT}</kbd>
      </button>
      <button
        class="nav-item primary capture"
        aria-keyshortcuts={aria(CAPTURE_SHORTCUT)}
        onclick={() => api.openCapture()}
        title={t("nav.capture_hint")}
      >
        <Icon name="plus" />
        <span class="label">{t("nav.capture")}</span>
        <kbd aria-hidden="true">{CAPTURE_SHORTCUT}</kbd>
      </button>
    </nav>

    <main class="content" class:paper={paperViews.includes(app.view)}>
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
    grid-template-columns: 216px 1fr;
    height: 100%;
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    padding: var(--sp-4) var(--sp-2) var(--sp-3);
    background: var(--sidebar);
    border-right: 1px solid var(--border);
  }

  .brand {
    font-family: var(--font-serif);
    font-weight: 600;
    font-size: var(--fs-xl);
    line-height: 1.1;
    letter-spacing: -0.01em;
    color: var(--accent);
    padding: 0 var(--sp-3) var(--sp-4);
  }

  .brand small {
    display: block;
    margin-top: 3px;
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--text-muted);
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
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-muted);
    text-align: left;
    padding: var(--sp-2) var(--sp-3);
  }

  .nav-item:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .nav-item.active {
    background: var(--surface);
    color: var(--accent);
    font-weight: 600;
    box-shadow:
      var(--shadow-1),
      inset 3px 0 0 var(--accent);
  }

  /* Capture is the one primary action of the shell. */
  .nav-item.capture {
    flex-wrap: wrap;
    margin-top: var(--sp-1);
    background: var(--accent);
    color: var(--accent-text);
    font-weight: 600;
  }

  /* If both do not fit, the shortcut moves to a second line and the label stays whole. */
  .capture .label {
    white-space: nowrap;
  }

  .nav-item.capture:hover {
    background: var(--accent);
    color: var(--accent-text);
    filter: brightness(1.08);
  }

  .label {
    min-width: 0;
  }

  kbd {
    margin-left: auto;
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    font-weight: 400;
    white-space: nowrap;
    color: var(--text-muted);
  }

  .capture kbd {
    color: inherit;
    opacity: 0.85;
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

    .label,
    kbd,
    .brand {
      display: none;
    }

    .nav-item {
      justify-content: center;
      padding-inline: var(--sp-2);
    }
  }
</style>
