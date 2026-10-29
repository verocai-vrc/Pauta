# ADR 0004: Um cartão por rede, com cartões irmãos

- **Data:** 2026-10-28
- **Estado:** aceita (decisão do autor)

## Contexto

A seção 7.2 recomendava um cartão por ideia, com `publications.linkedin` e `publications.instagram`. O autor preferiu um cartão por rede.

## Decisão

```yaml
network: linkedin
publication:
  at: 2026-11-04T09:00:00-03:00
  status: scheduled       # planned | scheduled | published | skipped
  url: null
  published_at: null      # preenchido por "marcar como publicado"
links:
  siblings: [01JBX3K9Q3CD]   # o mesmo post para outras redes
  article: 01J…              # se for divulgação de um artigo
```

- Cada cartão anda sozinho pelas colunas: o post do LinkedIn pode estar em Agendado enquanto o do Instagram ainda está em Produção.
- "Criar versão para Instagram" (`duplicate_for_network`) copia título, etiquetas e corpo e mantém `links.siblings` simétrico em todos os irmãos.
- O texto copiado para postar é a seção `## LinkedIn` (ou o nome da rede), se existir; senão, o corpo inteiro menos a seção `## Notas`.
- "Marcar como publicado" move o cartão para a coluna de papel `done`.

## Consequências

- O cálculo do próximo horário livre olha só os cartões da rede em questão.
- Uma ideia ainda sem rede (captura rápida) é um cartão com `network` ausente; ela recebe a rede quando sai da coluna Ideia.
