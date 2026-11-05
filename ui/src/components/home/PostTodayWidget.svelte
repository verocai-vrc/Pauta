<script lang="ts">
  import type { AgendaItem } from "../../lib/api";
  import { hm, whenLabel } from "../../lib/dates";
  import { isLate, nextScheduled, splitPosts } from "../../lib/home/brief";
  import { t } from "../../lib/i18n";
  import { app, networkLabel, openItem, openToday } from "../../lib/store.svelte";
  import Icon from "../Icon.svelte";
  import PostActions from "../PostActions.svelte";
  import Problems from "../Problems.svelte";
  import Widget from "./Widget.svelte";

  let { agenda, limit }: { agenda: AgendaItem[]; limit: number } = $props();

  const today = $derived(app.today!);
  const posts = $derived(today.posts);
  const split = $derived(splitPosts(posts));
  const lead = $derived(split.lead);
  const shown = $derived(split.rest.slice(0, limit));
  const hidden = $derived(Math.max(0, split.rest.length - limit));
  const pending = $derived(posts.filter((p) => !p.done).length);
  const late = $derived(lead ? isLate(lead, today.date, today.time) : false);
  const upNext = $derived(posts.length === 0 ? nextScheduled(agenda, today.date) : null);
</script>

<Widget id="post" title={t("today.post_today")} count={pending} {hidden} onmore={openToday}>
  {#if posts.length === 0}
    <p class="muted small">{t("today.no_posts")}</p>
    {#if upNext}
      <ul class="w-list">
        <li>
          <button class="ghost w-row" title={upNext.title} onclick={() => openItem("card", upNext.id)}>
            <span class="ell">{t("home.next_scheduled", { when: whenLabel(upNext.when, today.date) })}</span>
          </button>
        </li>
      </ul>
    {/if}
  {:else}
    {#if lead}
      <div class="lead">
        <div class="top">
          {#if hm(lead.when)}<span class="time" class:late>{hm(lead.when)}</span>{/if}
          <span class="chip {lead.network ?? ''}"><span class="dot" aria-hidden="true"></span>{networkLabel(lead.network)}</span>
          {#if late}<span class="chip danger">{t("home.late")}</span>{/if}
        </div>
        <button class="ghost title" title={lead.title} onclick={() => openItem("card", lead.id)}>{lead.title}</button>
        <Problems problems={app.index!.readiness[lead.id] ?? []} />
        <PostActions id={lead.id} compact />
      </div>
    {:else}
      <p class="muted small">{t("home.all_posted")}</p>
    {/if}
    {#if shown.length}
      <ul class="w-list">
        {#each shown as p (p.id)}
          <li>
            <button class="ghost w-row" class:done={p.done} title={p.title} onclick={() => openItem("card", p.id)}>
              <span class="when">{hm(p.when)}</span>
              <span class="chip {p.network ?? ''}">
                {#if p.done}<Icon name="check" size={13} />{:else}<span class="dot" aria-hidden="true"></span>{/if}
                {networkLabel(p.network)}
              </span>
              <span class="ell">{p.title}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</Widget>

<style>
  .lead {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    min-width: 0;
    padding-bottom: var(--sp-2);
  }

  .top {
    display: flex;
    align-items: baseline;
    gap: var(--sp-2);
  }

  .time {
    font-family: var(--font-serif);
    font-size: var(--fs-lg);
    font-variant-numeric: tabular-nums lining-nums;
  }

  .time.late {
    color: var(--danger);
  }

  .title {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    overflow: hidden;
    overflow-wrap: anywhere;
    min-width: 0;
    padding: 0;
    text-align: left;
    font-family: var(--font-serif);
    font-size: var(--fs-lg);
    font-weight: 500;
    line-height: 1.25;
  }

  .when {
    flex: none;
    min-width: 2.9rem;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .done,
  .done .when {
    color: var(--text-muted);
  }
</style>
