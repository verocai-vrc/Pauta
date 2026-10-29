# ADR 0003: Conector MCP com o SDK oficial `rmcp`

- **Data:** 2026-10-28
- **Estado:** aceita

## Contexto

A seção 11 pede um binário `pauta-mcp` por stdio, com o SDK oficial de Rust "se estiver maduro". Na data da implementação, o `rmcp` está na versão 3.8 e implementa a revisão `2026-07-28` do protocolo, que eliminou o aperto de mão `initialize`. Com isso, escrever o protocolo à mão ficou mais arriscado do que usar o SDK.

## Decisão

- `rmcp` 3.8 com as macros `#[tool_router]` e `#[tool]`. Os parâmetros são structs com `schemars::JsonSchema`, e os comentários `///` viram as descrições que o Claude lê.
- O servidor abre a pasta de dados a cada chamada (`Workspace::open`), e por isso sempre vê as edições feitas no app ou à mão. Não há estado em memória.
- As ferramentas da seção 11.3, mais `pauta_duplicate_for_network` (exigida pelo modelo de um cartão por rede).
- Os erros de domínio (coluna inexistente, data inválida) voltam como resultado de ferramenta com `is_error`, que o Claude consegue ler e corrigir, e não como falha do protocolo.
- Toda escrita usa `Workspace::with_actor("mcp:<ferramenta>")`, e o `pauta-core` registra cada uma em `.pauta/log.jsonl`.
- Nenhuma ferramenta exclui definitivamente nem publica. A mais destrutiva é `pauta_archive_card`.

## Configuração verificada

`claude mcp add --help` (Claude Code 2.1.271) confirma a sintaxe usada pela tela de Configurações:

```sh
claude mcp add --scope user pauta -- /caminho/pauta-mcp --root ~/Documentos/Pauta
```

O suporte do Claude Desktop no Linux **não foi verificado**. A tela mostra o trecho `mcpServers` para o caso de existir.

## Consequências

O teste `crates/pauta-mcp/tests/protocol.rs` inicia o binário real como processo filho, conecta como cliente `rmcp`, chama cada ferramenta e confere os arquivos e o log.
