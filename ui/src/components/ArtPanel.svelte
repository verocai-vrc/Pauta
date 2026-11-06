<script lang="ts">
  // Ordering artwork from Claude Design (ADR 0007). There is no API behind this panel:
  // it keeps the hand-off visible. Copy the brief, paste it into the tool, then bring the
  // exported files back and mark the art as ready.
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, type Card } from "../lib/api";
  import { t, tn } from "../lib/i18n";
  import { app, fail, toast } from "../lib/store.svelte";
  import Icon from "./Icon.svelte";

  let { card, flush, onchange }: { card: Card; flush: () => Promise<void>; onchange: (c: Card) => void } = $props();

  const net = $derived(app.index?.config.networks.find((n) => n.id === card.network));
  const formats = $derived(net?.media.formats ?? []);
  const status = $derived(card.art?.status ?? null);
  const current = $derived(card.art?.format ?? formats[0]?.id ?? "");
  // Follows the card's format, and can be changed in the select before (re)requesting.
  let chosen = $derived(current);
  let busy = $state(false);
  const spec = $derived(formats.find((f) => f.id === chosen));

  async function run(fn: () => Promise<Card | void>, ok?: string) {
    busy = true;
    try {
      await flush();
      const c = await fn();
      if (c) onchange(c);
      if (ok) toast(ok);
    } catch (e) {
      fail(e);
    } finally {
      busy = false;
    }
  }

  const request = () => run(() => api.requestArt(card.id, chosen), t("art.requested_toast"));
  const copy = () => run(async () => void (await api.copyArtRequest(card.id)), t("art.copied"));
  const ready = () => run(() => api.setArtStatus(card.id, "ready"), t("art.ready_toast"));
  const reopen = () => run(() => api.setArtStatus(card.id, "requested"));
  const clear = () => run(() => api.setArtStatus(card.id, "none"));

  async function attach() {
    const picked = await open({
      multiple: true,
      filters: [{ name: t("art.files"), extensions: ["png", "jpg", "jpeg", "webp", "pdf", "zip", "svg", "mp4", "mov", "gif"] }],
    });
    const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
    if (paths.length) await run(async () => void (await api.attachAssets(card.id, paths)), tn("art.attached", paths.length));
  }
</script>

<section class="art" aria-labelledby="art-h">
  <div class="row between rule-heading">
    <h3 id="art-h">{t("art.title")}</h3>
    <span class="chip" class:ok={status === "ready"} class:warn={status === "requested"}>
      {status === "ready" ? t("art.status_ready") : status === "requested" ? t("art.status_requested") : t("art.status_none")}
    </span>
  </div>

  <div class="row">
    <label class="field">
      {t("art.format")}
      <select bind:value={chosen} disabled={busy}>
        {#each formats as f (f.id)}<option value={f.id}>{f.label} · {f.size}</option>{/each}
      </select>
    </label>
    {#if spec?.note}<span class="small muted note">{spec.note}</span>{/if}
  </div>

  {#if status === null}
    <p class="small muted">{t("art.intro")}</p>
    <div class="row"><button class="with-icon" disabled={busy} onclick={request}><Icon name="palette" />{t("art.request")}</button></div>
  {:else}
    {#if chosen !== card.art?.format && status === "requested"}
      <p class="small muted">{t("art.format_changed")}</p>
      <div class="row"><button disabled={busy} onclick={request}>{t("art.update_request")}</button></div>
    {/if}
    <p class="small muted">{t("art.brief_hint")}</p>
    <div class="row">
      <button class="with-icon" disabled={busy} onclick={copy}><Icon name="copy" />{t("art.copy_request")}</button>
      <button class="with-icon" disabled={busy} onclick={() => api.openArtTool().catch(fail)}><Icon name="external-link" />{t("art.open_tool")}</button>
      <button class="with-icon" disabled={busy} onclick={attach}><Icon name="plus" />{t("art.attach")}</button>
      <button class="with-icon" disabled={busy} onclick={() => api.openAssets(card.id).catch(fail)}><Icon name="folder" />{t("art.open_folder")}</button>
    </div>

    <div class="files">
      <strong class="small">{t("art.files_title", { n: card.assets.length })}</strong>
      {#if card.assets.length}
        <ul>{#each card.assets as f (f)}<li class="small">{f}</li>{/each}</ul>
      {:else}
        <p class="small muted">{t("art.no_files")}</p>
      {/if}
    </div>

    <div class="row">
      {#if status === "requested"}
        <button class="primary with-icon" disabled={busy || card.assets.length === 0} onclick={ready} title={card.assets.length === 0 ? t("art.ready_needs_files") : ""}><Icon name="check" />{t("art.mark_ready")}</button>
      {:else}
        <button disabled={busy} onclick={reopen}>{t("art.reopen")}</button>
      {/if}
      <button class="ghost" disabled={busy} onclick={clear}>{t("art.clear")}</button>
    </div>
  {/if}
</section>

<style>
  .art {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .between {
    justify-content: space-between;
  }

  .rule-heading h3 {
    font-size: inherit;
  }

  p {
    margin: 0;
  }

  .note {
    align-self: flex-end;
    padding-bottom: var(--sp-2);
  }

  .files ul {
    margin: var(--sp-1) 0 0;
    padding-left: var(--sp-4);
  }
</style>
