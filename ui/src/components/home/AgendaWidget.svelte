<script lang="ts">
  import type { AgendaItem } from "../../lib/api";
  import { t } from "../../lib/i18n";
  import { app } from "../../lib/store.svelte";
  import Widget from "./Widget.svelte";

  let { agenda, limit }: { agenda: AgendaItem[]; limit: number } = $props();
  void agenda;
  void limit;

  function more() {
    app.calendarMode = "agenda";
    app.view = "calendar";
  }
</script>

<Widget id="agenda" title={t("home.agenda")} onmore={more}>
  <p class="muted small">…</p>
</Widget>
