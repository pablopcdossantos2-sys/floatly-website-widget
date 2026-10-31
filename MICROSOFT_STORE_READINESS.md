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

## Correções implementadas após os relatórios

- Política CSP alinhada ao suporte anunciado a sites HTTP e HTTPS.
- Manifesto MSIX atualizado para Windows 10 build 19041 (versão 2004) como mínimo, em razão dos recursos uap10 utilizados.
- Job de assinatura NSIS não é executado em pull_request; os testes de compilação continuam disponíveis nesses eventos.
- Preferência de atalho do usuário preservada quando o atalho alternativo é necessário temporariamente; combinações reservadas pelo Windows são rejeitadas.
- Rotas de autenticação menos ambíguas, limites de abertura externa, validação de origem interna e emissão restrita de eventos de configuração.
- Limpeza da URL retorna à tela inicial; backups de configuração e redução de dados sensíveis nos logs implementados.
- Testes automatizados de scripts incluídos nos dois workflows; upgrades minor/major de Tauri foram reservados para atualização coordenada.

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
- [ ] Instalar o MSIX em Windows de teste, abrir o aplicativo e localizar o caminho efetivo dos dados e logs. Confirmar se o Windows redireciona %APPDATA%/WidgetWebsite para a área privada do pacote; só depois fechar a redação definitiva da Política de Privacidade.
- [ ] Conferir as importações do executável com dumpbin /dependents e testar a instalação em uma VM Windows limpa sem pressupor a presença do Visual C++ Redistributable.
- [ ] Testar sites HTTP e HTTPS, incluindo URL com porta explícita, e confirmar que o bootstrap local não exibe falso aviso de falta de conexão.
- [ ] Limpar a URL nas Configurações e confirmar que a janela retorna à tela local Escolher site.
- [ ] Testar páginas comuns como /account/profile, /accounts/overview, /docs/auth e fluxos OAuth com parâmetros de autorização/callback.
- [ ] Testar conflitos de atalhos, combinações reservadas, alternância de idioma e se o atalho preferido permanece salvo após um conflito temporário.
- [ ] Minimizar o widget por mais de cinco minutos e confirmar que o watchdog não registra falso aviso; testar também uma página realmente sem heartbeat.
- [ ] Testar arraste com cliques rápidos e botões do mouse invertidos, escalas de 125%/150% e mais de um monitor.
- [ ] Em uma instalação NSIS, habilitar a inicialização automática, verificar a entrada real em HKCU/Software/Microsoft/Windows/CurrentVersion/Run, desinstalar e confirmar que a entrada foi removida.

**Não é necessário comprar certificado Code Signing, cadastrar segredos PFX ou gerar chave do Tauri Updater para o canal Microsoft Store MSIX.**
