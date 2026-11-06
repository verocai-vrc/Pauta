<script lang="ts">
  import { t } from "../../lib/i18n";
  import { projectSummaries, updatedLabel, type ProjectLine } from "../../lib/home/projects";
  import { app, openItem } from "../../lib/store.svelte";
  import Widget from "./Widget.svelte";

  let { limit }: { limit: number } = $props();

  const date = $derived(app.today!.date);
  const all = $derived(projectSummaries(app.index!, date));
  const shown = $derived(all.slice(0, limit));

  function openBoard(id: string) {
    app.boardId = id;
    app.view = "boards";
  }
</script>

{#snippet line(label: string, entry: ProjectLine | null)}
  <dt>{label}</dt>
  <dd>
    {#if entry}
      <button class="ghost ell" title={entry.text} onclick={() => openItem(entry.kind, entry.id)}>{entry.text}</button>
    {:else}
      <span class="muted">{t("home.proj_nothing")}</span>
    {/if}
  </dd>
{/snippet}

<Widget
  id="projects"
  title={t("home.projects")}
  count={all.length}
  hidden={Math.max(0, all.length - limit)}
  onmore={() => {
    // The boards screen opens on the last board used; the most recent project is what the card led with.
    if (all[0]) app.boardId = all[0].id;
    app.view = "boards";
  }}
>
  {#if all.length === 0}
    <p class="muted small">{t("home.no_projects")}</p>
  {:else}
    <ul class="w-list">
      {#each shown as p (p.id)}
        <li class="proj">
          <div class="head">
            <button class="ghost name ell" title={p.name} onclick={() => openBoard(p.id)}>{p.name}</button>
            {#if p.updated}<span class="upd">{updatedLabel(p.updated, date)}</span>{/if}
          </div>
          {#if p.empty}
            <p class="muted small">{t("home.proj_empty")}</p>
          {:else}
            <dl>
              {#if p.last}{@render line(t("home.proj_last"), p.last)}{/if}
              {@render line(t("home.proj_next"), p.next)}
            </dl>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</Widget>

<style>
  .proj {
    padding: var(--sp-2) 0;
  }

  .proj:first-child {
    padding-top: 0;
  }

  .head {
    display: flex;
    align-items: baseline;
    gap: var(--sp-2);
  }

  /* The name gives way before "atualizado…" does, so a name with no spaces stays inside the card. */
  button.name {
    display: block;
    flex: 0 1 auto;
    min-width: 0;
    min-height: 24px;
    padding: 0 var(--sp-1);
    margin-left: calc(var(--sp-1) * -1);
    border: 0;
    font-family: var(--font-serif);
    font-weight: 500;
    font-size: var(--fs-base);
  }

  .upd {
    flex: none;
    margin-left: auto;
    font-size: var(--fs-xs);
    color: var(--text-muted);
    white-space: nowrap;
  }

  dl {
    margin: 0;
    display: grid;
    grid-template-columns: 4.4em minmax(0, 1fr);
    column-gap: var(--sp-2);
    align-items: center;
    font-size: var(--fs-sm);
  }

  dt {
    color: var(--text-muted);
  }

  dd {
    margin: 0;
    min-width: 0;
  }

  dd button {
    display: block;
    min-width: 0;
    min-height: 24px;
    padding: 0 var(--sp-1);
    margin-left: calc(var(--sp-1) * -1);
    width: calc(100% + var(--sp-1));
    font-size: inherit;
    text-align: left;
  }

  dd > .muted {
    display: block;
    min-height: 24px;
    line-height: 24px;
  }

  .proj p {
    margin: 0;
  }
</style>
