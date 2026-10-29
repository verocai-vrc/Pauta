<script lang="ts">
  import { api, type AgendaItem } from "../lib/api";
  import { relativeDay, whenLabel, hm } from "../lib/dates";
  import { t } from "../lib/i18n";
  import { app, board, fail, networkLabel, openItem } from "../lib/store.svelte";
  import CadenceMeter from "./CadenceMeter.svelte";
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

<div class="today">
  <header>
    <h1>{t("today.title")}</h1>
    <p class="muted">{date && relativeDay(date, date)} · {date}</p>
  </header>

  {#if today}
    {#if today.invalid_files > 0}
      <section class="notice danger" aria-labelledby="inv">
        <h2 id="inv">{t("today.invalid", { n: today.invalid_files })}</h2>
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
      <h2 id="post-today">{t("today.post_today")}</h2>
      {#if today.posts.length === 0}
        <p class="muted">{t("today.no_posts")}</p>
      {:else}
        <ul class="items">
          {#each today.posts as p (p.id)}
            <li class="item post" class:done={p.done}>
              <div class="line">
                <span class="time">{hm(p.when)}</span>
                <span class="chip {p.network}">{networkLabel(p.network)}</span>
                <button class="ghost title" onclick={() => openItem("card", p.id)}>{p.title}</button>
                {#if p.done}<span class="chip ok">{t(`status.${p.status}`)}</span>{/if}
              </div>
              {#if !p.done}<Problems problems={app.index?.readiness[p.id] ?? []} />{/if}
              {#if !p.done}<PostActions id={p.id} compact />{/if}
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    {#if today.cadence.length}
      <section aria-labelledby="cadence">
        <div class="row between">
          <h2 id="cadence">{t("cadence.title")}</h2>
          <button class="ghost small" onclick={() => (app.view = "rhythm")}>{t("cadence.see_rhythm")} →</button>
        </div>
        <div class="meters">
          {#each today.cadence as c (c.network)}<CadenceMeter progress={c} compact />{/each}
        </div>
      </section>
    {/if}

    {#if today.overdue.length}
      <section aria-labelledby="overdue">
        <h2 id="overdue">{t("today.overdue")}</h2>
        <ul class="items">
          {#each today.overdue as i (i.kind + i.id)}
            <li class="item">
              <div class="line">
                <span class="chip danger">{whenLabel(i.when, date)}</span>
                <span class="chip {i.network ?? ''}">{kindLabel(i)}</span>
                <button class="ghost title" onclick={() => openItem("card", i.id)}>{i.title}</button>
                <span class="small muted">{boardName(i.board)}</span>
              </div>
              {#if i.kind === "publication"}<PostActions id={i.id} compact />{/if}
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    {#each today.empty_queues as q (q.network)}
      <section class="notice warn" aria-label={t("today.empty_queue", { net: q.label, days: q.days })}>
        <h2>{t("today.empty_queue", { net: q.label, days: q.days })}</h2>
        {#if q.candidates.length}
          <p class="small">{t("today.candidates")}</p>
          <div class="row">
            {#each q.candidates as c (c.id)}
              <button onclick={() => openItem("card", c.id)}>{c.title}</button>
            {/each}
          </div>
        {:else}
          <p class="small">{t("today.no_candidates")}</p>
        {/if}
      </section>
    {/each}

    <section aria-labelledby="upcoming">
      <h2 id="upcoming">{t("today.upcoming")}</h2>
      {#if today.upcoming.length === 0}
        <p class="muted">{t("today.no_upcoming")}</p>
      {:else}
        <ul class="items">
          {#each today.upcoming as i (i.kind + i.id)}
            <li class="item">
              <div class="line">
                <span class="chip" class:warn={i.when.date === date}>{whenLabel(i.when, date)}</span>
                <span class="chip">{kindLabel(i)}</span>
                <button class="ghost title" onclick={() => openItem(i.kind === "event" ? "event" : "card", i.id)}>
                  {i.title}
                </button>
                {#if i.board}<span class="small muted">{boardName(i.board)}</span>{/if}
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    {#if today.stale.length}
      <section aria-labelledby="stale">
        <h2 id="stale">{t("today.stale")}</h2>
        <div class="row">
          {#each today.stale as c (c.id)}
            <button class="ghost stale" onclick={() => openItem("card", c.id)}>
              {c.title} <span class="small muted">· {boardName(c.board)}</span>
            </button>
          {/each}
        </div>
      </section>
    {/if}
  {/if}
</div>

<style>
  .today {
    max-width: 860px;
    margin: 0 auto;
    padding: var(--sp-5) var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-5);
  }

  header p {
    margin: var(--sp-1) 0 0;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .items {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .item {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--sp-2) var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .item.done .title {
    color: var(--text-muted);
  }

  .line {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    flex-wrap: wrap;
  }

  .between {
    justify-content: space-between;
  }

  .meters {
    display: flex;
    gap: var(--sp-5);
    flex-wrap: wrap;
  }

  .time {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .title {
    font-weight: 600;
    text-align: left;
    padding: 0 var(--sp-1);
  }

  .notice {
    border-radius: var(--radius-md);
    padding: var(--sp-3);
  }

  .notice.warn {
    background: var(--warning-soft);
    color: var(--warning);
  }

  .notice.danger {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .notice ul {
    margin: 0;
    padding-left: var(--sp-4);
  }

  .link {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    text-decoration: underline;
    padding: 0;
  }

  .stale {
    border: 1px dashed var(--border);
  }
</style>
