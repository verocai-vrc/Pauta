<script lang="ts">
  import { api } from "../../lib/api";
  import { t } from "../../lib/i18n";
  import { ideaBoard, latestIdeas } from "../../lib/home/ideas";
  import { app, fail, openItem, toast } from "../../lib/store.svelte";
  import Icon from "../Icon.svelte";
  import Widget from "./Widget.svelte";

  let { limit }: { limit: number } = $props();

  const target = $derived(ideaBoard(app.index!));
  const ideas = $derived(latestIdeas(app.index!));
  const shown = $derived(ideas.slice(0, limit));

  let text = $state("");
  // A second Enter while the first save is on its way must not create the card twice.
  let saving = $state(false);

  async function save(e: SubmitEvent) {
    e.preventDefault();
    const title = text.trim();
    if (!target || !title || saving) return;
    saving = true;
    try {
      await api.createCard(target.id, { title, top: true });
      text = "";
      toast(t("home.idea_saved"));
    } catch (err) {
      fail(err);
    } finally {
      saving = false;
    }
  }

  function more() {
    if (target) app.boardId = target.id;
    app.view = "boards";
  }
</script>

<Widget id="ideas" title={t("home.ideas")} count={ideas.length} hidden={Math.max(0, ideas.length - limit)} onmore={more}>
  {#if target}
    <form onsubmit={save}>
      <input type="text" bind:value={text} placeholder={t("home.idea_placeholder")} aria-label={t("home.idea_placeholder")} />
      <button type="submit" class="add" aria-label={t("home.idea_add")} title={t("home.idea_add")}>
        <Icon name="plus" />
      </button>
    </form>
  {:else}
    <p class="muted small">{t("home.no_content_board")}</p>
  {/if}
  {#if shown.length}
    <ul class="w-list">
      {#each shown as c (c.id)}
        <li>
          <button class="w-row" onclick={() => openItem("card", c.id)}>
            <span class="ell" title={c.title}>{c.title}</span>
          </button>
        </li>
      {/each}
    </ul>
  {:else if target}
    <p class="muted small">{t("home.no_ideas")}</p>
  {/if}
</Widget>

<style>
  form {
    display: flex;
    gap: var(--sp-1);
  }

  /* The large-window scale reaches the field, which would otherwise keep the body's size. */
  input {
    flex: 1;
    min-width: 0;
    font-size: var(--fs-md);
  }

  /* The button stretches to the height of the field, and stays a square next to it. */
  .add {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: none;
    width: 32px;
    min-height: 32px;
    padding: 0;
  }
</style>
