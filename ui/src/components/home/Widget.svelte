<script lang="ts">
  // The card every widget lives in: title, count and the way to the detail screen.
  import type { Snippet } from "svelte";
  import { t } from "../../lib/i18n";
  import Icon from "../Icon.svelte";

  let {
    id,
    title,
    count = 0,
    hidden = 0,
    grow = true,
    onmore,
    children,
  }: { id: string; title: string; count?: number; hidden?: number; grow?: boolean; onmore: () => void; children: Snippet } = $props();
</script>

<section class="widget w-card" class:grow data-widget={id} aria-labelledby={`w-${id}`}>
  <header>
    <h2 id={`w-${id}`}>{title}</h2>
    {#if count > 0}<span class="n">{count}</span>{/if}
    <button class="ghost more" onclick={onmore}>
      {hidden > 0 ? t("home.see_more_n", { n: hidden }) : t("home.see_more")}<span class="sr-only">: {title}</span>
      <Icon name="arrow-right" size={13} />
    </button>
  </header>
  <div class="body">{@render children()}</div>
</section>

<style>
  .widget {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  /* Spare height is shared between the cards of a column, so none of them holds it all. */
  .grow {
    flex: 1 1 auto;
  }

  header {
    display: flex;
    align-items: baseline;
    gap: var(--sp-2);
    padding-bottom: 5px;
    border-bottom: 1.5px solid var(--head-rule);
  }

  h2 {
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .n {
    font-size: var(--fs-xs);
    font-weight: 600;
    color: var(--text-muted);
  }

  .more {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
    min-height: 24px;
    padding: 0 var(--sp-1);
    font-size: var(--fs-xs);
    white-space: nowrap;
    color: var(--text-muted);
  }

  .more:hover {
    color: var(--text);
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    min-width: 0;
  }
</style>
