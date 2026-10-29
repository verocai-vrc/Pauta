<script lang="ts">
  import { api, type BoardKind, type Column, type ColumnRole } from "../lib/api";
  import { t } from "../lib/i18n";
  import { app, board as boardById, fail, toast } from "../lib/store.svelte";
  import BoardView from "./BoardView.svelte";
  import Modal from "./Modal.svelte";

  const boards = $derived(app.index?.boards ?? []);
  const current = $derived(boardById(app.boardId));

  let creating = $state(false);
  let newName = $state("");
  let newKind = $state<BoardKind>("project");

  let editing = $state(false);
  let name = $state("");
  let color = $state("#7a2e3b");
  let columns = $state<Column[]>([]);
  let originalIds = $state<string[]>([]);

  function slug(s: string) {
    return (
      s
        .normalize("NFD")
        .replace(/[̀-ͯ]/g, "")
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-|-$/g, "") || "coluna"
    );
  }

  async function create(e: SubmitEvent) {
    e.preventDefault();
    try {
      const b = await api.createBoard(newName.trim(), newKind);
      creating = false;
      newName = "";
      app.boardId = b.id;
    } catch (err) {
      fail(err);
    }
  }

  function openSettings() {
    if (!current) return;
    name = current.name;
    color = current.color ?? "#7a2e3b";
    columns = current.columns.map((c) => ({ ...c }));
    originalIds = current.columns.map((c) => c.id);
    editing = true;
  }

  function addColumn() {
    columns.push({ id: "", name: "", role: null, limit: null });
  }

  function moveColumn(i: number, d: number) {
    const j = i + d;
    if (j < 0 || j >= columns.length) return;
    [columns[i], columns[j]] = [columns[j], columns[i]];
  }

  async function save(e: SubmitEvent) {
    e.preventDefault();
    if (!current) return;
    const used: string[] = [];
    const cols = columns
      .filter((c) => c.name.trim())
      .map((c) => {
        let id = c.id || slug(c.name);
        let n = 2;
        while (used.includes(id)) id = `${slug(c.name)}-${n++}`;
        used.push(id);
        return { ...c, id, name: c.name.trim(), limit: c.limit ? Number(c.limit) : null };
      });
    try {
      await api.updateBoard(current.id, { name: name.trim(), color, columns: cols }, current.hash);
      editing = false;
      toast(t("board.saved"));
    } catch (err) {
      fail(err);
    }
  }

  async function shiftBoard(d: number) {
    if (!current) return;
    const order = boards.map((b) => b.id);
    const i = order.indexOf(current.id);
    const j = i + d;
    if (j < 0 || j >= order.length) return;
    [order[i], order[j]] = [order[j], order[i]];
    await api.setBoardOrder(order).catch(fail);
  }

  async function archiveBoard() {
    if (!current || !confirm(t("board.archive_confirm", { name: current.name }))) return;
    try {
      await api.archiveBoard(current.id);
      editing = false;
      toast(t("board.archived"));
    } catch (err) {
      fail(err);
    }
  }

  function onTabKey(e: KeyboardEvent, idx: number) {
    if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
      e.preventDefault();
      if (e.altKey) {
        shiftBoard(e.key === "ArrowRight" ? 1 : -1);
        return;
      }
      const next = boards[(idx + (e.key === "ArrowRight" ? 1 : -1) + boards.length) % boards.length];
      app.boardId = next.id;
      (document.getElementById(`tab-${next.id}`) as HTMLElement | null)?.focus();
    }
  }

  const roles: (ColumnRole | "")[] = ["", "inbox", "scheduled", "done"];
</script>

<div class="boards">
  <div class="tabs-bar">
    <div class="tabs" role="tablist" aria-label={t("board.tabs")}>
      {#each boards as b, i (b.id)}
        <button
          id={`tab-${b.id}`}
          role="tab"
          class="tab"
          aria-selected={b.id === app.boardId}
          aria-controls="board-panel"
          tabindex={b.id === app.boardId ? 0 : -1}
          style:--board-color={b.color ?? "var(--accent)"}
          onclick={() => (app.boardId = b.id)}
          onkeydown={(e) => onTabKey(e, i)}
        >
          <span class="dot" aria-hidden="true"></span>{b.name}
          <span class="sr-only">({t(`board.kind_${b.kind}`)})</span>
        </button>
      {/each}
    </div>
    <div class="row actions">
      <button class="ghost" onclick={() => (creating = true)}>＋ {t("board.new")}</button>
      {#if current}<button class="ghost" onclick={openSettings}>⚙ {t("board.settings")}</button>{/if}
      <button class="ghost" onclick={() => (app.trashOpen = true)}>🗑 {t("trash.title")}</button>
    </div>
  </div>

  <div id="board-panel" role="tabpanel" aria-labelledby={current ? `tab-${current.id}` : undefined}>
    {#if current}
      {#key current.id}
        <BoardView board={current} />
      {/key}
    {:else}
      <p class="muted pad">{t("board.none")}</p>
    {/if}
  </div>
</div>

{#if creating}
  <Modal title={t("board.new")} onclose={() => (creating = false)}>
    <form class="form" onsubmit={create}>
      <label class="field">
        {t("board.name")}
        <!-- svelte-ignore a11y_autofocus -->
        <input type="text" bind:value={newName} required autofocus />
      </label>
      <fieldset>
        <legend class="small muted">{t("board.kind")}</legend>
        {#each ["content", "project", "generic"] as k (k)}
          <label class="radio">
            <input type="radio" name="kind" value={k} bind:group={newKind} />
            <span><strong>{t(`board.kind_${k}`)}</strong> <span class="small muted">{t(`board.kind_${k}_hint`)}</span></span>
          </label>
        {/each}
      </fieldset>
      <div class="row">
        <button class="primary" type="submit">{t("common.create")}</button>
        <button type="button" onclick={() => (creating = false)}>{t("common.cancel")}</button>
      </div>
    </form>
  </Modal>
{/if}

{#if editing && current}
  <Modal title={t("board.settings_of", { name: current.name })} onclose={() => (editing = false)} width="640px">
    <form class="form" onsubmit={save}>
      <div class="row">
        <label class="field" style="flex: 1">
          {t("board.name")}
          <input type="text" bind:value={name} required />
        </label>
        <label class="field">
          {t("board.color")}
          <input type="color" bind:value={color} />
        </label>
      </div>
      <fieldset>
        <legend class="small muted">{t("board.columns")}</legend>
        <table>
          <thead>
            <tr>
              <th>{t("board.col_name")}</th>
              <th>{t("board.col_role")}</th>
              <th>{t("board.col_limit")}</th>
              <th><span class="sr-only">{t("common.actions")}</span></th>
            </tr>
          </thead>
          <tbody>
            {#each columns as col, i (i)}
              <tr>
                <td>
                  <input type="text" bind:value={col.name} aria-label={t("board.col_name")} />
                  {#if col.id}<span class="small muted mono">{col.id}</span>{/if}
                </td>
                <td>
                  <select
                    value={col.role ?? ""}
                    onchange={(e) => (col.role = ((e.currentTarget as HTMLSelectElement).value || null) as ColumnRole | null)}
                    aria-label={t("board.col_role")}
                  >
                    {#each roles as r (r)}<option value={r}>{t(`role.${r || "none"}`)}</option>{/each}
                  </select>
                </td>
                <td>
                  <input type="number" min="1" style="width: 64px" bind:value={col.limit} aria-label={t("board.col_limit")} />
                </td>
                <td class="row">
                  <button type="button" class="ghost" aria-label={t("common.up")} onclick={() => moveColumn(i, -1)}>↑</button>
                  <button type="button" class="ghost" aria-label={t("common.down")} onclick={() => moveColumn(i, 1)}>↓</button>
                  <button
                    type="button"
                    class="ghost danger"
                    aria-label={t("board.col_remove")}
                    title={originalIds.includes(col.id) ? t("board.col_remove_hint") : ""}
                    onclick={() => columns.splice(i, 1)}>✕</button
                  >
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
        <button type="button" onclick={addColumn}>＋ {t("board.col_add")}</button>
      </fieldset>
      <div class="row">
        <button type="button" onclick={() => shiftBoard(-1)}>◀ {t("board.move_left")}</button>
        <button type="button" onclick={() => shiftBoard(1)}>{t("board.move_right")} ▶</button>
        <button type="button" onclick={() => api.openPath(current.dir).catch(fail)}>🗀 {t("board.open_folder")}</button>
      </div>
      <div class="row">
        <button class="primary" type="submit">{t("common.save")}</button>
        <button type="button" onclick={() => (editing = false)}>{t("common.cancel")}</button>
        <span class="spacer"></span>
        <button type="button" class="danger" onclick={archiveBoard}>{t("board.archive")}</button>
      </div>
    </form>
  </Modal>
{/if}

<style>
  .boards {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .tabs-bar {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-3) var(--sp-4);
    flex-wrap: wrap;
  }

  .tabs {
    display: flex;
    gap: 2px;
    flex-wrap: wrap;
    flex: 1;
  }

  .tab {
    border: none;
    background: transparent;
    border-bottom: 3px solid transparent;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    display: flex;
    gap: var(--sp-2);
    align-items: center;
  }

  .tab[aria-selected="true"] {
    border-bottom-color: var(--board-color);
    font-weight: 600;
    background: var(--surface);
  }

  .dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--board-color);
  }

  .pad {
    padding: var(--sp-4);
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  fieldset {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  fieldset > button {
    align-self: flex-start;
  }

  .radio {
    display: flex;
    gap: var(--sp-2);
    align-items: baseline;
  }

  table {
    border-collapse: collapse;
    width: 100%;
  }

  th {
    text-align: left;
    font-size: var(--fs-xs);
    color: var(--text-muted);
    font-weight: 500;
  }

  td {
    padding: 2px 4px 2px 0;
    vertical-align: top;
  }

  td input[type="text"] {
    width: 100%;
  }

  .mono {
    font-family: var(--font-mono);
    display: block;
  }

  .spacer {
    flex: 1;
  }
</style>
