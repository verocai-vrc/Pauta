<script lang="ts">
  import { app } from "../lib/store.svelte";
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each app.toasts as toast (toast.id)}
    <div class="toast" class:error={toast.kind === "error"}>{toast.text}</div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    bottom: var(--sp-4);
    right: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    z-index: 100;
    max-width: min(420px, calc(100vw - 32px));
  }

  .toast {
    background: var(--text);
    color: var(--bg);
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-2);
  }

  .toast.error {
    background: var(--danger);
    color: #fff;
  }
</style>
