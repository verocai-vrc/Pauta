<script lang="ts">
  import { api } from "../lib/api";
  import { t } from "../lib/i18n";
  import { app, board } from "../lib/store.svelte";
  import Modal from "./Modal.svelte";

  interface Cmd {
    label: string;
    hint?: string;
    run: () => void;
  }

  let query = $state("");
  let selected = $state(0);

  function close() {
    app.paletteOpen = false;
  }

  const commands = $derived.by<Cmd[]>(() => {
    const base: Cmd[] = [
      { label: t("nav.today"), hint: t("palette.view"), run: () => (app.view = "today") },
      { label: t("nav.boards"), hint: t("palette.view"), run: () => (app.view = "boards") },
      { label: t("nav.calendar"), hint: t("palette.view"), run: () => (app.view = "calendar") },
      { label: t("nav.rhythm"), hint: t("palette.view"), run: () => (app.view = "rhythm") },
      { label: t("nav.settings"), hint: t("palette.view"), run: () => (app.view = "settings") },
      { label: t("nav.capture"), hint: t("palette.action"), run: () => api.openCapture() },
      {
        label: t("cal.new_event"),
        hint: t("palette.action"),
        run: () => {
          app.newEventDate = app.today?.date ?? null;
          app.openEvent = "new";
        },
      },
      { label: t("trash.title"), hint: t("palette.action"), run: () => (app.trashOpen = true) },
    ];
    for (const b of app.index?.boards ?? []) {
      base.push({
        label: b.name,
        hint: t("palette.board"),
        run: () => {
          app.view = "boards";
          app.boardId = b.id;
        },
      });
    }
    for (const c of app.index?.cards ?? []) {
      base.push({ label: c.title, hint: board(c.board)?.name, run: () => (app.openCard = c.id) });
    }
    for (const e of app.index?.events ?? []) {
      base.push({ label: e.title, hint: `${t("palette.event")} · ${e.when.date}`, run: () => (app.openEvent = e.id) });
    }
    return base;
  });

  const fold = (s: string) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
  const results = $derived.by(() => {
    const q = fold(query.trim());
    const list = q ? commands.filter((c) => fold(c.label).includes(q)) : commands.slice(0, 8);
    return list.slice(0, 30);
  });

  $effect(() => {
    void query;
    selected = 0;
  });

  function run(c: Cmd | undefined) {
    if (!c) return;
    close();
    c.run();
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selected = Math.min(selected + 1, results.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selected = Math.max(selected - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      run(results[selected]);
    } else if (e.key === "Escape") {
      // A search field would only clear itself on Esc; close the palette instead.
      e.preventDefault();
      close();
    }
  }
</script>

<Modal title={t("palette.title")} onclose={close} width="600px">
  <!-- svelte-ignore a11y_autofocus -->
  <input
    type="search"
    bind:value={query}
    onkeydown={onKey}
    placeholder={t("palette.placeholder")}
    aria-label={t("palette.placeholder")}
    role="combobox"
    aria-expanded="true"
    aria-controls="palette-list"
    aria-activedescendant={results.length ? `palette-${selected}` : undefined}
    autofocus
  />
  <ul id="palette-list" role="listbox">
    {#each results as c, i (i)}
      <li
        id={`palette-${i}`}
        role="option"
        aria-selected={i === selected}
        class:sel={i === selected}
        onclick={() => run(c)}
        onkeydown={() => {}}
        onmousemove={() => (selected = i)}
      >
        <span>{c.label}</span>
        {#if c.hint}<span class="small muted">{c.hint}</span>{/if}
      </li>
    {/each}
    {#if results.length === 0}<li class="muted">{t("palette.none")}</li>{/if}
  </ul>
</Modal>

<style>
  input {
    width: 100%;
    font-size: var(--fs-lg);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 50vh;
    overflow-y: auto;
  }

  li {
    display: flex;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-2);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  li.sel {
    background: var(--accent-soft);
  }
</style>
