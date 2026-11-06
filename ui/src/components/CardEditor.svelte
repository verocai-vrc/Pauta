<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { api, isCmdError, type Card, type CardPatch, type PubStatus } from "../lib/api";
  import { localInput } from "../lib/dates";
  import { t } from "../lib/i18n";
  import { charCount, postText, renderMarkdown, toggleTask } from "../lib/markdown";
  import { app, board as boardById, card as cardById, fail, networkLabel, toast } from "../lib/store.svelte";
  import Modal from "./Modal.svelte";
  import ArtPanel from "./ArtPanel.svelte";
  import PostActions from "./PostActions.svelte";
  import Problems from "./Problems.svelte";

  let { id }: { id: string } = $props();

  type Field =
    | "title" | "body" | "tags" | "reminders" | "due" | "network" | "kind" | "column" | "links_board"
    | "pub_at" | "pub_status" | "pub_url" | "metrics";

  const stored = $derived(cardById(id));
  const brd = $derived(boardById(stored?.board));
  const networks = $derived(app.index?.config.networks ?? []);

  let baseHash = $state("");
  let title = $state("");
  let body = $state("");
  let tags = $state("");
  let reminders = $state("");
  let dueDate = $state("");
  let dueTime = $state("");
  let network = $state("");
  let kind = $state("");
  let column = $state("");
  let linksBoard = $state("");
  let pubAt = $state("");
  let pubStatus = $state<PubStatus>("planned");
  let pubUrl = $state("");
  const METRIC_KEYS = ["impressions", "reactions", "comments", "shares", "saves", "clicks"] as const;
  let metrics = $state<Record<(typeof METRIC_KEYS)[number], string>>({
    impressions: "", reactions: "", comments: "", shares: "", saves: "", clicks: "",
  });
  let metricsRecorded = $state<string | null>(null);
  let dirty = new SvelteSet<Field>();
  // Bumped on every edit of a field. A save only clears the fields whose counter is
  // unchanged since it started, so text typed while a save is in flight stays dirty.
  const revision: Partial<Record<Field, number>> = {};
  // Hashes this editor has moved past. The index lags behind a write until the watcher's reload
  // arrives, and that old copy must not be mistaken for an outside change.
  const superseded = new SvelteSet<string>();
  let preview = $state(false);
  let slots = $state<{ local: string; date: string; time: string }[]>([]);
  let assets = $state<{ name: string; path: string }[]>([]);
  let conflict = $state<{ disk: string; hash: string } | null>(null);
  let saving = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let panel: HTMLElement;
  let previewEl = $state<HTMLElement>();

  function advanceTo(hash: string) {
    if (baseHash && baseHash !== hash) superseded.add(baseHash);
    baseHash = hash;
  }

  function load(c: Card) {
    advanceTo(c.hash);
    title = c.title;
    body = c.body.replace(/^\n/, "");
    tags = c.tags.join(", ");
    reminders = c.reminders?.join(", ") ?? "";
    dueDate = c.due?.date ?? "";
    dueTime = c.due?.time?.slice(0, 5) ?? "";
    network = c.network ?? "";
    kind = c.kind;
    column = c.column;
    linksBoard = c.links.board ?? "";
    pubAt = localInput(c.publication?.at);
    pubStatus = c.publication?.status ?? "planned";
    pubUrl = c.publication?.url ?? "";
    for (const k of METRIC_KEYS) metrics[k] = c.metrics?.[k] != null ? String(c.metrics[k]) : "";
    metricsRecorded = c.metrics?.recorded ?? null;
    dirty.clear();
  }

  // Follow outside changes (MCP, editor, other window) while nothing is pending.
  $effect(() => {
    const c = stored;
    if (!c) return;
    if (c.hash === baseHash) superseded.clear();
    else if (!superseded.has(c.hash) && dirty.size === 0 && !saving) load(c);
  });

  $effect(() => {
    void stored?.assets.join("\n");
    api.listAssets(id).then((a) => (assets = a)).catch(() => {});
  });

  onMount(() => {
    if (stored) load(stored);
    panel?.focus();
    api.listAssets(id).then((a) => (assets = a)).catch(() => {});
  });

  onDestroy(() => {
    clearTimeout(timer);
    if (dirty.size) save();
  });

  function touch(f: Field, delay = 700) {
    revision[f] = (revision[f] ?? 0) + 1;
    dirty.add(f);
    clearTimeout(timer);
    timer = setTimeout(save, delay);
  }

  const list = (s: string) =>
    s
      .split(/[,\n]/)
      .map((x) => x.trim().replace(/^#/, ""))
      .filter(Boolean);

  function buildPatch(fields: Field[]): CardPatch {
    const p: CardPatch = {};
    if (fields.includes("title")) p.title = title;
    if (fields.includes("body")) p.body = body;
    if (fields.includes("tags")) p.tags = list(tags);
    if (fields.includes("reminders")) p.reminders = reminders.trim() ? list(reminders) : null;
    if (fields.includes("due")) p.due = dueDate ? (dueTime ? `${dueDate}T${dueTime}` : dueDate) : null;
    if (fields.includes("network")) p.network = network || null;
    if (fields.includes("kind")) p.kind = kind;
    if (fields.includes("column")) p.column = column;
    if (fields.includes("links_board")) p.links_board = linksBoard || null;
    if (fields.includes("pub_at") || fields.includes("pub_status") || fields.includes("pub_url")) {
      p.publication = {};
      if (fields.includes("pub_at")) p.publication.at = pubAt || null;
      if (fields.includes("pub_status")) p.publication.status = pubStatus;
      if (fields.includes("pub_url")) p.publication.url = pubUrl.trim() || null;
    }
    if (fields.includes("metrics")) {
      // Empty boxes are left out; an all-empty form removes the block.
      p.metrics = Object.fromEntries(METRIC_KEYS.map((k) => [k, metrics[k].trim() === "" ? null : Number(metrics[k])]));
    }
    return p;
  }

  async function save(expected = baseHash) {
    clearTimeout(timer);
    if (dirty.size === 0 || saving) return;
    if (dirty.has("title") && !title.trim()) return;
    const fields = [...dirty];
    const sent = Object.fromEntries(fields.map((f) => [f, revision[f] ?? 0])) as Record<Field, number>;
    saving = true;
    let saved = false;
    try {
      const updated = await api.updateCard(id, buildPatch(fields), expected);
      advanceTo(updated.hash);
      for (const f of fields) if ((revision[f] ?? 0) === sent[f]) dirty.delete(f);
      conflict = null;
      saved = true;
    } catch (e) {
      if (isCmdError(e) && e.kind === "conflict") conflict = { disk: e.disk_content ?? "", hash: e.disk_hash ?? "" };
      else fail(e);
    } finally {
      saving = false;
    }
    // Edits made during the save: the timer that fired meanwhile returned early.
    if (saved && dirty.size > 0) await save();
  }

  async function flush() {
    if (dirty.size) await save();
  }

  function close() {
    app.openCard = null;
  }

  async function schedule(at?: string) {
    await flush();
    try {
      const c = await api.schedulePost(id, at);
      load(c);
      toast(t("card.scheduled"));
    } catch (e) {
      fail(e);
    }
  }

  async function suggest() {
    try {
      slots = await api.findSlots(network, 3, id);
      if (slots.length === 0) toast(t("card.no_slots"));
    } catch (e) {
      fail(e);
    }
  }

  async function duplicate(net: string) {
    await flush();
    try {
      const copy = await api.duplicateCard(id, net);
      app.openCard = copy.id;
    } catch (e) {
      fail(e);
    }
  }

  async function archive() {
    await flush();
    await api.archiveCard(id).then(() => (toast(t("card.archived")), close())).catch(fail);
  }

  async function trash() {
    if (!confirm(t("card.trash_confirm"))) return;
    await api.trashCard(id).then(() => (toast(t("card.trashed")), close())).catch(fail);
  }

  function resolveKeepDisk() {
    dirty.clear();
    conflict = null;
    if (stored) load(stored);
  }

  function resolveKeepMine() {
    const hash = conflict?.hash;
    conflict = null;
    save(hash);
  }

  const netCfg = $derived(networks.find((n) => n.id === network));
  const count = $derived(charCount(postText(body, [netCfg?.label ?? "", network].filter(Boolean))));
  const otherNetworks = $derived(networks.filter((n) => n.id !== network));
  const siblings = $derived((stored?.links.siblings ?? []).map((s) => cardById(s)).filter(Boolean) as Card[]);
  const article = $derived(cardById(stored?.links.article));

  // Checkboxes in the preview toggle the matching `- [ ]` in the text.
  $effect(() => {
    if (!preview || !previewEl) return;
    void body;
    tick().then(() => {
      previewEl?.querySelectorAll<HTMLInputElement>('input[type="checkbox"]').forEach((box, i) => {
        box.disabled = false;
        box.onchange = () => {
          body = toggleTask(body, i);
          touch("body", 0);
        };
      });
    });
  });
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !conflict && !document.querySelector("dialog[open]") && close()} />

<div class="backdrop" role="presentation" onclick={close}></div>
<aside class="editor" aria-label={t("card.editor")} tabindex="-1" bind:this={panel}>
  {#if !stored}
    <p class="muted">{t("card.missing")}</p>
    <button onclick={close}>{t("common.close")}</button>
  {:else}
    <header>
      <input
        class="title"
        type="text"
        bind:value={title}
        oninput={() => touch("title")}
        aria-label={t("card.title")}
      />
      <span class="small muted status" aria-live="polite">
        {saving ? t("card.saving") : dirty.size ? t("card.pending") : t("card.saved")}
      </span>
      <button class="ghost" aria-label={t("common.close")} onclick={close}>✕</button>
    </header>

    <div class="grid">
      <label class="field">
        {t("card.column")}
        <select bind:value={column} onchange={() => touch("column", 0)}>
          {#each brd?.columns ?? [] as c (c.id)}<option value={c.id}>{c.name}</option>{/each}
        </select>
      </label>
      <label class="field">
        {t("card.kind")}
        <select bind:value={kind} onchange={() => touch("kind", 0)}>
          {#each ["post", "article", "task", "note"] as k (k)}<option value={k}>{t(`kind.${k}`)}</option>{/each}
        </select>
      </label>
      <label class="field">
        {t("card.network")}
        <select bind:value={network} onchange={() => touch("network", 0)}>
          <option value="">{t("card.no_network")}</option>
          {#each networks as n (n.id)}<option value={n.id}>{n.label}</option>{/each}
        </select>
      </label>
      <label class="field wide">
        {t("card.due")}
        <div class="row nowrap">
          <input type="date" bind:value={dueDate} onchange={() => touch("due", 0)} />
          <input type="time" bind:value={dueTime} onchange={() => touch("due", 0)} aria-label={t("card.due_time")} />
        </div>
      </label>
      <label class="field">
        {t("card.tags")}
        <input type="text" bind:value={tags} oninput={() => touch("tags")} placeholder="rust, divulgacao" />
      </label>
      <label class="field">
        {t("card.reminders")}
        <input
          type="text"
          bind:value={reminders}
          oninput={() => touch("reminders", 1200)}
          placeholder={(network ? app.index?.config.reminders.post_default : app.index?.config.reminders.deadline_default)?.join(", ")}
        />
      </label>
      <label class="field">
        {t("card.related_board")}
        <select bind:value={linksBoard} onchange={() => touch("links_board", 0)}>
          <option value="">—</option>
          {#each app.index?.boards ?? [] as b (b.id)}<option value={b.id}>{b.name}</option>{/each}
        </select>
      </label>
    </div>

    {#if network}
      <section class="publication" aria-label={t("card.publication")}>
        <h3>{t("card.publication_on", { net: networkLabel(network) })}</h3>
        <div class="row">
          <label class="field">
            {t("card.pub_at")}
            <input type="datetime-local" bind:value={pubAt} onchange={() => touch("pub_at", 0)} />
          </label>
          <label class="field">
            {t("card.pub_status")}
            <select bind:value={pubStatus} onchange={() => touch("pub_status", 0)}>
              {#each ["planned", "scheduled", "published", "skipped"] as s (s)}<option value={s}>{t(`status.${s}`)}</option>{/each}
            </select>
          </label>
          <label class="field grow">
            {t("card.pub_url")}
            <input type="url" bind:value={pubUrl} oninput={() => touch("pub_url")} placeholder="https://…" />
          </label>
        </div>
        <div class="row">
          <button onclick={suggest}>⌚ {t("card.suggest_slot")}</button>
          {#each slots as s (s.local)}
            <button class="chip-btn" onclick={() => schedule(s.local)}>{s.date} {s.time}</button>
          {/each}
          {#if pubAt && pubStatus !== "published"}
            <button class="primary" onclick={() => schedule(pubAt)}>{t("card.schedule")}</button>
          {/if}
        </div>
        <Problems problems={app.index?.readiness[id] ?? []} />
        <PostActions {id} />
      </section>

      <ArtPanel
        card={stored}
        {flush}
        onchange={(c) => {
          load(c);
          api.listAssets(id).then((a) => (assets = a)).catch(() => {});
        }}
      />
    {/if}

    {#if stored.board && brd?.kind === "content"}
      <div class="row small">
        {#each otherNetworks as n (n.id)}
          {#if !siblings.some((s) => s.network === n.id)}
            <button class="ghost" onclick={() => duplicate(n.id)}>⇄ {t("card.duplicate_for", { net: n.label })}</button>
          {/if}
        {/each}
        {#each siblings as s (s.id)}
          <button class="ghost" onclick={() => (app.openCard = s.id)}>
            ⇄ {t("card.sibling", { net: networkLabel(s.network) })}
          </button>
        {/each}
        {#if article}
          <button class="ghost" onclick={() => (app.openCard = article.id)}>↩ {t("card.promotes", { title: article.title })}</button>
        {/if}
      </div>
    {/if}

    {#if network && pubStatus === "published"}
      <section class="results" aria-labelledby="results-h">
        <div class="row between">
          <h3 id="results-h">{t("metrics.title")}</h3>
          {#if metricsRecorded}<span class="small muted">{t("metrics.recorded", { date: metricsRecorded })}</span>{/if}
        </div>
        <p class="small muted">{t("metrics.hint")}</p>
        <div class="metrics">
          {#each METRIC_KEYS as k (k)}
            <label class="field">
              {t(`metrics.${k}`)}
              <input type="number" min="0" step="1" inputmode="numeric" bind:value={metrics[k]} oninput={() => touch("metrics", 1200)} />
            </label>
          {/each}
        </div>
      </section>
    {/if}

    <section class="body">
      <div class="row between">
        <div class="row" role="group" aria-label={t("card.body_mode")}>
          <button class:active={!preview} aria-pressed={!preview} onclick={() => (preview = false)}>{t("card.edit")}</button>
          <button class:active={preview} aria-pressed={preview} onclick={() => (preview = true)}>{t("card.preview")}</button>
        </div>
        {#if network}
          <span class="chip" class:danger={netCfg?.char_limit != null && count > netCfg.char_limit} aria-live="polite">
            {t("card.chars", { n: count })}{#if netCfg?.char_limit != null}&nbsp;/ {netCfg.char_limit}{/if}
          </span>
        {:else if stored.checklist.total}
          <span class="chip">☑ {stored.checklist.done}/{stored.checklist.total}</span>
        {/if}
      </div>
      {#if preview}
        <div class="markdown preview" bind:this={previewEl}>{@html renderMarkdown(body)}</div>
      {:else}
        <textarea
          bind:value={body}
          oninput={() => touch("body")}
          aria-label={t("card.body")}
          placeholder={network ? t("card.body_post_hint") : t("card.body_hint")}
          spellcheck="true"
          lang="pt-BR"
        ></textarea>
      {/if}
    </section>

    <section class="assets">
      <div class="row between">
        <h3>{t("card.assets")}</h3>
        <button class="ghost" onclick={() => api.openAssets(id).catch(fail)}>🗀 {t("card.open_assets")}</button>
      </div>
      {#if assets.length}
        <ul>
          {#each assets as a (a.path)}
            <li><button class="ghost" onclick={() => api.openPath(a.path).catch(fail)}>{a.name}</button></li>
          {/each}
        </ul>
      {:else}
        <p class="small muted">{t("card.no_assets")}</p>
      {/if}
    </section>

    <footer class="row">
      <button onclick={archive}>{t("card.archive")}</button>
      <button class="danger" onclick={trash}>{t("card.trash")}</button>
      <span class="spacer"></span>
      <button class="ghost small" onclick={() => api.openPath(stored.path).catch(fail)} title={stored.path}>
        {t("card.open_file")}
      </button>
    </footer>
  {/if}
</aside>

{#if conflict}
  <Modal title={t("conflict.title")} onclose={resolveKeepDisk} width="900px">
    <p>{t("conflict.intro")}</p>
    <div class="conflict">
      <div>
        <h3>{t("conflict.disk")}</h3>
        <pre>{conflict.disk}</pre>
      </div>
      <div>
        <h3>{t("conflict.mine")}</h3>
        <pre>{title}

{body}</pre>
      </div>
    </div>
    <div class="row">
      <button onclick={resolveKeepDisk}>{t("conflict.keep_disk")}</button>
      <button class="primary" onclick={resolveKeepMine}>{t("conflict.keep_mine")}</button>
      <button onclick={() => api.copyText(body).then(() => toast(t("conflict.copied")))}>{t("conflict.copy_mine")}</button>
    </div>
  </Modal>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.2);
    z-index: 40;
  }

  .editor {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(720px, 100vw);
    background: var(--bg);
    border-left: 1px solid var(--border);
    box-shadow: var(--shadow-2);
    z-index: 50;
    overflow-y: auto;
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  .editor:focus {
    outline: none;
  }

  header {
    display: flex;
    gap: var(--sp-2);
    align-items: center;
  }

  .title {
    flex: 1;
    font-size: var(--fs-xl);
    font-weight: 600;
    border-color: transparent;
    background: transparent;
  }

  .title:hover,
  .title:focus {
    border-color: var(--border);
    background: var(--surface);
  }

  .status {
    white-space: nowrap;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--sp-3);
  }

  .nowrap {
    flex-wrap: nowrap;
  }

  .wide {
    grid-column: span 2;
  }

  .nowrap input[type="date"] {
    flex: 1;
    min-width: 0;
  }

  .publication {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .grow {
    flex: 1;
    min-width: 200px;
  }

  .results {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .results p {
    margin: 0;
  }

  .metrics {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: var(--sp-3);
  }

  .chip-btn {
    border-radius: 999px;
    font-size: var(--fs-sm);
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .between {
    justify-content: space-between;
  }

  .active {
    background: var(--accent-soft);
    color: var(--accent);
    border-color: var(--accent);
  }

  textarea {
    min-height: 320px;
    font-family: var(--font-mono);
    font-size: var(--fs-md);
    line-height: 1.55;
  }

  .preview {
    min-height: 320px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--sp-3);
  }

  .assets ul {
    margin: 0;
    padding-left: var(--sp-4);
  }

  footer {
    border-top: 1px solid var(--border);
    padding-top: var(--sp-3);
  }

  .spacer {
    flex: 1;
  }

  .conflict {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-3);
  }

  .conflict pre {
    background: var(--surface-2);
    padding: var(--sp-2);
    border-radius: var(--radius-sm);
    max-height: 50vh;
    overflow: auto;
    white-space: pre-wrap;
    font-size: var(--fs-sm);
  }
</style>
