<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, errorMessage } from "../lib/api";
  import { t } from "../lib/i18n";
  import { app, reload } from "../lib/store.svelte";

  let root = $state(app.status?.suggested ?? "");
  let busy = $state(false);
  let error = $state("");

  async function choose() {
    const picked = await open({ directory: true, defaultPath: root || undefined });
    if (typeof picked === "string") root = picked;
  }

  async function create(e: SubmitEvent) {
    e.preventDefault();
    busy = true;
    error = "";
    try {
      app.status = await api.setup(root);
      await reload();
    } catch (err) {
      error = errorMessage(err);
    } finally {
      busy = false;
    }
  }
</script>

<div class="setup">
  <form class="panel" onsubmit={create}>
    <h1>{t("setup.title")}</h1>
    <p class="muted">{t("setup.intro")}</p>
    <label class="field">
      {t("setup.folder")}
      <div class="row">
        <input type="text" bind:value={root} required style="flex: 1" />
        <button type="button" onclick={choose}>{t("setup.choose")}</button>
      </div>
    </label>
    <p class="small muted">{t("setup.seed")}</p>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <button class="primary" type="submit" disabled={busy || !root}>{t("setup.create")}</button>
  </form>
</div>

<style>
  .setup {
    display: grid;
    place-items: center;
    height: 100%;
    padding: var(--sp-4);
  }

  form {
    max-width: 520px;
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .error {
    color: var(--danger);
  }
</style>
