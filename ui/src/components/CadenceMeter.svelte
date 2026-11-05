<script lang="ts">
  // "1 of 2 this week": filled = published, outlined = scheduled, empty = open.
  // Shape and text carry the meaning; color is only a hint. No alarms, no streaks.
  import type { CadenceProgress } from "../lib/api";
  import { shortDate } from "../lib/dates";
  import { t, tn } from "../lib/i18n";

  // `inline` is the Início row: network, dots and progress on one line. Without it nothing changes.
  let { progress, compact = false, inline = false }: { progress: CadenceProgress; compact?: boolean; inline?: boolean } = $props();

  const slots = $derived(
    Array.from({ length: Math.min(progress.goal, 12) }, (_, i) =>
      i < progress.done ? "done" : i < progress.done + progress.scheduled ? "scheduled" : "open",
    ),
  );
  const missing = $derived(Math.max(0, progress.goal - progress.done - progress.scheduled));
  const period = $derived(progress.per === "week" ? t("cadence.this_week") : t("cadence.this_month"));
</script>

{#snippet dots()}
  <div class="dots" role="img" aria-label={t("cadence.aria", { done: progress.done, scheduled: progress.scheduled, goal: progress.goal })}>
    {#each slots as s, i (i)}<span class="dot {s}"></span>{/each}
    {#if progress.goal > 12}<span class="small muted">…</span>{/if}
  </div>
{/snippet}

{#if inline}
  <div class="meter inline">
    <strong class="chip {progress.network} net ell" title={progress.label}>{progress.label}</strong>
    {@render dots()}
    <span class="of">{t("cadence.of", { done: progress.done, goal: progress.goal })} {period}</span>
  </div>
{:else}
  <div class="meter">
    <div class="row head">
      <strong class="chip {progress.network}">{progress.label}</strong>
      <span>{t("cadence.of", { done: progress.done, goal: progress.goal })} {period}</span>
    </div>
    {@render dots()}
    {#if !compact}
      <p class="small muted">
        {progress.scheduled > 0 ? tn("cadence.scheduled", progress.scheduled) : ""}
        {#if missing > 0}{tn("cadence.missing", missing)}{:else}{t("cadence.covered")}{/if}
        · {shortDate(progress.period_start)} – {shortDate(progress.period_end)}
      </p>
    {/if}
  </div>
{/if}

<style>
  .meter {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .head {
    justify-content: space-between;
  }

  .dots {
    display: flex;
    gap: var(--sp-1);
    align-items: center;
  }

  .dot {
    width: var(--dot, 22px);
    height: var(--dot, 22px);
    border-radius: 50%;
    border: 2px solid var(--text-muted);
    box-sizing: border-box;
  }

  .dot.done {
    background: var(--ok);
    border-color: var(--ok);
  }

  .dot.scheduled {
    border-style: dashed;
    border-color: var(--accent);
  }

  p {
    margin: 0;
  }

  .inline {
    /* From the page's font size, so the dots grow with the large-window scale. */
    --dot: calc(var(--fs-base) * 1.05);
    flex-direction: row;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
  }

  /* Block, not the chip's inline-flex, or the ellipsis of .ell would not apply. */
  .net {
    display: block;
    flex: 0 0 5.5rem;
    min-width: 0;
  }

  /* Wraps inside the card when the goal is large (up to 12 dots). */
  .inline .dots {
    flex: 0 1 auto;
    flex-wrap: wrap;
    min-width: 0;
  }

  .of {
    margin-left: auto;
    flex: none;
    white-space: nowrap;
    color: var(--text-muted);
  }
</style>
