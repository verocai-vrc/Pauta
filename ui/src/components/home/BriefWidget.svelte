<script lang="ts">
  import type { AgendaItem } from "../../lib/api";
  import { longDate } from "../../lib/dates";
  import { app } from "../../lib/store.svelte";
  import { todaySummary } from "../../lib/summary";

  let { agenda }: { agenda: AgendaItem[] } = $props();

  const today = $derived(app.today!);
  void agenda;
</script>

<section class="brief w-card" aria-labelledby="w-brief">
  <div class="day">
    <div class="kicker">{longDate(today.date)}</div>
    <h1 id="w-brief" class="display">Início</h1>
  </div>
  <div class="lines">
    <p class="dek">{todaySummary(today)}</p>
  </div>
</section>

<style>
  .brief {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);
    align-items: end;
    gap: 0 var(--sp-3);
    padding: var(--sp-3) var(--sp-4);
  }

  h1 {
    margin-top: 2px;
  }

  /* The notebook margin, the one trace of the ruled paper on this screen. */
  .lines {
    padding-left: var(--sp-4);
    border-left: 1.5px solid var(--margin-line);
  }

  .dek {
    line-height: 1.3;
  }
</style>
