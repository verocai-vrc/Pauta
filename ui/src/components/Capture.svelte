<script lang="ts">
  // Small always-on-top window for quick capture (section 12.1).
  import { onMount } from "svelte";
  import { api, errorMessage, type Board } from "../lib/api";
  import { t } from "../lib/i18n";

  let boards = $state<Board[]>([]);
  let boardId = $state("");
  let title = $state("");
  let notes = $state("");
  let error = $state("");
  let input: HTMLInputElement;

  onMount(async () => {
    try {
      const index = await api.index();
      boards = index.boards;
      boardId = boards.find((b) => b.kind === "content")?.id ?? boards[0]?.id ?? "";
    } catch (e) {
      error = errorMessage(e);
    }
    input?.focus();
  });

  async function save(e: SubmitEvent) {
    e.preventDefault();
    if (!title.trim()) return;
    try {
      await api.createCard(boardId, {
        title,
        body: notes.trim() ? `## Notas\n\n${notes.trim()}` : undefined,
        top: true,
      });
      title = "";
      notes = "";
      await api.closeCapture();
    } catch (err) {
      error = errorMessage(err);
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && api.closeCapture()} />

<form onsubmit={save}>
  <h1>{t("capture.title")}</h1>
  <input type="text" bind:this={input} bind:value={title} placeholder={t("capture.placeholder")} aria-label={t("capture.placeholder")} required />
  <textarea rows="3" bind:value={notes} placeholder={t("capture.notes")} aria-label={t("capture.notes")}></textarea>
  <div class="row">
    <select bind:value={boardId} aria-label={t("capture.board")}>
      {#each boards as b (b.id)}<option value={b.id}>{b.name}</option>{/each}
    </select>
    <span class="spacer"></span>
    <button type="button" onclick={() => api.closeCapture()}>{t("common.cancel")}</button>
    <button class="primary" type="submit">{t("capture.save")}</button>
  </div>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
</form>

<style>
  form {
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    height: 100%;
  }

  h1 {
    font-size: var(--fs-lg);
  }

  input {
    font-size: var(--fs-lg);
  }

  /* The notes take the room left in the small window, so no empty band stays under the buttons. */
  textarea {
    flex: 1;
    resize: none;
  }

  .spacer {
    flex: 1;
  }

  .error {
    color: var(--danger);
    margin: 0;
  }
</style>
