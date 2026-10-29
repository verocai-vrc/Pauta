#!/usr/bin/env bash
# Dumps the JSON the UI receives from the Rust backend, at a fixed clock, so the
# browser checks run against real output instead of hand-written mocks.
# Usage: ui/e2e/make-fixtures.sh <output dir>
set -euo pipefail
out="${1:?output dir}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$out"
cp -r "$root/examples/sample-data/." "$tmp/"
rm -f "$tmp/boards/dungeon-meister/cards/cabecalho-quebrado--broken.md"   # keep the checked screens free of the deliberate error

# Twelve weeks of published posts, so the history charts have something to show.
i=0
for week in 0 1 2 3 4 5 6 7 8 9 10; do
  for net in linkedin instagram; do
    # Skip some weeks so the charts are not flat.
    if [ $(( (week + i) % 3 )) -eq 2 ]; then i=$((i+1)); continue; fi
    day=$(date -d "2026-10-26 - $((week*7)) days + $(( (i % 4) + 1 )) days" +%Y-%m-%d)
    id="01FIXTURE$(printf '%02d' $i)$(printf '%02d' $week)"
    metrics=""
    if [ $((i % 2)) -eq 0 ]; then   # numbers recorded on every other post
      metrics="metrics:
  impressions: $((800 + i * 130))
  reactions: $((10 + i * 3))
  comments: $((1 + i % 5))
  shares: $((i % 4))
  recorded: ${day}"
    fi
    cat > "$tmp/boards/conteudo/cards/post-$id--fx$i.md" <<CARD
---
id: $id
title: "Post publicado de exemplo $i"
kind: post
column: published
rank: "m$i"
network: $net
publication:
  at: ${day}T09:00:00-03:00
  status: published
  url: https://example.com/post/$i
  published_at: ${day}T09:05:00-03:00
${metrics}
created: ${day}T08:00:00-03:00
updated: ${day}T09:05:00-03:00
---

Texto.
CARD
    i=$((i+1))
  done
done

# A pending Instagram post whose art was requested and whose files already arrived.
cat > "$tmp/boards/conteudo/cards/carrossel-de-exemplo--fxart1.md" <<CARD
---
id: 01FIXTUREART1
title: "Carrossel de exemplo com arte pedida"
kind: post
column: producing
rank: "n1"
network: instagram
publication:
  at: 2026-11-04T19:00:00-03:00
  status: planned
  url: null
art:
  status: requested
  format: carousel
  requested_at: 2026-10-27T10:00:00-03:00
created: 2026-10-27T09:00:00-03:00
updated: 2026-10-27T10:00:00-03:00
---

Legenda do carrossel de exemplo.

## Arte

Cinco slides, fundo vinho, um conceito por slide.
CARD
mkdir -p "$tmp/boards/conteudo/assets/01FIXTUREART1"
printf x > "$tmp/boards/conteudo/assets/01FIXTUREART1/01.png"
printf x > "$tmp/boards/conteudo/assets/01FIXTUREART1/02.png"

export PAUTA_NOW="2026-10-28T21:00:00-03:00"
cli=(cargo run -q --manifest-path "$root/Cargo.toml" -p pauta-cli -- --root "$tmp")
"${cli[@]}" index-json > "$out/index.json"
"${cli[@]}" today > "$out/today.json"
"${cli[@]}" rhythm > "$out/rhythm.json"
"${cli[@]}" agenda-json 2026-10-01 2026-12-31 > "$out/agenda.json"
echo "fixtures written to $out"
