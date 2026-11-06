<script lang="ts">
  import type { Snippet } from "svelte";
  import { onMount } from "svelte";
  import { t } from "../lib/i18n";
  import Icon from "./Icon.svelte";

  let {
    title,
    onclose,
    children,
    width = "520px",
  }: { title: string; onclose: () => void; children: Snippet; width?: string } = $props();

  let dialog: HTMLDialogElement;

  onMount(() => {
    dialog.showModal();
    return () => dialog.close();
  });
</script>

<!-- Native <dialog>: focus trap, Esc and inert background come for free. -->
<dialog
  bind:this={dialog}
  style:width
  aria-label={title}
  oncancel={(e) => {
    e.preventDefault();
    onclose();
  }}
  onclick={(e) => {
    if (e.target === dialog) onclose();
  }}
>
  <div class="body">
    <header>
      <h2>{title}</h2>
      <button class="ghost" aria-label={t("common.close")} onclick={onclose}><Icon name="x" /></button>
    </header>
    {@render children()}
  </div>
</dialog>

<style>
  dialog {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    color: var(--text);
    padding: 0;
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 64px);
    box-shadow: var(--shadow-2);
  }

  dialog::backdrop {
    /* ::backdrop only inherits custom properties in recent engines. */
    background: var(--scrim, rgb(0 0 0 / 0.35));
  }

  .body {
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  header button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
</style>
