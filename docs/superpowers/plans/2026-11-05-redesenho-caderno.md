# Redesenho Caderno: plano de implementação

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Aplicar a identidade Caderno editorial a toda a interface, sem mudar comportamento.

**Architecture:** Os valores mudam em `ui/src/app.css`, com os nomes atuais dos tokens. Duas peças novas: `Icon.svelte` e `ui/src/lib/summary.ts`. Cada tela ajusta só o próprio estilo com escopo.

**Tech Stack:** Svelte 5, TypeScript, Vite, Vitest, playwright-core e axe.

**Spec:** `docs/superpowers/specs/2026-11-05-redesenho-caderno-design.md`

## Global Constraints

- Nenhum recurso, atalho, dado ou comando entra ou sai.
- Valores de cor, tipo, raio e sombra: exatamente os da seção 2 do spec.
- Sem dependência em tempo de execução para ícones. Fontes só de `@fontsource-variable/*`, locais.
- Todo texto novo passa por `pt-BR.ts`.
- Seletores de `tauri.mjs` continuam válidos: `h1` "Hoje", `section.column`, `h3` da coluna, `aside.editor`, `.chip-btn`, `.item`, rótulos dos botões.
- `package.json` fica na raiz: rode `pnpm` em `/home/arthur/Pauta`. `test:browser` usa `ui/dist`, então rode `pnpm build` antes.

## Review Focus

1. Título longo em Hoje e nos cartões: quebra a linha e não invade a margem.
2. Janela na largura mínima (720px, barra lateral recolhida): a margem de Hoje não espreme o conteúdo.
3. Tela longa com rolagem: a pauta e a margem acompanham o conteúdo.
4. Hoje sem nenhum item: frase "Nada pendente por aqui." e nada quebrado.
5. Janela de captura (`?capture`): recebe fontes e tokens sem a barra lateral.

---

### Task 1: Fundamentos

**Files:** `package.json`, `ui/src/main.ts`, `ui/src/app.css`, `ui/src/components/{Modal,CardEditor,Toasts}.svelte`, `ui/e2e/harness.mjs`, `ui/e2e/screens.mjs`, `ui/e2e/tauri.mjs`

**Interfaces:** Produz os tokens da seção 2 e `--ruling`, `--font-serif`, `--fs-display`, `--sidebar`, `--danger-text`, `--scrim`, `--rule`, `--margin-line`, `--head-rule`.

- [ ] Instalar `@fontsource-variable/fraunces` e `@fontsource-variable/source-sans-3`, importando `opsz.css`, `opsz-italic.css`, `wght.css` e `wght-italic.css` em `main.ts`.
- [ ] Teste: `harness.mjs` falha se `document.fonts.check('16px "Fraunces Variable"')` ou Source Sans for falso, e `audit()` define `--ruling: none`. Rodar e ver falhar.
- [ ] Trocar os tokens nos três blocos de tema de `app.css`, e as cores fixas por `--scrim` e `--danger-text`.
- [ ] `screens.mjs` audita um aviso de erro nos dois temas. `tauri.mjs` verifica as fontes sob a CSP real.
- [ ] `pnpm build && pnpm test:browser` passa.

### Task 2: Ícones

**Files:** criar `ui/src/components/Icon.svelte` e `ui/src/lib/glyphs.test.ts`; modificar os componentes da seção 3.

**Interfaces:** `<Icon name="plus" size={16} />`, com `name: IconName`, um tipo união exportado do módulo.

- [ ] Teste: `glyphs.test.ts` lê `ui/src/**/*.svelte` e falha em qualquer glifo da seção 3, exceto `← → ↑ ↓`. Rodar e ver falhar.
- [ ] Criar `Icon.svelte` com os caminhos do pacote `lucide-static`, copiados e com o aviso ISC.
- [ ] Substituir os glifos, inclusive `↑ ↓` de `Boards` e `→` de `Today`. `Calendar.icon()` passa a devolver `IconName`.
- [ ] `pnpm test && pnpm check` passam.

### Task 3: Barra lateral e atalhos

**Files:** `ui/src/App.svelte`, `ui/src/lib/locales/pt-BR.ts`, `ui/e2e/screens.mjs`

- [ ] Teste em `screens.mjs`: cada `<kbd>` da barra lateral existe, e `Alt+1` a `Alt+5` mudam de tela. Rodar e ver falhar.
- [ ] Largura 216px, fundo `--sidebar`, marca com `nav.tagline`, item ativo e botão primário de captura.
- [ ] `<kbd aria-hidden="true">` e `aria-keyshortcuts`, ambos vindos da lista `views`. Buscar e Capturar usam constantes.
- [ ] Conferir a captura em 1360px: `Ctrl+Shift+I` não pode ser cortado. Aplicar a segunda linha do spec se precisar.

### Task 4: Hoje

**Files:** criar `ui/src/lib/summary.ts`; modificar `ui/src/lib/dates.ts`, `ui/src/lib/lib.test.ts`, `ui/src/lib/locales/pt-BR.ts`, `ui/src/components/Today.svelte`

**Interfaces:** `longDate(d: string): string` em `dates.ts`, e `todaySummary(today: Today): string` em `summary.ts`.

- [ ] Testes: `longDate("2026-10-28") === "Quarta-feira, 28 de outubro"`; `todaySummary` com um post, um atrasado e três próximos, com plurais, com zeros omitidos, e tudo zero dando "Nada pendente por aqui." Rodar e ver falhar.
- [ ] Implementar as duas funções e as chaves de tradução.
- [ ] Reescrever a marcação de Hoje com a margem, cartões para ação e linhas para consulta. Pauta com `background-attachment: local` e margem que acompanha a rolagem. Abaixo de 860px a sarjeta encolhe.
- [ ] `pnpm test && pnpm build && pnpm test:browser` passam, e a captura confere com o protótipo.

### Task 5: Quadros

**Files:** `ui/src/components/{Boards,BoardView,CardTile}.svelte`, `ui/src/app.css`

- [ ] Colunas planas com rótulo e fio, título em Fraunces, chips sem fundo exceto `ok`, `warn` e `danger`.
- [ ] `pnpm build && pnpm test:browser` passam, com título longo conferido na captura.

### Task 6: Calendário

**Files:** `ui/src/components/{Calendar,WeekGrid}.svelte`

- [ ] Teste em `screens.mjs`: um item publicado não tem `line-through`. Rodar e ver falhar.
- [ ] Fios finos no lugar das caixas, título em Fraunces, publicado apagado com `check`.
- [ ] `pnpm build && pnpm test:browser` passam.

### Task 7: Editor do cartão

**Files:** `ui/src/components/{CardEditor,ArtPanel}.svelte`

- [ ] Título em Fraunces, Publicação e Arte como seções com fio.
- [ ] `pnpm build && pnpm test:browser` passam nos três estados do editor.

### Task 8: Ritmo

**Files:** `ui/src/components/{Rhythm,StackedBars,CadenceMeter}.svelte`

- [ ] Teste em `screens.mjs`: o rótulo de cada barra tem `scrollHeight <= clientHeight`. Rodar e ver falhar.
- [ ] Corrigir o corte em `StackedBars`. Painéis viram seções com fio, com pauta na tela.
- [ ] `pnpm build && pnpm test:browser` passam.

### Task 9: Demais telas e fechamento

**Files:** `ui/src/components/{Settings,Setup,Modal,CommandPalette,Capture,Toasts,TrashArchive,MetricsImport}.svelte`, `README.md`

- [ ] Pauta em Configurações e títulos em Fraunces. Captura e Primeira execução conferidas.
- [ ] README: registrar a identidade visual e a licença dos ícones e das fontes.
- [ ] `pnpm lint && pnpm check && pnpm test && pnpm build && pnpm test:browser` e `cargo test --workspace` passam.
- [ ] Capturas de antes e depois nos dois temas para a revisão do autor.
