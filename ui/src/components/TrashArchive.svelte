<script lang="ts">
  import { onMount } from "svelte";
  import { api, type ArchivedItem, type TrashEntry } from "../lib/api";
  import { t } from "../lib/i18n";
  import { app, fail, toast } from "../lib/store.svelte";
  import Modal from "./Modal.svelte";

  let trash = $state<TrashEntry[]>([]);
  let archive = $state<ArchivedItem[]>([]);

  async function load() {
    try {
      [trash, archive] = await Promise.all([api.listTrash(), api.listArchive()]);
    } catch (e) {
      fail(e);
    }
  }

  onMount(load);

  async function restore(fn: () => Promise<void>) {
    try {
      await fn();
      toast(t("trash.restored"));
      await load();
    } catch (e) {
      fail(e);
    }
  }

  function stamp(s: string) {
    // 20261028T201500 -> 2026-10-28 20:15
    return s.length >= 13 ? `${s.slice(0, 4)}-${s.slice(4, 6)}-${s.slice(6, 8)} ${s.slice(9, 11)}:${s.slice(11, 13)}` : s;
  }
</script>

<Modal title={t("trash.title")} onclose={() => (app.trashOpen = false)} width="680px">
  <section>
    <h3>{t("trash.archive")}</h3>
    {#if archive.length === 0}<p class="small muted">{t("trash.empty")}</p>{/if}
    <ul>
      {#each archive as a (a.path)}
        <li>
          <span><span class="chip">{t(`trash.kind_${a.kind}`)}</span> {a.title} {#if a.board && a.kind !== "board"}<span class="small muted">· {a.board}</span>{/if}</span>
          <button onclick={() => restore(() => api.restoreArchived(a.path))}>{t("trash.restore")}</button>
        </li>
      {/each}
    </ul>
  </section>
  <section>
    <h3>{t("trash.trash")}</h3>
    {#if trash.length === 0}<p class="small muted">{t("trash.empty")}</p>{/if}
    <ul>
      {#each trash as e (e.path)}
        <li>
          <span>{e.title} <span class="small muted">· {stamp(e.deleted_at)} · {e.original}</span></span>
          <button onclick={() => restore(() => api.restoreTrash(e.path))}>{t("trash.restore")}</button>
        </li>
      {/each}
    </ul>
    <p class="small muted">{t("trash.hint")}</p>
  </section>
</Modal>

<style>
  ul {
    list-style: none;
    padding: 0;
    margin: var(--sp-2) 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    max-height: 30vh;
    overflow-y: auto;
  }

  li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--sp-3);
  }
</style>
