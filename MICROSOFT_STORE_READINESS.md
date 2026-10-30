# Microsoft Store — estado de preparação

**Última revisão:** 29 de outubro de 2026  
**Status:** <strong>em preparação; não submeter ainda</strong>.

O tutorial completo, com as tarefas do desenvolvedor, os comandos, as verificações e o fluxo do Partner Center, está em [GUIA_PUBLICACAO_MICROSOFT_STORE.md](GUIA_PUBLICACAO_MICROSOFT_STORE.md). O conteúdo de apresentação está em [MICROSOFT_STORE_LISTING_DRAFT.md](MICROSOFT_STORE_LISTING_DRAFT.md).

## O que já foi preparado no repositório

- Produto enquadrado como navegador compacto Win32/WebView2 para um endereço escolhido pelo usuário; não é cliente de um serviço específico. Investing.com permanece apenas como exemplo de uso.
- URL vazia para novas instalações e assistente de primeira execução.
- Configuração adicional `src-tauri/tauri.microsoftstore.conf.json` usando WebView2 `offlineInstaller`.
- Verificador `npm run check:store-config`, integrado ao CI normal e ao workflow Store.
- Script `npm run build:store`: assina o executável, materializa o toolset NSIS, assina os DLLs originais da pasta `Plugins` usada pelo `makensis`, reconstrói o pacote, usa o `signCommand` do Tauri para o desinstalador e o setup externo e valida com 7-Zip a assinatura de todos os PE extraídos. Gera `SHA256SUMS.txt` e interrompe o build se algum PE estiver sem assinatura válida.
- Workflow manual **Build Microsoft Store package**, executável na branch `main`, que publica o instalador assinado como artefato temporário do Actions por 14 dias.

O script exige um valor real em `bundle.publisher`, além dos segredos `WINDOWS_PFX_BASE64` e `WINDOWS_PFX_PASSWORD`. Ele falha sem esses dados. O artefato do Actions não é uma URL permanente e não deve ser usado diretamente como endereço do pacote no Partner Center.

## Bloqueios que ainda dependem de decisão ou ação humana

- [x] Conta Partner Center existente e já utilizada para publicar outro widget; não é necessário repetir o cadastro.
- [ ] Reservar o nome do Widget Website como novo produto em Apps and Games.
- [ ] Definir a identidade real do publisher e configurar `bundle.publisher` em `src-tauri/tauri.conf.json`.
- [ ] Obter certificado Authenticode confiável e registrar os dois segredos exigidos pelo build Store.
- [ ] Implementar e testar o Tauri Updater. Atualmente não existem plugin, chaves, endpoint nem artefatos assinados para atualização.
- [ ] Executar o workflow Store com publisher e certificado reais.
- [ ] Validar em Windows limpo que o WebView2 Evergreen Runtime está atualizado, que a versão Chromium carregada está dentro do limite de duas versões principais da regra 10.2.1 e que o aplicativo adota a versão nova após reiniciar.
- [ ] Testar assinatura, instalação silenciosa, atualização do aplicativo, desinstalação, Adicionar/Remover Programas e execução em uma máquina Windows limpa.
- [ ] Hospedar o EXE assinado em URL HTTPS direta, versionada, pública e imutável.
- [ ] Revisar e finalizar a [política de privacidade](PRIVACY_POLICY.md): preencher nome legal do controlador e e-mail de privacidade, confirmar as práticas descritas e só então informar a URL pública no Partner Center (https://github.com/pablopcdossantos2-sys/widget-website/blob/main/PRIVACY_POLICY.md).
- [ ] Finalizar screenshots reais, classificação etária e contato de suporte da listagem.
- [ ] Executar Package Validation no Partner Center, corrigir eventuais falhas e enviar para certificação.

## Lembrete técnico

O caminho atual é **EXE/MSI**, com instalador NSIS EXE para um navegador compacto Win32/WebView2. A regra 10.1.1 sobre publicação pelo proprietário do domínio aplica-se a produtos enviados como web apps; não descreve o modelo deste aplicativo genérico, que permite ao usuário escolher o destino. Como navegador, o produto deve observar a regra 10.2.1 sobre Chromium/Gecko e atualidade do mecanismo. Para EXE/MSI, a Microsoft também exige instalador independente/offline, assinatura do instalador e de todos os arquivos PE incluídos; o build inspeciona o pacote extraído e gera um manifesto, URL HTTPS versionada e instalação silenciosa. Para o instalador NSIS deste projeto, o parâmetro silencioso indicado pelo Tauri é `/S`. O script exige 7-Zip para inspecionar o conteúdo real do EXE e falha se encontrar PE sem assinatura Authenticode válida.

A política Microsoft Store 7.20 foi publicada em 15 de setembro de 2026 e entrou em vigor em 22 de outubro de 2026. Para este produto, o requisito técnico diretamente relevante é a regra 10.2.1 para aplicativos que navegam na Web: usar Chromium ou Gecko e manter o mecanismo dentro de duas versões principais. O Tauri utiliza WebView2/Chromium; a configuração Store usa o instalador offline do Evergreen Runtime, e não uma versão fixa do mecanismo. O Runtime recebe atualizações automáticas, embora uma instância já aberta possa precisar ser reiniciada para adotar a atualização.

Consulte as fontes e o procedimento detalhado no [guia](GUIA_PUBLICACAO_MICROSOFT_STORE.md).
