# Redesenho visual: Caderno editorial

Data: 2026-11-05. Estado: aguardando a revisão do autor.

## 1. Objetivo

O autor achou a interface "genérica e inacabada". Este trabalho dá ao Pauta uma identidade própria, a do **Caderno editorial**: papel, pauta, margem de caderno e títulos com serifa. O nome do app significa tanto agenda quanto papel pautado, e é isso que o distingue de qualquer outro kanban.

**O comportamento não muda.** Nenhum recurso, atalho, dado ou comando entra ou sai. O layout só muda onde as seções 4 e 5 disserem.

Decisões do autor:

- Direção visual: A, Caderno editorial (2026-10-29).
- Tema escuro: Grafite, carvão neutro com o acento rosado (2026-10-29).
- Abordagem: ajustar os tokens primeiro, com poucas peças compartilhadas (2026-11-04).
- Atalhos visíveis na barra lateral: emprestados da direção B, sem mudar a direção A (2026-11-05).

Os protótipos aprovados estão em `.superpowers/brainstorm/*/content/` (fora do Git).

## 2. Tokens

Continuam os nomes atuais (`--bg`, `--surface`, `--text-muted`...), porque os componentes já os usam quase sempre: não há tamanho de fonte literal e só há três cores fixas. Muda o valor, não o nome. As cores fixas viram tokens: o fundo de `Modal.svelte` e de `CardEditor.svelte` passam a usar `--scrim`, e o `#fff` de `Toasts.svelte` passa a usar `--danger-text`. A cor padrão de quadro em `Boards.svelte` (`#7a2e3b`) é dado, não estilo, e fica.

### 2.1 Cores

| Token | Claro | Grafite |
| --- | --- | --- |
| `--bg` | `#faf7f2` | `#18191b` |
| `--sidebar` (novo) | `#f3eee6` | `#121315` |
| `--surface` | `#ffffff` | `#222326` |
| `--surface-2` | `#f3eee6` | `#2a2b2f` |
| `--surface-3` | `#e9e2d6` | `#35373b` |
| `--border` | `#e6ddd0` | `#35373b` |
| `--text` | `#2a211d` | `#ececea` |
| `--text-muted` | `#6f6156` | `#a6a6a2` |
| `--accent` | `#7a2e3b` | `#e3909c` |
| `--accent-text` | `#ffffff` | `#22100f` |
| `--accent-soft` | `#f3e3e5` | `#3a2629` |
| `--warning` | `#7d5100` | `#f0c26a` |
| `--warning-soft` | `#fbf1dc` | `#33291a` |
| `--danger` | `#a3271f` | `#ff9a8f` |
| `--danger-soft` | `#f9e4e1` | `#3b2220` |
| `--danger-text` (novo) | `#ffffff` | `#2a0f0c` |
| `--ok` | `#2e6b3a` | `#95cf9f` |
| `--ok-soft` | `#e3f0e2` | `#1f2e22` |
| `--c-linkedin` | `#0a5fb4` | `#6aaef0` |
| `--c-instagram` | `#a82a72` | `#ec86c0` |
| `--c-due` | `#7d5100` | `#f0c26a` |
| `--c-event` | `#2e5a7a` | `#7fb3d5` |
| `--rule` (novo) | `#efe8dd` | `#1f2023` |
| `--margin-line` (novo) | `rgb(122 46 59 / 0.3)` | `rgb(227 144 156 / 0.26)` |
| `--head-rule` (novo) | `#2a211d` | `#c4c4c0` |
| `--scrim` (novo) | `rgb(0 0 0 / 0.35)` | `rgb(0 0 0 / 0.55)` |

`--focus` continua azul (`#1d5fd1` e `#7fb0ff`), para o foco do teclado nunca se confundir com o acento.

Estes valores diferem dos protótipos em cinco pontos, para passar no WCAG AA: texto apagado, aviso, LinkedIn e Instagram no claro, e texto apagado no Grafite. Foram medidos 60 pares de texto e fundo por tema, com mínimo de 4,63 no claro e 4,88 no Grafite.

### 2.2 Tipografia

- `--font-serif` (novo): Fraunces, com tamanho óptico e itálico. Títulos, títulos de cartão e a frase de resumo.
- `--font-ui`: Source Sans 3, com itálico. Todo o resto.
- `--font-mono`: sem mudança.
- `--fs-display` (novo): `2.375rem`, só para o título das telas de leitura.

As fontes vêm dos pacotes `@fontsource-variable/fraunces` e `@fontsource-variable/source-sans-3` (OFL-1.1), importando `opsz.css`, `opsz-italic.css`, `wght.css` e `wght-italic.css`. O app roda sem rede, e a CSP (`default-src 'self'`) bloquearia fontes externas. Os arquivos latinos carregados somam cerca de 200 KB. Nenhum `woff2` desses pacotes tem menos de 4 KB, então o Vite não os embute como `data:`, que a CSP bloquearia, e `vite.config.ts` não muda.

### 2.3 Forma

Raios `--radius-sm` 4px, `--radius-md` 6px e `--radius-lg` 8px. Sombras mais suaves, como nos protótipos: `--shadow-1` é `0 1px 0 var(--border), 0 2px 6px rgb(60 40 20 / 0.05)` no claro e `0 1px 0 #0d0e0f, 0 2px 8px rgb(0 0 0 / 0.3)` no Grafite. `--shadow-2` não muda.

### 2.4 Pauta do papel

O papel pautado é o token `--ruling`, uma `repeating-linear-gradient` de 32px com `--rule`. A margem de caderno é um pseudoelemento com `--margin-line`. Ambos são decoração e não recebem texto próprio.

## 3. Ícones

`ui/src/components/Icon.svelte` recebe `name` e guarda os caminhos dos ícones Lucide em linha, sem dependência em tempo de execução, com o aviso da licença ISC no arquivo. O SVG é sempre `aria-hidden="true"`: o nome acessível continua vindo do texto ou do `aria-label` do botão.

| Glifo atual | Ícone | Glifo atual | Ícone |
| --- | --- | --- | --- |
| ☀ | `sun` | ✕ | `x` |
| ▦ | `columns-3` | ✓ | `check` |
| ▤ | `calendar` | ☑ | `square-check` |
| ▥ | `chart-column` | ⚠ | `triangle-alert` |
| ⚙ | `settings` | ⚑ | `flag` |
| ＋ | `plus` | ★ | `star` |
| ⌕ | `search` | ▲ | `rocket` |
| ⌚ | `clock` | ◆ | `diamond` |
| ⧉ | `copy` | 🎨 | `palette` |
| ↗ | `external-link` | ⇄ | `arrow-left-right` |
| 🗀 | `folder` | ↩ | `undo-2` |
| 🗑 | `trash-2` | ⇩ | `download` |
| ◀ ▶ | `chevron-left`, `chevron-right` | ← → ↑ ↓ | `arrow-left`, `arrow-right`, `arrow-up`, `arrow-down` |

As setas viram ícone nos botões: `↑` e `↓` em `Boards.svelte` e `→` em `Today.svelte`. Continuam como texto as que fazem parte de uma frase ou do nome de um atalho: `←` no mapeamento de colunas de `MetricsImport.svelte:61` e `Alt+← → ↑ ↓` em `Settings.svelte:258`. Os glifos dentro de `pt-BR.ts` também continuam como texto. `🚀` em `lib.test.ts` é dado de teste e fica.

## 4. Atalhos visíveis

Cada item da barra lateral mostra o seu atalho à direita, num `<kbd>` discreto: `Alt+1` a `Alt+5`, `Ctrl+K` em Buscar e `Ctrl+Shift+I` em Capturar ideia.

- O `<kbd>` é `aria-hidden="true"`. O botão recebe `aria-keyshortcuts`, então o leitor de tela anuncia o atalho sem repetir o texto.
- O atributo `title` continua.
- Abaixo de 860px, quando a barra lateral mostra só os ícones, os atalhos somem junto com os rótulos.
- A barra lateral passa de 200px para 216px, para caber o atalho mais longo. Se `Ctrl+Shift+I` não couber em uma linha ao lado de "Capturar ideia", ele vai para uma segunda linha pequena dentro do botão. O texto nunca é cortado.
- `aria-keyshortcuts` e o texto vêm da mesma lista de `App.svelte`, para não divergirem.

## 5. Telas

**Onde o caderno aparece.** A pauta fica nas telas de leitura: Hoje, Ritmo e Configurações. Não fica no Calendário, no editor nem nos quadros, porque linhas atrás de uma grade ou de um formulário só fazem ruído. A margem de caderno fica só em Hoje, a única tela com uma divisão real entre data e conteúdo.

| Tela | Mudança |
| --- | --- |
| Barra lateral | Fundo `--sidebar`, marca em Fraunces com a linha "caderno de publicação" e atalhos (seção 4). O item ativo vira um cartão `--surface` com borda de acento à esquerda. "Capturar ideia" vira o botão primário. |
| Hoje | Como no protótipo: data por extenso acima de "Hoje", frase de resumo em itálico e horários à esquerda da margem. O que pede ação é cartão; o que é só para saber é linha pautada. Posts feitos ficam tracejados e apagados. |
| Quadros | Colunas planas, com rótulo em caixa alta sobre um fio `--head-rule`, sem as caixas cinzas. Título do cartão em Fraunces. Chips sem fundo, exceto os de estado (`ok`, `warn`, `danger`). |
| Calendário | Mesma estrutura em mês, semana e agenda. Fios finos entre as células no lugar das caixas brancas, e título em Fraunces. |
| Editor do cartão | Título em Fraunces. Publicação e Arte viram seções com fio, não caixas aninhadas. |
| Ritmo | Painéis viram seções com fio e títulos em Fraunces. |
| Configurações, Primeira execução, modais, paleta, avisos, lixeira e captura | Recebem tokens e ícones, e os títulos passam para Fraunces. Sem mudança de layout. |

### 5.1 Correções junto com o redesenho

- **Gráfico do Ritmo:** os rótulos abaixo das barras semanais são cortados (`overflow: hidden` em `StackedBars.svelte`). Os rótulos passam a aparecer inteiros.
- **Aviso de erro no escuro:** o texto branco sobre `--danger` tem contraste de 2,28:1 hoje. Com `--danger-text`, passa a 8,76:1 no Grafite e 7,32:1 no claro.
- **Calendário:** posts publicados deixam de ser riscados (`line-through` em `Calendar.svelte` e `WeekGrid.svelte`) e passam a aparecer apagados, com o ícone `check`.

## 6. Textos novos

Todos passam pelas chaves de tradução de `pt-BR.ts`:

- `nav.tagline`: "caderno de publicação".
- Data por extenso em `dates.ts`, como "Quarta-feira, 28 de outubro".
- Frase de resumo de Hoje, montada com as contagens que `today` já traz: posts de hoje ainda não feitos, atrasados e próximos 7 dias. Cada contagem tem singular e plural, e a contagem zero some da frase. Sem nenhuma contagem, a frase é "Nada pendente por aqui." A lógica fica numa função pura em `ui/src/lib`, com teste no Vitest.

## 7. Fora do escopo

- Colunas embutidas da direção B.
- Animações novas.
- Reestruturar o Calendário ou o editor.
- Mudanças no núcleo em Rust, no conector MCP ou no formato dos dados.
- Identidade da bandeja e do ícone do app.

## 8. Testes

- `pnpm test:browser`: os 24 estados, com axe (WCAG 2.2 AA), nos dois temas, sem violações.
- **Contraste sobre a pauta:** o axe não mede contraste sobre `background-image` e deixaria de checar o texto dessas telas sem avisar. `harness.mjs` define `--ruling: none` antes de `audit()`, de modo que o texto é medido contra `--bg`.
- **Fontes:** `harness.mjs` e `tauri.mjs` verificam que Fraunces e Source Sans 3 carregaram (`document.fonts.check`), este último sob a CSP real do webview.
- **Ícones:** um teste no Vitest garante que nenhum componente use mais os glifos da seção 3. As setas `← → ↑ ↓` ficam de fora do teste, porque seguem válidas como texto.
- **Avisos:** `screens.mjs` ganha um estado com um aviso de erro, que hoje não é auditado.
- **Atalhos:** um teste em `harness.mjs` verifica que cada atalho exibido na barra lateral muda de tela.
- Os seletores de `tauri.mjs` continuam válidos: `h1` "Hoje", `section.column`, `aside.editor`, `.chip-btn`, `.item` e os rótulos dos botões.
- Conferência visual das capturas de `screens.mjs` nos dois temas.

## 9. Critérios de aceite

1. Todas as verificações do CI passam.
2. Nenhum glifo da seção 3 sobra nos componentes, exceto as setas de texto que ela lista.
3. As fontes carregam sem rede, sob a CSP real.
4. Cada atalho da barra lateral está visível e funciona.
5. O autor revisa capturas de antes e depois e concorda que o app deixou de parecer genérico e inacabado.

## 10. Ajustes feitos na implementação

Decididos depois da revisão de código e da revisão visual:

- **Abas dos quadros:** seguem o protótipo aprovado, com a aba ativa em `--text` sobre `--bg`. Antes não estavam na seção 5, e a aba ativa tinha contraste baixo no Grafite.
- **Hierarquia dos botões:** um botão primário por seção no editor. "Agendar", "Pedir arte" e "Copiar pedido" passam a botões comuns.
- **Telas de leitura:** Hoje, Ritmo e Configurações alinham o título à mesma borda esquerda, em vez de centralizar o conteúdo.
- **Pauta em Ritmo:** gráficos e listas têm fundo `--bg`, para as linhas não parecerem grade do gráfico.
- **Controles nativos:** caixas de seleção usam `accent-color`, seletores usam uma seta própria (`--chevron`) e o seletor de arquivo é estilizado.
- **Agenda do Calendário:** linhas com fio, como em Hoje, sem caixas cinzas.
- **Novo token:** `--fs-base` (15px), para os títulos das linhas de Hoje.
- **Resumo de Hoje:** um post de hoje já atrasado conta uma vez, como post a publicar. Um prazo atrasado do mesmo cartão continua contando como atrasado.
- **Testes:** a auditoria falha se o axe deixar de medir contraste por causa de imagem de fundo ou pseudoelemento. A tela Hoje ganhou o estado `today-full`.
