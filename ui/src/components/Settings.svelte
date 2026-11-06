<script lang="ts">
  import { onMount } from "svelte";
  import { open, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api, type McpSnippet, type NetworkConfig, type Preferences } from "../lib/api";
  import { WEEKDAYS_SHORT } from "../lib/dates";
  import { t, tn } from "../lib/i18n";
  import { app, fail, reload, toast } from "../lib/store.svelte";
  import Icon from "./Icon.svelte";

  const DAYS = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
  const cfg = app.index!.config;

  let timezone = $state(cfg.timezone);
  let networks = $state<NetworkConfig[]>(structuredClone($state.snapshot(cfg.networks)) as NetworkConfig[]);
  // Goal inputs live apart from the network objects: 0 means "no goal".
  let goals = $state(
    cfg.networks.map((n) => ({ posts: n.cadence_goal?.posts ?? 0, per: (n.cadence_goal?.per ?? "week") as "week" | "month" })),
  );
  let postDefault = $state(cfg.reminders.post_default.join(", "));
  let deadlineDefault = $state(cfg.reminders.deadline_default.join(", "));
  let quietOn = $state(cfg.quiet_hours !== null);
  let quietFrom = $state(cfg.quiet_hours?.from.slice(0, 5) ?? "23:00");
  let quietTo = $state(cfg.quiet_hours?.to.slice(0, 5) ?? "07:00");
  let emptyDays = $state(cfg.empty_queue_days);
  let staleDays = $state(cfg.stale_after_days);
  let autoArchive = $state(cfg.auto_archive_days ?? 0);
  let prefs = $state<Preferences | null>(null);
  let mcp = $state<McpSnippet | null>(null);

  onMount(async () => {
    prefs = await api.preferences().catch(() => null);
    mcp = await api.mcpSnippet().catch(() => null);
  });

  const list = (s: string) =>
    s
      .split(",")
      .map((x) => x.trim())
      .filter(Boolean);

  async function save(e: SubmitEvent) {
    e.preventDefault();
    try {
      await api.updateConfig({
        timezone,
        networks: networks.map((n, i) => ({
          ...n,
          char_limit: n.char_limit ? Number(n.char_limit) : null,
          url: n.url || null,
          cadence_goal: Number(goals[i].posts) > 0 ? { posts: Number(goals[i].posts), per: goals[i].per } : null,
        })),
        reminders: { post_default: list(postDefault), deadline_default: list(deadlineDefault) },
        quiet_hours: quietOn ? { from: quietFrom, to: quietTo } : null,
        empty_queue_days: Number(emptyDays),
        stale_after_days: Number(staleDays),
        auto_archive_days: Number(autoArchive) > 0 ? Number(autoArchive) : null,
        art: { tool_url: toolUrl, brand },
      });
      toast(t("settings.saved"));
    } catch (err) {
      fail(err);
    }
  }

  function toggleDay(n: NetworkConfig, s: number, day: string) {
    const days = n.slots[s].days;
    n.slots[s].days = days.includes(day) ? days.filter((d) => d !== day) : DAYS.filter((d) => d === day || days.includes(d));
  }

  async function setPref(key: string, value: unknown) {
    try {
      prefs = await api.setPreference(key, value);
      if (key === "theme") {
        if (value === "system") document.documentElement.removeAttribute("data-theme");
        else document.documentElement.dataset.theme = String(value);
      }
    } catch (e) {
      fail(e);
    }
  }

  async function changeFolder() {
    const picked = await open({ directory: true });
    if (typeof picked !== "string") return;
    if (!confirm(t("settings.change_folder_confirm", { path: picked }))) return;
    try {
      await api.setup(picked);
      await reload();
      toast(t("settings.folder_changed"));
    } catch (e) {
      fail(e);
    }
  }

  let toolUrl = $state(cfg.art.tool_url);
  let brand = $state(cfg.art.brand ?? "");
  let icsAll = $state(false);

  async function exportIcs() {
    const path = await saveDialog({ defaultPath: "pauta.ics", filters: [{ name: "iCalendar", extensions: ["ics"] }] });
    if (!path) return;
    try {
      const n = await api.exportIcs(path, icsAll);
      toast(tn("settings.ics_done", n));
    } catch (e) {
      fail(e);
    }
  }

  async function copy(text: string) {
    await api.copyText(text).then(() => toast(t("common.copied"))).catch(fail);
  }
</script>

<div class="settings">
  <h1 class="display">{t("settings.title")}</h1>

  <section class="panel">
    <h2>{t("settings.data")}</h2>
    <div class="row">
      <code>{app.status?.root}</code>
      <button class="with-icon" onclick={() => app.status?.root && api.openPath(app.status.root).catch(fail)}><Icon name="folder" />{t("settings.open_folder")}</button>
      <button onclick={changeFolder}>{t("settings.change_folder")}</button>
      <button class="with-icon" onclick={() => (app.trashOpen = true)}><Icon name="trash-2" />{t("trash.title")}</button>
    </div>
    <p class="small muted">{t("settings.data_hint")}</p>
  </section>

  <form class="panel form" onsubmit={save}>
    <h2>{t("settings.schedule")}</h2>
    <label class="field narrow">
      {t("settings.timezone")}
      <input type="text" bind:value={timezone} />
    </label>

    {#each networks as n, ni (ni)}
      <fieldset>
        <legend>{n.label || n.id}</legend>
        <div class="row">
          <label class="field">{t("settings.net_label")}<input type="text" bind:value={n.label} /></label>
          <label class="field grow">{t("settings.net_url")}<input type="url" bind:value={n.url} /></label>
          <label class="field">{t("settings.net_limit")}<input type="number" min="1" bind:value={n.char_limit} style="width: 90px" /></label>
        </div>
        <div class="row">
          <label class="field">
            {t("settings.goal_posts")}
            <input type="number" min="0" max="100" bind:value={goals[ni].posts} style="width: 90px" />
          </label>
          <label class="field">
            {t("settings.goal_per")}
            <select bind:value={goals[ni].per}>
              <option value="week">{t("settings.goal_week")}</option>
              <option value="month">{t("settings.goal_month")}</option>
            </select>
          </label>
          <span class="small muted hint">{t("settings.goal_hint")}</span>
        </div>
        <label class="row small">
          <input type="checkbox" bind:checked={n.media.requires_media} /> {t("settings.requires_media")}
        </label>
        <p class="small muted">{t("settings.slots")}</p>
        {#each n.slots as slot, si (si)}
          <div class="row slot">
            <div class="row" role="group" aria-label={t("settings.slot_days")}>
              {#each DAYS as d, di (d)}
                <button
                  type="button"
                  class="day"
                  class:on={slot.days.includes(d)}
                  aria-pressed={slot.days.includes(d)}
                  onclick={() => toggleDay(n, si, d)}>{WEEKDAYS_SHORT[di]}</button
                >
              {/each}
            </div>
            <input type="time" bind:value={slot.time} aria-label={t("settings.slot_time")} />
            <button type="button" class="ghost danger" aria-label={t("settings.slot_remove")} onclick={() => n.slots.splice(si, 1)}><Icon name="x" /></button>
          </div>
        {/each}
        <button type="button" class="with-icon" onclick={() => n.slots.push({ days: ["tue"], time: "09:00" })}><Icon name="plus" />{t("settings.slot_add")}</button>
      </fieldset>
    {/each}

    <div class="row">
      <label class="field grow">{t("settings.post_reminders")}<input type="text" bind:value={postDefault} /></label>
      <label class="field grow">{t("settings.deadline_reminders")}<input type="text" bind:value={deadlineDefault} /></label>
    </div>
    <p class="small muted">{t("settings.reminders_hint")}</p>

    <div class="row">
      <label class="row small"><input type="checkbox" bind:checked={quietOn} /> {t("settings.quiet")}</label>
      {#if quietOn}
        <input type="time" bind:value={quietFrom} aria-label={t("settings.quiet_from")} />
        <span>–</span>
        <input type="time" bind:value={quietTo} aria-label={t("settings.quiet_to")} />
      {/if}
    </div>

    <div class="row">
      <label class="field">{t("settings.empty_days")}<input type="number" min="1" bind:value={emptyDays} /></label>
      <label class="field">{t("settings.stale_days")}<input type="number" min="1" bind:value={staleDays} /></label>
      <label class="field">{t("settings.auto_archive")}<input type="number" min="0" bind:value={autoArchive} /></label>
    </div>
    <h3>{t("settings.art")}</h3>
    <p class="small muted">{t("settings.art_hint")}</p>
    <label class="field">
      {t("settings.art_url")}
      <input type="url" bind:value={toolUrl} />
    </label>
    <label class="field">
      {t("settings.art_brand")}
      <textarea rows="4" bind:value={brand} placeholder={t("settings.art_brand_hint")}></textarea>
    </label>
    <div><button class="primary" type="submit">{t("common.save")}</button></div>
  </form>

  <section class="panel form">
    <h2>{t("settings.ics")}</h2>
    <p class="small muted">{t("settings.ics_hint")}</p>
    <div class="row">
      <label class="row small"><input type="checkbox" bind:checked={icsAll} /> {t("settings.ics_all")}</label>
      <button class="with-icon" onclick={exportIcs}><Icon name="download" />{t("settings.ics_export")}</button>
    </div>
  </section>

  {#if prefs}
    <section class="panel form">
      <h2>{t("settings.app")}</h2>
      <label class="row"><input type="checkbox" checked={prefs.autostart} onchange={(e) => setPref("autostart", e.currentTarget.checked)} /> {t("settings.autostart")}</label>
      <label class="row"><input type="checkbox" checked={prefs.close_to_tray} onchange={(e) => setPref("close_to_tray", e.currentTarget.checked)} /> {t("settings.close_to_tray")}</label>
      <label class="field narrow">
        {t("settings.theme")}
        <select value={prefs.theme} onchange={(e) => setPref("theme", e.currentTarget.value)}>
          {#each ["system", "light", "dark"] as th (th)}<option value={th}>{t(`settings.theme_${th}`)}</option>{/each}
        </select>
      </label>
    </section>
  {/if}

  <section class="panel form">
    <h2>{t("settings.mcp")}</h2>
    <p class="small muted">{t("settings.mcp_intro")}</p>
    {#if mcp}
      {#if !mcp.binary_exists}<p class="chip warn">{t("settings.mcp_missing", { path: mcp.binary })}</p>{/if}
      <h3>Claude Code</h3>
      <div class="snippet"><pre>{mcp.claude_code}</pre><button onclick={() => copy(mcp!.claude_code)}>{t("common.copy")}</button></div>
      <h3>Claude Desktop</h3>
      <p class="small muted">{t("settings.mcp_desktop_hint")}</p>
      <div class="snippet"><pre>{mcp.claude_desktop}</pre><button onclick={() => copy(mcp!.claude_desktop)}>{t("common.copy")}</button></div>
    {/if}
  </section>

  <section class="panel">
    <h2>{t("settings.shortcuts")}</h2>
    <dl>
      <dt>Ctrl+K</dt><dd>{t("settings.sc_palette")}</dd>
      <dt>Alt+1…5</dt><dd>{t("settings.sc_views")}</dd>
      <dt>Ctrl+Shift+I</dt><dd>{t("settings.sc_capture")}</dd>
      <dt>Ctrl+Alt+P</dt><dd>{t("settings.sc_global")}</dd>
      <dt>Alt+← → ↑ ↓</dt><dd>{t("settings.sc_move")}</dd>
      <dt>Esc</dt><dd>{t("settings.sc_close")}</dd>
    </dl>
  </section>
</div>

<style>
  /* The text starts where Today's does, so the reading screens share one left edge. */
  .settings {
    --indent: 148px;
    max-width: calc(var(--indent) + 844px);
    margin: 0;
    padding: var(--sp-5) var(--sp-4) var(--sp-5) var(--indent);
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  /* The gap does the spacing; a paragraph margin on top of it opened holes. */
  p {
    margin: 0;
  }

  /* A small heading belongs to the text under it, not to the block above. */
  h3 {
    margin: var(--sp-2) 0 calc(-1 * var(--sp-1));
  }

  fieldset {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  legend {
    font-family: var(--font-serif);
    font-size: var(--fs-md);
    font-weight: 500;
  }

  fieldset > button {
    align-self: flex-start;
  }

  .narrow {
    max-width: 280px;
  }

  /* The hint reads on the line of the input, not in the middle of label and input. */
  .hint {
    align-self: flex-end;
    line-height: 32px;
  }

  .grow {
    flex: 1;
    min-width: 200px;
  }

  .day {
    min-width: 40px;
    padding: 0 var(--sp-1);
    font-size: var(--fs-sm);
  }

  .day.on {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-text);
  }

  .snippet {
    display: flex;
    gap: var(--sp-2);
    align-items: flex-start;
  }

  pre {
    flex: 1;
    margin: 0;
    background: var(--surface-2);
    padding: var(--sp-2);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    white-space: pre-wrap;
    word-break: break-all;
  }

  code {
    font-family: var(--font-mono);
    background: var(--surface-2);
    padding: 2px 6px;
    border-radius: 4px;
  }

  dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: var(--sp-1) var(--sp-4);
    margin: var(--sp-2) 0 0;
  }

  dt {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }

  dd {
    margin: 0;
  }

  @media (max-width: 860px) {
    .settings {
      --indent: 108px;
    }
  }
</style>
