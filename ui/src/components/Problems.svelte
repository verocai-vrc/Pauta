<script lang="ts">
  // "Before posting": what is still missing, in plain words.
  import type { Problem } from "../lib/api";
  import { t } from "../lib/i18n";
  import { problemText } from "../lib/readiness";
  import Icon from "./Icon.svelte";

  let { problems }: { problems: Problem[] } = $props();
</script>

{#if problems.length}
  <ul class="problems" aria-label={t("ready.title")}>
    {#each problems as p (p.code)}
      <li><Icon name="triangle-alert" size={14} /> {problemText(p)}</li>
    {/each}
  </ul>
{/if}

<style>
  .problems {
    list-style: none;
    margin: 0;
    padding: var(--sp-2) var(--sp-3);
    background: var(--warning-soft);
    color: var(--warning);
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
