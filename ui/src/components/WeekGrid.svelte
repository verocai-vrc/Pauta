<script lang="ts">
  // Week view as an hourly grid (section 8.1). Each day is a labelled list of items;
  // the blocks are positioned by time but stay in chronological order for readers.
  import { onMount } from "svelte";
  import type { AgendaItem } from "../lib/api";
  import { hm, relativeDay, WEEKDAYS_SHORT, shortDate, weekday } from "../lib/dates";
  import { t } from "../lib/i18n";
  import { HOUR_PX, layoutDay, minutesOf, timeAtOffset } from "../lib/weekgrid";

  let {
    days,
    items,
    today,
    now,
    kindLabel,
    icon,
    label,
    ondragstart,
    ondragend,
    onopen,
    oncreate,
    onreschedule,
    dragging,
  }: {
    days: string[];
    items: AgendaItem[];
    today: string;
    /** Local `HH:MM[:SS]` right now, in the configured zone. */
    now: string;
    kindLabel: (i: AgendaItem) => string;
    icon: (i: AgendaItem) => string;
    label: (i: AgendaItem) => string;
    ondragstart: (i: AgendaItem, e: DragEvent) => void;
    ondragend: () => void;
    onopen: (i: AgendaItem) => void;
    oncreate: (day: string) => void;
    /** `time` is null when dropped on the all-day row. */
    onreschedule: (i: AgendaItem, day: string, time: string | null) => void;
    dragging: AgendaItem | null;
  } = $props();

  const hours = Array.from({ length: 24 }, (_, h) => h);
  let scroller: HTMLDivElement;

  const allDay = (day: string) =>
    items.filter((i) => {
      const end = i.end?.date ?? i.when.date;
      return i.when.time === null && i.when.date <= day && end >= day;
    });

  // Open scrolled to the first item of the week (or 07:00), a little above it. In the current
  // week, if the now-line would be below the visible hours, scroll down until it shows.
  onMount(() => {
    const timed = items.filter((i) => i.when.time).map((i) => minutesOf(i.when.time));
    const first = Math.min(...timed, 7 * 60);
    let top = Math.max(0, first / 60 - 1);
    if (days.includes(today)) {
      const visible = scroller.clientHeight / HOUR_PX;
      const nowH = minutesOf(now) / 60;
      if (nowH > top + visible - 1) top = nowH - visible + 2;
    }
    scroller.scrollTop = Math.max(0, top * HOUR_PX);
  });

  function drop(e: DragEvent, day: string) {
    e.preventDefault();
    if (!dragging) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    onreschedule(dragging, day, timeAtOffset(e.clientY - rect.top));
  }
</script>

{#snippet block(i: AgendaItem)}
  <button
    class="item {i.kind} {i.network ?? ''}"
    class:done={i.done}
    draggable="true"
    title={label(i)}
    ondragstart={(e) => ondragstart(i, e)}
    {ondragend}
    onclick={() => onopen(i)}
  >
    <span aria-hidden="true">{icon(i)}</span>
    <span class="sr-only">{kindLabel(i)}:</span>
    {#if hm(i.when)}<span class="t">{hm(i.when)}</span>{/if}
    <span class="txt">{i.title}</span>
    {#if i.done}<span class="sr-only">({t("cal.done")})</span>{/if}
  </button>
{/snippet}

<div class="week" style:--hour={`${HOUR_PX}px`}>
  <div class="head">
    <div class="gutter"></div>
    {#each days as day (day)}
      <div class="dayhead" class:today={day === today}>
        <span class="wd">{WEEKDAYS_SHORT[weekday(day)]}</span>
        <button class="daynum ghost" aria-label={t("cal.create_on", { date: shortDate(day) })} onclick={() => oncreate(day)}>
          {Number(day.slice(8))}
        </button>
      </div>
    {/each}
  </div>

  <div class="alldayrow">
    <div class="gutter small muted">{t("cal.all_day")}</div>
    {#each days as day (day)}
      <section
        class="allday"
        aria-label={`${relativeDay(day, today)}: ${t("cal.all_day")}`}
        ondragover={(e) => dragging?.when.time === null && e.preventDefault()}
        ondrop={(e) => {
          e.preventDefault();
          if (dragging?.when.time === null) onreschedule(dragging, day, null);
        }}
      >
        <ul>
          {#each allDay(day) as i (i.kind + i.id)}<li>{@render block(i)}</li>{/each}
        </ul>
      </section>
    {/each}
  </div>

  <div class="scroller" bind:this={scroller}>
    <div class="body">
      <div class="gutter hours" aria-hidden="true">
        {#each hours as h (h)}<div class="hour small muted">{String(h).padStart(2, "0")}:00</div>{/each}
      </div>
      {#each days as day (day)}
        {@const placed = layoutDay(items, day)}
        <section
          class="col"
          class:today={day === today}
          aria-label={relativeDay(day, today)}
          ondragover={(e) => dragging && e.preventDefault()}
          ondrop={(e) => drop(e, day)}
        >
          {#if day === today}
            <div class="now" style:top={`${(minutesOf(now) / 60) * HOUR_PX}px`} aria-hidden="true"></div>
          {/if}
          <ul>
            {#each placed as p (p.item.kind + p.item.id)}
              <li
                style:top={`${(p.start / 60) * HOUR_PX}px`}
                style:height={`${((p.end - p.start) / 60) * HOUR_PX - 2}px`}
                style:left={`${(p.lane / p.lanes) * 100}%`}
                style:width={`${100 / p.lanes}%`}
              >
                {@render block(p.item)}
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  </div>
</div>

<style>
  .week {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    background: var(--surface);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .head,
  .alldayrow,
  .body {
    display: grid;
    grid-template-columns: 64px repeat(7, minmax(0, 1fr));
  }

  .head {
    border-bottom: 1px solid var(--border);
  }

  .dayhead {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    padding: var(--sp-1) var(--sp-2);
    border-left: 1px solid var(--border);
  }

  .wd {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .dayhead.today .daynum {
    background: var(--accent);
    color: var(--accent-text);
    border-radius: 50%;
  }

  .daynum {
    min-height: 26px;
    min-width: 26px;
    padding: 0;
  }

  .alldayrow {
    border-bottom: 1px solid var(--border);
    min-height: 30px;
  }

  .allday {
    border-left: 1px solid var(--border);
    padding: 2px;
  }

  .gutter {
    padding: 2px var(--sp-1);
    text-align: right;
  }

  .scroller {
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .body {
    height: calc(24 * var(--hour));
    position: relative;
  }

  .hours {
    display: flex;
    flex-direction: column;
  }

  .hour {
    height: var(--hour);
    transform: translateY(-0.6em);
  }

  .col {
    position: relative;
    border-left: 1px solid var(--border);
    background-image: repeating-linear-gradient(to bottom, transparent 0, transparent calc(var(--hour) - 1px), var(--border) calc(var(--hour) - 1px), var(--border) var(--hour));
  }

  .col.today {
    background-color: var(--accent-soft);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .allday ul {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  /* A thin line across today's column; a dot on the left edge keeps it visible without color. */
  .now {
    position: absolute;
    left: 0;
    right: 0;
    height: 0;
    border-top: 2px solid var(--danger);
    pointer-events: none;
    z-index: 1;
  }

  .now::before {
    content: "";
    position: absolute;
    left: -5px;
    top: -6px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--danger);
  }

  .col li {
    position: absolute;
    padding: 0 1px;
  }

  .item {
    display: flex;
    gap: 4px;
    align-items: flex-start;
    width: 100%;
    height: 100%;
    text-align: left;
    font-size: var(--fs-xs);
    min-height: 22px;
    padding: 2px 5px;
    border-radius: 4px;
    border: none;
    border-left: 3px solid var(--c-event);
    background: var(--surface-2);
    cursor: grab;
    overflow: hidden;
  }

  .allday .item {
    height: auto;
    align-items: center;
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
  }
</style>
