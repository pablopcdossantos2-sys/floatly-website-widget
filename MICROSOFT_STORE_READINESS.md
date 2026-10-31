# Microsoft Store — prontidão do Floatly (MSIX)

**Última revisão:** 31 de outubro de 2026  
**Formato:** MSIX x64. Não usar o fluxo EXE/MSI.

## Identidade confirmada no Partner Center

- Package/Identity/Name: `pablopcdsantos.Floatly`
- Package/Identity/Publisher: `CN=A3E973B8-1411-4314-B754-F3DADD99C070`
- Package/Properties/PublisherDisplayName: `pablopcdsantos`

Os três valores já estão definidos como padrões do workflow. O build MSIX é acionado automaticamente a cada push para `main`.

## Próxima ação humana

- [ ] Aguardar o workflow **Build MSIX for Microsoft Store** concluir e baixar o artefato `floatly-msix-x64`.
- [ ] Enviar o arquivo .msix ao Partner Center e executar a validação do pacote.
- [ ] Finalizar listagem, screenshots reais e notas de certificação.

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
