# Microsoft Store — estado de preparação

**Última revisão:** 30 de outubro de 2026  
**Status:** preparação avançada; ainda não submeter o pacote.

O procedimento completo está em [GUIA_PUBLICACAO_MICROSOFT_STORE.md](GUIA_PUBLICACAO_MICROSOFT_STORE.md). O texto de apresentação está em [MICROSOFT_STORE_LISTING_DRAFT.md](MICROSOFT_STORE_LISTING_DRAFT.md). Os canais públicos do projeto são a [política de privacidade](PRIVACY_POLICY.md) e a [página de suporte](SUPPORT.md).

## Situação confirmada no Partner Center

Com base nas telas compartilhadas e nas atualizações feitas pelo desenvolvedor:

- [x] Produto criado com o nome **Floatly** em Apps and Games. O nome não precisa mais ser reservado.
- [x] Disponibilidade configurada para 240 de 240 regiões, com descoberta habilitada na Microsoft Store.
- [x] Preço definido como gratuito.
- [x] Categoria principal definida como **Productivity / Produtividade**; categoria secundária deixada em branco.
- [x] Classificação etária concluída no questionário IARC.
- [x] URL da política de privacidade informada no Partner Center. O documento público agora identifica o responsável individual pelo projeto e informa o e-mail de privacidade.
- [x] Notas para certificação preenchidas com instruções para o avaliador configurar um endereço de teste na primeira execução.
- [x] Campos de requisitos de sistema deixados em branco, sem requisitos adicionais de RAM ou GPU sem validação.
- [x] Endereço residencial e telefone pessoal removidos dos dados de contato públicos.
- [x] Canal de e-mail mantido para contato de suporte.

**Ajuste recomendado no Partner Center:** atualizar o campo de site de suporte para apontar diretamente para [SUPPORT.md](https://github.com/pablopcdossantos2-sys/floatly-website-widget/blob/main/SUPPORT.md), em vez da raiz do repositório. O endereço de suporte é público; mantenha somente canais que possam ser monitorados.

## O que já foi preparado no repositório

- Produto descrito como navegador compacto Win32/WebView2 para um endereço escolhido pelo usuário; não é cliente de um serviço específico.
- Marca Floatly aplicada ao nome do pacote, à janela principal, à tela inicial, às configurações, aos menus da bandeja e aos textos da documentação.
- Identificador interno do Tauri e pastas locais legadas `WidgetWebsite` preservados para evitar quebrar atualizações e dados de instalações anteriores.
- URL vazia para novas instalações e assistente de primeira execução.
- Configuração adicional `src-tauri/tauri.microsoftstore.conf.json` usando WebView2 `offlineInstaller`.
- Verificador `npm run check:store-config`, integrado ao CI normal e ao workflow Store; a verificação agora exige o nome de produto Floatly.
- Script `npm run build:store`: assina o executável, materializa o toolset NSIS, assina os DLLs originais da pasta `Plugins` usada pelo `makensis`, reconstrói o pacote, usa o `signCommand` do Tauri para o desinstalador e o setup externo e valida com 7-Zip a assinatura de todos os PE extraídos. Gera `SHA256SUMS.txt` e interrompe o build se algum PE estiver sem assinatura válida.
- Workflow manual **Build Microsoft Store package**, que publica o instalador como artefato temporário do Actions.
- Política de privacidade em português, inglês e espanhol, com responsável e contato preenchidos.
- Página de suporte em português, inglês e espanhol, com orientação para primeira execução e compartilhamento seguro de logs.

## Bloqueios técnicos e decisões ainda pendentes

- [ ] Definir o valor de `bundle.publisher` somente depois de confirmar o nome que consta no certificado Authenticode e a identidade do publicador no Partner Center. O nome da pessoa responsável pela política de privacidade não deve ser copiado automaticamente para esse campo.
- [ ] Obter certificado Authenticode confiável e registrar os segredos `WINDOWS_PFX_BASE64` e `WINDOWS_PFX_PASSWORD` no GitHub Actions.
- [ ] Executar o workflow **Build Microsoft Store package** em Windows com publisher e certificado reais.
- [ ] Validar em Windows limpo que o WebView2 Evergreen Runtime está atualizado, que a versão Chromium carregada está dentro do limite de duas versões principais da regra 10.2.1 e que o aplicativo adota a versão nova após reiniciar.
- [ ] Testar instalação, execução, atualização, desinstalação, Adicionar/Remover Programas, DPI, escalas de 125%/150% e monitores múltiplos em uma máquina Windows limpa.
- [ ] Hospedar o EXE assinado em URL HTTPS direta, pública, versionada e imutável. Um artefato temporário do GitHub Actions não serve como URL permanente do pacote.
- [ ] Capturar screenshots reais da versão final em Windows e revisar os textos da listagem nos idiomas da Store.
- [ ] Atualizar o site de suporte no Partner Center para a página dedicada `SUPPORT.md`, se ainda estiver apontando para a raiz do repositório.
- [ ] Executar Package Validation no Partner Center, corrigir eventuais falhas e enviar para certificação.

## CI — situação observada

A execução **Build Windows** do commit `6e3b0fe6d12d6007bc7868834daa8275e0a99520` falhou em `cargo fmt --check`, por uma quebra de linha na função `show_login` de `src-tauri/src/commands.rs`. Essa formatação foi corrigida na `main`. A execução acionada após o conjunto atual de alterações deve ser consultada antes de considerar o CI aprovado.

## Lembrete técnico

O caminho atual é **EXE/MSI**, com instalador NSIS EXE para um navegador compacto Win32/WebView2. A regra 10.1.1 sobre publicação pelo proprietário do domínio aplica-se a produtos enviados como web apps; não descreve o modelo deste aplicativo genérico, que permite ao usuário escolher o destino. Como navegador, o produto deve observar a regra 10.2.1 sobre Chromium/Gecko e atualidade do mecanismo. Para EXE/MSI, a Microsoft também exige instalador independente/offline, assinatura do instalador e dos arquivos PE incluídos, instalação silenciosa e uma URL HTTPS direta para o pacote.

O mecanismo Tauri Updater ainda não foi integrado. Isso deve ser tratado como uma decisão de arquitetura de atualização e manutenção do produto, não como um requisito já cumprido ou um bloqueio automático de certificação. Definir como novas versões serão entregues e testadas antes da publicação continua sendo necessário.

Consulte as fontes oficiais e o procedimento detalhado no [guia](GUIA_PUBLICACAO_MICROSOFT_STORE.md).
