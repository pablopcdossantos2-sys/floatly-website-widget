# Widget Website — Windows

Widget desktop sem moldura para abrir o calendário econômico do Investing.com em uma pequena janela WebView2, com sessão persistente.

> O repositório foi inicializado com Tauri 2. O alvo principal é Windows 10/11 de 64 bits; a validação gráfica final e o primeiro login precisam ser confirmados em um computador Windows.

## Recursos previstos

- Janela compacta sem barra de título, bandeja do sistema e instância única.
- Sessão persistente em `%APPDATA%\WidgetWebsite\webview-data`; o app não armazena senhas.
- Configurações em `%APPDATA%\WidgetWebsite\config.json`.
- Atalho global, inicialização com o Windows, opacidade, zoom, posição, tamanho, bloqueio e sempre no topo.
- Redirecionamento de links para o navegador padrão e fluxo de autenticação dentro do WebView.
- Build para Windows x64 e instalador NSIS via GitHub Actions.

## Pré-requisitos

- Windows 10/11 x64.
- Node.js LTS e npm.
- Rust estável (o toolchain atual dos plugins Tauri 2 exige Rust 1.90 ou superior).
- Microsoft Edge WebView2 Runtime. Se estiver ausente, instale o [Evergreen Runtime oficial](https://developer.microsoft.com/microsoft-edge/webview2/).
- Para desenvolvimento local, Visual Studio Build Tools com a carga de trabalho **Desenvolvimento para desktop com C++** e Windows SDK.

## Desenvolvimento

```powershell
npm install
npm run tauri dev
```

## Gerar instalador

O ícone-fonte fica em `icons/widget.svg`. Gere os formatos nativos antes do empacotamento:

```powershell
npm install
npm run icons
npm run tauri build
```

O instalador NSIS e o executável ficam em `src-tauri/target/release/bundle/nsis` e `src-tauri/target/release`. Também há um workflow do GitHub Actions que compila para Windows x64.

## Primeiro login

1. Inicie o widget e aguarde o calendário carregar.
2. Entre preferencialmente com e-mail e senha em uma conta Investing.com. A janela de autenticação pode ser aberta pelo menu da bandeja.
3. Refaça filtros se eles não forem sincronizados pela conta ou se estiver usando uma conta nova.
4. Feche e reabra o widget para conferir a persistência. Os dados do WebView2 não são apagados ao fechar.

A sessão pode expirar por decisão do site. Use **Entrar / mostrar janela de login**, autentique-se e retorne ao widget. O aplicativo não lê, copia nem salva senhas.

## Privacidade e limitações

- Sem telemetria. A navegação do widget é destinada ao site configurado e a links de autenticação relacionados; links externos comuns são encaminhados ao navegador padrão.
- O User-Agent padrão do WebView2 é preservado.
- A tela de configuração não armazena credenciais.
- O suporte de arredondamento e a nitidez das bordas podem variar entre Windows 10, Windows 11 e configurações de DPI.
- O build automatizado não substitui o teste manual em Windows: login do Investing.com, autenticação social, monitores múltiplos, escala de 125%/150% e comportamento em suspensão devem ser validados na máquina-alvo.
