<script lang="ts">
  // Stacked bars of posts per period and network. Colors come from the network
  // tokens, and an equivalent table is always present for screen readers.
  import type { Bucket } from "../lib/api";
  import { t } from "../lib/i18n";
  import { networkLabel } from "../lib/store.svelte";

  let {
    buckets,
    networks,
    label,
    caption,
  }: { buckets: Bucket[]; networks: string[]; label: (b: Bucket) => string; caption: string } = $props();

  const max = $derived(Math.max(1, ...buckets.map((b) => b.total)));
  const color = (n: string) => (n === "linkedin" ? "var(--c-linkedin)" : n === "instagram" ? "var(--c-instagram)" : "var(--c-event)");
</script>

<figure>
  <figcaption class="small muted">{caption}</figcaption>
  <div class="chart" aria-hidden="true">
    {#each buckets as b (b.start)}
      <div class="col" title={`${label(b)}: ${b.total}`}>
        <div class="plot">
          <span class="n small">{b.total || ""}</span>
          <div class="stack" style:height={`calc((100% - var(--n-h)) * ${b.total / max})`}>
            {#each networks as n (n)}
              {#if b.counts[n]}
                <div class="seg" style:flex={b.counts[n]} style:background={color(n)}></div>
              {/if}
            {/each}
          </div>
        </div>
        <span class="lbl small muted">{label(b)}</span>
      </div>
    {/each}
  </div>
  <div class="legend row small" aria-hidden="true">
    {#each networks as n (n)}
      <span class="row"><span class="swatch" style:background={color(n)}></span>{networkLabel(n)}</span>
    {/each}
  </div>
  <table class="sr-only">
    <caption>{caption}</caption>
    <thead>
      <tr><th>{t("rhythm.period")}</th>{#each networks as n (n)}<th>{networkLabel(n)}</th>{/each}<th>{t("rhythm.total")}</th></tr>
    </thead>
    <tbody>
      {#each buckets as b (b.start)}
        <tr><th>{label(b)}</th>{#each networks as n (n)}<td>{b.counts[n] ?? 0}</td>{/each}<td>{b.total}</td></tr>
      {/each}
    </tbody>
  </table>
</figure>

<style>
  figure {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .chart {
    display: flex;
    align-items: stretch;
    gap: var(--sp-1);
    height: 150px;
  }

  .col {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  /* The bars get the height left after the label, so a tall bar can no longer squeeze the label. */
  .plot {
    --n-h: 1.4em;
    flex: 1;
    min-height: 0;
    width: 100%;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: center;
  }

  .stack {
    width: 100%;
    max-width: 34px;
    min-height: 2px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column-reverse;
    border-radius: 3px 3px 0 0;
    overflow: hidden;
    background: var(--surface-2);
  }

  .seg {
    min-height: 3px;
  }

  .n {
    font-variant-numeric: tabular-nums;
    height: var(--n-h);
    line-height: var(--n-h);
    flex-shrink: 0;
  }

  .lbl {
    font-size: var(--fs-xs);
    line-height: 1.5;
    flex-shrink: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 2px;
    display: inline-block;
  }
</style>
