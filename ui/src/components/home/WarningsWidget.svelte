<script lang="ts">
  import type { AgendaItem } from "../../lib/api";
  import { buildWarnings } from "../../lib/home/warnings";
  import { t } from "../../lib/i18n";
  import { app, openItem, openToday } from "../../lib/store.svelte";
  import Icon from "../Icon.svelte";
  import Widget from "./Widget.svelte";

  let { agenda, limit }: { agenda: AgendaItem[]; limit: number } = $props();

  const today = $derived(app.today!);
  // The same post Postar hoje shows first, which already carries its own problems.
  const leadId = $derived(today.posts.find((p) => !p.done)?.id ?? null);
  const warnings = $derived(buildWarnings(today, app.index!.readiness, agenda, leadId));
</script>

<Widget id="warnings" title={t("home.warnings")} count={warnings.length} hidden={Math.max(0, warnings.length - limit)} onmore={openToday}>
  {#if warnings.length === 0}
    <p class="clear"><Icon name="check" size={16} /> {t("home.all_clear")}</p>
  {:else}
    <ul class="w-list">
      {#each warnings.slice(0, limit) as w (w.key)}
        <li>
          <button class="w-row" onclick={() => (w.card ? openItem("card", w.card) : openToday())}>
            <span class="ic {w.tone}"><Icon name={w.tone === "quiet" ? "clock" : "triangle-alert"} size={14} /></span>
            <span class="ell grow-text" title={w.text}>{w.text}</span>
            {#if w.aux}<span class="aux">{w.aux}</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</Widget>

<style>
  .ic {
    display: inline-flex;
    flex: none;
  }

  .danger {
    color: var(--danger);
  }

  .warn {
    color: var(--warning);
  }

  .quiet {
    color: var(--text-muted);
  }

  .grow-text {
    flex: 1 1 auto;
  }

  .aux {
    flex: none;
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .clear {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin: 0;
    color: var(--ok);
  }
</style>
