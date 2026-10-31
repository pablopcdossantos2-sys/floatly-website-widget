# Microsoft Store — prontidão do Floatly (MSIX)

**Última revisão:** 30 de outubro de 2026  
**Formato:** MSIX x64. Não usar o fluxo EXE/MSI.

## Próxima ação humana

- [ ] Criar uma nova ficha em Partner Center → Apps and games → New product → **MSIX or PWA app** e reservar o nome Floatly.
- [ ] Iniciar a submissão e copiar, em Product management → Product identity, o Package/Identity Name, Publisher e Publisher display name.
- [ ] Informar esses três valores ao workflow **Build MSIX for Microsoft Store** na branch main e baixar o artefato floatly-msix-x64.

Siga o [guia passo a passo](GUIA_PUBLICACAO_MICROSOFT_STORE.md). Não escolha EXE/MSI.

## Trabalho técnico preparado no repositório

- Manifesto MSIX com identidade preenchida no momento do build.
- Pacote sem assinatura Authenticode de terceiros; a Microsoft assina o MSIX depois da aprovação.
- Workflow Windows x64 que compila, verifica as permissões e gera o MSIX.
- Extensão windows.startupTask e integração Rust com a API StartupTask para manter a opção de iniciar com o Windows.
- runFullTrust declarado para o aplicativo desktop Tauri.

## O que ainda precisa de validação

- [ ] Executar o workflow com os dados reais de identidade fornecidos pelo Partner Center.
- [ ] Confirmar que o MSIX é aceito no upload do Partner Center e passa pela validação.
- [ ] Concluir a revisão da listagem, capturas reais e notas de certificação.
- [ ] Submeter o pacote à certificação e testar a versão instalada pela Store em Windows 10/11 x64.
- [ ] Verificar a inicialização automática, bandeja, atalhos, janela flutuante, WebView2, atualização e desinstalação.

**Não é necessário comprar certificado Code Signing, cadastrar segredos PFX ou gerar chave do Tauri Updater para o canal Microsoft Store MSIX.**
