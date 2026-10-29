<script lang="ts">
  import { api, type AgendaItem, type AgendaKind } from "../lib/api";
  import {
    addDays, addMonths, hm, monthGrid, monthTitle, moveToDay, relativeDay, shortDate, weekDays, WEEKDAYS_SHORT,
  } from "../lib/dates";
  import { t } from "../lib/i18n";
  import { app, board, fail, networkLabel, openItem } from "../lib/store.svelte";
  import Modal from "./Modal.svelte";
  import WeekGrid from "./WeekGrid.svelte";

  type Mode = "month" | "week" | "agenda";
  const today = $derived(app.today?.date ?? new Date().toISOString().slice(0, 10));

  let mode = $state<Mode>("month");
  let cursor = $state("");
  let items = $state<AgendaItem[]>([]);
  let fBoard = $state("");
  let fNetwork = $state("");
  let fKinds = $state<Record<AgendaKind, boolean>>({ publication: true, due: true, event: true });
  let dragging = $state<AgendaItem | null>(null);
  let creatingOn = $state<string | null>(null);
  let newCardTitle = $state("");
  let newCardBoard = $state("");

  $effect(() => {
    if (!cursor && today) cursor = today;
  });

  const days = $derived(
    !cursor ? [] : mode === "month" ? monthGrid(cursor) : mode === "week" ? weekDays(cursor) : [],
  );
  const range = $derived.by(() => {
    if (!cursor) return null;
    if (mode === "agenda") return { from: cursor, to: addDays(cursor, 60) };
    return { from: days[0], to: days[days.length - 1] };
  });

  $effect(() => {
    void app.version;
    const r = range;
    if (r) api.agenda(r.from, r.to).then((x) => (items = x)).catch(fail);
  });

  const filtered = $derived(
    items.filter(
      (i) =>
        fKinds[i.kind] &&
        (!fBoard || i.board === fBoard) &&
        (!fNetwork || i.network === fNetwork),
    ),
  );

  function onDay(day: string) {
    return filtered.filter((i) => {
      const end = i.end?.date ?? i.when.date;
      return i.when.date <= day && end >= day;
    });
  }

  function go(d: number) {
    if (mode === "month") cursor = addMonths(cursor, d);
    else if (mode === "week") cursor = addDays(cursor, 7 * d);
    else cursor = addDays(cursor, 30 * d);
  }

  const title = $derived(
    !cursor
      ? ""
      : mode === "month"
        ? monthTitle(cursor)
        : mode === "week"
          ? `${shortDate(days[0])} – ${shortDate(days[6])}`
          : t("cal.from", { date: shortDate(cursor) }),
  );

  function icon(i: AgendaItem) {
    if (i.kind === "publication") return "↗";
    if (i.kind === "due") return "⚑";
    return ({ deadline: "⚑", milestone: "★", launch: "▲" } as Record<string, string>)[i.status ?? ""] ?? "◆";
  }

  function label(i: AgendaItem) {
    return `${kindLabel(i)}: ${i.title}${hm(i.when) ? ", " + hm(i.when) : ""}${i.done ? ", " + t("cal.done") : ""}`;
  }

  /** `time` is given when dropped on an hour of the week grid; otherwise the time of day is kept. */
  async function reschedule(i: AgendaItem, day: string, time?: string | null) {
    const to = time ? `${day}T${time}` : moveToDay(i.when, day);
    if (to === (i.when.time ? `${i.when.date}T${i.when.time.slice(0, 5)}` : i.when.date)) return;
    try {
      if (i.kind === "publication") await api.updateCard(i.id, { publication: { at: to } });
      else if (i.kind === "due") await api.updateCard(i.id, { due: to });
      else await api.updateEvent(i.id, { date: to });
    } catch (e) {
      fail(e);
    }
  }

  function startCreate(day: string) {
    creatingOn = day;
    newCardTitle = "";
    newCardBoard = fBoard || app.index?.boards[0]?.id || "";
  }

  async function createCard(e: SubmitEvent) {
    e.preventDefault();
    if (!creatingOn) return;
    try {
      const c = await api.createCard(newCardBoard, { title: newCardTitle, due: creatingOn });
      creatingOn = null;
      app.openCard = c.id;
    } catch (err) {
      fail(err);
    }
  }

  function createEvent() {
    app.newEventDate = creatingOn;
    app.openEvent = "new";
    creatingOn = null;
  }

  /** Days split into week rows, so the grid has the row > cell structure ARIA requires. */
  const weeks = $derived(Array.from({ length: Math.ceil(days.length / 7) }, (_, i) => days.slice(i * 7, i * 7 + 7)));

  function kindLabel(i: AgendaItem) {
    return i.kind === "publication" ? networkLabel(i.network) : i.kind === "due" ? t("kind.due") : t(`event.${i.status ?? "event"}`);
  }

  const agendaDays = $derived.by(() => {
    if (mode !== "agenda") return [];
    const byDay: Record<string, AgendaItem[]> = {};
    for (const i of filtered) (byDay[i.when.date] ??= []).push(i);
    return Object.entries(byDay).sort(([a], [b]) => a.localeCompare(b));
  });
</script>

{#snippet chip(i: AgendaItem)}
  <button
    class="item {i.kind} {i.network ?? ''}"
    class:done={i.done}
    draggable="true"
    title={label(i)}
    ondragstart={(e) => {
      dragging = i;
      e.dataTransfer?.setData("text/plain", i.id);
    }}
    ondragend={() => (dragging = null)}
    onclick={(e) => {
      e.stopPropagation();
      openItem(i.kind === "event" ? "event" : "card", i.id);
    }}
  >
    <span aria-hidden="true">{icon(i)}</span>
    <span class="sr-only">{kindLabel(i)}:</span>
    {#if hm(i.when)}<span class="t">{hm(i.when)}</span>{/if}
    <span class="txt">{i.title}</span>
    {#if i.done}<span class="sr-only">({t("cal.done")})</span>{/if}
  </button>
{/snippet}

<div class="calendar">
  <header class="row">
    <h1>{t("cal.title")}</h1>
    <div class="row" role="group" aria-label={t("cal.view")}>
      {#each ["month", "week", "agenda"] as m (m)}
        <button class:active={mode === m} aria-pressed={mode === m} onclick={() => (mode = m as Mode)}>{t(`cal.${m}`)}</button>
      {/each}
    </div>
    <div class="row">
      <button aria-label={t("cal.prev")} onclick={() => go(-1)}>‹</button>
      <button onclick={() => (cursor = today)}>{t("cal.today")}</button>
      <button aria-label={t("cal.next")} onclick={() => go(1)}>›</button>
      <strong class="period" aria-live="polite">{title}</strong>
    </div>
    <span class="spacer"></span>
    <button
      onclick={() => {
        app.newEventDate = today;
        app.openEvent = "new";
      }}>＋ {t("cal.new_event")}</button
    >
  </header>

  <div class="filters row" role="group" aria-label={t("cal.filters")}>
    {#each ["publication", "due", "event"] as k (k)}
      <label class="row small"><input type="checkbox" bind:checked={fKinds[k as AgendaKind]} /> {t(`cal.kind_${k}`)}</label>
    {/each}
    <select bind:value={fBoard} aria-label={t("cal.filter_board")}>
      <option value="">{t("cal.all_boards")}</option>
      {#each app.index?.boards ?? [] as b (b.id)}<option value={b.id}>{b.name}</option>{/each}
    </select>
    <select bind:value={fNetwork} aria-label={t("cal.filter_network")}>
      <option value="">{t("cal.all_networks")}</option>
      {#each app.index?.config.networks ?? [] as n (n.id)}<option value={n.id}>{n.label}</option>{/each}
    </select>
  </div>

  {#if mode === "agenda"}
    <div class="agenda">
      {#if agendaDays.length === 0}<p class="muted">{t("cal.nothing")}</p>{/if}
      {#each agendaDays as [day, list] (day)}
        <section>
          <h2 class:today={day === today}>{relativeDay(day, today)} <span class="muted small">{day}</span></h2>
          <ul>
            {#each list as i (i.kind + i.id)}
              <li>
                {@render chip(i)}
                {#if i.board}<span class="small muted">{board(i.board)?.name ?? i.board}</span>{/if}
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  {:else if mode === "week"}
    <WeekGrid
      {days}
      items={filtered}
      {today}
      now={app.today?.time ?? "00:00:00"}
      {kindLabel}
      {icon}
      {label}
      {dragging}
      ondragstart={(i, e) => {
        dragging = i;
        e.dataTransfer?.setData("text/plain", i.id);
      }}
      ondragend={() => (dragging = null)}
      onopen={(i) => openItem(i.kind === "event" ? "event" : "card", i.id)}
      oncreate={startCreate}
      onreschedule={(i, day, time) => {
        dragging = null;
        reschedule(i, day, time);
      }}
    />
  {:else}
    <div class="grid" role="grid" aria-label={title}>
      <div class="weekdays" role="row">
        {#each WEEKDAYS_SHORT as w (w)}<div role="columnheader" class="wd">{w}</div>{/each}
      </div>
      <div class="cells" role="rowgroup">
        {#each weeks as weekDays, w (w)}
        <div class="weekrow" role="row">
        {#each weekDays as day (day)}
          {@const list = onDay(day)}
          <div
            class="cell"
            class:other={mode === "month" && day.slice(0, 7) !== cursor.slice(0, 7)}
            class:today={day === today}
            class:over={dragging !== null}
            role="gridcell"
            tabindex="-1"
            ondragover={(e) => dragging && e.preventDefault()}
            ondrop={(e) => {
              e.preventDefault();
              if (dragging) reschedule(dragging, day);
              dragging = null;
            }}
          >
            <button class="daynum ghost" aria-label={t("cal.create_on", { date: shortDate(day) })} onclick={() => startCreate(day)}>
              {Number(day.slice(8))}
            </button>
            {#each list.slice(0, 4) as i (i.kind + i.id)}
              {@render chip(i)}
            {/each}
            {#if list.length > 4}
              <button
                class="more ghost small"
                onclick={() => {
                  cursor = day;
                  mode = "week";
                }}>+{list.length - 4}</button
              >
            {/if}
          </div>
        {/each}
        </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

{#if creatingOn}
  <Modal title={t("cal.create_on", { date: shortDate(creatingOn) })} onclose={() => (creatingOn = null)}>
    <button class="primary" onclick={createEvent}>◆ {t("cal.new_event")}</button>
    <form class="form" onsubmit={createCard}>
      <p class="small muted">{t("cal.or_card")}</p>
      <label class="field">
        {t("board.name")}
        <select bind:value={newCardBoard}>
          {#each app.index?.boards ?? [] as b (b.id)}<option value={b.id}>{b.name}</option>{/each}
        </select>
      </label>
      <label class="field">
        {t("card.title")}
        <input type="text" bind:value={newCardTitle} required />
      </label>
      <button type="submit">{t("cal.create_card")}</button>
    </form>
  </Modal>
{/if}

<style>
  .calendar {
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    height: 100%;
  }

  .spacer {
    flex: 1;
  }

  .period {
    margin-left: var(--sp-2);
  }

  .active {
    background: var(--accent-soft);
    color: var(--accent);
    border-color: var(--accent);
  }

  .grid {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .weekdays,
  .weekrow {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 2px;
  }

  .cells {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .weekrow {
    flex: 1;
    min-height: 96px;
  }

  .wd {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    padding: var(--sp-1);
  }

  .cell {
    background: var(--surface);
    border-radius: var(--radius-sm);
    padding: 2px 4px 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    overflow: hidden;
  }

  .cell.other {
    background: var(--surface-2);
  }

  .cell.other .daynum {
    font-weight: 400;
  }

  .cell.today {
    box-shadow: inset 0 0 0 2px var(--accent);
  }

  .cell.over:hover {
    box-shadow: inset 0 0 0 2px var(--focus);
  }

  .daynum {
    align-self: flex-start;
    font-size: var(--fs-xs);
    min-height: 20px;
    padding: 0 var(--sp-1);
    color: var(--text-muted);
  }

  .item {
    display: flex;
    gap: 4px;
    align-items: center;
    width: 100%;
    text-align: left;
    font-size: var(--fs-xs);
    min-height: 22px;
    padding: 1px 5px;
    border-radius: 4px;
    border: none;
    border-left: 3px solid var(--c-event);
    background: var(--surface-2);
    cursor: grab;
  }

  .item.due {
    border-left-color: var(--c-due);
  }

  .item.publication.linkedin {
    border-left-color: var(--c-linkedin);
  }

  .item.publication.instagram {
    border-left-color: var(--c-instagram);
  }

  /* Done items: struck through and muted by color, never by opacity (contrast). */
  .item.done {
    color: var(--text-muted);
    text-decoration: line-through;
  }

  .t {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }

  .txt {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .more {
    min-height: 18px;
    padding: 0 4px;
    align-self: flex-start;
  }

  .agenda {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
    max-width: 760px;
  }

  .agenda h2::first-letter {
    text-transform: uppercase;
  }

  .agenda h2.today {
    color: var(--accent);
  }

  .agenda ul {
    list-style: none;
    padding: 0;
    margin: var(--sp-2) 0 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .agenda li {
    display: flex;
    gap: var(--sp-2);
    align-items: center;
  }

  .agenda .item {
    font-size: var(--fs-md);
    min-height: 30px;
    max-width: 560px;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
</style>
