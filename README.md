# Widget Website — Windows

Widget desktop para manter o calendário econômico do Investing.com aberto em uma pequena janela WebView2, sem moldura. Projeto Tauri 2 direcionado a Windows 10/11 x64.

## Recursos
- Janela sem barra de título, bandeja do sistema, instância única e atalho global padrão CTRL+ALT+C.
- Sessão WebView2 persistente em %APPDATA%\WidgetWebsite\webview-data; configuração em %APPDATA%\WidgetWebsite\config.json.
- URL, tamanho, posição, travamento, sempre no topo, barra de tarefas, opacidade, zoom, cantos e recarga automática configuráveis.
- Links externos encaminhados ao navegador padrão; domínios de autenticação reconhecidos abrem em janela secundária com uma barra de endereço simples e a pasta de dados compartilhada do WebView2. A janela tenta fechar quando o fluxo retorna ao endereço configurado após a autenticação.
- Workflow de compilação Windows x64 e instalador NSIS.

## Pré-requisitos
- Windows 10/11 x64, Node.js LTS e npm.
- Rust estável 1.90 ou mais recente.
- Microsoft Edge WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/
- Para compilar localmente: Visual Studio Build Tools com Desenvolvimento para desktop com C++ e Windows SDK.

## Desenvolvimento
Execute no PowerShell (a geração dos ícones é necessária na primeira compilação):
    npm install
    npm run icons
    npm run tauri dev

## Instalar / empacotar
Execute:
    npm install
    npm run icons
    npm run tauri build

O instalador aparece em src-tauri/target/release/bundle/nsis e o executável em src-tauri/target/release. O workflow Build Windows compila a saída no GitHub Actions e publica-a como artefato.

## Primeiro login
1. Abra o widget e aguarde o calendário carregar.
2. Prefira autenticação por e-mail e senha. O menu da bandeja oferece Entrar / mostrar janela de login.
3. Feche e abra o app para verificar a persistência. Filtros salvos podem depender da conta e talvez precisem ser refeitos em uma conta nova.
4. Se a sessão expirar, use Entrar / mostrar janela de login e autentique-se novamente.

O programa não lê, copia ou grava senhas. O User-Agent padrão do WebView2 é preservado.

## Privacidade, decisões e limitações
- Sem telemetria. A navegação principal é limitada ao site configurado e a recursos de autenticação conhecidos; links HTTP/HTTPS para outros domínios abrem no navegador padrão.
- Arquivo de configuração corrompido é substituído por valores padrão.
- A janela de login secundária compartilha a pasta de dados do WebView2 e inclui uma barra de endereço injetada. A navegação para domínios não autorizados continua bloqueada dentro do app e abre no navegador padrão. O fechamento automático depende de o provedor retornar ao caminho configurado; alguns fluxos sociais podem exigir fechar a janela manualmente.
- O zoom é aplicado pelo controle nativo de ampliação do WebView2, afetando a renderização da página de forma semelhante ao zoom de um navegador. Opacidade e recorte dos cantos continuam usando CSS; o arredondamento pode variar no Windows 10.
- A verificação de rede mostra uma tela própria antes da navegação e monitora a conexão enquanto o site está aberto. O wrapper padrão do Tauri não fornece um evento portátil que permita substituir toda falha de navegação do Chromium em todas as situações.
- É necessário testar login real, filtros, atalho em conflito, escalas 125%/150%, monitores múltiplos, DPI e consumo de memória em Windows. O ambiente de desenvolvimento atual não permite comprovar esses comportamentos gráficos.

## Estrutura
- src-tauri/src: configuração, janela, comandos e bandeja em Rust.
- src: inicialização e painel de configurações em TypeScript.
- icons/widget.svg: ícone-fonte vetorial usado para gerar os ícones nativos do Windows.