<script module lang="ts">
  // Icons from Lucide (https://lucide.dev), copied to avoid a runtime dependency.
  // ISC License, Copyright (c) 2026 Lucide Icons and Contributors.
  //
  // Permission to use, copy, modify, and/or distribute this software for any purpose with or
  // without fee is hereby granted, provided that the above copyright notice and this permission
  // notice appear in all copies.
  //
  // THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS
  // SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE
  // AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
  // WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT,
  // NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
  // PERFORMANCE OF THIS SOFTWARE.
  const icons = {
    "sun": [["circle", { cx: "12", cy: "12", r: "4" }], ["path", { d: "M12 2v2" }], ["path", { d: "M12 20v2" }], ["path", { d: "m4.93 4.93 1.41 1.41" }], ["path", { d: "m17.66 17.66 1.41 1.41" }], ["path", { d: "M2 12h2" }], ["path", { d: "M20 12h2" }], ["path", { d: "m6.34 17.66-1.41 1.41" }], ["path", { d: "m19.07 4.93-1.41 1.41" }]],
    "columns-3": [["rect", { width: "18", height: "18", x: "3", y: "3", rx: "2" }], ["path", { d: "M9 3v18" }], ["path", { d: "M15 3v18" }]],
    "calendar": [["path", { d: "M8 2v3" }], ["path", { d: "M16 2v3" }], ["rect", { x: "3", y: "3", width: "18", height: "18", rx: "2" }], ["path", { d: "M3 9h18" }]],
    "chart-column": [["path", { d: "M3 3v16a2 2 0 0 0 2 2h16" }], ["path", { d: "M18 17V9" }], ["path", { d: "M13 17V5" }], ["path", { d: "M8 17v-3" }]],
    "settings": [["path", { d: "M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915" }], ["circle", { cx: "12", cy: "12", r: "3" }]],
    "plus": [["path", { d: "M5 12h14" }], ["path", { d: "M12 5v14" }]],
    "search": [["path", { d: "m21 21-4.34-4.34" }], ["circle", { cx: "11", cy: "11", r: "8" }]],
    "clock": [["circle", { cx: "12", cy: "12", r: "10" }], ["path", { d: "M12 6v6l4 2" }]],
    "copy": [["rect", { width: "14", height: "14", x: "8", y: "8", rx: "2", ry: "2" }], ["path", { d: "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" }]],
    "external-link": [["path", { d: "M15 3h6v6" }], ["path", { d: "M10 14 21 3" }], ["path", { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }]],
    "folder": [["path", { d: "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" }]],
    "trash-2": [["path", { d: "M10 11v6" }], ["path", { d: "M14 11v6" }], ["path", { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" }], ["path", { d: "M3 6h18" }], ["path", { d: "M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }]],
    "chevron-left": [["path", { d: "m15 18-6-6 6-6" }]],
    "chevron-right": [["path", { d: "m9 18 6-6-6-6" }]],
    "x": [["path", { d: "M18 6 6 18" }], ["path", { d: "m6 6 12 12" }]],
    "check": [["path", { d: "M20 6 9 17l-5-5" }]],
    "square-check": [["rect", { width: "18", height: "18", x: "3", y: "3", rx: "2" }], ["path", { d: "m16 9-5.5 5.5L8 12" }]],
    "triangle-alert": [["path", { d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" }], ["path", { d: "M12 9v4" }], ["path", { d: "M12 17h.01" }]],
    "flag": [["path", { d: "M4 22V4a1 1 0 0 1 .4-.8A6 6 0 0 1 8 2c3 0 5 2 7.333 2q2 0 3.067-.8A1 1 0 0 1 20 4v10a1 1 0 0 1-.4.8A6 6 0 0 1 16 16c-3 0-5-2-8-2a6 6 0 0 0-4 1.528" }]],
    "star": [["path", { d: "M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z" }]],
    "rocket": [["path", { d: "M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5" }], ["path", { d: "M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09" }], ["path", { d: "M9 12a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.4 22.4 0 0 1-4 2z" }], ["path", { d: "M9 12H4s.55-3.03 2-4c1.62-1.08 5 .05 5 .05" }]],
    "diamond": [["path", { d: "M2.7 10.3a2.41 2.41 0 0 0 0 3.41l7.59 7.59a2.41 2.41 0 0 0 3.41 0l7.59-7.59a2.41 2.41 0 0 0 0-3.41l-7.59-7.59a2.41 2.41 0 0 0-3.41 0Z" }]],
    "palette": [["path", { d: "M12 22a1 1 0 0 1 0-20 10 9 0 0 1 10 9 5 5 0 0 1-5 5h-2.25a1.75 1.75 0 0 0-1.4 2.8l.3.4a1.75 1.75 0 0 1-1.4 2.8z" }], ["circle", { cx: "13.5", cy: "6.5", r: ".5", fill: "currentColor" }], ["circle", { cx: "17.5", cy: "10.5", r: ".5", fill: "currentColor" }], ["circle", { cx: "6.5", cy: "12.5", r: ".5", fill: "currentColor" }], ["circle", { cx: "8.5", cy: "7.5", r: ".5", fill: "currentColor" }]],
    "arrow-left-right": [["path", { d: "M8 3 4 7l4 4" }], ["path", { d: "M4 7h16" }], ["path", { d: "m16 21 4-4-4-4" }], ["path", { d: "M20 17H4" }]],
    "undo-2": [["path", { d: "M9 14 4 9l5-5" }], ["path", { d: "M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5a5.5 5.5 0 0 1-5.5 5.5H11" }]],
    "download": [["path", { d: "M12 15V3" }], ["path", { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }], ["path", { d: "m7 10 5 5 5-5" }]],
    "arrow-left": [["path", { d: "m12 19-7-7 7-7" }], ["path", { d: "M19 12H5" }]],
    "arrow-right": [["path", { d: "M5 12h14" }], ["path", { d: "m12 5 7 7-7 7" }]],
    "arrow-up": [["path", { d: "m5 12 7-7 7 7" }], ["path", { d: "M12 19V5" }]],
    "arrow-down": [["path", { d: "M12 5v14" }], ["path", { d: "m19 12-7 7-7-7" }]],
  } satisfies Record<string, [string, Record<string, string>][]>;

  export type IconName = keyof typeof icons;
</script>

<script lang="ts">
  let { name, size = 16 }: { name: IconName; size?: number } = $props();
</script>

<!-- Always decorative: the accessible name comes from the text or aria-label beside it. -->
<svg
  class="icon"
  data-icon={name}
  aria-hidden="true"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
>
  {#each icons[name] as [tag, attrs], i (i)}<svelte:element this={tag} {...attrs} />{/each}
</svg>

<style>
  .icon {
    flex: none;
    vertical-align: -0.24em;
  }
</style>
