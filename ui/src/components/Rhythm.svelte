<script lang="ts">
  import { api, type Rhythm } from "../lib/api";
  import { shortDate, WEEKDAYS_SHORT } from "../lib/dates";
  import { t } from "../lib/i18n";
  import { app, fail, networkLabel, openItem } from "../lib/store.svelte";
  import CadenceMeter from "./CadenceMeter.svelte";
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
    <h1>{t("rhythm.title")}</h1>
    <p class="muted">{t("rhythm.intro")}</p>
  </header>

  {#if data}
    <section class="panel" aria-labelledby="goals">
      <h2 id="goals">{t("rhythm.goals")}</h2>
      {#if data.cadence.length === 0}
        <p class="muted">{t("rhythm.no_goals")}</p>
      {:else}
        <div class="meters">
          {#each data.cadence as c (c.network)}<CadenceMeter progress={c} />{/each}
        </div>
      {/if}
    </section>

    <section class="panel" aria-labelledby="weekly">
      <h2 id="weekly">{t("rhythm.weekly")}</h2>
      <StackedBars
        buckets={data.history.weeks}
        {networks}
        caption={t("rhythm.weekly_caption", { n: data.history.weeks.length })}
        label={(b) => shortDate(b.start)}
      />
    </section>

    <section class="panel" aria-labelledby="monthly">
      <h2 id="monthly">{t("rhythm.monthly")}</h2>
      <StackedBars
        buckets={data.history.months}
        {networks}
        caption={t("rhythm.monthly_caption", { n: data.history.months.length })}
        label={(b) => MONTHS[Number(b.start.slice(5, 7)) - 1]}
      />
    </section>

    <section class="panel" aria-labelledby="results">
      <div class="head">
        <h2 id="results">{t("insights.title")}</h2>
        <button onclick={() => (importing = true)}>{t("import.open")}</button>
      </div>
      {#if data.insights.posts_with_metrics === 0}
        <p class="muted">{t("insights.none")}</p>
      {:else}
        <p class="small muted">{t("insights.intro", { n: data.insights.posts_with_metrics })}</p>
        {#each data.insights.networks as n (n.network)}
          {@const max = Math.max(1, ...n.by_weekday.map((d) => d.avg_engagement))}
          <div class="insight">
            <div class="row">
              <strong class="chip {n.network}">{n.label}</strong>
              <span>{t("insights.summary", { posts: n.posts, avg: n.avg_engagement.toFixed(1) })}</span>
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

    <section class="panel" aria-labelledby="recent">
      <h2 id="recent">{t("rhythm.recent")} <span class="muted small">· {t("rhythm.total_all", { n: data.history.total_posts })}</span></h2>
      {#if data.recent.length === 0}
        <p class="muted">{t("rhythm.none_yet")}</p>
      {:else}
        <ul>
          {#each data.recent as p (p.id)}
            <li class="row">
              <span class="small muted date">{shortDate(p.date)}</span>
              <span class="chip {p.network}">{networkLabel(p.network)}</span>
              {#if p.archived}
                <span>{p.title}</span><span class="chip">{t("rhythm.archived")}</span>
              {:else}
                <button class="ghost" onclick={() => openItem("card", p.id)}>{p.title}</button>
              {/if}
              {#if p.url}<button class="ghost small" onclick={() => api.openUrl(p.url!).catch(fail)}>↗ {t("rhythm.open_post")}</button>{/if}
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</div>

{#if importing}<MetricsImport onclose={() => (importing = false)} />{/if}

<style>
  .rhythm {
    max-width: 860px;
    margin: 0 auto;
    padding: var(--sp-5) var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  header p {
    margin: var(--sp-1) 0 0;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
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

  .date {
    width: 56px;
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
</style>
