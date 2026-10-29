<script lang="ts">
  import { api, errorMessage, type ImportPreview } from "../lib/api";
  import { t } from "../lib/i18n";
  import { fail, networkLabel, reload, toast } from "../lib/store.svelte";
  import Modal from "./Modal.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let text = $state("");
  let preview = $state<ImportPreview | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);

  const FIELDS = ["impressions", "reactions", "comments", "shares", "saves", "clicks"] as const;

  async function pick(e: Event) {
    const file = (e.currentTarget as HTMLInputElement).files?.[0];
    preview = null;
    error = null;
    if (!file) return;
    try {
      text = await file.text();
      preview = await api.previewMetricsImport(text);
    } catch (err) {
      error = errorMessage(err);
    }
  }

  async function apply() {
    busy = true;
    try {
      const r = await api.importMetrics(text);
      if (r.applied.length) toast(t("import.done", { n: r.applied.length }));
      if (r.failed.length) toast(t("import.failed", { n: r.failed.length }), "error");
      await reload();
      onclose();
    } catch (err) {
      fail(err);
    } finally {
      busy = false;
    }
  }

  const header = (f: string) => (f === "url" ? t("import.field_url") : t(`metrics.${f}`));
  const num = (n: number | null | undefined) => (n == null ? "–" : String(n));
</script>

<Modal title={t("import.title")} {onclose} width="680px">
  <p class="muted">{t("import.intro")}</p>
  <label>
    {t("import.file")}
    <input type="file" accept=".csv,.tsv,.txt,text/csv" onchange={pick} />
  </label>

  {#if error}<p class="error" role="alert">{error}</p>{/if}

  {#if preview}
    <h3>{t("import.columns")}</h3>
    <ul class="cols">
      {#each preview.columns as c (c.field)}
        <li><strong>{header(c.field)}</strong> ← “{c.header}”</li>
      {/each}
    </ul>

    {#if preview.matched.length === 0}
      <p class="muted">{t("import.none_matched")}</p>
    {:else}
      <h3>{t("import.matched", { n: preview.matched.length })}</h3>
      <p class="small muted">{t("import.kept")}</p>
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div class="scroll" role="region" tabindex="0" aria-label={t("import.matched", { n: preview.matched.length })}>
        <table>
          <thead>
            <tr>
              <th scope="col">Post</th>
              {#each FIELDS as f (f)}<th scope="col">{t(`metrics.${f}`)}</th>{/each}
            </tr>
          </thead>
          <tbody>
            {#each preview.matched as m (m.card_id)}
              <tr>
                <th scope="row">{m.title}{#if m.network}<span class="small muted"> · {networkLabel(m.network)}</span>{/if}</th>
                {#each FIELDS as f (f)}
                  <td>
                    {#if m.before?.[f] != null && m.before[f] !== m.after[f]}
                      {t("import.change", { before: num(m.before[f]), after: num(m.after[f]) })}
                    {:else}
                      {num(m.after[f])}
                    {/if}
                  </td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    {#if preview.unmatched.length}
      <p class="small">{t("import.unmatched", { n: preview.unmatched.length })}</p>
      <ul class="small muted">
        {#each preview.unmatched.slice(0, 8) as [line, url] (line)}<li>{t("import.line", { n: line })}: {url}</li>{/each}
        {#if preview.unmatched.length > 8}<li>…</li>{/if}
      </ul>
    {/if}
    {#if preview.skipped_cells.length}
      <p class="small muted">
        {t("import.skipped", {
          n: preview.skipped_cells.length,
          list: preview.skipped_cells
            .slice(0, 5)
            .map(([l, h, c]) => `${t("import.line", { n: l })} ${h} “${c}”`)
            .join("; "),
        })}
      </p>
    {/if}

    <div class="actions">
      <button class="ghost" onclick={onclose}>{t("common.cancel")}</button>
      <button class="primary" disabled={busy || preview.matched.length === 0} onclick={apply}>
        {t("import.apply", { n: preview.matched.length })}
      </button>
    </div>
  {/if}
</Modal>

<style>
  label {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    margin: var(--sp-3) 0;
  }

  .cols {
    margin: 0 0 var(--sp-3);
    padding-left: var(--sp-4);
  }

  .scroll {
    overflow-x: auto;
  }

  table {
    border-collapse: collapse;
    width: 100%;
    font-size: var(--fs-sm);
  }

  th,
  td {
    text-align: left;
    padding: 4px var(--sp-2);
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
  }

  tbody th {
    white-space: normal;
    font-weight: 500;
    min-width: 15rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--sp-2);
    margin-top: var(--sp-4);
  }

  .error {
    color: var(--danger);
  }
</style>
