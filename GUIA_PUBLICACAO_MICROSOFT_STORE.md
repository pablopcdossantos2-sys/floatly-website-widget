# Guia passo a passo — publicação do Widget Website na Microsoft Store

**Revisado em:** 29 de outubro de 2026  
**Estado atual:** preparação técnica inicial concluída; ainda não pronto para submissão.

Este guia foi escrito para que o desenvolvedor consiga executar a preparação e a publicação sem precisar conhecer previamente o Partner Center, o Tauri ou o fluxo de distribuição Win32 da Microsoft Store.

## 1. Entenda o caminho de publicação escolhido

O Widget Website é um **navegador compacto para desktop Windows**, desenvolvido para manter um endereço HTTP/HTTPS escolhido pela pessoa usuária em uma janela flutuante. Ele pode exibir um site próprio, uma ferramenta hospedada pela própria pessoa ou outro site compatível. Não é um aplicativo dedicado a um serviço específico e não vem associado a nenhum domínio.

A janela oferece integração própria com o desktop: aba para movimentação, redimensionamento, opção de manter no topo, bandeja do sistema, atalho global, sessão WebView2 persistente e zoom. Seu escopo é intencionalmente compacto: a pessoa configura o endereço nas Configurações e usa uma janela dedicada, em vez de substituir um navegador completo com abas e histórico.

O projeto usa Tauri 2 e, no momento, gera um instalador **NSIS EXE**. Por isso, o caminho técnico preparado é o tipo de produto **EXE or MSI app** do Partner Center. A Microsoft também aceita MSIX e recomenda esse formato em vários cenários, mas migrar este projeto para MSIX exigiria uma etapa própria de empacotamento e validação.

A publicação EXE/MSI é diferente de enviar um ZIP ou um artefato do GitHub Actions. O Partner Center recebe uma URL HTTPS direta para o instalador EXE/MSI hospedado pelo desenvolvedor ou em sua infraestrutura/CDN. A Microsoft baixa e valida esse arquivo.

### Enquadramento nas políticas da Microsoft Store

A regra 10.1.1 que exige que aplicativos da Web sejam publicados pelo proprietário do domínio é destinada a produtos enviados como web apps. O Widget Website não é a publicação de um site dentro de um invólucro: é um aplicativo Win32 genérico, distribuído como EXE, cuja função é fornecer uma janela de navegação configurável. O usuário escolhe o destino; o produto não depende de um site ou serviço específico. Portanto, essa regra não deve ser tratada como um bloqueio prévio para este projeto.

Como o Widget Website navega na Web, a regra diretamente relevante é a **10.2.1**: o mecanismo deve ser Chromium ou Gecko e precisa permanecer dentro de duas versões principais do projeto de código aberto correspondente. O Tauri usa WebView2, baseado em Chromium. A configuração de Store seleciona offlineInstaller, que instala o Evergreen WebView2 Runtime; ela não fixa uma versão privada do Chromium. O Evergreen recebe atualizações automáticas da Microsoft.

Há, porém, um detalhe operacional: uma instância do aplicativo que já está aberta pode continuar usando a versão anterior do Runtime até ser reiniciada. O processo de validação deve confirmar a versão efetivamente carregada e verificar que o aplicativo adota a versão atualizada após reiniciar. Se a pessoa mantiver o widget aberto por períodos longos, convém avaliar um aviso de reinicialização quando uma nova versão do Runtime estiver disponível.

O produto deve ser apresentado com precisão como navegador compacto de um endereço por vez, não como navegador completo. A compatibilidade de páginas, autenticação e recursos depende do site escolhido; a pessoa usuária continua responsável por respeitar os termos de cada serviço. A Microsoft fará a certificação final do pacote enviado, como ocorre com qualquer aplicativo da Store.

## 2. O que já está preparado no repositório

Os itens abaixo foram implementados diretamente na branch `main`:

- O campo da URL padrão está vazio. Uma instalação nova não abre automaticamente o Investing.com nem outro site de terceiros.
- Na primeira execução, a janela pede que a pessoa escolha um site e oferece um botão para abrir as Configurações.
- O Investing.com aparece nos READMEs somente como exemplo de endereço.
- A opção Opacidade foi removida da interface e da configuração persistente. Configurações antigas descartam esse campo sem perder as demais preferências.
- O tamanho padrão continua em 350 × 550 px e o zoom padrão em 80%.
- `src-tauri/tauri.microsoftstore.conf.json` seleciona o instalador offline do WebView2 sem alterar a compilação normal.
- `npm run check:store-config` verifica alguns requisitos locais de configuração.
- `npm run build:store` executa o build e assinatura no Windows. Ele assina o executável principal antes do empacotamento; usa o `bundle.windows.signCommand` do Tauri para assinar o desinstalador NSIS e o instalador externo; faz uma primeira compilação para materializar o toolset NSIS; assina também as DLLs originais da pasta de plugins usada pelo `makensis`; e reconstrói o instalador. Depois extrai o EXE e verifica a assinatura Authenticode de **todos os arquivos PE** encontrados. Se houver um executável ou DLL sem assinatura válida, interrompe o build. O processo gera `SHA256SUMS.txt` com a versão, o hash do instalador e a lista dos PE verificados.
- O workflow manual **Build Microsoft Store package** executa os testes e gera o instalador assinado como artefato temporário do GitHub Actions.

Esses recursos são preparação, não aprovação de certificação. O build de Store para quando não encontra `bundle.publisher` preenchido com a identidade real do desenvolvedor ou quando faltam os segredos do certificado.

## 3. Tarefas que exigem o desenvolvedor humano

### Etapa 1 — Acessar a conta Partner Center existente

O desenvolvedor já possui uma conta em storedeveloper.microsoft.com e já publicou outro widget por meio dela. Portanto, **não é necessário criar outra conta nem repetir o cadastro de desenvolvedor**.

1. Acesse o Partner Center com a mesma conta Microsoft usada na publicação anterior.
2. Abra **Apps and Games**.
3. Confirme que a conta e o painel de publicação continuam acessíveis.
4. Continue usando essa conta para reservar e administrar o Widget Website.

### Etapa 2 — Reservar o nome do produto

1. No Partner Center, abra **Apps and Games**.
2. Escolha **New Product**.
3. Selecione **EXE or MSI app**.
4. Reserve um nome exclusivo e apropriado.
5. Registre o nome e o identificador do produto em um local seguro para a equipe.

O nome **Widget Website** é o nome atual no código, não uma garantia de disponibilidade na Store. Se a reserva exigir outro nome, decida a marca final antes da primeira submissão. A listagem deve usar um nome único, sem empilhar palavras-chave ou descrições no título.

### Etapa 3 — Definir a identidade do publisher

O Tauri exige que o nome do publisher seja diferente do nome do produto. O valor deve representar a pessoa ou entidade que realmente publica o aplicativo e deve ser coerente com a identidade usada no certificado de assinatura e no Partner Center.

1. Defina o nome de publisher que aparecerá no Windows.
2. Confirme a grafia com a conta Individual/Company e com o certificado de assinatura.
3. Abra `src-tauri/tauri.conf.json`.
4. Dentro de `bundle`, adicione o campo `publisher` com o nome verdadeiro, por exemplo:

```json
"bundle": {
  "publisher": "NOME REAL DO PUBLISHER",
  "active": true
}
```

Não copie literalmente o texto de exemplo. Não use um nome fictício. Preserve as demais propriedades já existentes no objeto `bundle`.

5. Faça commit dessa alteração na `main`.
6. Execute `npm run check:store-config` e confirme que a verificação passa sem o aviso de publisher ausente.

**Esta etapa não pode ser concluída automaticamente:** o repositório não contém e não deve inventar a identidade legal/comercial do publicador.

### Etapa 4 — Obter um certificado de assinatura de código

Para o caminho EXE/MSI, a Microsoft exige que o instalador e todos os arquivos PE que ele contém sejam assinados por um certificado de assinatura de código que encadeie até uma autoridade certificadora participante do Microsoft Trusted Root Program.

1. Obtenha um certificado de assinatura de código de uma autoridade certificadora aceita pela Microsoft.
2. Confirme com a autoridade certificadora o nome que será exibido como publisher.
3. Exporte o certificado com chave privada em formato PFX.
4. Guarde o PFX e sua senha em um gerenciador de segredos. Nunca coloque o arquivo PFX, sua senha ou a chave privada no Git, README, issue ou artefato público.
5. No GitHub, abra **Settings → Secrets and variables → Actions → New repository secret**.
6. Crie os segredos:
   - `WINDOWS_PFX_BASE64`: conteúdo do PFX codificado em Base64, sem cabeçalhos de arquivo PEM.
   - `WINDOWS_PFX_PASSWORD`: senha do PFX.

Para obter o Base64 em PowerShell sem criar um arquivo de texto intermediário:

```powershell
[Convert]::ToBase64String([IO.File]::ReadAllBytes("C:\caminho\publisher.pfx"))
```

Copie a saída diretamente para o segredo do GitHub. Não a salve no repositório.

O script de Store verifica a validade do certificado, a finalidade Code Signing, a cadeia de confiança e o resultado da assinatura. O executável do aplicativo é assinado antes de entrar no NSIS; o instalador é assinado depois de empacotado. O instalador offline do WebView2 vem da Microsoft e também deve ser confirmado como componente assinado pelo fornecedor.

**Importante:** os segredos Authenticode acima são diferentes da chave que será usada pelo atualizador Tauri. Não os confunda nem reutilize uma chave privada em finalidades diferentes.

### Etapa 5 — Definir como as atualizações do aplicativo serão entregues

Esta é uma pendência técnica real. O projeto ainda não tem o plugin Tauri Updater configurado, não possui chave pública/privada de atualização, endpoint de atualização nem artefatos assinados de atualização.

O guia do Tauri para Microsoft Store indica que o instalador EXE/MSI precisa tratar atualizações. A solução Tauri usual é o plugin `tauri-plugin-updater`, que usa uma assinatura própria dos arquivos de atualização.

O desenvolvedor precisa:

1. Decidir onde os arquivos de atualização serão hospedados (por exemplo, um servidor HTTPS ou um feed estático em uma infraestrutura/CDN de confiança).
2. Seguir o [guia oficial do Tauri Updater](https://v2.tauri.app/plugin/updater/) para adicionar o plugin Rust e o pacote JavaScript correspondente.
3. Gerar o par de chaves de atualização pelo Tauri CLI. O comando documentado é:

```powershell
npm run tauri signer generate -- -w "$HOME\.tauri\widget-website.key"
```

4. Guardar a chave privada em local seguro e adicionar seu conteúdo/senha aos segredos do GitHub usados pelo processo de release (por exemplo, `TAURI_SIGNING_PRIVATE_KEY` e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`).
5. Colocar somente a chave pública em `plugins.updater.pubkey` na configuração Tauri.
6. Configurar o endpoint HTTPS real em `plugins.updater.endpoints`.
7. Ativar a geração dos artefatos do atualizador, implementar a consulta/aviso ao usuário e testar a atualização de uma versão anterior para a nova.
8. Confirmar que a estratégia de atualização escolhida é compatível com a distribuição EXE/MSI cadastrada no Partner Center e com as obrigações da Store.

A chave privada do atualizador nunca deve ser publicada. Se for perdida, os usuários das versões existentes podem ficar impossibilitados de validar atualizações futuras.

Não configure um endpoint de exemplo ou uma chave falsa: o atualizador só deve ser ativado quando o endpoint, o feed, as chaves e o processo de release forem reais e testados.

### Etapa 6 — Conferir a versão do aplicativo

Antes de cada pacote que será enviado à Store, mantenha os seguintes valores sincronizados:

- `package.json` → `version`
- `src-tauri/Cargo.toml` → `version`
- `src-tauri/tauri.conf.json` → `version`

Hoje esses arquivos estão alinhados em `0.1.0`. Antes da submissão, defina a versão de lançamento que será publicada e atualize os três campos no mesmo commit. Não reutilize a mesma URL de pacote para binários diferentes.

## 4. Como gerar o instalador de Store

### Pré-requisitos no computador Windows

- Windows 10/11 x64.
- Node.js 22 ou mais recente e npm.
- Rust 1.90 ou mais recente, com target `x86_64-pc-windows-msvc`.
- Visual Studio Build Tools com **Desktop development with C++** e Windows SDK.
- Certificado PFX válido e segredos carregados no ambiente.
- Publisher real definido em `src-tauri/tauri.conf.json`.

### Opção A — Gerar localmente pelo PowerShell

1. Abra o PowerShell na pasta do repositório.
2. Atualize a branch `main` e instale as dependências:

```powershell
git switch main
git pull
npm ci
```

3. Disponibilize temporariamente `WINDOWS_PFX_BASE64` e `WINDOWS_PFX_PASSWORD` no ambiente do PowerShell, sem gravá-los em arquivos ou no histórico do terminal.
4. Execute:

```powershell
npm run build:store
```

5. O script executa as verificações, compila o aplicativo sem empacotar, assina o executável principal, gera o instalador usando a configuração offline do WebView2 e assina o instalador final.
6. O script faz uma primeira compilação para materializar o toolset NSIS, assina os DLLs originais da pasta `Plugins` usada pelo `makensis` e reconstrói o instalador. O script também exige que o **7-Zip** esteja instalado e disponível em `PATH` ou em uma das pastas padrão do Windows. Ele é usado para extrair o conteúdo do NSIS e verificar as assinaturas de cada executável/DLL. O build falha caso não consiga fazer essa inspeção.
7. O instalador deverá estar em:

```text
src-tauri/target/release/bundle/nsis/
```

8. O script remove as variáveis de ambiente do processo ao terminar e apaga o arquivo PFX temporário.
8. Verifique o resultado:

```powershell
Get-AuthenticodeSignature ".\src-tauri\target\release\bundle\nsis\NOME-DO-INSTALADOR.exe" |
  Format-List Status, StatusMessage, SignerCertificate
Get-FileHash ".\src-tauri\target\release\bundle\nsis\NOME-DO-INSTALADOR.exe" -Algorithm SHA256
```

O status da assinatura precisa ser `Valid`. O manifesto `SHA256SUMS.txt` registra o SHA-256 do instalador e os arquivos PE extraídos. Revise esse manifesto antes de hospedar o pacote e guarde-o no registro do release.

### Opção B — Gerar pelo GitHub Actions

Esta é a opção recomendada para uma compilação reproduzível:

1. Confirme que `bundle.publisher` está configurado na `main`.
2. Confirme que os segredos `WINDOWS_PFX_BASE64` e `WINDOWS_PFX_PASSWORD` estão cadastrados no repositório.
3. Abra a aba **Actions** no GitHub.
4. Selecione **Build Microsoft Store package**.
5. Clique em **Run workflow** e execute usando a branch `main`.
6. Aguarde o workflow concluir todas as verificações e a assinatura.
7. Baixe o artefato `widget-website-microsoft-store-x64` e extraia o EXE.

O artefato do Actions é temporário e tem retenção limitada. **Não use o ZIP do Actions nem o link temporário do artefato como URL permanente de pacote do Partner Center.** Extraia o instalador EXE assinado e hospede o arquivo em uma URL pública e permanente.

O workflow é manual e não publica o aplicativo automaticamente na Store.

### Risco conhecido do empacotador NSIS

Há um relato upstream no Tauri ([issue #16208](https://github.com/tauri-apps/tauri/issues/16208), aberto em 4 de outubro de 2026) em que algumas DLLs padrão de plugins NSIS — como `System.dll`, `nsDialogs.dll`, `StartMenu.dll` e `NSISdl.dll` — continuam sem assinatura no instalador, embora o empacotador informe que assinou cópias desses plugins. O relato descreve Tauri CLI 2.11.2 / NSIS 3.11; o projeto atualmente resolve uma versão mais recente da CLI, então não se deve presumir que o mesmo comportamento esteja presente ou corrigido sem inspeção real.

O script `build-microsoft-store.ps1` tenta contornar esse problema assinando, após materializar o NSIS, os DLLs originais da pasta `Plugins` do toolset que contém `makensis.exe`, incluindo `LangDLL.dll`, antes de reconstruir o instalador. Ele não confia apenas nos logs do Tauri: extrai o EXE com 7-Zip e verifica os cabeçalhos PE e a assinatura de cada binário. **Se qualquer PE estiver sem assinatura válida, não publique o pacote.** Essa mitigação foi preparada a partir do relato upstream, mas ainda precisa ser validada em um runner Windows com certificado real. Se falhar, será necessário corrigir a integração do toolset NSIS ou avaliar outra estratégia de empacotamento (por exemplo, MSIX).
## 5. Testar o instalador antes de enviá-lo

Faça estes testes em uma máquina virtual ou computador Windows limpo, de preferência diferente do ambiente de desenvolvimento. Preserve uma cópia do instalador exato que será enviado.

### Etapa 8 — Validar assinatura e integridade

1. Confirme que `Get-AuthenticodeSignature` retorna `Valid` para o instalador e para o executável principal.
2. Confira o nome do certificado e sua cadeia de confiança.
3. Confirme que o instalador contém a instalação offline do WebView2. A configuração Tauri Store já seleciona `offlineInstaller`; esse modo aumenta consideravelmente o tamanho do arquivo, mas evita baixar o Runtime durante a instalação.
4. Revise `SHA256SUMS.txt`, confirme que todos os PE extraídos constam como assinados e registre o nome do arquivo, versão, tamanho, data e SHA-256.
5. Não altere o binário depois de validar e enviar essa versão.

### Etapa 9 — Testar a instalação silenciosa

O projeto usa NSIS. Para o Partner Center, o parâmetro silencioso documentado pelo Tauri é **`/S`** (S maiúsculo).

Em um PowerShell elevado, execute o EXE que foi extraído do artefato:

```powershell
Start-Process -FilePath ".\NOME-DO-INSTALADOR.exe" -ArgumentList "/S" -Wait -PassThru
```

O instalador deve completar sem exibir uma interface de instalação. Um aviso do UAC pode ser permitido pelo requisito Microsoft. Registre o código de saída e confirme que o aplicativo foi instalado.

### Etapa 10 — Conferir a entrada do Windows e a desinstalação

1. Abra **Configurações do Windows → Aplicativos → Aplicativos instalados** (ou **Painel de Controle → Programas e Recursos**).
2. Confirme que nome do aplicativo, publisher e versão aparecem corretamente; nenhum campo pode ficar em branco ou mostrar uma identidade sem relação com o produto.
3. Confirme que a instalação criou somente a entrada do próprio aplicativo, sem software adicional ou bundleware.
4. Abra o Widget Website. A primeira execução deve pedir que você escolha um site.
5. Configure uma página HTTPS de teste sob seu controle, se possível. Teste também um site de terceiros somente quando seus termos permitirem.
6. Teste janela flutuante, aba de movimentação, redimensionamento, sempre no topo, bandeja, atalho global, zoom, sessão persistente, idioma e fluxo de login quando aplicável.
7. Teste reinício do Windows, atualização e encerramento.
8. Desinstale e confira as opções de preservação/remoção dos dados. Reinstale e teste o fluxo de primeira execução novamente.

A instalação offline do WebView2 permite instalar o Runtime sem baixá-lo durante o setup; isso não significa que o site escolhido funcionará sem conexão à internet.

## 6. Hospedar o instalador e criar a submissão

### Etapa 11 — Hospedar uma URL HTTPS versionada

A Microsoft exige um link HTTPS direto ao EXE/MSI. O instalador deve estar hospedado em infraestrutura própria ou CDN, sem login, cookies ou etapas intermediárias para baixá-lo.

1. Escolha o servidor ou CDN que ficará sob controle do desenvolvedor.
2. Crie um caminho versionado, por exemplo:

```text
https://downloads.seudominio.example/widget-website/0.1.0/widget-website-setup.exe
```

3. Substitua o domínio de exemplo pelo domínio real. Não use esse endereço fictício.
4. Teste o link em uma janela anônima e confirme que ele baixa diretamente o EXE por HTTPS.
5. Compare o SHA-256 do arquivo hospedado com o arquivo assinado localmente.
6. Depois de submeter o binário, não substitua o conteúdo dessa URL. Para uma nova versão, publique outro caminho versionado e atualize a submissão do Partner Center.
7. Mantenha as versões antigas disponíveis enquanto forem referenciadas por uma submissão publicada.

### Etapa 12 — Criar a submissão no Partner Center

1. Entre em **Partner Center → Apps and Games**.
2. Abra o produto reservado na Etapa 3.
3. Complete as seções da submissão:
   - **Availability:** mercados, disponibilidade, preço e agenda de publicação.
   - **Properties:** categoria, subcategoria quando disponível, requisitos de sistema e informações de produto.
   - **Age ratings:** responda ao questionário de classificação etária IARC.
   - **Packages:** informe a URL HTTPS do EXE assinado e os dados solicitados pelo Partner Center.
   - **Store listings:** título, descrição, screenshots, ícone/logo e idiomas.
4. Para este instalador NSIS, informe o argumento de instalação silenciosa `/S` no campo de parâmetros de instalação.
5. Se o Partner Center pedir informações adicionais, use os dados do arquivo efetivamente testado: versão, arquitetura x64, requisitos e SHA-256.
6. Salve cada seção antes de avançar.

A Microsoft recomenda completar a listagem em todos os idiomas suportados pelo produto. Este projeto oferece interface PT-BR, inglês e espanhol; prepare textos e capturas coerentes nesses idiomas.

### Etapa 13 — Preparar os materiais da página da Store

Use o arquivo [MICROSOFT_STORE_LISTING_DRAFT.md](MICROSOFT_STORE_LISTING_DRAFT.md) como rascunho de título, descrições, palavras-chave e roteiro de screenshots.

Antes de publicar:

- Use capturas reais do aplicativo executando no Windows. Não use mockups que mostrem funções ainda não implementadas.
- Mostre a primeira execução, a janela flutuante com uma página de demonstração autorizada e os controles nativos de desktop.
- Não use telas do Investing.com, logotipos de sites de terceiros, marcas, imagens ou termos de pesquisa que sugiram parceria ou publicação por esses serviços.
- Use no máximo sete termos/frases de pesquisa únicos, relevantes e próprios do Widget Website.
- Não prometa que todos os sites, anúncios, players, autenticações ou páginas protegidas funcionarão. O comportamento depende da tecnologia e dos termos de cada site.
- Defina a categoria correta. Não escolha uma categoria de navegador, calendário ou finanças só porque o usuário pode abrir um desses conteúdos.
- Informe um canal real de suporte e uma página de política de privacidade acessível.
- Revise o texto e as capturas para garantir que o valor principal esteja claro já na primeira execução.

### Etapa 14 — Política de privacidade e notas para certificação

O produto mantém uma sessão WebView2 local, grava configuração/logs no perfil do usuário e acessa o site escolhido por essa pessoa. O repositório informa que não há telemetria e que os logs omitem query strings e fragments das URLs. O desenvolvedor deve verificar se essa descrição continua precisa na versão que será enviada.

1. Defina e publique uma política de privacidade em URL estável, caso exigida pelas políticas ou pela legislação aplicável. A página deve explicar quais dados são armazenados, onde ficam, por quanto tempo, e como apagar a sessão/configuração.
2. Informe na listagem o canal de suporte e os contatos de suporte exigidos pelo Partner Center.
3. Em **Certification notes**, explique que é um utilitário desktop de sites escolhidos pelo usuário, e descreva como o testador configura a URL na primeira execução.
4. Forneça uma página de demonstração pública e controlada pelo desenvolvedor para a avaliação, caso a Microsoft solicite uma URL de teste. Não dependa de uma conta pessoal ou de credenciais privadas.
5. Explique as limitações conhecidas: compatibilidade depende do site, alguns provedores bloqueiam certas navegações ou autenticações, e o Widget Website não é afiliado aos serviços exibidos.

## 7. Validação e envio

### Etapa 15 — Executar a validação de pacote

Para produtos EXE/MSI, o Partner Center oferece a seção de **Package validation**. Na página **Manage Packages**, use **Run** no pacote e depois consulte **View status** para inspecionar os resultados.

Antes de enviar, confirme também manualmente:
- assinatura Authenticode válida;
- URL de pacote HTTPS direta e versionada;
- instalação silenciosa sem interface (UAC pode aparecer);
- nome, publisher e versão corretos em Aplicativos instalados;
- uma única entrada do produto, sem bundleware;
- instalador completo/offline, sem downloader stub;
- nenhuma alteração no binário depois de submetido.

Se a validação indicar falhas, corrija o build e produza um novo instalador. Quando o binário mudar, use outra URL versionada e atualize o pacote no Partner Center.

### Etapa 16 — Submeter para certificação

1. Revise todas as seções e confirme que não há campos pendentes.
2. Clique em **Submit to the Store**.
3. Acompanhe o status no Partner Center e as mensagens enviadas ao e-mail de contato.
4. Se houver reprovação, leia cada item do relatório e as notas do avaliador. Corrija o problema, gere um pacote novo quando necessário e crie uma nova submissão.
5. Só anuncie que o aplicativo está disponível depois que o status indicar publicação na Microsoft Store.

A Microsoft informa que a certificação de produtos MSI/EXE pode levar até três dias úteis, mas isso é uma referência documental, não uma garantia de prazo.

## 8. Atualizações depois da publicação

Para cada atualização:

1. Atualize a versão nos três arquivos de metadados.
2. Gere e teste os artefatos de atualização Tauri com a chave privada protegida.
3. Compile, assine, instale e teste em cima da versão anterior.
4. Publique os arquivos de atualização e o instalador final em uma nova URL versionada.
5. Registre o hash e guarde uma cópia imutável do binário enviado.
6. Atualize a submissão existente no Partner Center com o pacote e a URL da nova versão.
7. Não substitua o binário que corresponde a uma versão já publicada.

## 9. Checklist de responsabilidade

| Tarefa | Responsável |
|---|---|
| Escolher conta Individual ou Company e concluir verificação | Desenvolvedor humano |
| Obter uma resposta da Microsoft sobre a elegibilidade do widget genérico | Desenvolvedor humano |
| Reservar nome e preencher Partner Center | Desenvolvedor humano |
| Definir publisher real e preencher `bundle.publisher` | Desenvolvedor humano |
| Obter certificado Authenticode de uma CA confiável | Desenvolvedor humano |
| Guardar o PFX e criar GitHub Actions Secrets | Desenvolvedor humano |
| Escolher endpoint/host de atualizações e criar a chave do Tauri Updater | Desenvolvedor humano |
| Integrar e testar o mecanismo de atualizações | Trabalho técnico futuro no repositório |
| Verificar versão, permissões e configuração Store | Automatizado pelo script/CI |
| Compilar e assinar app antes do NSIS e assinar instalador | Automatizado por `npm run build:store` ou workflow manual, após os segredos e publisher existirem |
| Testar instalação, desinstalação, atualização e comportamento em Windows limpo | Desenvolvedor humano |
| Hospedar pacote em URL HTTPS versionada e imutável | Desenvolvedor humano |
| Criar listagem, capturas reais, classificação etária e política de privacidade | Desenvolvedor humano |
| Executar package validation, submeter e responder à certificação | Desenvolvedor humano |

## 10. Bloqueios atuais — não marcar como pronto

- [ ] Confirmação da Microsoft de que este modelo de widget genérico é elegível para a Store, dadas as regras sobre web apps e conteúdo de terceiros.
- [ ] Conta Partner Center criada e nome do produto reservado.
- [ ] Nome real do publisher definido e configurado em `src-tauri/tauri.conf.json`.
- [ ] Certificado Authenticode obtido e segredos GitHub configurados.
- [ ] Integração do Tauri Updater, chaves e endpoint de atualização implementados e testados.
- [ ] Build Store executado com sucesso no Windows usando certificado real, com `SHA256SUMS.txt` validando todos os PE extraídos.
- [ ] Instalador silencioso, atualização, desinstalação e teste em máquina limpa aprovados.
- [ ] Instalador hospedado em URL HTTPS direta, versionada e imutável.
- [ ] Capturas reais, textos finais, classificação etária, suporte e privacidade prontos.
- [ ] Package validation aprovada no Partner Center.
- [ ] Submissão enviada e certificação concluída.

## Fontes oficiais consultadas

- [Microsoft Store Policies — versão 7.20 (publicada em 15/09/2026; efetiva em 22/10/2026)](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies), especialmente as regras 10.1.1 (representação correta do produto) e 10.2.1 (mecanismo de navegação Web).
- [WebView2: boas práticas de desenvolvimento](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/developer-guide) — uso do Evergreen Runtime e adoção de atualizações.
- [Guia Tauri 2 — Microsoft Store](https://v2.tauri.app/distribute/microsoft-store/).
- [Tauri 2 — Windows Installer](https://v2.tauri.app/distribute/windows-installer/).
- [Tauri 2 — versões dos mecanismos WebView](https://v2.tauri.app/reference/webview-versions/).
- [Requisitos de pacote EXE/MSI](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/app-package-requirements).
- [Validação de pacotes EXE/MSI](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/package-validation-pre-check).
- [Validação manual de instaladores EXE/MSI](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/manual-package-validation).
- [Listagem da Store para produtos EXE/MSI](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/add-and-edit-store-listing-info).
- [Tauri 2 — Updater plugin](https://v2.tauri.app/plugin/updater/).
