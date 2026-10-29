<script lang="ts">
  // "1 of 2 this week": filled = published, outlined = scheduled, empty = open.
  // Shape and text carry the meaning; color is only a hint. No alarms, no streaks.
  import type { CadenceProgress } from "../lib/api";
  import { shortDate } from "../lib/dates";
  import { t } from "../lib/i18n";

  let { progress, compact = false }: { progress: CadenceProgress; compact?: boolean } = $props();

  const slots = $derived(
    Array.from({ length: Math.min(progress.goal, 12) }, (_, i) =>
      i < progress.done ? "done" : i < progress.done + progress.scheduled ? "scheduled" : "open",
    ),
  );
  const missing = $derived(Math.max(0, progress.goal - progress.done - progress.scheduled));
  const period = $derived(progress.per === "week" ? t("cadence.this_week") : t("cadence.this_month"));
</script>

<div class="meter">
  <div class="row head">
    <strong class="chip {progress.network}">{progress.label}</strong>
    <span>{t("cadence.of", { done: progress.done, goal: progress.goal })} {period}</span>
  </div>
  <div class="dots" role="img" aria-label={t("cadence.aria", { done: progress.done, scheduled: progress.scheduled, goal: progress.goal })}>
    {#each slots as s, i (i)}<span class="dot {s}"></span>{/each}
    {#if progress.goal > 12}<span class="small muted">…</span>{/if}
  </div>
  {#if !compact}
    <p class="small muted">
      {progress.scheduled > 0 ? t("cadence.scheduled", { n: progress.scheduled }) : ""}
      {#if missing > 0}{t("cadence.missing", { n: missing })}{:else}{t("cadence.covered")}{/if}
      · {shortDate(progress.period_start)} – {shortDate(progress.period_end)}
    </p>
  {/if}
</div>

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
    width: 22px;
    height: 22px;
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
</style>
