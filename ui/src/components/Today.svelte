<script lang="ts">
  import { api, type AgendaItem, type When } from "../lib/api";
  import { hm, longDate, relativeDay } from "../lib/dates";
  import { t, tn } from "../lib/i18n";
  import { app, board, fail, networkLabel, openItem } from "../lib/store.svelte";
  import { todaySummary } from "../lib/summary";
  import CadenceMeter from "./CadenceMeter.svelte";
  import Icon from "./Icon.svelte";
  import PostActions from "./PostActions.svelte";
  import Problems from "./Problems.svelte";

  const today = $derived(app.today);
  const date = $derived(today?.date ?? "");

  function kindLabel(i: AgendaItem): string {
    if (i.kind === "publication") return networkLabel(i.network);
    if (i.kind === "due") return t("kind.due");
    return t(`event.${i.status ?? "event"}`);
  }

  function boardName(id: string | null) {
    return board(id)?.name ?? id ?? "";
  }
</script>

{#snippet when(w: When, late = false)}
  <div class="gut" class:late class:warn={!late && w.date === date}>
    <span>{relativeDay(w.date, date)}</span>
    {#if hm(w)}<span class="at">{hm(w)}</span>{/if}
  </div>
{/snippet}

{#snippet kind(i: AgendaItem)}
  <span class="chip {i.network ?? ''}">
    {#if i.kind === "due"}<Icon name="flag" size={13} />
    {:else if i.kind === "event"}<Icon name="diamond" size={13} />
    {:else}<span class="dot" aria-hidden="true"></span>{/if}
    {kindLabel(i)}
  </span>
{/snippet}

<div class="today">
  <header class="indent">
    {#if date}<div class="kicker">{longDate(date)}</div>{/if}
    <h2 class="display">{t("today.title")}</h2>
    {#if today}<p class="dek">{todaySummary(today)}</p>{/if}
  </header>

  {#if today}
    {#if today.invalid_files > 0}
      <section class="notice danger indent" aria-labelledby="inv">
        <h3 id="inv">{tn("today.invalid", today.invalid_files)}</h3>
        <ul>
          {#each app.index?.invalid ?? [] as f (f.path)}
            <li>
              <button class="ghost link" onclick={() => api.openPath(f.path).catch(fail)}>{f.path}</button>
              <span class="small">{f.error}</span>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    <section aria-labelledby="post-today">
      <h3 class="section-label indent" id="post-today">
        {t("today.post_today")} <span class="n">{today.posts.length}</span>
      </h3>
      {#if today.posts.length === 0}
        <p class="muted indent">{t("today.no_posts")}</p>
      {:else}
        <ul class="entries">
          {#each today.posts as p (p.id)}
            <li class="entry" class:done={p.done}>
              <div class="gut time">{hm(p.when)}</div>
              <div class="card item post">
                <div class="line">
                  <span class="chip {p.network}"><span class="dot" aria-hidden="true"></span>{networkLabel(p.network)}</span>
                  <button class="ghost title" onclick={() => openItem("card", p.id)}>{p.title}</button>
                  {#if p.done}
                    <span class="chip ok"><Icon name="check" size={13} />{t(`status.${p.status}`)}</span>
                  {/if}
                </div>
                {#if !p.done}<Problems problems={app.index?.readiness[p.id] ?? []} />{/if}
                {#if !p.done}<PostActions id={p.id} compact />{/if}
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    {#if today.cadence.length}
      <section aria-labelledby="cadence">
        <h3 class="section-label indent" id="cadence">{t("cadence.title")}</h3>
        <div class="entry">
          <div class="gut"></div>
          <div class="meters">
            {#each today.cadence as c (c.network)}<CadenceMeter progress={c} compact />{/each}
            <button class="ghost small see" onclick={() => (app.view = "rhythm")}>
              {t("cadence.see_rhythm")}
              <Icon name="arrow-right" size={14} />
            </button>
          </div>
        </div>
      </section>
    {/if}

    {#if today.overdue.length}
      <section aria-labelledby="overdue">
        <h3 class="section-label indent" id="overdue">
          {t("today.overdue")} <span class="n">{today.overdue.length}</span>
        </h3>
        <ul class="entries">
          {#each today.overdue as i (i.kind + i.id)}
            <li class="entry">
              {@render when(i.when, true)}
              <div class="card item">
                <div class="line">
                  {@render kind(i)}
                  <button class="ghost title" onclick={() => openItem("card", i.id)}>{i.title}</button>
                  <span class="small muted board">{boardName(i.board)}</span>
                </div>
                {#if i.kind === "publication"}<PostActions id={i.id} compact />{/if}
              </div>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    {#each today.empty_queues as q (q.network)}
      <section
        class="notice warn indent"
        aria-label={t("today.empty_queue", { net: q.label, days: q.days })}
      >
        <h3><Icon name="triangle-alert" size={15} />{t("today.empty_queue", { net: q.label, days: q.days })}</h3>
        {#if q.candidates.length}
          <p class="small">{t("today.candidates")}</p>
          <div class="row">
            {#each q.candidates as c (c.id)}
              <button class="pill" onclick={() => openItem("card", c.id)}>{c.title}</button>
            {/each}
          </div>
        {:else}
          <p class="small">{t("today.no_candidates")}</p>
        {/if}
      </section>
    {/each}

    <section aria-labelledby="upcoming">
      <h3 class="section-label indent" id="upcoming">
        {t("today.upcoming")} <span class="n">{today.upcoming.length}</span>
      </h3>
      {#if today.upcoming.length === 0}
        <p class="muted indent">{t("today.no_upcoming")}</p>
      {:else}
        <ul class="entries ledger">
          {#each today.upcoming as i (i.kind + i.id)}
            <li class="entry">
              {@render when(i.when)}
              <div class="line">
                {@render kind(i)}
                <button
                  class="ghost title"
                  onclick={() => openItem(i.kind === "event" ? "event" : "card", i.id)}
                >
                  {i.title}
                </button>
                {#if i.board}<span class="small muted board">{boardName(i.board)}</span>{/if}
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    {#if today.stale.length}
      <section aria-labelledby="stale">
        <h3 class="section-label indent" id="stale">{t("today.stale")}</h3>
        <ul class="entries ledger">
          {#each today.stale as c (c.id)}
            <li class="entry">
              <div class="gut"></div>
              <div class="line bare">
                <button class="ghost title" onclick={() => openItem("card", c.id)}>{c.title}</button>
                <span class="small muted board">{boardName(c.board)}</span>
              </div>
            </li>
          {/each}
        </ul>
      </section>
    {/if}
  {/if}
</div>

<style>
  /* Dates and times live left of the notebook margin, everything you read to the right of it. */
  .today {
    --gut: 96px;
    --gap: 28px;
    --pad-l: 24px;
    --pad-r: var(--sp-5);
    position: relative;
    box-sizing: border-box;
    max-width: 860px;
    min-height: 100%;
    margin: 0;
    padding: var(--sp-5) var(--pad-r) var(--sp-6) var(--pad-l);
    display: flex;
    flex-direction: column;
    gap: var(--sp-5);
  }

  /* Inside the scrolling content, so the line moves with the text. It sits in the gap between the
     gutter and the content, where there is no text. */
  .today::before {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: calc(var(--pad-l) + var(--gut) + var(--gap) / 2);
    width: 1.5px;
    background: var(--margin-line);
    pointer-events: none;
  }

  .indent {
    margin-left: calc(var(--gut) + var(--gap));
  }

  header h2 {
    margin: 2px 0;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  h3 {
    margin: 0;
  }

  /* The count is told apart from the label by weight, not by opacity (contrast). */
  .section-label .n {
    margin-left: 2px;
    font-weight: 600;
  }

  .entries {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .entry {
    display: grid;
    grid-template-columns: var(--gut) minmax(0, 1fr);
    column-gap: var(--gap);
    align-items: first baseline;
  }

  /* The gutter and the entry share the baseline of their first line. */
  .gut {
    text-align: right;
    font-size: var(--fs-md);
    line-height: 1.25;
    color: var(--text-muted);
  }

  .gut .at {
    display: block;
    font-variant-numeric: tabular-nums;
  }

  .gut.time {
    font-family: var(--font-serif);
    font-size: var(--fs-lg);
    font-weight: 500;
    font-variant-numeric: tabular-nums lining-nums;
    color: var(--text);
  }

  .gut.late {
    font-weight: 600;
    color: var(--danger);
  }

  .gut.warn {
    font-weight: 600;
    color: var(--warning);
  }

  /* What asks for action is a card. */
  .card {
    min-width: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-1);
    padding: var(--sp-2) var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  /* What is done is only a trace of a card. */
  .done .card {
    background: transparent;
    box-shadow: none;
    border-style: dashed;
  }

  .done .gut.time,
  .done .title {
    color: var(--text-muted);
  }

  /* The kind has a column of its own, so the titles start at the same place. */
  .line {
    min-width: 0;
    display: grid;
    grid-template-columns: 5.5rem minmax(0, 1fr) auto;
    align-items: first baseline;
    gap: var(--sp-1) var(--sp-3);
  }

  .line.bare .title {
    grid-column: 2;
  }

  .title {
    min-width: 0;
    padding: 0 var(--sp-1);
    font-family: var(--font-serif);
    font-size: var(--fs-lg);
    font-weight: 500;
    line-height: 1.25;
    text-align: left;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .board {
    margin-left: auto;
  }

  /* Chips are plain text here, except the ones that say a state. As in the mockup, all are semibold. */
  .chip {
    gap: 5px;
    font-weight: 600;
    /* The text gives the baseline of the chip, so the kind lines up with the title beside it. */
    align-items: baseline;
  }

  /* An icon has its baseline at its bottom edge; this sits it on the middle of the text. */
  .chip :global(.icon) {
    margin-bottom: -2.5px;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: currentColor;
  }

  /* What is only for reading is a ruled line. */
  .ledger {
    gap: 0;
    /* The rows do not follow the 32px ruling, so the ledger has its own plain paper and only
       its own lines show. The margin line is positioned and still paints above it. */
    background: var(--bg);
    margin: 0 calc(-1 * var(--pad-r)) 0 calc(-1 * var(--pad-l));
    padding: 0 var(--pad-r) 0 var(--pad-l);
  }

  .ledger .entry {
    padding: var(--sp-1) 0;
    border-bottom: 1px solid var(--border);
  }

  .ledger .title {
    font-size: var(--fs-base);
  }

  .meters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--sp-3) var(--sp-5);
  }

  .see {
    margin-left: auto;
    margin-right: calc(-1 * var(--sp-3));
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
  }

  .notice {
    padding: var(--sp-3) var(--sp-4);
    border-left: 3px solid currentColor;
    border-radius: 0 var(--radius-md) var(--radius-md) 0;
  }

  .notice h3 {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-family: var(--font-ui);
    font-size: var(--fs-md);
    font-weight: 700;
  }

  .notice p {
    margin: 0;
  }

  .notice.warn {
    background: var(--warning-soft);
    color: var(--warning);
  }

  .notice.warn p {
    color: var(--text-muted);
  }

  .notice.danger {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .notice ul {
    margin: 0;
    padding-left: var(--sp-4);
  }

  .pill {
    border-radius: 999px;
    background: var(--surface);
    color: var(--text);
    text-align: left;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .pill:hover {
    background: var(--surface-2);
  }

  .link {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    text-decoration: underline;
    padding: 0;
  }

  /* Narrow window: the gutter shrinks so the content keeps its room. */
  @media (max-width: 860px) {
    .today {
      --gut: 76px;
      --gap: 20px;
      --pad-l: 12px;
      --pad-r: var(--sp-4);
    }

    .gut.time {
      font-size: var(--fs-md);
    }

    /* No room for a column: the kind, the title and the board wrap as they fit. */
    .line {
      display: flex;
      flex-wrap: wrap;
      gap: var(--sp-1) var(--sp-3);
    }

    .title {
      flex: 1 1 14ch;
    }
  }
</style>
