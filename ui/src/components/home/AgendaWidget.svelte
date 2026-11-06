<script lang="ts">
  import type { AgendaItem } from "../../lib/api";
  import { hm } from "../../lib/dates";
  import { groupByDay, homeAgenda } from "../../lib/home/agenda";
  import { t } from "../../lib/i18n";
  import { app, networkLabel, openItem } from "../../lib/store.svelte";
  import Icon from "../Icon.svelte";
  import Widget from "./Widget.svelte";

  let { agenda, limit }: { agenda: AgendaItem[] | null; limit: number } = $props();

  const date = $derived(app.today!.date);
  const items = $derived(homeAgenda(agenda ?? [], date));
  const days = $derived(groupByDay(items.slice(0, limit), date));

  function more() {
    app.calendarMode = "agenda";
    app.view = "calendar";
  }

  // What the mark says to the eye, in words (as the calendar does): the icon is hidden from screen readers.
  const kindLabel = (i: AgendaItem) => (i.kind === "publication" ? networkLabel(i.network) : i.kind === "due" ? t("kind.due") : t(`event.${i.status ?? "event"}`));

  const netColor = (n: string | null) => (n === "linkedin" ? "var(--c-linkedin)" : n === "instagram" ? "var(--c-instagram)" : "var(--c-event)");

  // An event that began earlier is listed under today, so its start time would mislead.
  const time = (i: AgendaItem) => (i.when.date < date ? "" : hm(i.when));
</script>

<Widget id="agenda" title={t("home.agenda")} count={items.length} hidden={Math.max(0, items.length - limit)} onmore={more}>
  {#if agenda === null}
    <!-- Not here yet: saying "nothing" now would be a guess. -->
  {:else if items.length === 0}
    <p class="muted small">{t("home.agenda_empty")}</p>
  {:else}
    {#each days as d (d.date)}
      <section class="day">
        <h3 class:now={d.today}>{d.label}</h3>
        <ul class="w-list">
          {#each d.items as i (`${i.kind}:${i.id}`)}
            <li>
              <button class="w-row" onclick={() => openItem(i.kind === "event" ? "event" : "card", i.id)}>
                <span class="t">{time(i)}</span>
                <span class="sr-only">{kindLabel(i)}:</span>
                <span class="mark" aria-hidden="true">
                  {#if i.kind === "due"}
                    <span style:color="var(--c-due)"><Icon name="flag" size={16} /></span>
                  {:else if i.kind === "event"}
                    <span style:color="var(--c-event)"><Icon name="diamond" size={16} /></span>
                  {:else}
                    <span class="dot" style:color={netColor(i.network)}></span>
                  {/if}
                </span>
                <span class="ell title" title={i.title}>{i.title}</span>
              </button>
            </li>
          {/each}
        </ul>
      </section>
    {/each}
  {/if}
</Widget>

<style>
  .day h3 {
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    font-weight: 700;
    color: var(--text);
    padding: 0 0 2px;
  }

  .day .w-row {
    gap: 6px;
  }

  .day h3.now {
    color: var(--accent);
  }

  .day .w-list {
    border-top: 1px solid var(--border);
  }

  .t {
    flex: none;
    width: 2.9rem;
    text-align: right;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .mark {
    flex: none;
    display: inline-flex;
    justify-content: center;
    width: 14px;
  }

  .mark > span {
    display: inline-flex;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: currentColor;
  }

  .title {
    flex: 1 1 auto;
  }
</style>
