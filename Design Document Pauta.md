# Design Document: Pauta

> Documento de design para a criação do Pauta, aplicativo desktop de produtividade pessoal com conector MCP, produzido a partir de uma sessão de planejamento no claude.ai. Destina-se a ser anexado na IDE para que um agente (Claude Code) planeje e implemente o programa.
>
> **Data:** 28 de outubro de 2026
> **Autor do projeto:** Arthur (estudante de Ciência da Computação, UNICAP)
> **Nome:** "Pauta" é o nome de trabalho, decidido pelo autor. No jornalismo, a pauta é o planejamento do que será publicado e quando.
> **Próximo passo previsto:** sessão de alinhamento com a skill "Planeje Comigo" na IDE, seguida da implementação em sprints.

---

## 0. Como ler este documento

Este documento é dirigido a um agente de IA de planejamento e implementação. Ele descreve **o que** foi decidido e **por quê**, sem ditar a implementação linha a linha. O planejamento detalhado (tipos, módulos, ordem de tarefas) é responsabilidade do agente, em diálogo com o autor.

### 0.1 Convenção de status

- **[DECIDIDO]**: decisão fechada pelo autor. Não reabrir por iniciativa própria. Diante de um conflito técnico real, apresentar o problema e as alternativas ao autor antes de desviar.
- **[RECOMENDAÇÃO]**: sugestão técnica feita durante a sessão de planejamento, ainda não validada explicitamente pelo autor. O agente pode refinar ou substituir, com justificativa.
- **[ABERTO]**: pergunta que precisa de resposta antes ou durante o planejamento. Todas estão reunidas na seção 20.

### 0.2 Regras de interpretação

1. Se dois trechos parecerem conflitar, prevalece o mais específico. Persistindo a dúvida, perguntar ao autor.
2. O Pauta é um projeto **separado** do VideoAutomation. Nada deste documento depende daquele projeto, e as decisões de um não valem para o outro (seção 17.6).
3. Afirmações sobre produtos de terceiros (Claude Desktop, Claude Code, protocolo MCP, APIs do LinkedIn e do Instagram, bibliotecas) devem ser **conferidas na documentação oficial no momento da implementação**. Os pontos marcados "a verificar" são suposições razoáveis, não fatos confirmados.
4. O código, os identificadores e as ferramentas MCP são em inglês. Este documento é em português; a seção 21 traz um glossário para nomear tipos e módulos.

### 0.3 Mapa do documento

| Seções | Conteúdo |
| --- | --- |
| 1 a 3 | Contexto, princípios e escopo |
| 4 e 5 | Arquitetura e formato dos dados |
| 6 a 10 | Funcionalidades: quadros, conteúdo, calendário, agendamento e constância |
| 11 | Conector MCP |
| 12 a 14 | Interface, stack e integridade dos dados |
| 15 a 17 | Testes, marcos e evolução futura |
| 18 a 22 | Riscos, processo, pontos em aberto, glossário e referências |

---

## 1. Contexto e propósito

### 1.1 O problema

O autor quer manter uma presença on-line constante como desenvolvedor e pesquisador em tecnologia da informação, principalmente no **LinkedIn** e no **Instagram**, onde já tem contas profissionais. O objetivo é divulgar projetos, publicar sempre que sair um artigo, fazer divulgação científica (algo que ele gosta de escrever) e ampliar o networking.

**[DECIDIDO] O gargalo é a constância, não a criação.** Ter ideias e escrever não é a dificuldade. O que falta é organizar o processo, saber o que está em cada etapa e ser lembrado na hora de postar. O Pauta é, antes de tudo, um **organizador de processo**.

### 1.2 A ampliação: produtividade pessoal

**[DECIDIDO]** O Pauta não se limita à presença on-line. Ele também organiza:

- o andamento dos **projetos de código** do autor, cada um no seu próprio quadro;
- **prazos e datas de entrega**, como as do TCC2;
- lançamentos, marcos e qualquer outra data relevante.

O Pauta é, portanto, o aplicativo de produtividade pessoal do autor, com a presença on-line como primeiro caso de uso.

### 1.3 Como a criação acontece

A criação de conteúdo fica com ferramentas que o autor já usa: o Claude para ideias e textos, com o contexto dos Projetos e da memória que ele já construiu, e o Claude Design para a parte visual dos posts. O autor pretende criar um Projeto no Claude dedicado a posts. O Pauta não reimplementa nada disso; ele organiza o resultado e se conecta ao Claude pelo MCP (seção 11).

### 1.4 Ambiente

**[DECIDIDO]** Tudo local, no desktop Linux do autor (Ubuntu, ambiente principal em todas as máquinas). Sem servidor, sem conta, sem sincronização na versão 0.

---

## 2. Princípios e invariantes

Estas regras devem servir de checklist de revisão de código.

1. **Os arquivos são a fonte da verdade.** Todos os dados vivem em arquivos Markdown e YAML legíveis numa pasta do autor. Não há banco de dados que seja a verdade; qualquer índice ou cache é descartável e reconstruível a partir dos arquivos.
2. **Dois pés, um núcleo.** O aplicativo e o conector MCP leem e escrevem os mesmos arquivos por meio da mesma biblioteca central. Nenhuma regra de negócio é implementada duas vezes.
3. **O app funciona sem o Claude.** A interface é completa por si só. O conector MCP é uma porta a mais, nunca um requisito.
4. **Edições externas são bem-vindas.** O autor, o Claude ou outro editor de texto podem alterar os arquivos. O Pauta detecta e recarrega, e **preserva tudo o que não entende**: campos desconhecidos no cabeçalho e o corpo do texto exatamente como estavam.
5. **Nada se perde em silêncio.** Gravações são atômicas, exclusões vão para uma lixeira, e conflitos entre edições simultâneas são detectados e mostrados, nunca resolvidos sobrescrevendo um dos lados sem aviso.
6. **Tempo com fuso explícito.** Toda data com horário é gravada com fuso horário. Datas sem horário (prazos de dia inteiro) são gravadas como data pura.
7. **Nada é publicado automaticamente na versão 0.** O Pauta lembra, organiza e prepara; quem posta é o autor.
8. **O formato dos arquivos é um contrato estável e versionado.** Mudanças de formato vêm com migração automática.

---

## 3. Escopo

### 3.1 Versão 0 [DECIDIDO]

1. **Quadros estilo Trello**, com abas para vários quadros: quadros de conteúdo (posts, artigos) e quadros de projetos separados.
2. **Cartões editáveis**, com tela de edição que mostra o texto, as datas e os detalhes.
3. **Calendário** com os dias que têm posts marcados, lançamentos, prazos e outros eventos, incluindo eventos avulsos como as entregas do TCC2.
4. **Agendamento por rede**, com dias e horários de postagem definidos para cada conteúdo em cada rede.
5. **Notificação no desktop** na hora de postar.
6. **Conector MCP**, para que o Claude Desktop e o Claude Code leiam e operem o Pauta.

### 3.2 Depois da versão 0 [DECIDIDO como direção]

- dashboard de visualizações e métricas das contas;
- sugestões de temas em alta;
- chatbot dentro do app (por enquanto, o Pauta aberto em tela dividida com o Claude Desktop resolve);
- notificação no celular;
- publicação automática pelas APIs das redes.

### 3.3 Fora do escopo [DECIDIDO]

- **Integração com o Obsidian.** Fica fora, mas com a porta aberta: o formato dos cartões (Markdown com cabeçalho YAML) é lido nativamente pelo Obsidian, então o autor pode abrir a pasta nele um dia sem nenhuma linha de código a mais (seção 5.6).
- Multiusuário, colaboração e sincronização em nuvem.
- Qualquer mistura com o VideoAutomation nesta fase.

---

## 4. Arquitetura

### 4.1 Três camadas, dois pés [DECIDIDO]

```text
                ┌──────────────────────────┐        ┌──────────────────────────┐
                │   App Pauta (Tauri)      │        │   Conector MCP           │
                │   quadros, calendário,   │        │   (processo próprio,     │
                │   bandeja, notificações  │        │   iniciado pelo Claude)  │
                └────────────┬─────────────┘        └────────────┬─────────────┘
                             │                                   │
                             └──────────────┬────────────────────┘
                                            ▼
                             ┌──────────────────────────────┐
                             │   pauta-core (biblioteca)    │
                             │   modelo, leitura, gravação, │
                             │   validação, agenda, regras  │
                             └──────────────┬───────────────┘
                                            ▼
                             ┌──────────────────────────────┐
                             │   Pasta de dados do autor    │
                             │   Markdown + YAML            │
                             └──────────────────────────────┘
```

- **Dados:** a pasta com os arquivos (seção 5).
- **App:** a interface própria do autor, que funciona sem o Claude aberto.
- **Conector MCP:** permite ao Claude ler e operar os mesmos dados quando o autor quiser conversar com ele.

Qualquer alteração, venha de onde vier, é vista pelas outras portas porque todas leem os mesmos arquivos.

### 4.2 Por que MCP em vez de chatbot embutido

O autor queria inicialmente um chatbot dentro do app para perguntar sobre o contexto dos seus Projetos no Claude. A solução adotada inverte isso: em vez de o Claude entrar no Pauta, **o Pauta entra no Claude**, como um conjunto de ferramentas. Os motivos:

- o Claude Desktop e o Claude Code já têm o contexto dos Projetos e da memória do autor;
- o MCP aproveita a assinatura que o autor já tem, sem chave de API paga à parte;
- a verificação está pendente, mas até onde se sabe, o conhecimento dos Projetos do Claude **não é acessível por uma API pública**, o que tornaria um chatbot embutido sem esse contexto. **A verificar** na implementação.

### 4.3 Workspace Cargo [RECOMENDAÇÃO]

| Crate | Papel | Depende de |
| --- | --- | --- |
| `pauta-core` | Modelo de dados, leitura e gravação dos arquivos, validação, migrações, cálculo de agenda, lembretes e horários livres | apenas bibliotecas puras |
| `pauta-app` | Aplicativo Tauri: comandos expostos à interface, observador de arquivos, agendador de notificações, bandeja, inicialização automática | `pauta-core` |
| `pauta-mcp` | Binário do servidor MCP por stdio | `pauta-core` |
| `ui/` | Interface web do Tauri (Svelte e TypeScript) | comandos do `pauta-app` |

O `pauta-core` não depende do Tauri, do relógio real nem do sistema de notificações: o relógio e o sistema de arquivos entram por interfaces, o que permite testar toda a lógica de agenda com datas fixas.

---

## 5. Formato dos dados

### 5.1 Pasta de dados [RECOMENDAÇÃO]

Uma pasta escolhida pelo autor na primeira execução (padrão sugerido: `~/Pauta`):

```text
~/Pauta/
  pauta.yaml                 # configuração geral (seção 5.2)
  boards/
    conteudo/                # um quadro de conteúdo
      board.yaml
      cards/
        2026-11-04-rust-sem-garbage-collector--01JBX3.md
        ...
      assets/
        01JBX3/              # imagens e anexos do cartão
    dungeon-meister/         # um quadro de projeto
      board.yaml
      cards/
      assets/
    tcc2/
      ...
  calendar/                  # eventos avulsos: prazos, entregas, marcos
    2026-12-05-entrega-tcc2-capitulo-3--01JBY7.md
  archive/                   # cartões e eventos arquivados, por quadro
  .trash/                    # exclusões recuperáveis
  .pauta/                    # cache, log e trava; descartável, ignorável pelo Git
```

A pasta inteira pode ser versionada com Git se o autor quiser, já que tudo é texto.

### 5.2 Configuração geral (`pauta.yaml`)

```yaml
schema_version: 1
timezone: America/Recife
language: pt-BR
board_order: [conteudo, dungeon-meister, abstract-tex, tcc2]
networks:
  linkedin:
    slots:                       # faixas de postagem (seção 9.1)
      - { days: [tue, thu], time: "09:00" }
    cadence_goal: { posts: 2, per: week }
  instagram:
    slots:
      - { days: [wed, sat], time: "19:00" }
    cadence_goal: { posts: 2, per: week }
reminders:
  post_default: ["-1h", "0"]     # antecedência e hora exata
  deadline_default: ["-7d", "-1d", "09:00"]
notifications:
  quiet_hours: { from: "23:00", to: "07:00" }
```

### 5.3 Quadro (`board.yaml`)

```yaml
schema_version: 1
id: conteudo
name: "Conteúdo"
kind: content            # content | project | generic
color: "#7a2e3b"
columns:
  - { id: idea,      name: "Ideia" }
  - { id: draft,     name: "Rascunho" }
  - { id: producing, name: "Produção" }
  - { id: scheduled, name: "Agendado" }
  - { id: published, name: "Publicado" }
```

- Colunas têm **identificador estável** e nome editável. Renomear uma coluna não toca nos cartões.
- Os quadros de projeto usam o mesmo formato com outras colunas (seção 6.2).

### 5.4 Cartão

Cada cartão é **um arquivo Markdown com cabeçalho YAML** (frontmatter). Exemplo de cartão de conteúdo:

```markdown
---
id: 01JBX3K9Q2
title: "Por que Rust não tem coletor de lixo"
kind: post
column: draft
rank: "a3"
tags: [rust, divulgacao]
links:
  board: dungeon-meister      # projeto relacionado, opcional
publications:
  linkedin:
    at: 2026-11-04T09:00:00-03:00
    status: scheduled         # planned | scheduled | published | skipped
    url: null
  instagram:
    at: 2026-11-05T19:00:00-03:00
    status: planned
    url: null
reminders: ["-1h", "0"]
created: 2026-10-28T20:15:00-03:00
updated: 2026-10-28T21:02:00-03:00
---

## LinkedIn

Texto do post para o LinkedIn...

## Instagram

Legenda para o Instagram...

## Notas

Fontes, ideias soltas, prompt usado no Claude Design...
```

Exemplo de cartão de projeto:

```markdown
---
id: 01JBZ8M4T1
title: "Resolução em etapas com janelas de reação"
kind: task
column: doing
rank: "b1"
tags: [submarco-1]
due: 2026-11-20
checklist_progress: auto      # calculado das caixas de seleção do corpo
created: 2026-10-28T22:00:00-03:00
updated: 2026-10-28T22:00:00-03:00
---

- [x] Esboçar a pilha de decisões pendentes
- [ ] Testes de snapshot do log de eventos

Link: https://github.com/...
```

Regras do cartão:

- **`id`** é estável e único (por exemplo, ULID). O nome do arquivo é derivado do título e do id, e pode mudar sem quebrar referências.
- **`rank`** é uma chave de ordenação fracionária dentro da coluna. Mover um cartão altera só o seu próprio arquivo, sem renumerar os vizinhos.
- **Seções do corpo** com títulos convencionados (`## LinkedIn`, `## Instagram`, `## Notas`) são reconhecidas pelo app nos cartões de conteúdo. Qualquer outro conteúdo do corpo é preservado e exibido como Markdown.
- **Caixas de seleção** do Markdown (`- [ ]`) formam a lista de verificação do cartão, com progresso exibido no quadro.
- **Campos desconhecidos** no cabeçalho são preservados na regravação (invariante 4).

### 5.5 Evento de calendário

Prazos, entregas e marcos que não são cartões ficam em `calendar/`, no mesmo formato:

```markdown
---
id: 01JBY7P0D5
title: "Entrega do capítulo 3 do TCC2"
kind: deadline            # deadline | event | milestone | launch
date: 2026-12-05          # dia inteiro; ou start/end com horário e fuso
reminders: ["-7d", "-1d", "09:00"]
links:
  board: tcc2
tags: [tcc2]
---

Detalhes da entrega, formato exigido, link do sistema da universidade...
```

### 5.6 Compatibilidade com o Obsidian

Não é requisito, mas o formato escolhido a garante de graça: Markdown comum com propriedades YAML no topo é o formato nativo do Obsidian. Para preservar isso, o Pauta deve evitar sintaxe própria que o Obsidian não entenda, e não deve criar nem depender de arquivos dentro de uma pasta `.obsidian`.

### 5.7 Versionamento e migração

Cada arquivo de configuração e cada cartão carregam `schema_version` quando necessário (os cartões podem herdar a versão do quadro). Quando o formato mudar, o `pauta-core` migra os arquivos antigos passo a passo, com cópia de segurança antes da primeira migração.

---

## 6. Quadros

### 6.1 Abas [DECIDIDO]

Cada quadro é uma aba. O autor cria, renomeia, reordena, colore e arquiva quadros. Os quadros de conteúdo (posts e artigos) e os quadros de projetos ficam separados, cada um na sua aba.

### 6.2 Tipos e modelos [RECOMENDAÇÃO]

O tipo do quadro define as colunas iniciais e os campos que o cartão oferece. As colunas continuam editáveis depois.

| Tipo | Colunas iniciais | Campos próprios do cartão |
| --- | --- | --- |
| **Conteúdo** | Ideia, Rascunho, Produção, Agendado, Publicado | Publicações por rede (data, horário, estado, link), texto por rede, anexos |
| **Projeto** | Backlog, A fazer, Fazendo, Revisão, Concluído | Prazo, lista de verificação, links (repositório, issue) |
| **Genérico** | A fazer, Fazendo, Feito | Prazo, lista de verificação |

Quadros iniciais sugeridos na primeira execução: "Conteúdo" e um quadro de projeto de exemplo. Os projetos atuais do autor (Dungeon Meister, Abstract-Tex, TCC2 e outros) viram quadros quando ele quiser.

### 6.3 Interações no quadro

- Arrastar e soltar cartões entre colunas e dentro da coluna, com equivalente completo pelo teclado (seção 12.4).
- Criar cartão rápido no topo de qualquer coluna, só com o título.
- Filtro por etiqueta e busca por texto no quadro.
- Contador de cartões por coluna; limite opcional de cartões em andamento por coluna, com aviso visual ao ultrapassar.
- Indicadores no cartão: etiquetas, prazo (com destaque se vencido ou próximo), progresso da lista de verificação, redes e datas de publicação.

### 6.4 Tela de edição do cartão [DECIDIDO]

Uma tela (painel lateral ou janela) para editar o cartão: título, coluna, etiquetas, datas, lembretes, campos do tipo e o texto em Markdown.

**[RECOMENDAÇÃO]** Editor de Markdown com alternância entre edição e visualização, sem tentar ser um editor visual completo. Mudanças são salvas automaticamente após uma pausa curta na digitação.

### 6.5 Arquivamento e lixeira

- **Arquivar** move o cartão para `archive/`, tirando-o do quadro sem perdê-lo. Cartões publicados ou concluídos há mais de um período configurável podem ser arquivados automaticamente.
- **Excluir** move para `.trash/`, recuperável pela interface.

---

## 7. Fluxo de conteúdo

### 7.1 Ciclo do post [DECIDIDO]

**Ideia → Rascunho → Produção → Agendado → Publicado.** Na prática:

- **Ideia:** captura rápida, só título e notas.
- **Rascunho:** texto sendo escrito, em geral com o Claude.
- **Produção:** arte sendo feita, em geral no Claude Design; o arquivo final vai para os anexos do cartão.
- **Agendado:** texto e arte prontos, com data e horário por rede.
- **Publicado:** postado, com o link do post registrado.

### 7.2 Um cartão, várias redes [RECOMENDAÇÃO]

Um cartão representa **uma ideia de post**, com uma entrada de publicação para cada rede (LinkedIn, Instagram). Cada entrada tem data e horário, estado e link próprios, porque o texto e o momento de cada rede costumam ser diferentes. O cartão vai para "Publicado" quando todas as suas publicações estiverem publicadas ou dispensadas.

**[ABERTO]** Confirmar esse modelo, ou preferir um cartão por rede.

### 7.3 Facilitar a postagem manual [RECOMENDAÇÃO]

Como a versão 0 não publica automaticamente, o momento de postar precisa levar poucos minutos:

- botão **"Copiar texto"** por rede, com o texto já pronto;
- acesso direto aos **anexos** (abrir a pasta ou copiar a imagem);
- botão para **abrir a rede** no navegador;
- ação **"Marcar como publicado"**, que pede o link do post e registra data e hora reais;
- **contador de caracteres** por rede, com aviso quando o texto passar do limite da plataforma. Os limites devem vir de configuração e ser conferidos na implementação, porque mudam.

### 7.4 Artigos

Artigos (científicos, de blog ou de portfólio) são cartões de conteúdo como os posts, com uma etiqueta ou um tipo próprio. **[RECOMENDAÇÃO]** Quando um artigo é publicado, o Pauta oferece criar um cartão de post de divulgação ligado a ele, porque o autor quer postar sempre que publicar um artigo.

---

## 8. Calendário

### 8.1 Tela de calendário [DECIDIDO]

Uma tela de calendário mostra, num só lugar, todos os itens com data:

- publicações agendadas e realizadas, por rede;
- prazos dos cartões de todos os quadros;
- eventos avulsos (prazos, entregas, marcos, lançamentos), como as entregas do TCC2.

**[RECOMENDAÇÃO]** Três visões: mês, semana e agenda (lista cronológica). Filtro por quadro, por tipo de item e por rede. Cada tipo de item com cor e ícone próprios (a cor nunca é o único indicador). Clicar num item abre o cartão ou o evento; clicar num dia vazio cria um evento ou um cartão com aquela data. Arrastar um item para outro dia reagenda.

### 8.2 Tela "Hoje" [RECOMENDAÇÃO]

A tela de detalhe do dia, na aba Hoje do Calendário (o app abre no Início, seção 8.4):

- o que postar hoje, com acesso direto às ações da seção 7.3;
- prazos de hoje e dos próximos dias;
- itens atrasados (publicações que passaram do horário sem serem marcadas, prazos vencidos);
- avisos de constância (seção 10).

### 8.3 Bandeja e widget

**[RECOMENDAÇÃO]** Um ícone na bandeja do sistema mostra os próximos itens (próxima publicação, próximo prazo) e dá acesso rápido para abrir o app, abrir a tela "Hoje" (a aba do Calendário) e capturar uma ideia. No Ubuntu, isso usa o suporte a indicadores de aplicativo, que vem ativo por padrão; conferir em outras distribuições.

**[ABERTO]** Widget na área de trabalho. O ambiente GNOME do Ubuntu não tem widgets nativos; seria preciso uma extensão própria. Fica como possibilidade futura.

### 8.4 Tela Início [DECIDIDO]

A tela que o app abre ao iniciar e na primeira abertura da janela em cada novo dia. Mostra de relance o que precisa do autor hoje, sem rolar, em sete widgets fixos: **Brief** (saudação, resumo e próximo compromisso), **Postar hoje**, **Avisos**, **Agenda** (7 dias), **Ritmo**, **Ideias** (com campo para guardar uma nova) e **Projetos** (último e próximo passo de cada quadro). Cada lista tem limite e um "Ver mais" que leva à tela de detalhe. A disposição, os limites e as regras estão em `docs/superpowers/specs/2026-11-05-inicio-design.md`; a decisão de compô-la na interface, em `docs/adr/0009-inicio-na-interface.md`.

---

## 9. Agendamento, lembretes e notificações

### 9.1 Faixas de postagem por rede [RECOMENDAÇÃO]

O autor define faixas fixas de postagem por rede no `pauta.yaml` ou na tela de configurações (por exemplo, LinkedIn às terças e quintas às 9h). Ao mover um cartão para "Agendado", o Pauta sugere o **próximo horário livre** de cada rede; o autor pode aceitar ou escolher outro horário qualquer.

Mais tarde, quando houver dados de desempenho (seção 17.1), as sugestões passam a vir dos resultados do próprio autor, que valem mais do que regras genéricas.

### 9.2 Lembretes

- **Publicações:** lembrete com antecedência (para conferir se o material está pronto) e na hora exata. Padrão configurável, ajustável por cartão.
- **Prazos e eventos:** lembretes em dias de antecedência e no próprio dia, num horário configurável.
- **Adiar:** toda notificação pode ser adiada (por exemplo, 15 minutos, 1 hora, amanhã).
- **Horário de silêncio:** lembretes que cairiam no período de silêncio são entregues ao fim dele.

### 9.3 Notificações no desktop [DECIDIDO]

Notificações nativas do sistema, só no desktop, na versão 0. **[RECOMENDAÇÃO]** Quando o sistema permitir, a notificação traz ações ("Abrir", "Copiar texto", "Adiar"); se não permitir, clicar nela abre o cartão.

### 9.4 Funcionamento em segundo plano [RECOMENDAÇÃO]

Para que as notificações cheguem com a janela fechada:

- fechar a janela **minimiza para a bandeja**, e o processo continua rodando;
- **inicialização automática** com a sessão do usuário, com opção para desligar;
- o agendador do `pauta-app` calcula o próximo lembrete a partir do `pauta-core` e espera até ele, recalculando sempre que os arquivos mudam.

### 9.5 Lembretes perdidos

Se o computador estava desligado ou suspenso no horário, os lembretes perdidos aparecem ao abrir ou ao acordar o sistema, agrupados numa única notificação ("2 posts atrasados") e destacados na tela "Hoje". Nada é perdido em silêncio.

---

## 10. Constância

O autor pediu que o produto mínimo não seja fraco a ponto de faltar motivação para abri-lo todo dia. Como o problema central é constância, o Pauta deve **tornar visível o ritmo**, sem virar um jogo nem gerar culpa.

**[RECOMENDAÇÃO]**

- **Aviso de fila vazia (versão 0):** quando uma rede não tem nada agendado para os próximos dias, a tela "Hoje" avisa, com atalho para os cartões em "Produção" ou "Rascunho" daquela rede.
- **Meta de cadência por rede (depois da versão 0):** por exemplo, dois posts por semana no LinkedIn, com indicador simples de progresso na semana.
- **Histórico de publicações:** contagem por semana e por mês a partir dos registros de publicação, base para o futuro dashboard.
- **Cartões parados:** destaque discreto para cartões que estão há muito tempo na mesma coluna.

---

## 11. Conector MCP

### 11.1 Papel [DECIDIDO]

O conector MCP é o segundo pé do Pauta. Ele permite que o Claude Desktop e o Claude Code, com todo o contexto que o autor já construiu nos Projetos e na memória, leiam e operem o quadro e o calendário. O autor conversa com o Claude normalmente, e o Claude cria cartões, escreve rascunhos, move cartões e agenda posts.

Casos de uso típicos:

- num Projeto do Claude dedicado a posts: "transforma as notas desta conversa num rascunho de post para o LinkedIn e coloca na coluna Rascunho";
- "o que eu tenho agendado esta semana, e o que está atrasado?";
- "cria um evento de prazo para a entrega do capítulo 3 do TCC2 no dia 5 de dezembro, com lembrete uma semana antes";
- no Claude Code, dentro do repositório do Dungeon Meister: "move o cartão da resolução em etapas para Concluído e cria uma ideia de post sobre ela".

### 11.2 Arquitetura [RECOMENDAÇÃO]

- Binário próprio, `pauta-mcp`, que fala MCP por **stdio** e é iniciado pelo próprio cliente (Claude Desktop ou Claude Code).
- Usa o `pauta-core` diretamente sobre a pasta de dados, sem depender de o app estar aberto. O app, se estiver aberto, percebe as mudanças pelo observador de arquivos.
- Biblioteca: o SDK oficial de MCP para Rust, se estiver maduro na época da implementação. **A verificar.**
- Caminho da pasta de dados recebido por argumento ou variável de ambiente, com o padrão de `pauta.yaml`.

### 11.3 Ferramentas [RECOMENDAÇÃO]

| Ferramenta | Função |
| --- | --- |
| `pauta_today` | Resumo do dia: o que postar, prazos, atrasados, avisos de fila vazia |
| `pauta_list_boards` | Lista os quadros e suas colunas |
| `pauta_list_cards` | Lista cartões com filtros (quadro, coluna, etiqueta, texto, intervalo de datas) |
| `pauta_get_card` | Lê um cartão completo, com cabeçalho e corpo |
| `pauta_add_idea` | Captura rápida de uma ideia na coluna inicial de um quadro de conteúdo |
| `pauta_create_card` | Cria um cartão com título, coluna, campos e corpo |
| `pauta_update_card` | Altera campos e seções do corpo (substituir ou acrescentar) |
| `pauta_move_card` | Move um cartão de coluna ou de posição |
| `pauta_schedule_post` | Agenda uma publicação de um cartão numa rede, num horário dado ou no próximo horário livre |
| `pauta_find_slots` | Lista os próximos horários livres de uma rede |
| `pauta_list_agenda` | Lista tudo o que tem data num intervalo |
| `pauta_create_event` | Cria um evento de calendário (prazo, entrega, marco, lançamento) |
| `pauta_mark_published` | Registra uma publicação como feita, com o link |
| `pauta_archive_card` | Arquiva um cartão |

As descrições das ferramentas devem ser claras o bastante para o Claude escolher a certa sem instruções extras, e os retornos devem ser compactos, para não consumir contexto à toa.

### 11.4 Segurança e limites [RECOMENDAÇÃO]

- **Nenhuma ferramenta exclui definitivamente.** No máximo arquiva ou move para a lixeira.
- **Nenhuma ferramenta publica nas redes.** A versão 0 não publica nada (invariante 7).
- Gravações pelo MCP seguem as mesmas regras de atomicidade e detecção de conflito do app (seção 14).
- O conector só acessa a pasta de dados configurada.
- Toda operação de escrita feita pelo MCP fica registrada num log em `.pauta/`, com data e ferramenta, para o autor auditar o que o Claude mudou.
- Os clientes do Claude pedem a aprovação do usuário para usar ferramentas; o Pauta não deve contornar isso.

### 11.5 Configuração nos clientes

- **Claude Code:** registro do servidor local pela linha de comando ou pelo arquivo de configuração de MCP. **A verificar** o comando exato na documentação.
- **Claude Desktop:** registro no arquivo de configuração de servidores MCP do aplicativo. **A verificar** o suporte e o caminho do arquivo no Linux; se o Claude Desktop não estiver disponível no Linux, o Claude Code cobre o uso local.
- **[RECOMENDAÇÃO]** A tela de configurações do Pauta mostra o trecho de configuração pronto para copiar, com o caminho correto do binário.

### 11.6 Recursos e prompts MCP [RECOMENDAÇÃO, depois da versão 0]

Além de ferramentas, o MCP permite expor recursos (por exemplo, o resumo do dia) e prompts prontos (por exemplo, "rascunhar post a partir de um marco de projeto"). Ficam para depois que as ferramentas estiverem estáveis.

---

## 12. Interface

### 12.1 Telas [RECOMENDAÇÃO]

1. **Início** (tela inicial): seção 8.4.
2. **Quadros:** abas, um quadro por aba, com o painel de edição do cartão.
3. **Calendário:** seção 8.1, com a aba **Hoje** (seção 8.2).
4. **Ritmo:** metas de cadência e histórico (seção 10).
5. **Configurações:** pasta de dados, fuso, idioma, redes e faixas de postagem, lembretes padrão, horário de silêncio, inicialização automática, tema, atalhos e o trecho de configuração do MCP.

Mais a bandeja (seção 8.3) e uma janela pequena de **captura rápida de ideia**, acionável por atalho global e pela bandeja.

### 12.2 Navegação e atalhos

Barra lateral com Início (`Alt+1`), Quadros, Calendário, Ritmo e Configurações. Paleta de comandos (por exemplo, Ctrl K) para buscar cartões, trocar de quadro e executar ações. Atalhos para criar cartão, mover cartão, abrir o calendário e capturar ideia.

### 12.3 Idioma

**[ABERTO]** Idioma da interface. **[RECOMENDAÇÃO]** Português do Brasil primeiro, com todos os textos em chaves de tradução desde o início, para que o inglês possa entrar depois sem retrabalho.

### 12.4 Acessibilidade [RECOMENDAÇÃO]

Todas as ações pelo teclado, inclusive mover cartões entre colunas e posições (sem depender de arrastar com o mouse), foco visível, rótulos acessíveis em todos os ícones, cor nunca como único indicador e respeito a `prefers-reduced-motion`. Meta: nível AA das diretrizes WCAG 2.2.

### 12.5 Visual [RECOMENDAÇÃO]

Cores, fontes, espaçamentos e raios centralizados em variáveis CSS (design tokens), com tema claro e escuro, para que um redesenho posterior seja troca de valores e não reescrita. A identidade visual será feita depois, possivelmente no Claude Design.

---

## 13. Stack

### 13.1 Recomendação

**[RECOMENDAÇÃO]** Tauri 2 com Rust no backend e Svelte com TypeScript na interface. Motivos:

- o autor já usa Tauri e Svelte no Abstract-Tex, e está aprendendo Rust;
- o Tauri tem plugins oficiais para notificações nativas, bandeja, inicialização automática e atalhos globais; **a verificar** os nomes e o estado atual dos plugins na implementação;
- o núcleo em Rust é compartilhado sem esforço entre o app e o binário MCP.

**[ABERTO]** Confirmar a stack.

### 13.2 Dependências candidatas [RECOMENDAÇÃO]

| Função | Candidatos | Observação |
| --- | --- | --- |
| Frontmatter YAML | biblioteca YAML mantida para Rust | O `serde_yaml` foi arquivado pelo autor; escolher uma alternativa mantida e que permita preservar campos desconhecidos. **A verificar** |
| Markdown | `pulldown-cmark` ou equivalente | Para reconhecer seções e caixas de seleção sem reescrever o corpo |
| Datas e fusos | `jiff` ou `chrono` com `chrono-tz` | Fuso explícito em toda data com horário |
| Identificadores | ULID | Ordenáveis por tempo |
| Observação de arquivos | `notify` | Recarregar ao detectar mudanças externas |
| MCP | SDK oficial de MCP para Rust | **A verificar** |
| Interface | Svelte, TypeScript, biblioteca de arrastar e soltar acessível | |
| Testes | `cargo test`, `proptest`, `insta`, Vitest, `tauri-driver` | Seção 15 |

### 13.3 Licença e publicação

**[ABERTO]** Se o Pauta será público no GitHub como parte do portfólio. Se for, licença `MIT OR Apache-2.0`, como os outros projetos em Rust do autor.

---

## 14. Integridade dos dados e concorrência

O app, o conector MCP, o Claude e o próprio autor podem alterar os mesmos arquivos. As regras abaixo evitam perda de dados.

**[RECOMENDAÇÃO]**

1. **Gravação atômica:** escrever num arquivo temporário na mesma pasta e renomear por cima do original.
2. **Detecção de conflito:** ao salvar, comparar o hash do arquivo em disco com o hash lido antes da edição. Se mudou, não sobrescrever: recarregar, mostrar a diferença e deixar o autor escolher.
3. **Observador de arquivos:** o app recarrega cartões alterados por fora, com um pequeno intervalo para agrupar eventos.
4. **Preservação:** ler e regravar um cartão sem alterar nada deve produzir um arquivo idêntico ao original, byte a byte. Alterar um campo só muda aquele campo.
5. **Índice em memória:** o app e o MCP montam um índice em memória a partir dos arquivos. Com algumas centenas de cartões isso é instantâneo; um cache em disco só entra se medições mostrarem necessidade.
6. **Arquivos inválidos:** um cartão com cabeçalho quebrado não derruba o app. Ele aparece marcado como inválido, com o erro e um botão para abrir o arquivo, e nunca é regravado automaticamente.
7. **Cópia de segurança** antes de qualquer migração de formato.

---

## 15. Testes

### 15.1 Núcleo (`pauta-core`)

- **Ida e volta:** ler e regravar sem mudanças produz arquivos idênticos; teste de propriedade com `proptest` sobre cabeçalhos e corpos gerados, incluindo campos desconhecidos e Markdown variado.
- **Edição pontual:** alterar um campo altera só aquele campo.
- **Ordenação fracionária:** inserções repetidas entre dois cartões nunca colidem.
- **Agenda:** cálculo do próximo horário livre, lembretes com antecedência, horário de silêncio, lembretes perdidos e datas de dia inteiro, sempre com relógio fixo e fuso explícito.
- **Validação e migração:** arquivos válidos, inválidos e de versões antigas.
- **Conflitos:** gravação com arquivo alterado no meio do caminho é recusada.

### 15.2 Conector MCP

Testes de integração que iniciam o binário `pauta-mcp` sobre uma pasta temporária, chamam cada ferramenta pelo protocolo e conferem os arquivos resultantes. Inclui as garantias da seção 11.4 (nenhuma exclusão definitiva, log de escrita).

### 15.3 Interface

- Testes de componentes com Vitest.
- Testes de ponta a ponta no app real com `tauri-driver` no Linux: criar quadro, criar e mover cartão pelo teclado e pelo mouse, agendar post, ver no calendário, receber o lembrete com relógio simulado.
- Verificações automáticas de acessibilidade.

### 15.4 Integração contínua

GitHub Actions em Ubuntu com formatação, lint (`clippy`, ESLint), verificação de tipos e todos os testes acima.

---

## 16. Marcos

**[RECOMENDAÇÃO]** A versão 0 é o conjunto dos marcos 0 a 5. Cada marco é testável isoladamente. **A partir do marco 2, o autor passa a usar o Pauta para organizar o próprio desenvolvimento do Pauta**, o que serve de teste diário e evita construir sem usar.

### Marco 0: Esqueleto

Workspace Cargo, projeto Tauri com Svelte, integração contínua, licenças, ADR inicial com as decisões deste documento e o formato de dados documentado.

**Aceite:** a integração contínua roda verde e o app abre uma janela vazia.

### Marco 1: Núcleo de dados

`pauta-core` com modelo, leitura e gravação, preservação de campos, ordenação, validação, gravação atômica e detecção de conflito. Uma linha de comando mínima de depuração (listar quadros e cartões) ajuda a testar.

**Aceite:** os testes de ida e volta e de propriedade passam; uma pasta de exemplo com quadros, cartões e eventos é lida e regravada sem nenhuma alteração.

### Marco 2: Quadros

Abas, tipos e modelos de quadro, colunas editáveis, cartões com arrastar e soltar e equivalente pelo teclado, painel de edição com Markdown, etiquetas, lista de verificação, arquivamento, lixeira e observador de arquivos.

**Aceite:** o autor organiza o desenvolvimento do Pauta num quadro de projeto e mantém um quadro de conteúdo; uma edição feita num editor de texto externo aparece no app sem reiniciar.

### Marco 3: Calendário e tela "Hoje"

Eventos avulsos, prazos dos cartões, publicações por rede, visões de mês, semana e agenda, reagendamento por arrastar e tela "Hoje".

**Aceite:** as datas de entrega do TCC2 cadastradas como eventos e os posts agendados aparecem juntos no calendário e na tela "Hoje".

### Marco 4: Agendamento e notificações

Faixas por rede, sugestão de próximo horário livre, lembretes, adiamento, horário de silêncio, bandeja, inicialização automática, captura rápida de ideia, lembretes perdidos, ações de postagem manual (copiar texto, marcar publicado) e aviso de fila vazia.

**Aceite:** com a janela fechada, a notificação de um post agendado chega no horário; ao ligar o computador depois de um horário perdido, o lembrete aparece como atrasado.

### Marco 5: Conector MCP

Binário `pauta-mcp` com as ferramentas da seção 11.3, log de escrita, testes de integração e o trecho de configuração na tela de configurações.

**Aceite:** a partir do Claude Code (e do Claude Desktop, se disponível no Linux), o autor cria uma ideia, escreve o rascunho, agenda o post e consulta o resumo do dia, e as mudanças aparecem no app aberto.

### Marco 6 em diante: evolução

Metas de cadência, histórico de publicações e os itens da seção 17, na ordem que o autor decidir.

### Marco 7: Tela Início

Tela Início como primeira tela, com os sete widgets da seção 8.4. O Hoje passa a aba do Calendário, e o app volta ao Início na primeira abertura de cada novo dia. Só interface: nenhum comando novo (ADR 0009).

**Aceite:** o app abre no Início; na janela padrão os sete widgets cabem sem rolagem; cada "Ver mais" leva ao detalhe certo; marcar como publicado, copiar o texto e guardar uma ideia funcionam a partir do Início; em um novo dia, a primeira abertura da janela leva a ele.

---

## 17. Evolução futura

Registrado para que a arquitetura não impeça; nada aqui entra na versão 0.

### 17.1 Dashboard de métricas

Visualizações, alcance e engajamento por post e por rede, e sugestão de melhores horários com base nos dados do próprio autor. Depende das APIs do LinkedIn e do Instagram, cujo acesso a métricas, sobretudo de perfis pessoais, pode ser restrito. **A verificar** antes de planejar. Alternativa sem API: o autor registra os números manualmente no cartão publicado, ou importa exportações das próprias plataformas.

### 17.2 Sugestões de temas em alta

Pode vir pelo MCP (o Claude pesquisa e cria ideias no quadro) antes de qualquer código próprio.

### 17.3 Notificação no celular

Uma opção barata seria exportar o calendário em formato iCalendar (.ics) para um calendário que o celular já sincroniza; outra, um serviço de notificações por push. Ambas exigem sair do modelo puramente local e ficam para depois.

### 17.4 Publicação automática

LinkedIn e Instagram têm APIs de publicação, com requisitos de tipo de conta, aprovação de aplicativo e permissões. **A verificar.** O modelo de publicações por rede (seção 7.2) já comporta um estado "publicando" e o registro do link.

### 17.5 Chatbot embutido e Obsidian

O chatbot embutido só faz sentido se surgir uma forma de acessar o contexto dos Projetos do Claude fora dos clientes oficiais. O Obsidian já funciona pela pasta, se o autor quiser (seção 5.6).

### 17.6 Relação com o VideoAutomation

Os dois projetos ficam separados agora, por decisão do autor, para que nenhum trave o outro. No futuro, o Pauta pode ganhar redes como TikTok e YouTube no modelo de publicações por cartão, e o pacote de exportação manual do VideoAutomation pode virar um cartão do Pauta. O ecossistema maior de organização de mídias, se vier, se constrói a partir dessas duas peças.

---

## 18. Riscos

| Risco | Impacto | Mitigação |
| --- | --- | --- |
| Construir a ferramenta virar o substituto de postar | O autor passa meses no Pauta e continua sem constância | Versão 0 enxuta; uso do Pauta desde o marco 2; enquanto não fica pronto, manter um ritmo mínimo de posts com o que já existe |
| Escopo crescer com a ampliação para produtividade geral | Versão 0 nunca termina | Seção 3 como fronteira; tudo o mais vai para a seção 17 |
| Perda de dados por edições simultâneas | Perda de textos de posts | Seção 14 |
| Notificação não chegar | Falha no objetivo central | Segundo plano, inicialização automática e lembretes perdidos (seções 9.4 e 9.5), com testes de ponta a ponta |
| Dependências do ecossistema MCP e do Claude Desktop no Linux | Segundo pé indisponível | O app funciona sozinho (invariante 3); o Claude Code cobre o uso local |
| Ritmo dos agentes maior que o aprendizado de Rust do autor | Código que o autor não entende | Revisões obrigatórias por marco e ADRs explicando escolhas de Rust (seção 19) |
| Já existirem ferramentas prontas (Trello, Notion, plugins de quadro do Obsidian) | Esforço sem diferencial | O diferencial é a combinação: arquivos locais, calendário único para posts e projetos, e operação pelo Claude via MCP. Vale reavaliar se algum dia isso deixar de ser verdade |

---

## 19. Processo de implementação

**[RECOMENDAÇÃO]**

- O autor usará a skill "Planeje Comigo" para fechar os pontos da seção 20 antes de iniciar.
- Trabalho em sprints, cada uma terminando num marco testável, com loops de desenvolvimento agêntico dentro de cada sprint.
- Passos pequenos, cada um terminando com testes verdes; ao fim de cada sprint, uma demonstração curta e um resumo do que mudou.
- Decisões não óbvias, inclusive escolhas de Rust que não sejam evidentes para quem está aprendendo (ownership, traits, tratamento de erros), registradas como ADRs curtos em `docs/adr/`.
- Lição trazida da revisão do Abstract-Tex: nenhum marco novo começa enquanto houver revisões do autor pendentes do marco anterior, e a documentação (README, ADRs, este documento) é mantida enxuta e alinhada ao estado real a cada sprint.

---

## 20. Pontos em aberto para a sessão "Planeje Comigo"

Em ordem aproximada de urgência:

1. **Stack:** confirmar Tauri 2, Rust e Svelte (13.1).
2. **Pasta de dados:** local padrão e se será versionada com Git (5.1).
3. **Modelo de publicação:** um cartão com várias redes ou um cartão por rede (7.2).
4. **Colunas iniciais** dos quadros de conteúdo e de projeto (6.2).
5. **Faixas de postagem** iniciais por rede e lembretes padrão (9.1, 9.2).
6. **Idioma da interface** (12.3).
7. **Biblioteca YAML** que preserve campos desconhecidos (13.2).
8. **SDK de MCP para Rust** e suporte do Claude Desktop no Linux (11.2, 11.5).
9. **Comportamento ao fechar a janela:** minimizar para a bandeja por padrão (9.4).
10. **Widget na área de trabalho:** manter como ideia futura ou descartar (8.3).
11. **Publicação no GitHub** e licença (13.3).
12. **Primeiros quadros de projeto** a criar (Dungeon Meister, Abstract-Tex, TCC2, VideoAutomation e outros).

---

## 21. Glossário português e inglês

| Português | Inglês |
| --- | --- |
| pasta de dados | data root, workspace |
| quadro | board |
| aba | tab |
| coluna | column |
| cartão | card |
| chave de ordenação | rank |
| etiqueta | tag |
| lista de verificação | checklist |
| cabeçalho (do arquivo) | frontmatter |
| publicação | publication |
| rede | network |
| faixa de postagem | posting slot |
| próximo horário livre | next free slot |
| lembrete | reminder |
| adiar | snooze |
| horário de silêncio | quiet hours |
| lembrete perdido | missed reminder |
| evento de calendário | calendar event |
| prazo | deadline |
| marco | milestone |
| lançamento | launch |
| tela "Início" | home screen |
| tela "Hoje" | today view |
| captura rápida | quick capture |
| bandeja | system tray |
| arquivar | archive |
| lixeira | trash |
| meta de cadência | cadence goal |
| fila vazia | empty queue |
| conector MCP | MCP server |
| ferramenta (MCP) | tool |

---

## 22. Referências

- **Model Context Protocol:** especificação e SDKs oficiais em https://modelcontextprotocol.io
- **Tauri 2:** documentação e plugins oficiais (notificações, bandeja, inicialização automática, atalhos globais) em https://tauri.app
- **Documentação do Claude Code** sobre servidores MCP locais, e documentação do Claude Desktop sobre configuração de servidores MCP.
- **Formato de propriedades do Obsidian** (YAML no topo do Markdown), para manter a compatibilidade da seção 5.6.
- **Documentos relacionados neste Projeto:** "Handoff VideoAutomation 2.0", "Handoff Dungeon Meister 2.0" e "Comunicado de revisão Abstract-Tex 2026-10", este último pelas lições de processo da seção 19.
