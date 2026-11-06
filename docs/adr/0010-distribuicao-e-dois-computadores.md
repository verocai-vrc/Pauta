# ADR 0010: Distribuição por `.deb` e dados sincronizados por Git

- **Data:** 2026-11-06
- **Estado:** aceita

## Contexto

O Pauta passa a ser instalado em dois computadores com Ubuntu 26.04 e a ter uma lista pública de versões no GitHub. Faltavam três decisões: o formato do pacote, como atualizar e como os dois computadores compartilham os dados. A seção 1.4 do documento de design deixa a sincronização de fora do aplicativo na versão 0, e isso continua valendo.

## Decisão

**Só `.deb`, gerado pelo CI.** Enviar uma tag `vX.Y.Z` roda o CI inteiro e publica o pacote como pré-lançamento. O AppImage saiu: ele não levaria o `pauta-mcp` e depende do `libfuse2`, que o Ubuntu não instala por padrão.

**`pauta-mcp` vai em `bundle.linux.deb.files`, não em `externalBin`.** O `externalBin` exige o binário já na compilação de qualquer `cargo build`, inclusive no clippy e nos testes, porque o `tauri-build` o procura. O `deb.files` só é lido ao empacotar. O custo é que o pacote precisa do `pauta-mcp` compilado antes, e por isso existe o `pnpm release:build`. O caminho instalado é `/usr/bin/pauta-mcp`, que a tela de Configurações já procurava.

**Sem atualização automática.** Para dois computadores, `sudo apt install ./Pauta_X.Y.Z_amd64.deb` basta. Um atualizador pediria chaves de assinatura e um manifesto publicado a cada versão, sem ganho real nessa escala.

**Dados num repositório Git privado, fora do app.** A pasta de dados já é texto, o `.gitignore` já exclui `.pauta/` e o observador de arquivos ignora `.git/`. Num conflito, o Git deixa marcadores no arquivo. Se eles caem no cabeçalho, o cartão aparece como inválido e nunca é regravado.

**Por que não Syncthing:** ele cria cópias de conflito com extensão `.md`, que o Pauta lê como um segundo cartão com o mesmo id, sem aviso.

**O custo:** a sincronização é manual, e esquecer um `git pull` gera conflitos. Como `.pauta/` não viaja, cada computador guarda seus próprios lembretes entregues, e os dois avisam se estiverem ligados.
