<script lang="ts">
  import { api, type Rhythm } from "../lib/api";
  import { shortDate, WEEKDAYS_SHORT } from "../lib/dates";
  import { t, tn } from "../lib/i18n";
  import { app, fail, networkLabel, openItem } from "../lib/store.svelte";
  import CadenceMeter from "./CadenceMeter.svelte";
  import Icon from "./Icon.svelte";
  import MetricsImport from "./MetricsImport.svelte";
  import StackedBars from "./StackedBars.svelte";

  let data = $state<Rhythm | null>(null);
  let importing = $state(false);

  $effect(() => {
    void app.version;
    api.rhythm().then((r) => (data = r)).catch(fail);
  });

  const networks = $derived((app.index?.config.networks ?? []).map((n) => n.id));
  const MONTHS = ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"];
</script>

<div class="rhythm">
  <header>
    <h1 class="display">{t("rhythm.title")}</h1>
    <p class="muted">{t("rhythm.intro")}</p>
  </header>

  {#if data}
    <section aria-labelledby="goals">
      <h2 id="goals" class="rule-heading">{t("rhythm.goals")}</h2>
      {#if data.cadence.length === 0}
        <p class="muted">{t("rhythm.no_goals")}</p>
      {:else}
        <div class="meters">
          {#each data.cadence as c (c.network)}<CadenceMeter progress={c} />{/each}
        </div>
      {/if}
    </section>

    <section aria-labelledby="weekly">
      <h2 id="weekly" class="rule-heading">{t("rhythm.weekly")}</h2>
      <StackedBars
        buckets={data.history.weeks}
        {networks}
        caption={t("rhythm.weekly_caption", { n: data.history.weeks.length })}
        label={(b) => shortDate(b.start)}
      />
    </section>

    <section aria-labelledby="monthly">
      <h2 id="monthly" class="rule-heading">{t("rhythm.monthly")}</h2>
      <StackedBars
        buckets={data.history.months}
        {networks}
        caption={t("rhythm.monthly_caption", { n: data.history.months.length })}
        label={(b) => MONTHS[Number(b.start.slice(5, 7)) - 1]}
      />
    </section>

    <section aria-labelledby="results">
      <div class="head rule-heading">
        <h2 id="results">{t("insights.title")}</h2>
        <button onclick={() => (importing = true)}>{t("import.open")}</button>
      </div>
      {#if data.insights.posts_with_metrics === 0}
        <p class="muted">{t("insights.none")}</p>
      {:else}
        <p class="small muted">{tn("insights.intro", data.insights.posts_with_metrics)}</p>
        {#each data.insights.networks as n (n.network)}
          {@const max = Math.max(1, ...n.by_weekday.map((d) => d.avg_engagement))}
          <div class="insight">
            <div class="row">
              <strong class="chip {n.network}">{n.label}</strong>
              <span>{tn("insights.summary", n.posts, { avg: n.avg_engagement.toFixed(1) })}</span>
              {#if n.engagement_rate != null}<span class="muted">· {t("insights.rate", { pct: (n.engagement_rate * 100).toFixed(1) })}</span>{/if}
            </div>
            <table>
              <caption class="sr-only">{t("insights.by_weekday", { net: n.label })}</caption>
              <thead>
                <tr><th scope="col">{t("insights.weekday")}</th><th scope="col">{t("insights.posts")}</th><th scope="col">{t("insights.avg")}</th><th scope="col"><span class="sr-only">{t("insights.bar")}</span></th></tr>
              </thead>
              <tbody>
                {#each n.by_weekday as d (d.weekday)}
                  <tr>
                    <th scope="row">{WEEKDAYS_SHORT[d.weekday]}</th>
                    <td>{d.posts}</td>
                    <td>{d.avg_engagement.toFixed(1)}</td>
                    <td class="barcell">
                      <span class="bar" aria-hidden="true" style:width={`${(d.avg_engagement / max) * 100}%`}></span>
                      {#if d.posts < data.insights.min_samples}<span class="small muted">{t("insights.few")}</span>{/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/each}
        {#if data.insights.top.length}
          <h3>{t("insights.top")}</h3>
          <ol>
            {#each data.insights.top as p (p.id)}
              <li>
                <button class="ghost" onclick={() => openItem("card", p.id)}>{p.title}</button>
                <span class="small muted">{networkLabel(p.network)} · {t("insights.engagement", { n: p.engagement })}</span>
              </li>
            {/each}
          </ol>
        {/if}
      {/if}
    </section>

    <section aria-labelledby="recent">
      <h2 id="recent" class="rule-heading">{t("rhythm.recent")} <span class="muted small">· {t("rhythm.total_all", { n: data.history.total_posts })}</span></h2>
      {#if data.recent.length === 0}
        <p class="muted">{t("rhythm.none_yet")}</p>
      {:else}
        <ul>
          {#each data.recent as p (p.id)}
            <li class="entry">
              <span class="small muted">{shortDate(p.date)}</span>
              <span class="chip {p.network}">{networkLabel(p.network)}</span>
              <span class="what">
                {#if p.archived}
                  <span>{p.title}</span><span class="chip">{t("rhythm.archived")}</span>
                {:else}
                  <button class="ghost" onclick={() => openItem("card", p.id)}>{p.title}</button>
                {/if}
              </span>
              {#if p.url}<button class="ghost small with-icon" onclick={() => api.openUrl(p.url!).catch(fail)}><Icon name="external-link" size={14} />{t("rhythm.open_post")}</button>{/if}
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</div>

{#if importing}<MetricsImport onclose={() => (importing = false)} />{/if}

<style>
  /* The text starts where Today's does, so the reading screens share one left edge. */
  .rhythm {
    --indent: 148px;
    max-width: calc(var(--indent) + 844px);
    margin: 0;
    padding: var(--sp-5) var(--sp-4) var(--sp-5) var(--indent);
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  header p {
    margin: var(--sp-2) 0 0;
  }

  /* Sections on the ruled paper: a heading over a hairline, no boxes. */
  section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    margin-top: var(--sp-3);
  }

  .head button {
    font-size: var(--fs-md);
    min-height: 0;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    flex-wrap: wrap;
  }

  .meters {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: var(--sp-4);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  /* The lists are tabular: the date, the network, the title and the link have a column each. They
     have plain paper, so the ruling does not run through the rows. */
  ul,
  ol,
  .insight table {
    background: var(--bg);
  }

  .entry {
    display: grid;
    grid-template-columns: 4rem 5rem minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--sp-2);
  }

  .what {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .entry .ghost {
    margin-inline: calc(-1 * (var(--sp-3) + 1px));
  }

  .entry .what .ghost {
    text-align: left;
  }

  .entry > .ghost {
    margin-left: 0;
  }

  .insight {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  table {
    border-collapse: collapse;
    width: 100%;
    max-width: 560px;
  }

  th,
  td {
    text-align: left;
    padding: 3px var(--sp-2) 3px 0;
    font-size: var(--fs-sm);
    font-weight: 400;
  }

  thead th {
    color: var(--text-muted);
    font-size: var(--fs-xs);
  }

  .barcell {
    width: 50%;
  }

  .bar {
    display: inline-block;
    height: 10px;
    border-radius: 3px;
    background: var(--accent);
    vertical-align: middle;
    margin-right: var(--sp-2);
  }

  ol {
    margin: 0;
    padding-left: var(--sp-5);
  }

  @media (max-width: 860px) {
    .rhythm {
      --indent: 108px;
    }
  }
</style>
