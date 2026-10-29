# ADR 0001: Decisões da sessão de planejamento

- **Data:** 2026-10-28
- **Estado:** aceita

## Contexto

O [documento de design](../../Design%20Document%20Pauta.md) deixou doze pontos em aberto (seção 20). Eles foram fechados numa entrevista com o autor antes da implementação.

## Decisões

| # | Ponto | Decisão |
| --- | --- | --- |
| 1 | Stack | Tauri 2 + Rust + Svelte 5 com TypeScript |
| 2 | Pasta de dados | `~/Documentos/Pauta` como padrão sugerido, escolhida na primeira execução e guardada em `~/.config/pauta/config.yaml`. Separada de `~/Pauta`, onde fica o código. Versionar com Git fica a critério do autor (a pasta recebe um `.gitignore` que ignora `.pauta/`) |
| 3 | Modelo de publicação | **Um cartão por rede**, com as versões da mesma ideia ligadas como irmãs (ADR 0004). Diverge da recomendação da seção 7.2 |
| 4 | Colunas iniciais | As da seção 6.2 |
| 5 | Faixas e lembretes | Os exemplos da seção 5.2: LinkedIn ter/qui 09:00, Instagram qua/sáb 19:00, posts `-1h` e `0`, prazos `-7d`, `-1d` e `09:00`, silêncio 23:00–07:00, fuso America/Recife |
| 6 | Idioma | pt-BR, com todos os textos em chaves de tradução (`ui/src/lib/locales/`) |
| 7 | Biblioteca YAML | `serde_yaml_ng` só para *ler*; a escrita é cirúrgica (ADR 0002) |
| 8 | SDK MCP | `rmcp` 3.8, o SDK oficial (ADR 0003) |
| 9 | Fechar a janela | Minimiza para a bandeja; inicialização automática ligada por padrão, com opção para desligar |
| 10 | Widget | Fica como ideia futura; a bandeja cobre a necessidade |
| 11 | Publicação | Público, `MIT OR Apache-2.0` |
| 12 | Quadros iniciais | Conteúdo, TCC2, Dungeon Meister, Abstract-Tex e Pauta |

O autor também pediu a versão 0 inteira (marcos 0 a 5) numa única sessão, em vez de parar para revisão após cada marco como sugere a seção 19. As revisões por marco continuam valendo antes do marco 6.

## Consequências

- `pauta-core` não conhece o conceito de "publicações" no plural: cada cartão tem no máximo uma `publication` e um `network`.
- Um campo `role` (`inbox`, `scheduled`, `done`) nas colunas permite que as regras de agenda funcionem com colunas renomeadas ou personalizadas.
