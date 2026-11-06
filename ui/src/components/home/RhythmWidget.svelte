<script lang="ts">
  import { t } from "../../lib/i18n";
  import { app } from "../../lib/store.svelte";
  import CadenceMeter from "../CadenceMeter.svelte";
  import Widget from "./Widget.svelte";

  let { limit }: { limit: number } = $props();

  const cadence = $derived(app.today!.cadence);
  const shown = $derived(cadence.slice(0, limit));
</script>

<Widget id="rhythm" title={t("home.rhythm")} hidden={Math.max(0, cadence.length - limit)} grow={false} onmore={() => (app.view = "rhythm")}>
  {#if shown.length}
    <div class="meters">
      {#each shown as c (c.network)}<CadenceMeter progress={c} inline />{/each}
    </div>
  {:else}
    <p class="muted small">{t("home.no_goals")}</p>
  {/if}
</Widget>

<style>
  .meters {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
</style>
