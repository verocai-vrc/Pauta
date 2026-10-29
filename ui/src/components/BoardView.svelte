<script lang="ts">
  import { tick } from "svelte";
  import { api, type Board, type Card } from "../lib/api";
  import { t } from "../lib/i18n";
  import { app, fail } from "../lib/store.svelte";
  import CardTile from "./CardTile.svelte";

  let { board }: { board: Board } = $props();

  let search = $state("");
  let tag = $state("");
  let adding = $state<string | null>(null);
  let newTitle = $state("");
  let dragId = $state<string | null>(null);
  let dropTarget = $state<{ column: string; before: string | null } | null>(null);
  let announcement = $state("");

  const allCards = $derived(app.index?.cards.filter((c) => c.board === board.id) ?? []);
  const tags = $derived([...new Set(allCards.flatMap((c) => c.tags))].sort());
  const known = $derived(new Set(board.columns.map((c) => c.id)));

  function columnCards(columnId: string, isFirst: boolean): Card[] {
    // Cards with an unknown column (hand-edited) show up in the first column.
    return allCards.filter((c) => c.column === columnId || (isFirst && !known.has(c.column)));
  }

  function visible(c: Card): boolean {
    const q = search.trim().toLowerCase();
    if (tag && !c.tags.includes(tag)) return false;
    if (q && !c.title.toLowerCase().includes(q) && !c.body.toLowerCase().includes(q)) return false;
    return true;
  }

  /** Position in the full column (without the moving card) for "drop before `before`". */
  function indexFor(column: string, before: string | null, moving: string): number | null {
    if (!before) return null;
    const ids = allCards.filter((c) => c.column === column && c.id !== moving).map((c) => c.id);
    const i = ids.indexOf(before);
    return i < 0 ? null : i;
  }

  async function move(id: string, column: string, index: number | null) {
    const c = allCards.find((x) => x.id === id);
    try {
      await api.moveCard(id, column, index);
      const colName = board.columns.find((x) => x.id === column)?.name ?? column;
      announcement = t("board.moved", { title: c?.title ?? "", column: colName });
      await tick();
      (document.querySelector(`[data-id="${id}"]`) as HTMLElement | null)?.focus();
    } catch (e) {
      fail(e);
    }
  }

  function focusCard(id: string | undefined) {
    if (id) (document.querySelector(`[data-id="${id}"]`) as HTMLElement | null)?.focus();
  }

  function onCardKey(e: KeyboardEvent, c: Card, colIdx: number) {
    const cols = board.columns;
    const col = cols[colIdx];
    const list = columnCards(col.id, colIdx === 0).filter(visible);
    const pos = list.findIndex((x) => x.id === c.id);
    const full = allCards.filter((x) => x.column === col.id);
    const fullPos = full.findIndex((x) => x.id === c.id);
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      app.openCard = c.id;
    } else if (e.altKey && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
      e.preventDefault();
      const target = cols[colIdx + (e.key === "ArrowLeft" ? -1 : 1)];
      if (target) move(c.id, target.id, Math.max(0, fullPos));
    } else if (e.altKey && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
      e.preventDefault();
      const to = fullPos + (e.key === "ArrowUp" ? -1 : 1);
      if (to >= 0 && to < full.length) move(c.id, col.id, to);
    } else if (e.key === "ArrowUp" || e.key === "ArrowDown") {
      e.preventDefault();
      focusCard(list[pos + (e.key === "ArrowUp" ? -1 : 1)]?.id);
    } else if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      e.preventDefault();
      const nextIdx = colIdx + (e.key === "ArrowLeft" ? -1 : 1);
      const nextCol = cols[nextIdx];
      if (nextCol) focusCard(columnCards(nextCol.id, nextIdx === 0).filter(visible)[0]?.id);
    }
  }

  function onDragOver(e: DragEvent, column: string) {
    if (!dragId) return;
    e.preventDefault();
    const zone = e.currentTarget as HTMLElement;
    const tiles = [...zone.querySelectorAll<HTMLElement>("[data-id]")].filter((el) => el.dataset.id !== dragId);
    const before = tiles.find((el) => {
      const r = el.getBoundingClientRect();
      return e.clientY < r.top + r.height / 2;
    });
    dropTarget = { column, before: before?.dataset.id ?? null };
  }

  function onDrop(e: DragEvent, column: string) {
    e.preventDefault();
    if (dragId && dropTarget?.column === column) {
      move(dragId, column, indexFor(column, dropTarget.before, dragId));
    }
    dragId = null;
    dropTarget = null;
  }

  async function quickAdd(e: SubmitEvent, column: string) {
    e.preventDefault();
    const title = newTitle.trim();
    if (!title) return;
    try {
      await api.createCard(board.id, { title, column, top: true });
      newTitle = "";
    } catch (err) {
      fail(err);
    }
  }
</script>

<div class="toolbar row">
  <input type="search" bind:value={search} placeholder={t("board.search")} aria-label={t("board.search")} />
  <select bind:value={tag} aria-label={t("board.filter_tag")}>
    <option value="">{t("board.all_tags")}</option>
    {#each tags as tg (tg)}<option value={tg}>#{tg}</option>{/each}
  </select>
  <span class="small muted">{t("board.kbd_short")}</span>
</div>
<p id="board-kbd-help" class="sr-only">{t("board.kbd_help")}</p>
<div class="sr-only" aria-live="polite">{announcement}</div>

<div class="columns">
  {#each board.columns as col, colIdx (col.id)}
    {@const cards = columnCards(col.id, colIdx === 0)}
    {@const shown = cards.filter(visible)}
    {@const over = col.limit != null && cards.length > col.limit}
    <section class="column" class:over aria-labelledby={`col-${col.id}`}>
      <header>
        <h3 id={`col-${col.id}`}>{col.name}</h3>
        <span class="count" class:warn={over} title={over ? t("board.over_limit") : ""}>
          {cards.length}{#if col.limit != null}/{col.limit}{/if}
          {#if over}<span class="sr-only">{t("board.over_limit")}</span> ⚠{/if}
        </span>
        <button
          class="ghost add"
          aria-label={t("board.add_to", { column: col.name })}
          onclick={() => {
            adding = adding === col.id ? null : col.id;
            newTitle = "";
          }}>＋</button
        >
      </header>
      {#if adding === col.id}
        <form onsubmit={(e) => quickAdd(e, col.id)}>
          <!-- svelte-ignore a11y_autofocus -->
          <input
            type="text"
            bind:value={newTitle}
            placeholder={t("board.new_card")}
            aria-label={t("board.new_card")}
            autofocus
            onkeydown={(e) => e.key === "Escape" && (adding = null)}
          />
        </form>
      {/if}
      <div
        class="drop"
        role="list"
        ondragover={(e) => onDragOver(e, col.id)}
        ondragleave={(e) => {
          if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node)) dropTarget = null;
        }}
        ondrop={(e) => onDrop(e, col.id)}
      >
        {#each shown as c (c.id)}
          {#if dropTarget?.column === col.id && dropTarget.before === c.id}<div class="marker" aria-hidden="true"></div>{/if}
          <div role="listitem" class:dragging={dragId === c.id}>
            <CardTile
              card={c}
              onkeydown={(e) => onCardKey(e, c, colIdx)}
              ondragstart={(e) => {
                dragId = c.id;
                e.dataTransfer?.setData("text/plain", c.id);
                if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
              }}
              ondragend={() => {
                dragId = null;
                dropTarget = null;
              }}
            />
          </div>
        {/each}
        {#if dropTarget?.column === col.id && dropTarget.before === null}<div class="marker" aria-hidden="true"></div>{/if}
      </div>
      {#if shown.length === 0}<p class="empty small muted">{t("board.empty_column")}</p>{/if}
    </section>
  {/each}
</div>

<style>
  .toolbar {
    padding: 0 var(--sp-4) var(--sp-3);
  }

  .toolbar input {
    min-width: 220px;
  }

  .columns {
    display: flex;
    gap: var(--sp-3);
    padding: 0 var(--sp-4) var(--sp-4);
    align-items: flex-start;
    overflow-x: auto;
    min-height: 60vh;
  }

  .column {
    flex: 0 0 272px;
    background: var(--surface-2);
    border-radius: var(--radius-md);
    padding: var(--sp-2);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    max-height: calc(100vh - 170px);
  }

  .column.over {
    box-shadow: inset 0 0 0 2px var(--warning);
  }

  header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 0 var(--sp-1);
  }

  header h3 {
    flex: 1;
  }

  .count {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .count.warn {
    color: var(--warning);
    font-weight: 700;
  }

  .add {
    min-height: 24px;
    padding: 0 var(--sp-2);
  }

  form input {
    width: 100%;
  }

  .drop {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    overflow-y: auto;
    min-height: 48px;
    padding: 2px;
  }

  .dragging {
    opacity: 0.4;
  }

  .marker {
    height: 3px;
    border-radius: 2px;
    background: var(--focus);
  }

  .empty {
    text-align: center;
    margin: var(--sp-2) 0;
  }
</style>
