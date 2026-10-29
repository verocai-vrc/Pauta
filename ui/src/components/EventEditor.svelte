<script lang="ts">
  import { api } from "../lib/api";
  import { localInput } from "../lib/dates";
  import { t } from "../lib/i18n";
  import { app, fail, toast } from "../lib/store.svelte";
  import Modal from "./Modal.svelte";

  let { id }: { id: string } = $props();

  // The editor is re-created per id ({#key} in App.svelte), so reading it once is intended.
  // svelte-ignore state_referenced_locally
  const isNew = id === "new";
  const ev = isNew ? undefined : app.index?.events.find((e) => e.id === id);

  let title = $state(ev?.title ?? "");
  let kind = $state(ev?.kind ?? "deadline");
  let allDay = $state(ev ? ev.when.time === null : true);
  let date = $state(ev?.when.date ?? app.newEventDate ?? app.today?.date ?? "");
  let time = $state(ev?.when.time?.slice(0, 5) ?? "09:00");
  let end = $state(localInput(ev?.end));
  let reminders = $state(ev?.reminders?.join(", ") ?? "");
  let boardId = $state(ev?.links.board ?? "");
  let tags = $state(ev?.tags.join(", ") ?? "");
  let body = $state(ev?.body.replace(/^\n/, "") ?? "");

  const list = (s: string) =>
    s
      .split(",")
      .map((x) => x.trim().replace(/^#/, ""))
      .filter(Boolean);

  function close() {
    app.openEvent = null;
    app.newEventDate = null;
  }

  async function save(e: SubmitEvent) {
    e.preventDefault();
    const when = allDay ? date : `${date}T${time}`;
    try {
      if (isNew) {
        await api.createEvent({
          title,
          kind,
          date: when,
          end: end || undefined,
          reminders: reminders.trim() ? list(reminders) : undefined,
          board: boardId || undefined,
          tags: list(tags),
          body,
        });
      } else {
        await api.updateEvent(
          id,
          {
            title,
            kind,
            date: when,
            end: end || null,
            reminders: reminders.trim() ? list(reminders) : null,
            board: boardId || null,
            tags: list(tags),
            body,
          },
          ev?.hash,
        );
      }
      toast(t("event.saved"));
      close();
    } catch (err) {
      fail(err);
    }
  }

  async function archive() {
    await api.archiveEvent(id).then(close).catch(fail);
  }

  async function trash() {
    if (confirm(t("event.trash_confirm"))) await api.trashEvent(id).then(close).catch(fail);
  }
</script>

<Modal title={isNew ? t("event.new") : t("event.edit")} onclose={close} width="560px">
  {#if !isNew && !ev}
    <p class="muted">{t("event.missing")}</p>
  {:else}
    <form class="form" onsubmit={save}>
      <label class="field">
        {t("event.title")}
        <!-- svelte-ignore a11y_autofocus -->
        <input type="text" bind:value={title} required autofocus />
      </label>
      <div class="row">
        <label class="field">
          {t("event.kind")}
          <select bind:value={kind}>
            {#each ["deadline", "event", "milestone", "launch"] as k (k)}<option value={k}>{t(`event.${k}`)}</option>{/each}
          </select>
        </label>
        <label class="field">
          {t("event.date")}
          <input type="date" bind:value={date} required />
        </label>
        <label class="row small allday"><input type="checkbox" bind:checked={allDay} /> {t("event.all_day")}</label>
        {#if !allDay}
          <label class="field">
            {t("event.time")}
            <input type="time" bind:value={time} required />
          </label>
        {/if}
      </div>
      <div class="row">
        <label class="field">
          {t("event.end")}
          <input type="datetime-local" bind:value={end} />
        </label>
        <label class="field">
          {t("event.board")}
          <select bind:value={boardId}>
            <option value="">—</option>
            {#each app.index?.boards ?? [] as b (b.id)}<option value={b.id}>{b.name}</option>{/each}
          </select>
        </label>
      </div>
      <label class="field">
        {t("event.reminders")}
        <input type="text" bind:value={reminders} placeholder={app.index?.config.reminders.deadline_default.join(", ")} />
      </label>
      <label class="field">
        {t("card.tags")}
        <input type="text" bind:value={tags} />
      </label>
      <label class="field">
        {t("event.notes")}
        <textarea rows="5" bind:value={body}></textarea>
      </label>
      <div class="row">
        <button class="primary" type="submit">{t("common.save")}</button>
        <button type="button" onclick={close}>{t("common.cancel")}</button>
        <span class="spacer"></span>
        {#if !isNew}
          <button type="button" onclick={archive}>{t("card.archive")}</button>
          <button type="button" class="danger" onclick={trash}>{t("card.trash")}</button>
        {/if}
      </div>
    </form>
  {/if}
</Modal>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .allday {
    align-self: flex-end;
    min-height: 30px;
  }

  .spacer {
    flex: 1;
  }
</style>
