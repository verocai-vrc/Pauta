# ADR 0002: Edição cirúrgica do cabeçalho YAML

- **Data:** 2026-10-28
- **Estado:** aceita

## Contexto

A invariante 4 e a seção 14.4 exigem que ler e regravar um cartão sem mudanças produza o mesmo arquivo, byte a byte, e que alterar um campo mude só aquele campo, preservando campos desconhecidos e comentários. O `serde_yaml` foi arquivado, e nenhuma biblioteca YAML mantida para Rust faz ida e volta preservando comentários e formatação.

## Decisão

- **Leitura:** `serde_yaml_ng` (fork mantido do `serde_yaml`) converte o cabeçalho num `Mapping` só para extrair os campos.
- **Escrita:** o módulo `yaml.rs` trata o cabeçalho como texto e o divide em blocos, um por chave de primeiro nível. Mudar `column` substitui só as linhas do bloco `column:`; todo o resto do arquivo é copiado como estava. Uma chave nova é acrescentada no fim do cabeçalho.
- Valores aninhados (`publication`, `links`) são regravados mesclando o bloco existente com as mudanças (`fields::merge_map`), de modo que subchaves desconhecidas sobrevivem. Comentários *dentro* de um bloco reescrito se perdem; fora dele, nunca.
- O corpo Markdown nunca é re-renderizado; seções `##` são localizadas com `pulldown-cmark` e editadas por recorte de texto.
- O estilo de saída (listas em linha, títulos entre aspas, datas sem aspas, quebras CRLF preservadas) segue os exemplos do documento de design.

## Consequências

- Testes de propriedade (`tests/roundtrip_prop.rs`) geram cabeçalhos e corpos aleatórios e verificam a identidade da ida e volta e a preservação das outras chaves.
- O formatador próprio é pequeno (cerca de 150 linhas) e precisa ser mantido junto com o formato.

## Nota de Rust

`YamlText::set(&mut self, …)` recebe `&mut self` porque altera o texto no lugar. Quem chama precisa de posse exclusiva (`let mut doc`), e o compilador impede dois trechos de código de editarem o mesmo documento ao mesmo tempo.
