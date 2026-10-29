# ADR 0007: Arte dos posts com o Claude Design

- **Data:** 2026-10-29
- **Estado:** aceita

## Contexto

O autor quer poder pedir, de dentro do Pauta, a arte que acompanha um post, e a criação visual fica com o Claude Design (documento de design, seção 1.3: "o Pauta não reimplementa nada disso").

Pesquisa feita em 2026-10-29 (fontes secundárias; o documento manda conferir na documentação oficial):

- Não achei **nenhuma API** do Claude Design. O que se descreve são exportações pela interface (.zip, PDF, PPTX, HTML, Canva) e o envio para o Claude Code. Uma das fontes diz que não há exportação de imagem avulsa (PNG/JPEG/SVG).
- Os tamanhos de imagem por rede **divergem entre guias**. O LinkedIn publica carrossel como documento PDF; o Instagram quer proporção única por carrossel (1:1 ou 4:5), e 1080×1920 é de story/reel.

## Decisão

O Pauta **não integra** com o Claude Design: acompanha a passagem de bastão.

1. **Pedido:** "Pedir arte" grava no cartão `art: { status: requested, format, requested_at }`, garante uma seção `## Arte` no corpo (a direção visual, escrita pelo autor ou pelo Claude) e leva o cartão para a coluna de produção, se ele ainda estiver antes dela.
2. **Texto para colar:** "Copiar pedido" monta o pedido com rede, formato e tamanho, o texto do post, a direção visual, a identidade visual (`art.brand` do `pauta.yaml`) e o que entregar (acessibilidade, nomes de arquivo, alternativa em PDF/zip). "Abrir Claude Design" abre `art.tool_url`, que o autor pode trocar.
3. **Volta:** os arquivos exportados entram na pasta de anexos do cartão (por "Anexar arquivos…", à mão, ou por um agente com acesso ao disco). Marcar como pronta exige ao menos um arquivo.
4. **Pronto para postar:** `art::readiness` aponta o que falta: sem texto, acima do limite, rede que exige mídia sem arquivos, arte pedida e não concluída. Aparece no editor, no cartão, na tela Hoje e no retorno de `pauta_schedule_post`. Agendar com pendências **não é bloqueado**: o autor decide.
5. **Formatos por rede** ficam em `pauta.yaml` (`networks.<rede>.media`), com padrões embutidos, porque os tamanhos mudam e as fontes divergem. A seção `## Arte` nunca é texto de post nem entra na contagem de caracteres.

No MCP: `pauta_request_art`, `pauta_list_art_requests`, `pauta_mark_art_ready` e o prompt `art_brief`. O servidor **não copia arquivos de fora da pasta de dados** (seção 11.4); por isso não há ferramenta de anexar, e `pauta_list_art_requests` devolve a pasta onde os arquivos devem ficar.

## Por que assim

- Sem API, qualquer "integração" seria automação de tela, frágil e fora do espírito do projeto.
- Guardar o estado no arquivo do cartão mantém tudo legível no Obsidian e operável pelo Claude (invariantes 1 e 4).
- Se um dia houver API ou exportação programática, só o passo 3 muda; o resto do fluxo já está pronto.

## Consequências e pontos a verificar

- O formato `.zip` ou `.pdf` exportado fica como anexo: o Pauta não abre nem converte (um carrossel do Instagram precisa de imagens por slide; se o Claude Design só exportar PDF, será preciso convertê-lo à parte).
- Conferir na ajuda de cada plataforma os tamanhos antes de confiar nos padrões; eles são ponto de partida.
- Conferir na conta do autor o que o Claude Design exporta de fato e se o endereço padrão (`https://claude.ai`) é o melhor ponto de entrada.
