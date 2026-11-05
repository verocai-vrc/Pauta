# Pauta

Organizador de processo para manter constância na presença on-line e nos projetos pessoais: quadros estilo Trello, um calendário único para posts, prazos e marcos, lembretes no desktop na hora de postar e um conector MCP para operar tudo pelo Claude.

Tudo fica em arquivos Markdown + YAML numa pasta sua (legíveis no Obsidian, versionáveis com Git). Nada é publicado automaticamente: o Pauta organiza e lembra, e quem posta é você.

## Partes

| Caminho | O que é |
| --- | --- |
| [`crates/pauta-core`](crates/pauta-core) | Biblioteca com modelo, leitura e gravação sem perdas, agenda, lembretes e horários livres. Sem Tauri e sem relógio real |
| [`crates/pauta-app`](crates/pauta-app) | App Tauri 2: comandos, bandeja, notificações, observador de arquivos e inicialização automática |
| [`crates/pauta-mcp`](crates/pauta-mcp) | Servidor MCP por stdio para o Claude Code e o Claude Desktop |
| [`crates/pauta-cli`](crates/pauta-cli) | Linha de comando mínima de depuração |
| [`ui/`](ui) | Interface em Svelte 5 + TypeScript |
| [`docs/`](docs) | [Formato dos dados](docs/formato-dos-dados.md) e [ADRs](docs/adr) |
| [`examples/sample-data`](examples/sample-data) | Pasta de exemplo, usada nos testes |

## Pré-requisitos (Ubuntu)

```sh
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev libssl-dev
# Rust ≥ 1.88, Node 22 e pnpm 9
pnpm install
```

## Uso no dia a dia

```sh
pnpm tauri dev                    # app em modo de desenvolvimento
pnpm tauri build                  # pacotes .deb e AppImage em target/release/bundle/
cargo build --release -p pauta-mcp
```

Na primeira execução, o app pede a pasta de dados (padrão `~/Documentos/Pauta`) e cria os quadros Conteúdo, TCC2, Dungeon Meister, Abstract-Tex e Pauta.

> Se o app abortar com `Settings schema … 'antialiasing'`, ele foi iniciado pelo terminal do VS Code em snap. Rode-o de um terminal comum (ADR 0005).

### Conectar ao Claude

A tela **Configurações → Conector MCP** mostra o comando pronto, com os caminhos certos. Para o Claude Code:

```sh
claude mcp add --scope user pauta -- /caminho/para/pauta-mcp --root ~/Documentos/Pauta
```

Ferramentas (20): `pauta_today`, `pauta_rhythm`, `pauta_list_boards`, `pauta_list_cards`, `pauta_get_card`, `pauta_add_idea`, `pauta_create_card`, `pauta_update_card`, `pauta_move_card`, `pauta_schedule_post`, `pauta_find_slots`, `pauta_list_agenda`, `pauta_create_event`, `pauta_mark_published`, `pauta_record_metrics`, `pauta_request_art`, `pauta_list_art_requests`, `pauta_mark_art_ready`, `pauta_archive_card` e `pauta_duplicate_for_network`. Nenhuma exclui definitivamente nem publica, e toda escrita fica em `.pauta/log.jsonl`.

Recursos (leitura, sem chamar ferramenta): `pauta://today`, `pauta://boards`, `pauta://rhythm`, `pauta://agenda`, `pauta://card/{id}` e `pauta://board/{id}`. Prompts: `draft_post_from_milestone`, `weekly_review`, `plan_posting_week`, `art_brief` e `suggest_topics` (pesquisa temas em alta, se o cliente tiver busca na web, e só cria ideias depois da sua escolha).

Depois de atualizar o Pauta, recompile o binário (`cargo build --release -p pauta-mcp`) e reinicie o cliente do Claude para ver as ferramentas novas.

### Atalhos

`Ctrl+K` abre a paleta, `Alt+1…5` troca de tela (Hoje, Quadros, Calendário, Ritmo, Configurações) e `Ctrl+Shift+I` captura uma ideia. No quadro, as setas mudam o foco, `Alt+setas` movem o cartão e `Enter` abre. O atalho global `Ctrl+Alt+P` só funciona no X11; no Wayland, crie um atalho do GNOME para `pauta --capture`.

## Desenvolvimento

```sh
cargo test --workspace            # núcleo (unidade, integração e propriedade) e MCP pelo protocolo
cargo clippy --workspace --all-targets -- -D warnings
pnpm lint && pnpm check && pnpm test   # ESLint, tipos do Svelte e Vitest
pnpm test:browser                 # cada tela em claro e escuro com axe (WCAG 2.2 AA) e a regressão do salvamento automático
xvfb-run -a pnpm test:e2e         # app real via tauri-driver (precisa de webkit2gtk-driver; ver abaixo)
cargo run -p pauta-cli -- --root examples/sample-data boards
```

## Identidade visual

O Pauta segue a identidade do **Caderno editorial**: papel, pauta, margem de caderno e títulos com serifa (o nome vem de "papel pautado"). Os valores ficam nos tokens de `ui/src/app.css`, em claro e em Grafite, e a decisão está em `docs/superpowers/specs/2026-11-05-redesenho-caderno-design.md`.

Os ícones são do [Lucide](https://lucide.dev) (ISC), copiados em `ui/src/components/Icon.svelte` com o aviso da licença. As fontes são Fraunces (títulos) e Source Sans 3 (interface), ambas OFL-1.1, empacotadas pelos pacotes `@fontsource-variable` para o app rodar sem rede.

## Estado

A versão 0 (marcos 0 a 5 do documento de design) está pronta e foi revisada pelo autor, incluindo o uso real do conector MCP a partir do Claude Code. O marco 6 e os pendentes da versão 0 foram feitos em seguida:

| Item | O que há |
| --- | --- |
| Constância (seção 10) | Meta de cadência por rede (ex.: 2 por semana) com progresso na tela Hoje; tela **Ritmo** com histórico por semana e por mês, que continua contando posts arquivados; "parado" passa a medir o tempo na mesma coluna (`column_since`), não a última edição |
| Fechando a versão 0 | ESLint; verificação de acessibilidade com axe em todas as telas (corrigiu problemas reais no calendário); teste de regressão do salvamento automático; adiar um lembrete para dentro do horário de silêncio agora espera o fim dele; visão semanal em grade de horas, com arrastar para reagendar; suíte de ponta a ponta com `tauri-driver` |
| MCP: recursos e prompts (11.6) | Seis recursos e quatro prompts, com os dados já embutidos |
| Resultados e `.ics` (17.1, 17.3) | Números de alcance e engajamento digitados no cartão publicado (sem API de rede), com o que eles dizem sobre os dias da semana; importação desses números de um CSV exportado da rede (colunas reconhecidas por nome, com prévia antes de gravar; ADR 0008); exportação do calendário em `.ics`, com os lembretes como alarmes |
| Arte com o Claude Design | Pedido de arte registrado no cartão, texto pronto para colar, anexos, estado e verificação de "pronto para postar" (ADR 0007) |

Limites conhecidos:

- **A suíte `tauri-driver` ainda não foi executada.** O `WebKitWebDriver` não está instalável na máquina de desenvolvimento; a primeira execução será no CI. Ela não cobre arrastar com o mouse (o WebKitWebDriver não gera eventos de arrastar do HTML5); o equivalente pelo teclado é coberto, e o arrastar é testado à mão.
- **Não há integração com o Claude Design.** Não existe API: o Pauta monta o pedido, guarda o estado e recebe os arquivos que você exportar. Os tamanhos de imagem por rede vêm de fontes que divergem entre si; confira-os na ajuda de cada plataforma e ajuste em `pauta.yaml` (ADR 0007).
- **A importação de CSV foi testada com arquivos escritos por mim**, não com exportações reais do LinkedIn ou do Instagram, cujo formato não pude verificar. Ela reconhece as colunas por nome (português ou inglês) e mostra o que entendeu antes de gravar; se a sua exportação usar outros nomes, a mensagem diz o que faltou e dá para acrescentá-los em `crates/pauta-core/src/import.rs`.
- O teste manual dos critérios de aceite que dependem do desktop real (notificação com a janela fechada em release, lembrete perdido após desligar, Claude Desktop no Linux) continua sendo seu.

## Licença

`MIT OR Apache-2.0`, à sua escolha.
