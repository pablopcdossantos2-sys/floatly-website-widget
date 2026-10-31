# Floatly — Windows

**Idiomas / Languages / Idiomas:** Português (Brasil) · [English](README-en.md) · [Español](README.es.md)

---

Navegador compacto para manter um endereço escolhido pelo usuário em uma janela flutuante WebView2 no desktop do Windows. O endereço é configurado na primeira execução; nenhum site de terceiros vem predefinido. Projeto Tauri 2 para Windows 10/11 x64.

## Recursos
- Janela sem barra de título, bandeja do sistema, instância única e atalho global padrão CTRL+ALT+C. Fechar pela tecla/ botão de fechar esconde o widget na bandeja; use Sair para encerrar o processo.
- Sessão WebView2 persistente em %APPDATA%\WidgetWebsite\webview-data; configuração em %APPDATA%\WidgetWebsite\config.json. A pasta mantém o nome interno antigo para preservar dados de instalações anteriores do Floatly.
- Interface em português do Brasil, inglês ou espanhol, selecionável nas configurações.
- Uma pequena aba na parte superior central permite arrastar o widget. As áreas de interação nas bordas continuam ativas, mas são transparentes e não desenham uma moldura ao redor do conteúdo. O tamanho e a posição podem ser alterados nas configurações.
- O tamanho padrão é 350 × 550 px e o zoom padrão do navegador embutido é 80%; ambos podem ser alterados nas configurações.
- URL, tamanho, posição, travamento, sempre no topo, barra de tarefas, zoom, cantos e recarga automática configuráveis.
- Links externos encaminhados ao navegador padrão; domínios de autenticação reconhecidos abrem em janela secundária com uma barra de endereço simples e a pasta de dados compartilhada do WebView2. A janela tenta fechar quando o fluxo retorna ao endereço configurado após a autenticação.
- Workflow de compilação Windows x64 e instalador NSIS.
- Pacote MSIX x64 para a Microsoft Store, gerado por workflow próprio do GitHub Actions; o guia de publicação está em [MICROSOFT_STORE_READINESS.md](MICROSOFT_STORE_READINESS.md).

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

O instalador aparece em src-tauri/target/release/bundle/nsis e o executável em src-tauri/target/release. O workflow Build Windows compila a saída no GitHub Actions e publica o instalador e o executável no artefato `widget-website-windows-x64`.

## Primeira execução

Na primeira abertura, o widget pede que você escolha um site. Clique em **Escolher site** e informe uma URL em **Configurações → URL do site**. O campo começa vazio: nenhum website de terceiros vem configurado por padrão. O site precisa permitir o carregamento no WebView2; alguns sites podem restringir recursos específicos ou fluxos de login.

Para um exemplo concreto de URL, consulte a seção de exemplo abaixo.

## Microsoft Store

O Floatly é preparado para a Store como **MSIX x64**. Siga o [guia passo a passo](GUIA_PUBLICACAO_MICROSOFT_STORE.md) e use o [rascunho de listagem multilíngue](MICROSOFT_STORE_LISTING_DRAFT.md).

O workflow **Build MSIX for Microsoft Store** é executado automaticamente na branch `main` quando há alterações em arquivos de código, dependências, configuração ou empacotamento relevantes. Ele também pode ser iniciado manualmente pela aba **Actions** e usa a identidade do pacote já confirmada no Partner Center. O workflow gera o arquivo `.msix` sem assinatura. Isso é intencional: a Microsoft assina o MSIX depois da aprovação. Não são necessários certificado Code Signing, arquivo PFX ou segredos de assinatura para esse fluxo.

## Idioma da interface

Abra **Configurações → Idioma** e escolha Português (Brasil), English ou Español. A escolha é salva em `%APPDATA%\\WidgetWebsite\\config.json` e afeta a janela de configurações, menus da bandeja, a janela auxiliar de login e as mensagens próprias do widget. O conteúdo do site escolhido continua no idioma fornecido pelo próprio site.

## Assinatura opcional do instalador independente (não usada pela Store)

O workflow do instalador NSIS para distribuição independente oferece assinatura Authenticode opcional. Isso é separado do pacote MSIX da Microsoft Store, que não usa certificado do desenvolvedor. Para assinar o NSIS distribuído fora da Store, obtenha um **certificado confiável de assinatura de código para Windows** e crie estes dois segredos nas configurações do repositório, em **Settings → Secrets and variables → Actions**:

- `WINDOWS_PFX_BASE64`: conteúdo do arquivo PFX codificado em Base64.
- `WINDOWS_PFX_PASSWORD`: senha desse arquivo PFX.

A chave privada não deve ser colocada no repositório nem no README. A compilação roda sem acesso ao certificado. Em um job separado, o workflow usa `signtool` para assinar e verificar o instalador NSIS e o executável publicado separadamente, com SHA-256 e timestamp; o arquivo PFX temporário é removido ao final. O executável distribuído separadamente é assinado, mas a cópia já embutida no instalador foi empacotada sem assinatura. Sem os dois segredos, os artefatos são publicados sem assinatura.

**Importante:** não é possível prometer que o Microsoft Defender SmartScreen nunca exibirá um aviso. Mesmo uma aplicação assinada por um certificado válido pode receber um aviso até que a reputação do editor e do arquivo se estabeleça. Publicar pela Microsoft Store é a forma mais direta de evitar avisos de download do SmartScreen. Consulte a [documentação de reputação do SmartScreen](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) e o [guia de assinatura do Tauri para Windows](https://tauri.app/distribute/sign/windows/).

## Exemplo de uso: Investing.com

O Investing.com é apenas um exemplo de site que pode ser exibido. Para usá-lo, abra **Configurações → URL do site** e informe `https://br.investing.com/economic-calendar`. O widget não é exclusivo desse serviço; você pode configurar outro site HTTP ou HTTPS.

Para sites que exigem login, use o fluxo de autenticação disponibilizado pelo próprio site. O menu da bandeja inclui a opção de entrar ou configurar o endereço do site.

O programa não lê, copia ou grava senhas. O User-Agent padrão do WebView2 é preservado.

## Desinstalação e dados

O desinstalador NSIS oferece a opção **Delete the application data**. Quando marcada, ela remove `%APPDATA%\\WidgetWebsite` (configuração, logs e sessão WebView2) e `%LOCALAPPDATA%\\WidgetWebsite`, se existir. Se a opção ficar desmarcada, esses dados são preservados para uma futura reinstalação. A entrada de início automático do Windows é removida sempre que o aplicativo é desinstalado.

## 👤 Autoria e desenvolvimento

Floatly é uma aplicação desktop para Windows, desenvolvida de forma independente por Pablo Phillipe Cândido dos Santos, que mantém um endereço HTTP ou HTTPS escolhido pelo usuário em uma janela flutuante compacta baseada no Microsoft Edge WebView2. O usuário pode configurar o site e ajustar aspectos da janela e da navegação, com acesso pela bandeja do sistema.

O desenvolvimento contou com a utilização de ferramentas de inteligência artificial generativa como recurso auxiliar, permanecendo sob responsabilidade do autor a concepção, a implementação, a integração e a verificação do projeto.

Currículo Lattes: [http://lattes.cnpq.br/9500873674712528](http://lattes.cnpq.br/9500873674712528)
