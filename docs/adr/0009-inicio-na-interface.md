# ADR 0009: A tela Início é composta na interface

- **Data:** 2026-11-05
- **Estado:** aceita

## Contexto

O Início passa a ser a primeira tela, com sete widgets: Brief, Postar hoje, Avisos, Agenda, Ritmo, Ideias e Projetos (marco 7; especificação em `docs/superpowers/specs/2026-11-05-inicio-design.md`). Cada um mistura dados que o backend já entrega: `get_index` (quadros e cartões), `get_today` (posts do dia, atrasados, cadência) e `get_agenda` (próximos 7 dias). A pergunta era onde montar a tela: em `pauta-core`, com um comando novo como `get_home`, ou na interface.

## Decisão

A composição fica na interface, em funções puras de `ui/src/lib/home/*.ts` com testes no Vitest, e nenhum comando novo foi criado. Não há código Rust novo.

**Por quê:**

- Tudo o que a tela mostra já vem de comandos existentes. Um `get_home` só reembalaria esses dados.
- O MCP não precisa da tela. O Claude já consulta o dia por `pauta_today` e `pauta_list_agenda`, e as regras do Início (saudação, próximo compromisso, quais avisos aparecem, qual é a "próxima ação" de um projeto) existem para o olhar do autor, não para quem opera por ferramentas.
- As regras mudam com a tela. Mantê-las ao lado dos componentes deixa ajustar o texto e os limites sem passar por Rust, `pauta-mcp` e os testes de integração.

**O custo:** se o Claude um dia precisar de uma dessas regras (por exemplo, "o que está parado nos projetos"), ela terá de ser reescrita em Rust, e as duas versões precisarão concordar. Hoje isso não acontece. Se acontecer, a regra migra para `pauta-core` e a interface passa a chamá-la, com os testes de `ui/src/lib/home` servindo de tabela de casos.

## Exceção: o dia de um carimbo de tempo

Até aqui a interface não fazia conta de fuso: o backend entrega `Today.date` e `Today.time` já no fuso de `pauta.yaml` (ADR 0008). O widget Projetos precisa do **dia** em que um cartão foi atualizado, e `updated`, `column_since` e `created` chegam como instantes em UTC (`2026-11-04T01:30:00Z` é 3 de novembro em Recife). Comparar o texto da data erraria o "atualizado hoje" perto da meia-noite.

`localDay(timestamp, timezone)` converte com `Intl.DateTimeFormat("en-CA", { timeZone })`, usando `config.timezone`, e **só** o Projetos a usa. Ela é a única conta de fuso na interface. A alternativa, expor o dia local de cada cartão no backend, mudaria o formato dos dados por um detalhe de exibição. O risco é a base de fusos do WebKit divergir da do `chrono-tz` em zonas raras; para os fusos de uso real isso não aparece, e o teste de `localDay` cobre o caso de virada de dia.

## Volta ao Início em um novo dia

A decisão de voltar ao Início ao abrir a janela em um novo dia também é uma função pura, `shouldReturnHome(seen, date, busy)`. O dia vem de `Today.date`, relido ao receber o foco, e não do relógio do navegador, pelo mesmo motivo do ADR 0008. O estado `seenDate` fica no `store.svelte.ts`, fora de `app`, porque nenhuma tela o lê.
