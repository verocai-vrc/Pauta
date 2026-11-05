<script lang="ts">
  // Manual posting in a few clicks (section 7.3): copy text, open the network,
  // open attachments, then record the link.
  import { api } from "../lib/api";
  import { t } from "../lib/i18n";
  import { app, card, fail, networkLabel, toast } from "../lib/store.svelte";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";

  let { id, compact = false }: { id: string; compact?: boolean } = $props();

  const c = $derived(card(id));
  const net = $derived(app.index?.config.networks.find((n) => n.id === c?.network));
  let publishing = $state(false);
  let url = $state("");

  async function copy() {
    try {
      const n = await api.copyPostText(id);
      toast(t("post.copied", { n }));
    } catch (e) {
      fail(e);
    }
  }

  async function publish(e: SubmitEvent) {
    e.preventDefault();
    try {
      const updated = await api.markPublished(id, url.trim() || undefined);
      publishing = false;
      toast(t("post.marked"));
      if (updated.kind === "article") offerPromo(updated.id, updated.title);
    } catch (err) {
      fail(err);
    }
  }

  // Section 7.4: every published article should get a promo post.
  async function offerPromo(articleId: string, title: string) {
    if (!confirm(t("post.promo_offer"))) return;
    try {
      const promo = await api.createCard(c!.board, {
        title: t("post.promo_title", { title }),
        links_article: articleId,
        network: c?.network ?? undefined,
      });
      app.openCard = promo.id;
    } catch (e) {
      fail(e);
    }
  }
</script>

{#if c?.network}
  <div class="row" class:compact>
    <button class="with-icon" onclick={copy} title={t("post.copy_hint")}><Icon name="copy" />{t("post.copy")}</button>
    {#if net?.url}
      <button class="with-icon" onclick={() => api.openUrl(net.url!).catch(fail)}><Icon name="external-link" />{t("post.open_network", { net: networkLabel(c.network) })}</button>
    {/if}
    <button class="with-icon" onclick={() => api.openAssets(id).catch(fail)}><Icon name="folder" />{t("post.assets")}</button>
    {#if c.publication?.status !== "published"}
      <button class="primary with-icon" onclick={() => (publishing = true)}><Icon name="check" />{t("post.mark_published")}</button>
    {/if}
  </div>
{/if}

{#if publishing}
  <Modal title={t("post.mark_published")} onclose={() => (publishing = false)}>
    <form onsubmit={publish} class="publish">
      <label class="field">
        {t("post.url")}
        <!-- svelte-ignore a11y_autofocus -->
        <input type="url" bind:value={url} placeholder="https://www.linkedin.com/posts/…" autofocus />
      </label>
      <p class="small muted">{t("post.url_hint")}</p>
      <div class="row">
        <button class="primary" type="submit">{t("post.confirm_published")}</button>
        <button type="button" onclick={() => (publishing = false)}>{t("common.cancel")}</button>
      </div>
    </form>
  </Modal>
{/if}

<style>
  .compact button {
    font-size: var(--fs-sm);
    min-height: 26px;
    padding: 0 var(--sp-2);
  }

  /* The one primary action sits at the end of the row, as in the mockup. */
  .compact .primary {
    margin-left: auto;
  }

  .publish {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
</style>
