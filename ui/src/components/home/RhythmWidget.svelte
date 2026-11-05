<script lang="ts">
  import { t } from "../../lib/i18n";
  import { app } from "../../lib/store.svelte";
  import Widget from "./Widget.svelte";

  let { limit }: { limit: number } = $props();
  void limit;
</script>

<Widget id="rhythm" title={t("home.rhythm")} grow={false} onmore={() => (app.view = "rhythm")}>
  <p class="muted small">…</p>
</Widget>
