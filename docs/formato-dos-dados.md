# Formato dos dados (versão 1)

Contrato estável entre o app, o conector MCP e quem edita os arquivos à mão (invariante 8). Um exemplo completo está em [`examples/sample-data/`](../examples/sample-data/).

## Pasta

```text
<pasta de dados>/
  pauta.yaml                         configuração geral
  boards/<quadro>/board.yaml         um quadro
  boards/<quadro>/cards/*.md         cartões do quadro
  boards/<quadro>/assets/<id>/       anexos de um cartão
  calendar/*.md                      eventos avulsos (prazos, marcos…)
  archive/cards/<quadro>/*.md        cartões arquivados
  archive/calendar/*.md              eventos arquivados
  archive/boards/<quadro>/           quadros arquivados (pasta inteira)
  .trash/<AAAAMMDDTHHMMSS>/<caminho original>   lixeira
  .pauta/                            descartável: log.jsonl, reminders.json, backups
```

Os nomes de arquivo (`<slug>--<fim do id>.md`) são só uma conveniência: a identidade de um cartão é o `id` no cabeçalho, e o arquivo pode ser renomeado à vontade. Já a identidade de um **quadro é o nome da sua pasta** (`boards/<quadro>/`): se o `id:` do `board.yaml` discordar da pasta, vale a pasta.

## `pauta.yaml`

```yaml
schema_version: 1
timezone: America/Recife
language: pt-BR
board_order: [conteudo, tcc2]
networks:
  linkedin:
    label: "LinkedIn"
    url: "https://www.linkedin.com/feed/"   # botão "Abrir LinkedIn"
    char_limit: 3000                        # contador de caracteres
    slots:
      - { days: [tue, thu], time: "09:00" } # faixas de postagem
reminders:
  post_default: ["-1h", "0"]
  deadline_default: ["-7d", "-1d", "09:00"]
notifications:
  quiet_hours: { from: "23:00", to: "07:00" }
rhythm:
  empty_queue_days: 7      # aviso de fila vazia
  stale_after_days: 14     # cartões parados
archive:
  auto_after_days: null    # arquiva concluídos após N dias; null = nunca
art:
  tool_url: "https://claude.ai"   # "Abrir Claude Design" (não há API)
  brand: "Vinho #7a2e3b, serifa, muito espaço em branco"   # entra em todo pedido de arte
```

Por rede, além de `slots`:

```yaml
networks:
  linkedin:
    cadence_goal: { posts: 2, per: week }      # week | month; 1 a 100. Só um indicador
    media:                                     # opcional; há padrões embutidos
      requires_media: false                    # true: post sem arquivo aparece como incompleto
      formats:
        - { id: image, label: "Imagem única", size: "1200x627" }
        - { id: carousel, label: "Carrossel (documento PDF)", size: "1080x1350", note: "entregar como um único PDF" }
```

Tamanhos de imagem mudam e as fontes divergem: confira-os na ajuda de cada plataforma. Os padrões embutidos são LinkedIn imagem 1200x627 e carrossel em PDF 1080x1350; Instagram 1080x1350 (feed e carrossel) e 1080x1920 (story).

Contagens de dias (`empty_queue_days`, `stale_after_days`, `auto_after_days`) aceitam de 1 a 3650. Valores fora disso são recusados com uma mensagem, em vez de arquivar tudo (0 ou negativo) ou estourar o cálculo de datas.

Dias: `mon tue wed thu fri sat sun`; a leitura também aceita `seg ter qua qui sex sab dom`.

## `board.yaml`

```yaml
schema_version: 1
id: conteudo
name: "Conteúdo"
kind: content          # content | project | generic
color: "#7a2e3b"
columns:
  - { id: idea, name: "Ideia", role: inbox }
  - { id: draft, name: "Rascunho" }
  - { id: doing, name: "Fazendo", limit: 3 }     # limite de cartões em andamento
  - { id: scheduled, name: "Agendado", role: scheduled }
  - { id: published, name: "Publicado", role: done }
```

O `role` diz às regras o que a coluna significa: `inbox` recebe as capturas, `scheduled` recebe os cartões agendados e `done` marca concluídos (nunca atrasados). Sem `role`, a primeira coluna vale como entrada e a última como concluída.

## Cartão

```markdown
---
id: 01JBX3K9Q2AB…        # obrigatório, único (ULID)
title: "Título"          # obrigatório
kind: post               # post | article | task | note
column: scheduled
rank: "a3"               # ordem dentro da coluna (texto, comparação lexicográfica)
tags: [rust, divulgacao]
network: linkedin        # só cartões de conteúdo; uma rede por cartão
publication:
  at: 2026-11-04T09:00:00-03:00
  status: scheduled      # planned | scheduled | published | skipped
  url: null
  published_at: null
due: 2026-11-20          # ou 2026-11-20T18:00:00-03:00
reminders: ["-1h", "0"]  # opcional; senão, os padrões do pauta.yaml
links:
  board: dungeon-meister # projeto relacionado
  siblings: [01J…]       # o mesmo post em outras redes
  article: 01J…          # artigo divulgado por este post
art:                     # arte pedida ao Claude Design (opcional)
  status: requested      # requested | ready
  format: carousel       # id de um formato da rede
  requested_at: 2026-10-28T20:30:00-03:00
metrics:                 # números copiados das estatísticas da rede (opcional)
  impressions: 1200
  reactions: 34
  comments: 5
  shares: 2
  saves: 1
  clicks: 12
  recorded: 2026-11-10
column_since: { column: draft, at: 2026-10-28T20:15:00-03:00 }   # quando entrou na coluna atual
created: 2026-10-28T20:15:00-03:00
updated: 2026-10-28T21:02:00-03:00
---

Texto do post. Seções `## LinkedIn` / `## Instagram` têm prioridade
ao copiar; `## Notas` e `## Arte` (a direção visual para o Claude Design) nunca são copiadas.

- [ ] itens assim formam a lista de verificação
```

## Evento

```markdown
---
id: 01JBY7P0D5EF…
title: "Entrega do capítulo 3 do TCC2"
kind: deadline           # deadline | event | milestone | launch
date: 2026-12-05         # dia inteiro…
# start: 2026-12-05T14:00:00-03:00   …ou com horário
# end:   2026-12-05T16:00:00-03:00
reminders: ["-7d", "-1d", "09:00"]
links:
  board: tcc2
tags: [tcc2]
---

Detalhes.
```

## Regras

- **Datas:** com horário, sempre com fuso (`-03:00`); sem horário, `AAAA-MM-DD`. Ao receber `AAAA-MM-DDTHH:MM` sem fuso, o Pauta interpreta no `timezone` do `pauta.yaml` e grava com o fuso.
- **Lembretes:** `-15m`, `-1h`, `-7d`, `-1w` e `+…` são relativos ao item; `0` é na hora; `HH:MM` é nesse horário do dia do item, e também define o horário dos lembretes `-Nd` em itens de dia inteiro (padrão 09:00).
- **Preservação:** campos desconhecidos, comentários e a ordem das chaves são mantidos; editar um campo reescreve só aquele bloco do cabeçalho. Quebras de linha CRLF são mantidas.
- **Arte:** os arquivos exportados ficam em `boards/<quadro>/assets/<id do cartão>/` (a pasta é a verdade: o estado `ready` só é aceito com ao menos um arquivo ali). `column_since` só vale enquanto o `column` do cartão for o mesmo; se a coluna for mudada à mão, o registro é ignorado e o "parado" volta a usar `updated`. `metrics` só é aceito em cartões com rede e publicados.
- **Relógio de teste:** a variável `PAUTA_NOW` (RFC 3339) fixa a data e hora na linha de comando e nos builds de depuração do app; builds de release a ignoram.
- **BOM:** um BOM UTF-8 no início de um cartão é aceito e preservado.
- **Arquivos inválidos** aparecem na tela Hoje com o erro e nunca são regravados automaticamente.
- **Migração:** se `schema_version` mudar, o Pauta copia a pasta para `.pauta/backup-v<N>-<data>/` antes de migrar. Uma pasta com versão mais nova do que a suportada é recusada.

## Importação de resultados por CSV

A tela Ritmo importa números de um CSV exportado das estatísticas da rede e grava o mesmo bloco `metrics` descrito acima. Regras:

- O separador (`,`, `;` ou tabulação) e o BOM são detectados. É obrigatória uma coluna com o link do post (`URL`, `Link`, `Permalink`, `Link da publicação`…) e ao menos uma de resultados (`Impressões`/`Impressions`/`Views`/`Alcance`, `Reações`/`Likes`/`Curtidas`, `Comentários`, `Compartilhamentos`/`Reposts`, `Salvamentos`/`Saves`, `Cliques`). Maiúsculas, acentos e pontuação dos cabeçalhos não importam.
- A linha é ligada ao cartão **publicado** cujo `publication.url` é o mesmo, ignorando `https://`, `www.`, `?parâmetros`, `#âncora` e a barra final. Linhas sem cartão aparecem na prévia e nada é criado.
- Só números inteiros (`1.250`, `1,250` e `1 250` valem `1250`). `1,5 mil` ou `12%` são listados como ignorados, nunca adivinhados.
- Célula vazia mantém o número que o cartão já tinha; os demais substituem. `recorded` passa a ser a data da importação.
