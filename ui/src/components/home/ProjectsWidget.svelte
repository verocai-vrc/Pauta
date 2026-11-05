<script lang="ts">
  import { t } from "../../lib/i18n";
  import { app } from "../../lib/store.svelte";
  import Widget from "./Widget.svelte";

  let { limit }: { limit: number } = $props();
  void limit;
</script>

<Widget id="projects" title={t("home.projects")} onmore={() => (app.view = "boards")}>
  <p class="muted small">…</p>
</Widget>
