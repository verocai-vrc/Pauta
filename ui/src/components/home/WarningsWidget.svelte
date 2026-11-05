<script lang="ts">
  import type { AgendaItem } from "../../lib/api";
  import { t } from "../../lib/i18n";
  import { openToday } from "../../lib/store.svelte";
  import Widget from "./Widget.svelte";

  let { agenda, limit }: { agenda: AgendaItem[]; limit: number } = $props();
  void agenda;
  void limit;
</script>

<Widget id="warnings" title={t("home.warnings")} onmore={openToday}>
  <p class="muted small">…</p>
</Widget>
