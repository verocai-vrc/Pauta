# Tela Início (marco 7)

Data: 2026-11-05. Estado: aprovado pelo autor na entrevista; implementação direta, sem paradas.

## 1. Objetivo

Abrir o app e ver de relance o que precisa do autor hoje, sem rolar a página. O Início passa a ser a primeira tela, com sete widgets fixos. O Hoje deixa a barra lateral e vira a tela de detalhe.

**Fora do escopo:** mudar a disposição dos widgets, escondê-los ou reordená-los; mudar o formato dos dados; mudar o conector MCP; código Rust novo.

O protótipo fica em `.superpowers/sdd/2026-11-05-inicio/mock/` (fora do Git).

## 2. Decisões do autor (2026-11-05)

- O Início substitui o Hoje na ordem das telas. O Hoje continua acessível pelo Calendário e pelos "Ver mais".
- Widgets: Brief, Postar hoje, Avisos, Agenda, Ritmo, Ideias e Projetos. Produção e Resultados ficam de fora por enquanto.
- Disposição fixa, em três colunas por tema, com o Brief em cima.
- Todos os widgets são cartões.
- Listas com limite e "Ver mais". Nada de rolagem dentro dos widgets.
- O Brief traz saudação, data, resumo do dia e o próximo compromisso.
- Só o próximo post tem as ações; os demais são linhas.
- Avisos fica no lugar quando não há nada, dizendo "Tudo em dia."
- A Agenda mostra posts, prazos e eventos dos próximos 7 dias, por dia.
- Avisos inclui posts dos próximos 3 dias que não estão prontos.
- Ideias usa só os quadros de conteúdo.
- Projetos mostra, para cada projeto: quando foi atualizado, a última ação e a próxima ação planejada.
- A última ação vem dos próprios cartões. A próxima ação é o prazo mais próximo; sem datas, o primeiro cartão ativo.
- Cartão ativo é o que está entre a caixa de entrada e a coluna de concluídos.
- O Calendário ganha a aba Hoje, e o botão que leva a grade à data atual vira "Ir para hoje".
- O app abre no Início ao iniciar e na primeira vez que a janela é aberta em um novo dia.
- Tamanhos que importam: a janela padrão (1280×820) e maximizada em tela grande (1920×1080 ou maior).

## 3. Navegação

- `View` ganha `"home"`, que passa a ser o valor inicial, e perde `"today"`.
- A barra lateral fica: Início (`house`, `Alt+1`), Quadros, Calendário, Ritmo e Configurações. O ícone `house`, da Lucide, entra em `Icon.svelte` no lugar de `sun`, que deixa de ser usado.
- `CalendarMode` passa para `store.svelte.ts` como `app.calendarMode`, com os valores `"today" | "month" | "week" | "agenda"` e padrão `"month"`.
- `openToday()` no store faz `app.calendarMode = "today"` e `app.view = "calendar"`. O evento `pauta://navigate` com a carga `"today"` chama essa função, e assim a bandeja e a notificação continuam levando ao Hoje sem mudança em Rust. Qualquer outra carga muda `app.view` como hoje.
- A paleta de comandos ganha "Início", e "Hoje" passa a chamar `openToday()`.

### 3.1 Volta ao Início

O app guarda `seenDate`, o último dia em que a janela esteve em uso.

- Ao receber o foco ou ficar visível, atualiza `app.today` e compara. Se o dia mudou, volta ao Início. Depois disso, `seenDate` recebe o dia atual.
- Com um editor, modal (qualquer `dialog` aberto), paleta ou lixeira aberta, não navega, para não tirar o autor do que está fazendo. `seenDate` é atualizado do mesmo jeito.
- Um pedido explícito vence: `pauta://navigate`, da bandeja ou da notificação, cancela a volta dessa abertura, em qualquer ordem de chegada.
- A cada minuto, se a janela tiver o foco, `seenDate` acompanha o dia. Quem atravessa a meia-noite trabalhando não é levado ao Início ao trocar de janela.

A decisão fica na função pura `shouldReturnHome(seen, date, busy)`.

## 4. Disposição

```
┌──────────────────────────────────────────────┐
│ Brief                                        │
├──────────────┬───────────────┬───────────────┤
│ Postar hoje  │ Agenda        │ Projetos      │
│ Avisos       │               │               │
│ Ritmo        │               │ Ideias        │
└──────────────┴───────────────┴───────────────┘
```

As três colunas são agir agora, tempo e trabalho em andamento. Fundo liso `--bg`, sem pauta: a pauta atrás de cartões vira ruído. A identidade fica na tipografia, nos rótulos sobre fio escuro e na linha de margem dentro do Brief.

- Grade de três colunas iguais, com `max-width: 1680px`, centralizada. O Início não entra em `paperViews`.
- `min-height: 100%`: as colunas enchem a altura da janela e as bases dos cartões se alinham. Se o conteúdo não couber, a página rola.
- Dentro de cada coluna, a sobra de altura é dividida entre os cartões (`flex: 1 1 auto`). Ritmo não cresce.
- **Janela grande** (`min-width: 1600px` e `min-height: 960px`): o Início redefine os tokens localmente, para o texto acompanhar a tela: `--fs-xs: 13px`, `--fs-sm: 15px`, `--fs-md: 16px`, `--fs-base: 17px`, `--fs-lg: 21px`, `--fs-display: 54px`, `--sp-2: 10px`, `--sp-3: 16px`, `--sp-4: 22px`.
- **Janela estreita** (abaixo de 1100px): duas colunas, agir e tempo. A coluna de trabalho desce e ocupa a largura toda, com Projetos e Ideias lado a lado. A página rola. A janela mínima (720×480) não pode ter rolagem horizontal.

### 4.1 Limites

Cada widget mostra no máximo este número de itens. O que sobra aparece como "+N · Ver mais".

| Widget | Janela normal | Altura de 960px ou mais |
| --- | --- | --- |
| Postar hoje (linhas além do próximo post) | 2 | 2 |
| Avisos | 5 | 6 |
| Agenda (itens) | 12 | 14 |
| Ritmo (redes) | 2 | 3 |
| Ideias | 3 | 4 |
| Projetos | 4 | 5 |

Os limites valem para o dia mais cheio caber sem rolagem em 1280×820 e em 1920×1000, a área útil de uma janela maximizada em 1920×1080, descontadas a barra do sistema e a do título.

Quem decide é `MediaQuery("(min-height: 960px)")` de `svelte/reactivity`.

## 5. Widgets

Todos usam `Widget.svelte`: uma `section` com `aria-labelledby`, cartão com `--surface`, borda, `--radius-md` e `--shadow-1`. O cabeçalho tem o título em versalete sobre fio escuro (como `.section-label`), a contagem, que some quando é zero, e, à direita, "Ver mais" com `arrow-right`. O nome de "Ver mais" para leitores de tela é o texto visível seguido do nome do widget, em `.sr-only`, para o rótulo visível continuar dentro do nome acessível (WCAG 2.5.3).

Toda linha clicável é um botão `ghost` que ocupa a linha, com `hover` em `--surface-2`. Títulos ficam em uma linha, com reticências; o título inteiro vai no atributo `title`.

### 5.1 Brief

Cartão de largura total, em duas partes separadas por uma linha vertical `--margin-line`, que lembra a margem do caderno. A linha vai de cima a baixo do cartão e fica sobre a borda da coluna do meio.

- À esquerda: `.kicker` com `longDate(date)` e, abaixo, um `h1.display` com a saudação. É o único `h1` da tela.
- À direita: `.dek` com `todaySummary(today)` e, abaixo, a linha do próximo compromisso, em `--fs-sm`, peso 600, em uma linha só. Sem próximo compromisso, a linha some.

**Saudação**, por `today.time`: de 05:00 a 11:59, "Bom dia."; de 12:00 a 17:59, "Boa tarde."; nos demais horários, "Boa noite."

**Próximo compromisso:** o primeiro item da agenda, por data e hora, que não esteja feito e ainda não tenha passado. Item de dia inteiro vale até o fim do dia e, no mesmo dia, vem depois dos que têm hora.

### 5.2 Postar hoje

Lista `today.posts`. A contagem é a dos posts pendentes. "Ver mais" abre o Hoje.

- **Próximo post:** o primeiro pendente. Mostra a hora em serifa, a rede, o título em serifa (até duas linhas), os problemas de prontidão em uma linha curta ("falta imagem · sem texto", com `triangle-alert` em `--warning`) e `PostActions compact`. O texto longo de `Problems` não cabe em uma coluna estreita; ele continua no Hoje e no editor. Neste cartão, os botões de `PostActions` não têm ícones, como no protótipo. Uma linha fina separa o próximo post dos demais. Se o horário passou, a hora fica em `--danger` e aparece o selo "atrasado".
- **Demais:** os outros pendentes e depois os publicados, cada um em uma linha com hora, rede e título. O publicado leva `check` e texto apagado.
- **Sem posts hoje:** "Nada para postar hoje." e, abaixo, o próximo post da agenda ("Próximo post: {quando}"), se houver.
- **Tudo publicado:** "Tudo publicado hoje." e as linhas dos publicados.

### 5.3 Avisos

A lista é montada nesta ordem:

| # | Aviso | Ícone | Ao clicar |
| --- | --- | --- | --- |
| 1 | Arquivos inválidos, uma linha com o total | `triangle-alert` em `--danger` | Hoje |
| 2 | Cada item de `today.overdue`, com o título e o dia à direita | `triangle-alert` em `--danger` | Abre o cartão |
| 3 | Cada post pendente até `date + 3` com problema de prontidão | `triangle-alert` em `--warning` | Abre o cartão |
| 4 | Cada fila vazia | `triangle-alert` em `--warning` | Hoje |
| 5 | Cartões parados, uma linha com o total. O backend manda no máximo 10; com 10, o texto é "10 ou mais cartões parados" | `clock` apagado | Hoje |

- Em 2, sai o post de hoje que está em `today.posts` e pendente, porque Postar hoje já o mostra. Prazos continuam, mesmo que sejam do mesmo cartão.
- Em 3, sai o próximo post de Postar hoje, porque ele já mostra os seus problemas. Se houver mais de um problema, a linha mostra o primeiro.
- O contador é o total de linhas. Sem nenhum aviso, o cartão fica no lugar com `check` em `--ok` e "Tudo em dia."
- "Ver mais" abre o Hoje.

### 5.4 Agenda

`api.agenda(date, date + 7)`, buscada uma vez em `Home.svelte` e entregue a Brief, Postar hoje, Avisos e Agenda. Refaz a busca quando `app.version` muda.

- Saem os itens feitos e os posts de hoje, que estão em Postar hoje.
- Evento que começou antes de hoje entra em Hoje.
- Agrupa por dia, com o rótulo de `relativeDay` e inicial maiúscula. O rótulo de hoje fica em `--accent`.
- Cada linha: hora (vazia em item de dia inteiro), marca do tipo (`flag` para prazo, `diamond` para evento, ponto na cor da rede para post) e título.
- Sem itens: "Nada nos próximos 7 dias." Enquanto a agenda não chegou, o widget não diz que está vazio.
- "Ver mais" abre o Calendário em `agenda`.

### 5.5 Ritmo

Cada rede de `today.cadence` em uma linha: rede, pontos e "1 de 2 nesta semana". Os pontos têm o mesmo significado do `CadenceMeter`, que ganha a propriedade `inline`. Nesse modo, são no máximo seis pontos, em uma linha só, seguidos de "…"; com meta acima de cinco, os pontos ficam menores. O número exato está no texto. Sem metas: "Nenhuma meta definida." "Ver mais" abre o Ritmo.

### 5.6 Ideias

- Campo "Nova ideia…" com botão `plus` ("Adicionar ideia"). Enter também salva, com `api.createCard(board, { title, top: true })`, e mostra o aviso "Ideia guardada."
- O quadro é o primeiro de tipo `content`, o mesmo que a janela de captura escolhe. Sem esse quadro, o campo some e aparece "Crie um quadro de conteúdo para guardar ideias."
- A lista traz os cartões da coluna `inbox` dos quadros de conteúdo, do mais novo ao mais antigo, por `created`. Sem ideias: "Nenhuma ideia guardada."
- A contagem é o total de ideias. "Ver mais" abre esse quadro.

### 5.7 Projetos

Os quadros que não são de conteúdo, do atualizado há menos tempo ao atualizado há mais tempo; os sem cartões vão para o fim, na ordem dos quadros. A contagem é o total de quadros. "Ver mais" abre os Quadros. Sem quadros: "Nenhum quadro de projeto."

Cada projeto mostra o nome (botão que abre o quadro) e "atualizado hoje", "atualizado ontem" ou "atualizado em 4 nov". Abaixo, duas linhas com rótulo: Último e Próximo. Quadro sem cartões mostra só "Nenhum cartão ainda".

**Momento de um cartão:** o maior entre `updated`, `column_since` e `created`. Cartão sem nenhum dos três não entra na conta. O dia é calculado no fuso de `config.timezone`.

**Último:** o cartão com o momento mais recente, descrito assim, nesta ordem:

| Condição | Texto |
| --- | --- |
| `created` igual a `updated` | Criado: {título} |
| Coluna de concluídos e `column_since` igual a `updated` | Concluído: {título} |
| `column_since` igual a `updated` | Movido para {coluna}: {título} |
| Os demais casos | Editado: {título} |

**Próximo:** a primeira opção que existir:

1. O item datado mais próximo do projeto: o prazo de um cartão não concluído, inclusive atrasado, ou um evento em andamento ou de hoje em diante com `links.board` igual ao quadro. Texto: "{título} · {dia}".
2. O primeiro cartão ativo: colunas da mais avançada para a menos avançada, sem caixa de entrada e sem concluídos; dentro da coluna, por `rank`.
3. O primeiro cartão da caixa de entrada, por `rank`.
4. "Nada planejado".

Clicar em Último ou Próximo abre o cartão ou o evento.

## 6. Hoje no Calendário

- Os botões de visão ficam: Hoje, Mês, Semana e Agenda. `Calendar.svelte` passa a usar `app.calendarMode`.
- Na aba Hoje somem as setas, o botão de data, o período e os filtros. "Novo evento" continua. Abaixo do cabeçalho entra `Today.svelte`, sem mudar o conteúdo.
- O botão que leva a grade à data atual passa a se chamar "Ir para hoje".
- O título de `Today.svelte` vira `h2`, e as seções, `h3`, com a mesma aparência, porque "Calendário" é o `h1` da página.
- A pauta continua nessa aba: `main` recebe `.paper` também nesse caso. A linha de margem começa abaixo do cabeçalho do Calendário.

## 7. Arquitetura

A tela é composta na interface, a partir do que o backend já entrega (`get_index`, `get_today`, `get_agenda`). Nenhum comando novo. A decisão fica registrada no ADR 0009.

| Arquivo | Responsabilidade |
| --- | --- |
| `ui/src/components/Home.svelte` | Grade, busca da agenda e limites |
| `ui/src/components/home/Widget.svelte` | Cartão, cabeçalho e "Ver mais" |
| `ui/src/components/home/*Widget.svelte` | Um arquivo por widget |
| `ui/src/lib/home/*.ts` | Funções puras por widget, com testes em Vitest |
| `ui/src/lib/readiness.ts` | Texto dos problemas de prontidão, usado também por `Problems.svelte` |

`Card` em `api.ts` ganha `column_since: string | null`, que o backend já envia.

## 8. Textos

| Chave | Texto |
| --- | --- |
| `nav.home` | Início |
| `cal.go_today` | Ir para hoje |
| `home.greeting_morning` | Bom dia. |
| `home.greeting_afternoon` | Boa tarde. |
| `home.greeting_evening` | Boa noite. |
| `home.next_post_today` | Próximo: {time} no {net}. |
| `home.next_post` | Próximo: {day}, {time} no {net}. |
| `home.next_item` | Próximo: {title}, {day}. |
| `home.next_item_time` | Próximo: {title}, {day} às {time}. |
| `home.see_more` | Ver mais |
| `home.see_more_n` | +{n} · Ver mais |
| `home.warnings` | Avisos |
| `home.agenda` | Agenda |
| `home.rhythm` | Ritmo |
| `home.ideas` | Ideias |
| `home.projects` | Projetos |
| `home.late` | atrasado |
| `home.all_posted` | Tudo publicado hoje. |
| `home.next_scheduled` | Próximo post: {when} |
| `home.all_clear` | Tudo em dia. |
| `home.warn_invalid_one` | {n} arquivo com erro |
| `home.warn_invalid_other` | {n} arquivos com erro |
| `home.warn_not_ready` | Post de {day}: {problem} |
| `home.warn_queue` | {net}: nada agendado em {days} dias |
| `home.warn_stale_one` | {n} cartão parado |
| `home.warn_stale_other` | {n} cartões parados |
| `home.warn_stale_many` | {n} ou mais cartões parados |
| `home.problem_no_text` | sem texto |
| `home.problem_over_limit` | texto acima do limite |
| `home.problem_media_missing` | falta imagem |
| `home.problem_art_pending_waiting` | arte a caminho |
| `home.problem_art_pending_files` | marcar a arte como pronta |
| `home.agenda_empty` | Nada nos próximos 7 dias. |
| `home.no_goals` | Nenhuma meta definida. |
| `home.idea_placeholder` | Nova ideia… |
| `home.idea_add` | Adicionar ideia |
| `home.idea_saved` | Ideia guardada. |
| `home.no_ideas` | Nenhuma ideia guardada. |
| `home.no_content_board` | Crie um quadro de conteúdo para guardar ideias. |
| `home.no_projects` | Nenhum quadro de projeto. |
| `home.proj_updated_today` | atualizado hoje |
| `home.proj_updated_yesterday` | atualizado ontem |
| `home.proj_updated_on` | atualizado em {date} |
| `home.proj_last` | Último |
| `home.proj_next` | Próximo |
| `home.proj_created` | Criado: {title} |
| `home.proj_done` | Concluído: {title} |
| `home.proj_moved` | Movido para {column}: {title} |
| `home.proj_edited` | Editado: {title} |
| `home.proj_dated` | {title} · {day} |
| `home.proj_nothing` | Nada planejado |
| `home.proj_empty` | Nenhum cartão ainda |

## 9. Testes

- **Vitest**, nas funções puras: limites da saudação, próximo compromisso, ordem e deduplicação dos avisos, janela de 3 dias, agrupamento da agenda, ordem das ideias, verbos de Último, as quatro regras de Próximo, ordem dos projetos e `shouldReturnHome`.
- **`screens.mjs`**, nos dois temas, com axe WCAG 2.2 AA: `home`, `home-full`, `home-empty`, `calendar-today` e `calendar-today-full`.
- **Comportamento em `screens.mjs`:**
  - O app abre no Início e `Alt+1` volta a ele.
  - `home-full` não rola em 1280×820, em 1920×1000 nem em 1920×1080, e os títulos não vazam dos cartões.
  - Não há rolagem horizontal em 1100, 900 e 720px.
  - "+N · Ver mais" mostra o número certo, e cada "Ver mais" chega ao destino da seção 5.
  - A aba Hoje esconde as setas e os filtros.
  - Salvar uma ideia chama `create_card` e limpa o campo.
  - No foco em um novo dia, volta ao Início; no mesmo dia, não volta; com o editor aberto, não volta.
- **`tauri.mjs`:** os passos que partem do Hoje passam a partir do Início.
- **WebKitGTK:** a mesma conferência visual do redesenho, com o app real em Xvfb, nos dois temas e nos dois tamanhos.

## 10. Documentação

- `Design Document Pauta.md`: seções 8.2, 12.1 e 12.2, e o marco 7 na seção 16.
- `README.md`: estado e atalhos.
- ADR 0009: por que a tela é composta na interface.

## 11. Critérios de aceite

1. O app abre no Início, e `Alt+1` leva a ele.
2. Na janela padrão, com o dia mais cheio do fixture, os sete widgets aparecem inteiros, sem rolagem.
3. Maximizada em 1920×1080, a tela preenche a janela com texto maior e mostra mais itens.
4. Todo "Ver mais" leva à tela de detalhe certa, e o Hoje continua completo na aba do Calendário.
5. Marcar como publicado, copiar o texto e guardar uma ideia funcionam a partir do Início.
6. Em um novo dia, a primeira abertura da janela leva ao Início.
7. O conteúdo passa no axe nos dois temas, e o app real em WebKitGTK confere com as capturas.
