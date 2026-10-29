# ADR 0006: Erros, posse e padrões de Rust usados no projeto

- **Data:** 2026-10-28
- **Estado:** aceita

Escolhas de Rust que podem não ser óbvias para quem está aprendendo (seção 19).

## Erros: `thiserror` na biblioteca, `anyhow` nos binários

`pauta-core` define um único `enum Error` (`error.rs`) com variantes que quem chama precisa distinguir: `Conflict` carrega o conteúdo do disco para o diálogo de conflito, e `NotFound` e `Rule` viram mensagens. O operador `?` propaga o erro para cima. O trait `IoContext` (`.at(path)`) anexa o caminho do arquivo aos erros de E/S, para a mensagem dizer qual arquivo falhou.

Nos binários (`pauta-cli`, `pauta-mcp`), o erro só é exibido, então `anyhow::Result` basta. No app, `CmdError` converte o `Error` num objeto JSON que a interface entende.

## Posse: `Workspace` é barato de clonar

`Workspace` guarda um `PathBuf`, uma `String` e um `Arc<dyn Clock>`. Clonar copia o caminho e incrementa um contador. Por isso `with_actor` e `with_clock` devolvem um `Workspace` novo em vez de mutar o existente: cada ferramenta MCP recebe o seu, sem `&mut` compartilhado.

`Arc<dyn Clock>` é um ponteiro com contagem de referências para "qualquer tipo que implemente `Clock`". Nos testes entra um `FixedClock`, e no app o `SystemClock`. O `+ Send + Sync` no trait permite usá-lo entre threads (o agendador roda numa thread separada).

## Estado compartilhado no app

`AppState` usa `RwLock<Option<…>>`: muitas leituras simultâneas (comandos da interface, bandeja, agendador) e escrita rara (troca de pasta ou recarga). `Mutex<()>` em `reminders_lock` serve só para serializar o acesso a `.pauta/reminders.json`.

## `Option<Option<T>>` nos patches

Num `CardPatch`, `due: None` significa "não mexa", `Some(None)` significa "apague o prazo" e `Some(Some(d))` significa "defina". O módulo `fields::double_option` faz o serde distinguir um campo ausente no JSON de um `"due": null`.

## `let`-chains

`if let Some(x) = a && cond(x) { … }` (Rust 1.88, edição 2024) substitui `if let` aninhados. Por isso o `rust-version` mínimo é 1.88.

## Acréscimo (2026-10-29): `Index` é um retrato, não um cache

`Workspace::load` lê a pasta inteira (incluindo os cartões arquivados, só para o histórico) e devolve um `Index`. Cálculos como `schedule::today`, `rhythm::cadence` e `art::readiness` são funções puras sobre esse retrato e um `now` explícito, e por isso os testes usam um `FixedClock`. Nada é guardado entre chamadas: o app recarrega depois de cada escrita e a cada mudança de arquivo.
