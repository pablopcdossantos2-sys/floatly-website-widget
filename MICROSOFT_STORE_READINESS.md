# Microsoft Store — estado de preparação

**Última revisão:** 29 de outubro de 2026  
**Status:** <strong>em preparação; não submeter ainda</strong>.

O tutorial completo, com as tarefas do desenvolvedor, os comandos, as verificações e o fluxo do Partner Center, está em [GUIA_PUBLICACAO_MICROSOFT_STORE.md](GUIA_PUBLICACAO_MICROSOFT_STORE.md). O conteúdo de apresentação está em [MICROSOFT_STORE_LISTING_DRAFT.md](MICROSOFT_STORE_LISTING_DRAFT.md).

## O que já foi preparado no repositório

- Produto descrito como widget desktop genérico; Investing.com só aparece nos READMEs como exemplo.
- URL vazia para novas instalações e assistente de primeira execução.
- Configuração adicional `src-tauri/tauri.microsoftstore.conf.json` usando WebView2 `offlineInstaller`.
- Verificador `npm run check:store-config`, integrado ao CI normal e ao workflow Store.
- Script `npm run build:store`: assina o executável, materializa o toolset NSIS, assina os DLLs originais da pasta `Plugins` usada pelo `makensis`, reconstrói o pacote, usa o `signCommand` do Tauri para o desinstalador e o setup externo e valida com 7-Zip a assinatura de todos os PE extraídos. Gera `SHA256SUMS.txt` e interrompe o build se algum PE estiver sem assinatura válida.
- Workflow manual **Build Microsoft Store package**, executável na branch `main`, que publica o instalador assinado como artefato temporário do Actions por 14 dias.

O script exige um valor real em `bundle.publisher`, além dos segredos `WINDOWS_PFX_BASE64` e `WINDOWS_PFX_PASSWORD`. Ele falha sem esses dados. O artefato do Actions não é uma URL permanente e não deve ser usado diretamente como endereço do pacote no Partner Center.

## Bloqueios que ainda dependem de decisão ou ação humana

- [ ] Confirmar com a Microsoft a elegibilidade de um utilitário Win32 que exibe sites escolhidos pelo usuário, considerando a regra de web apps publicados pelo dono do domínio e as regras sobre conteúdo de terceiros.
- [ ] Criar/confirmar conta Partner Center e reservar o nome do produto.
- [ ] Definir a identidade real do publisher e configurar `bundle.publisher` em `src-tauri/tauri.conf.json`.
- [ ] Obter certificado Authenticode confiável e registrar os dois segredos exigidos pelo build Store.
- [ ] Implementar e testar o Tauri Updater. Atualmente não existem plugin, chaves, endpoint nem artefatos assinados para atualização.
- [ ] Executar o workflow Store com publisher e certificado reais.
- [ ] Testar assinatura, instalação silenciosa, atualização, desinstalação, Adicionar/Remover Programas e execução em uma máquina Windows limpa.
- [ ] Hospedar o EXE assinado em URL HTTPS direta, versionada, pública e imutável.
- [ ] Finalizar listagem, screenshots reais, categoria, classificação etária, suporte e política de privacidade.
- [ ] Executar Package Validation no Partner Center, corrigir eventuais falhas e enviar para certificação.

## Lembrete técnico

O caminho atual é **EXE/MSI**, com instalador NSIS EXE. Para esse tipo de produto, a Microsoft exige instalador independente/offline, assinatura do instalador e de todos os arquivos PE incluídos; o build inspeciona o pacote extraído e gera um manifesto, URL HTTPS versionada e instalação silenciosa. Para o instalador NSIS deste projeto, o parâmetro silencioso indicado pelo Tauri é `/S`. O script exige 7-Zip para inspecionar o conteúdo real do EXE e falha se encontrar PE sem assinatura Authenticode válida.

A política Microsoft Store 7.20 foi publicada em 15 de setembro de 2026 e entrou em vigor em 22 de outubro de 2026. O aceite do produto não está garantido apenas por ser distribuído como EXE.

Consulte as fontes e o procedimento detalhado no [guia](GUIA_PUBLICACAO_MICROSOFT_STORE.md).
