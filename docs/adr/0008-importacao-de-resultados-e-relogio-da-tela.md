# ADR 0008: Importação de resultados por CSV e relógio da tela

- **Data:** 2026-10-29
- **Estado:** aceita

## Importação de resultados (seção 17.1, alternativa sem API)

O documento aponta duas saídas para métricas sem API: digitar no cartão (feito) ou importar as exportações das plataformas. A segunda foi feita como **prévia e depois gravação**: `preview_metrics_import` lê o CSV e diz que cartão cada linha atualizaria, e `import_metrics` aplica o mesmo plano. O arquivo é lido pelo navegador da interface (`<input type="file">`), então o app não ganhou permissão de leitura de arquivos.

**Por que reconhecer cabeçalhos por nome:** não consegui verificar o formato das exportações do LinkedIn e do Instagram, que diferem entre si e mudam. Um layout fixo quebraria em silêncio; o reconhecimento por nome falha alto (diz que coluna faltou) e a prévia mostra qual coluna alimentou cada campo. `Alcance` entra como último recurso para impressões, e por isso a prévia nunca esconde a substituição.

**Por que o link é a chave:** título e data se repetem; o link, não. Cartões sem link publicado não são atualizados, de propósito.

**Regras que evitam estragar dados:** só inteiros (nada de "1,5 mil"), célula vazia não apaga o número existente, nada é criado para linhas sem cartão, e uma falha num cartão não impede os outros. O `csv` é a única dependência nova.

## Relógio da tela

`Today` ganhou `time` (hora local em minutos, no fuso configurado) e a interface relê `get_today` a cada minuto e ao voltar o foco à janela. Antes, uma janela deixada aberta passava da meia-noite mostrando o dia anterior. A linha de "agora" da visão semanal usa esse horário, não o do navegador, para respeitar o fuso do `pauta.yaml`.
