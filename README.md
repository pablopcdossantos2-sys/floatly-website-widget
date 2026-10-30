# Widget Website — Windows

**Idiomas / Languages / Idiomas:** Português (Brasil) · [English](README.en.md) · [Español](README.es.md)

Navegador compacto para manter um endereço escolhido pelo usuário em uma janela flutuante WebView2 no desktop do Windows. O endereço é configurado na primeira execução; nenhum site de terceiros vem predefinido. Projeto Tauri 2 para Windows 10/11 x64.

## Recursos
- Janela sem barra de título, bandeja do sistema, instância única e atalho global padrão CTRL+ALT+C. Fechar pela tecla/ botão de fechar esconde o widget na bandeja; use Sair para encerrar o processo.
- Sessão WebView2 persistente em %APPDATA%\WidgetWebsite\webview-data; configuração em %APPDATA%\WidgetWebsite\config.json.
- Interface em português do Brasil, inglês ou espanhol, selecionável nas configurações.
- Uma pequena aba na parte superior central permite arrastar o widget. As áreas de interação nas bordas continuam ativas, mas são transparentes e não desenham uma moldura ao redor do conteúdo. O tamanho e a posição podem ser alterados nas configurações.
- O tamanho padrão é 350 × 550 px e o zoom padrão do navegador embutido é 80%; ambos podem ser alterados nas configurações.
- URL, tamanho, posição, travamento, sempre no topo, barra de tarefas, zoom, cantos e recarga automática configuráveis.
- Links externos encaminhados ao navegador padrão; domínios de autenticação reconhecidos abrem em janela secundária com uma barra de endereço simples e a pasta de dados compartilhada do WebView2. A janela tenta fechar quando o fluxo retorna ao endereço configurado após a autenticação.
- Workflow de compilação Windows x64 e instalador NSIS.
- Preparação inicial para a Microsoft Store documentada em [MICROSOFT_STORE_READINESS.md](MICROSOFT_STORE_READINESS.md); a submissão ainda depende das pendências descritas nesse documento.

## Pré-requisitos
- Windows 10/11 x64, Node.js 22 ou mais recente (LTS) e npm.
- Rust estável 1.90 ou mais recente.
- Microsoft Edge WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/. O aplicativo verifica as pastas usuais do Runtime e mostra esse endereço se não o encontrar.
- Para compilar localmente: Visual Studio Build Tools com Desenvolvimento para desktop com C++ e Windows SDK.

## Desenvolvimento
Execute no PowerShell (a geração dos ícones é necessária na primeira compilação):
    npm ci
    npm run icons
    npm run tauri dev

## Instalar / empacotar
Execute:
    npm ci
    npm run icons
    npm run tauri build

O instalador aparece em src-tauri/target/release/bundle/nsis e o executável em src-tauri/target/release. O workflow Build Windows compila a saída no GitHub Actions e publica-a como artefato.

## Primeira execução

Na primeira abertura, o widget pede que você escolha um site. Clique em **Escolher site** e informe uma URL em **Configurações → URL do site**. O campo começa vazio: nenhum website de terceiros vem configurado por padrão. O site precisa permitir o carregamento no WebView2; alguns sites podem restringir recursos específicos ou fluxos de login.

Para um exemplo concreto de URL, consulte a seção de exemplo abaixo.

## Microsoft Store

O processo de preparação e publicação está documentado no [guia passo a passo da Microsoft Store](GUIA_PUBLICACAO_MICROSOFT_STORE.md). O [rascunho de listagem multilíngue](MICROSOFT_STORE_LISTING_DRAFT.md) ajuda a preparar descrição e screenshots.

O build Store é manual: configure o publisher real e os segredos do certificado antes de executar `npm run build:store` ou o workflow **Build Microsoft Store package**. A existência do workflow não significa que o produto já esteja pronto ou aprovado para publicação.

## Idioma da interface

Abra **Configurações → Idioma** e escolha Português (Brasil), English ou Español. A escolha é salva em `%APPDATA%\\WidgetWebsite\\config.json` e afeta a janela de configurações, menus da bandeja, a janela auxiliar de login e as mensagens próprias do widget. O conteúdo do site escolhido continua no idioma fornecido pelo próprio site.

## Assinatura digital e alerta do Windows

A compilação no GitHub Actions oferece assinatura Authenticode opcional. Para ativá-la, obtenha um **certificado confiável de assinatura de código para Windows** e crie estes dois segredos nas configurações do repositório, em **Settings → Secrets and variables → Actions**:

- `WINDOWS_PFX_BASE64`: conteúdo do arquivo PFX codificado em Base64.
- `WINDOWS_PFX_PASSWORD`: senha desse arquivo PFX.

A chave privada não deve ser colocada no repositório nem no README. A compilação roda sem acesso ao certificado. Em um job separado, o workflow usa `signtool` para assinar e verificar o instalador NSIS e o executável publicado separadamente, com SHA-256 e timestamp; o arquivo PFX temporário é removido ao final. O executável distribuído separadamente é assinado, mas a cópia já embutida no instalador foi empacotada sem assinatura. Sem os dois segredos, os artefatos são publicados sem assinatura.

**Importante:** não é possível prometer que o Microsoft Defender SmartScreen nunca exibirá um aviso. Mesmo uma aplicação assinada por um certificado válido pode receber um aviso até que a reputação do editor e do arquivo se estabeleça. Publicar pela Microsoft Store é a forma mais direta de evitar avisos de download do SmartScreen. Consulte a [documentação de reputação do SmartScreen](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) e o [guia de assinatura do Tauri para Windows](https://tauri.app/distribute/sign/windows/).

## Exemplo de uso: Investing.com

O Investing.com é apenas um exemplo de site que pode ser exibido. Para usá-lo, abra **Configurações → URL do site** e informe `https://br.investing.com/economic-calendar`. O widget não é exclusivo desse serviço; você pode configurar outro site HTTP ou HTTPS.

Para sites que exigem login, use o fluxo de autenticação disponibilizado pelo próprio site. O menu da bandeja inclui a opção de entrar ou configurar o endereço do site.

O programa não lê, copia ou grava senhas. O User-Agent padrão do WebView2 é preservado.

## Diagnóstico de congelamentos

- O aplicativo grava registros persistentes em `%APPDATA%\WidgetWebsite\diagnostics.log` (normalmente, `C:\Users\<usuário>\AppData\Roaming\WidgetWebsite\diagnostics.log`). As datas do log estão em UTC. O arquivo anterior é rotacionado para `diagnostics.log.1` quando o log atinge aproximadamente 1 MB.
- A página envia um sinal de atividade a cada 5 segundos. Se o sinal deixar de chegar por 20 segundos, o monitor registra um aviso, verifica se a janela nativa responde e procura processos `msedgewebview2.exe` descendentes do widget. A ausência desses processos é uma pista de que o WebView2 pode ter sido encerrado, não uma prova de quem causou isso.
- O log inclui início do app, detecção do WebView2 Runtime, navegações, falhas de inicialização e recuperação da atividade. URLs são registradas sem parâmetros de consulta ou fragmentos para reduzir o risco de guardar tokens de sessão.
- As mensagens também são enviadas à saída padrão quando há um console anexado. Em builds gráficos do Windows, o terminal pode não estar disponível; por isso, o arquivo de log é a fonte confiável. Abra o arquivo com o Bloco de Notas depois de reproduzir o congelamento.
- O diagnóstico distingue ausência de atividade JavaScript de uma janela nativa sem resposta, mas não consegue provar sozinho que um aplicativo externo causou o congelamento. O monitor também não pode gravar novas mensagens se o processo inteiro for encerrado abruptamente ou deixar de executar todos os seus threads.

## Privacidade, decisões e limitações
- Sem telemetria. A navegação principal é limitada ao site configurado e a recursos de autenticação reconhecidos; os links para outros domínios abrem no navegador padrão. A permissão concedida ao site remoto é limitada ao sinal autenticado de atividade.
- Arquivo de configuração corrompido é substituído por valores padrão. O botão “Restaurar tamanho padrão” muda somente largura e altura para 350 × 550 px; as outras preferências são preservadas.
- A CI verifica a compatibilidade das versões Tauri, o manifesto de permissões e as capabilities; executa `npm ci`, verificação de tipos, `npm audit`, `cargo fmt --check`, `cargo clippy`, `cargo audit` e `cargo test`; por fim, compila o instalador no Windows.
- A janela de login secundária compartilha a pasta de dados do WebView2 e inclui uma barra de endereço injetada. A navegação para domínios não autorizados continua bloqueada dentro do app e abre no navegador padrão. O fechamento automático depende de o provedor retornar ao caminho configurado; alguns fluxos sociais podem exigir fechar a janela manualmente.
- O zoom é aplicado pelo controle nativo de ampliação do WebView2, afetando a renderização da página de forma semelhante ao zoom de um navegador. O recorte dos cantos pode variar no Windows 10.
- A verificação de rede mostra uma tela própria antes da navegação e monitora a conexão enquanto o site está aberto. O wrapper padrão do Tauri não fornece um evento portátil que permita substituir toda falha de navegação do Chromium em todas as situações.
- É necessário testar login real, filtros, atalho em conflito, escalas 125%/150%, monitores múltiplos, DPI e consumo de memória em Windows. O ambiente de desenvolvimento atual não permite comprovar esses comportamentos gráficos.

## Desinstalação e dados

O desinstalador NSIS oferece a opção **Delete the application data**. Quando marcada, ela remove `%APPDATA%\\WidgetWebsite` (configuração, logs e sessão WebView2) e `%LOCALAPPDATA%\\WidgetWebsite`, se existir. Se a opção ficar desmarcada, esses dados são preservados para uma futura reinstalação. A entrada de início automático do Windows é removida sempre que o aplicativo é desinstalado.

## Estrutura
- src-tauri/src: configuração, janela, comandos, diagnósticos e bandeja em Rust.
- src: inicialização e painel de configurações em TypeScript.
- icons/widget.svg: ícone-fonte vetorial usado para gerar os ícones nativos do Windows.