<script lang="ts">
  import type { AgendaItem } from "../../lib/api";
  import { longDate } from "../../lib/dates";
  import { greeting, nextUp, nextUpText } from "../../lib/home/brief";
  import { app, networkLabel } from "../../lib/store.svelte";
  import { todaySummary } from "../../lib/summary";

  let { agenda }: { agenda: AgendaItem[] | null } = $props();

  const today = $derived(app.today!);
  const next = $derived(nextUp(agenda ?? [], today.date, today.time));
</script>

<section class="brief w-card" aria-labelledby="w-brief">
  <div class="day">
    <div class="kicker">{longDate(today.date)}</div>
    <h1 id="w-brief" class="display">{greeting(today.time)}</h1>
  </div>
  <div class="lines">
    <p class="dek">{todaySummary(today)}</p>
    {#if next}
      {@const line = nextUpText(next, today.date, networkLabel)}
      <p class="next ell" title={line}>{line}</p>
    {/if}
  </div>
</section>

<style>
  /* On the page's own columns, so the margin line falls on the edge of the Agenda below it. */
  .brief {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: subgrid;
    align-items: end;
    column-gap: var(--sp-3);
    padding: var(--sp-3) var(--sp-4);
  }

  .day {
    grid-column: 1;
  }

  h1 {
    margin-top: 2px;
  }

  /* The notebook margin, the one trace of the ruled paper on this screen. */
  .lines {
    grid-column: 2 / -1;
    align-self: stretch;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    min-width: 0;
    /* Edge to edge of the card, whatever the day says, so it reads as the margin of a notebook. */
    margin-block: calc(-1 * var(--sp-3));
    padding: var(--sp-3) 0 var(--sp-3) var(--sp-3);
    border-left: 1.5px solid var(--margin-line);
  }

  .dek {
    line-height: 1.3;
    text-wrap: balance;
  }

  .next {
    margin: 4px 0 0;
    font-size: var(--fs-sm);
    font-weight: 600;
    color: var(--text);
  }
</style>
