<script lang="ts">
  import type { Card } from "../lib/api";
  import { addDays, whenLabel } from "../lib/dates";
  import { t } from "../lib/i18n";
  import { app, networkLabel } from "../lib/store.svelte";
  import Icon from "./Icon.svelte";

  let {
    card,
    onkeydown,
    ondragstart,
    ondragend,
  }: {
    card: Card;
    onkeydown: (e: KeyboardEvent) => void;
    ondragstart: (e: DragEvent) => void;
    ondragend: () => void;
  } = $props();

  const today = $derived(app.today?.date ?? "");
  const dueState = $derived.by(() => {
    if (!card.due || !today) return "";
    if (card.due.date < today) return "danger";
    if (card.due.date <= addDays(today, 3)) return "warn";
    return "";
  });
  const pub = $derived(card.publication);
</script>

<!-- A button-like card: Enter opens, Alt+arrows move (see BoardView). -->
<div
  class="tile"
  role="button"
  tabindex="0"
  data-id={card.id}
  draggable="true"
  aria-roledescription={t("board.card_role")}
  aria-describedby="board-kbd-help"
  {onkeydown}
  {ondragstart}
  {ondragend}
  onclick={() => (app.openCard = card.id)}
>
  <div class="title">{card.title}</div>
  <div class="meta">
    {#if card.network}
      <span class="chip {card.network}">
        {networkLabel(card.network)}{#if pub?.at}&nbsp;· {whenLabel(pub.at, today)}{/if}
      </span>
    {/if}
    {#if pub && pub.status !== "planned"}
      <span class="chip" class:ok={pub.status === "published"}>{t(`status.${pub.status}`)}</span>
    {/if}
    {#if card.due}
      <span class="chip {dueState}" title={t("card.due")}><Icon name="flag" size={12} /> {whenLabel(card.due, today)}</span>
    {/if}
    {#if card.checklist.total}
      <span class="chip" class:ok={card.checklist.done === card.checklist.total} title={t("card.checklist")}>
        <Icon name="square-check" size={12} /> {card.checklist.done}/{card.checklist.total}
      </span>
    {/if}
    {#if card.art}
      <span class="chip" class:ok={card.art.status === "ready"} class:warn={card.art.status === "requested"} title={t("art.title")}>
        <Icon name="palette" size={12} /> {card.art.status === "ready" ? t("art.badge_ready") : t("art.badge_requested")}
      </span>
    {/if}
    {#if app.index?.readiness[card.id]?.length}
      <span class="chip warn" title={t("ready.title")}><Icon name="triangle-alert" size={12} /> {t("ready.badge", { n: app.index.readiness[card.id].length })}</span>
    {/if}
    {#if card.links.siblings.length}
      <span class="chip" title={t("card.siblings")}><Icon name="arrow-left-right" size={12} /> {card.links.siblings.length}</span>
    {/if}
    {#if card.kind === "article"}<span class="chip">{t("kind.article")}</span>{/if}
    {#each card.tags as tag (tag)}
      <span class="chip tag">#{tag}</span>
    {/each}
  </div>
</div>

<style>
  .tile {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--sp-2) var(--sp-3);
    box-shadow: var(--shadow-1);
    cursor: grab;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    text-align: left;
  }

  .tile:hover {
    border-color: var(--text-muted);
  }

  .title {
    font-family: var(--font-serif);
    font-weight: 500;
    line-height: 1.3;
    overflow-wrap: anywhere;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1) var(--sp-2);
  }
</style>
