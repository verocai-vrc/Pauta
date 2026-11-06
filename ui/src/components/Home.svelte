<script lang="ts">
  // The first screen: seven fixed widgets in three columns, by theme (act now, time, ongoing work).
  // Composed from what the backend already sends (ADR 0009).
  import { MediaQuery } from "svelte/reactivity";
  import { api, type AgendaItem } from "../lib/api";
  import { addDays } from "../lib/dates";
  import { NORMAL, ROOMY } from "../lib/home/limits";
  import { app, fail } from "../lib/store.svelte";
  import AgendaWidget from "./home/AgendaWidget.svelte";
  import BriefWidget from "./home/BriefWidget.svelte";
  import IdeasWidget from "./home/IdeasWidget.svelte";
  import PostTodayWidget from "./home/PostTodayWidget.svelte";
  import ProjectsWidget from "./home/ProjectsWidget.svelte";
  import RhythmWidget from "./home/RhythmWidget.svelte";
  import WarningsWidget from "./home/WarningsWidget.svelte";

  const tall = new MediaQuery("(min-height: 960px)");
  const limits = $derived(tall.current ? ROOMY : NORMAL);
  const date = $derived(app.today?.date);

  // Null until the first answer, and after a failed fetch: the widgets read it as "no items", but only
  // the Agenda's empty message waits for a real answer, so the screen never claims a week is empty it has not seen.
  let agenda = $state<AgendaItem[] | null>(null);
  let request = 0;

  $effect(() => {
    void app.version;
    if (!date) return;
    const mine = ++request;
    api
      .agenda(date, addDays(date, 7))
      .then((items) => {
        if (mine === request) agenda = items;
      })
      .catch(fail);
  });
</script>

{#if app.today}
  <div class="home">
    <div class="top"><BriefWidget {agenda} /></div>
    <div class="col">
      <PostTodayWidget {agenda} limit={limits.posts} />
      <WarningsWidget {agenda} limit={limits.warnings} />
      <RhythmWidget limit={limits.rhythm} />
    </div>
    <div class="col">
      <AgendaWidget {agenda} limit={limits.agenda} />
    </div>
    <div class="col work">
      <ProjectsWidget limit={limits.projects} />
      <IdeasWidget limit={limits.ideas} />
    </div>
  </div>
{/if}

<style>
  .home {
    --w-row-y: 3px;
    box-sizing: border-box;
    min-height: 100%;
    max-width: 1680px;
    margin: 0 auto;
    padding: var(--sp-3) var(--sp-4) var(--sp-4);
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    grid-template-rows: auto minmax(0, 1fr);
    gap: var(--sp-3);
  }

  .top {
    grid-column: 1 / -1;
    display: grid;
    grid-template-columns: subgrid;
  }

  .col {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    min-width: 0;
    min-height: 0;
  }

  /* A large window: the text grows with the screen, instead of floating in empty cards. */
  @media (min-width: 1600px) and (min-height: 960px) {
    .home {
      --fs-xs: 13px;
      --fs-sm: 15px;
      --fs-md: 16px;
      --fs-base: 17px;
      --fs-lg: 21px;
      --fs-display: 54px;
      --sp-2: 10px;
      --sp-3: 16px;
      --sp-4: 22px;
      --w-row-y: 6px;
    }
  }

  /* No room for three columns: act now and time side by side, ongoing work below, and the page scrolls. */
  @media (max-width: 1099px) {
    .home {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      grid-template-rows: auto;
    }

    .work {
      grid-column: 1 / -1;
      flex-direction: row;
    }

    .work > :global(.widget) {
      flex: 1 1 0;
    }
  }
</style>
