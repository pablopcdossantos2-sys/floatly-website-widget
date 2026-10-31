# Guia passo a passo — publicação do Floatly na Microsoft Store

**Revisado em:** 30 de outubro de 2026  
**Situação:** o produto Floatly já existe no Partner Center. Ainda não enviar para certificação.

Este guia contém apenas tarefas humanas ainda pendentes e explica o que fazer sem pressupor experiência com certificados digitais ou ferramentas de desenvolvimento. As configurações básicas já concluídas no Partner Center não precisam ser refeitas.

## Antes de começar

O produto Floatly foi criado no Partner Center pelo caminho **EXE ou MSI app**. Portanto, estamos seguindo o modelo de instalador tradicional do Tauri. Nesse modelo, o instalador e os executáveis/DLLs contidos nele precisam receber assinatura Authenticode antes do envio à Store.

Não compre nenhum certificado antes de concluir a etapa 1 deste guia. Há detalhes importantes sobre emissão para pessoa física no Brasil e sobre a proteção da chave privada.

## 1. Entender e escolher o certificado de assinatura

### O que é esse certificado?

É uma identidade digital usada para assinar programas do Windows. Ela permite ao Windows verificar quem publicou o executável e se o arquivo foi alterado depois da assinatura.

Para o Floatly, precisamos de um certificado público **Code Signing OV**, compatível com **Microsoft Authenticode**, cuja cadeia de certificação termine em uma autoridade certificadora (AC/CA) participante do **Microsoft Trusted Root Program**.

Não confunda os produtos:

- **Code Signing / assinatura de código:** é o produto necessário para EXE, DLL e instaladores do Windows.
- **SSL/TLS para sites:** protege conexões HTTPS; não serve para assinar o instalador.
- **Certificado para assinar PDF ou documentos:** também não serve.
- **EV Code Signing:** é uma categoria mais cara. Não compre EV apenas para tentar evitar o SmartScreen: a Microsoft informa que, desde 2024, EV não oferece mais o antigo bypass imediato do SmartScreen. Para o envio EXE/MSI à Store, não há exigência específica de EV.

A Microsoft explica os tipos de assinatura nesta página:
https://learn.microsoft.com/pt-br/windows/apps/package-and-deploy/code-signing-options

E apresenta o requisito de confiança para instaladores EXE/MSI aqui:
https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/app-package-requirements

### Primeiro: você solicitará o certificado como pessoa física ou empresa?

Essa diferença é decisiva. A autoridade certificadora precisa validar a identidade que aparecerá no certificado.

**Se você tem uma empresa com CNPJ:** a solicitação normalmente será feita em nome da razão social registrada na Receita Federal. Não use nome fantasia por conta própria; pergunte qual nome legal será gravado no certificado e confirme que você tem autorização para solicitar o certificado em nome da empresa.

**Se você não tem CNPJ e pretende solicitar como pessoa física, usando CPF:** não presuma que qualquer revendedor aceitará a solicitação. É necessário encontrar uma AC que confirme expressamente que emite Code Signing OV para uma pessoa física residente no Brasil.

Um exemplo concreto ajuda a entender a diferença:

- A página brasileira da Sectigo atualmente anuncia Code Signing OV e mostra preço de referência de R$ 3.026 por ano: https://www.sectigo.com.br/produtos/certificado-codesigning.php
- Porém, o formulário de pedido da própria Sectigo Brasil diz que não emite esse produto para pessoas físicas e solicita CNPJ e razão social: https://www.sectigo.com.br/produtos/order.php
- A documentação global da Sectigo, atualizada em 2 de junho de 2026, diz que o produto OV pode ser emitido para indivíduos ou organizações. Isso não significa que o canal brasileiro ofereça a modalidade para CPF. É preciso obter confirmação por escrito da equipe que efetivamente atenderá o pedido: https://www.sectigo.com/faqs/detail/OV-Code-Signing-Validation-for-Organizations-and-Individuals

Esses links são exemplos para pesquisa, não uma ordem para comprar da Sectigo. Compare as condições e não pague antes de confirmar sua elegibilidade.

### O que perguntar à autoridade certificadora

Entre em contato com o atendimento comercial da AC ou do revendedor e peça uma resposta por escrito. Você pode copiar a mensagem abaixo e adaptar o trecho entre colchetes:

> Olá. Sou desenvolvedor de software residente no Brasil e preciso assinar um aplicativo Windows que será distribuído pela Microsoft Store usando um instalador EXE/NSIS.
>
> [Não possuo CNPJ e preciso contratar como pessoa física, usando CPF.]  
> [OU: Tenho uma empresa registrada no Brasil e preciso contratar em nome da razão social vinculada ao CNPJ.]
>
> Preciso de um certificado público **Code Signing OV para Microsoft Authenticode**, aceito para assinatura de executáveis Windows e cuja cadeia de certificação seja compatível com o Microsoft Trusted Root Program.
>
> Poderiam confirmar, por favor:
>
> 1. Vocês emitem esse certificado para [pessoa física brasileira com CPF / empresa brasileira com CNPJ]?
> 2. Qual nome legal aparecerá como titular/editor no certificado?
> 3. A chave privada será fornecida em token USB, HSM em nuvem ou outro dispositivo protegido?
> 4. Como um desenvolvedor consegue assinar os arquivos no Windows usando SignTool? Há suporte para assinatura remota em GitHub Actions ou a assinatura precisa ocorrer em um computador com o token conectado?
> 5. Qual é o custo total, incluindo certificado, token, frete internacional, impostos e eventuais serviços adicionais?
> 6. Qual é o prazo de emissão e quais documentos serão exigidos?
>
> Antes de contratar, preciso confirmar que a modalidade oferecida atende ao requisito de assinatura Authenticode da Microsoft Store para instaladores EXE/MSI. Obrigado.

Não é necessário enviar documentos pessoais neste primeiro contato comercial. A AC explicará quais documentos são necessários durante o processo formal de validação.

### Por que estamos perguntando sobre token USB ou HSM?

As regras atuais do setor exigem que a chave privada usada em certificados públicos Code Signing fique protegida em hardware apropriado, como um token USB ou HSM. Ela normalmente não pode ser copiada livremente para um arquivo comum.

Isso tem uma consequência para o Floatly: o workflow atual do repositório foi escrito supondo um arquivo PFX exportável e os segredos \`WINDOWS_PFX_BASE64\` e \`WINDOWS_PFX_PASSWORD\`. Esse método não deve ser presumido compatível com um certificado cuja chave esteja presa a um token ou HSM.

**Não gere esses segredos, não tente exportar a chave privada e não envie token, PIN, PFX ou senha para ninguém.** Primeiro vamos descobrir qual modalidade a AC oferece. Depois, o mantenedor ajustará o processo de compilação e assinatura do repositório para usar o método realmente contratado — por exemplo, assinatura local com o token ou integração com o serviço de assinatura remota, se disponível.

A página da Microsoft explica a exigência de HSM/token para certificados OV:
https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options

### Como verificar se a AC faz parte do programa da Microsoft

A AC deve confirmar que o certificado emitido encadeia até uma CA participante do Microsoft Trusted Root Program. Você também pode conferir a lista oficial atualizada:

https://ccadb.my.salesforce-sites.com/microsoft/IncludedCACertificateReportForMSFT

Não basta o vendedor dizer que comercializa certificados digitais ou que o certificado é ICP-Brasil. Peça a confirmação específica de compatibilidade do produto **Code Signing Authenticode** com o requisito da Store.

### Depois que a AC confirmar que pode emitir o certificado

1. Só então escolha a proposta e conclua a contratação.
2. Siga o procedimento de validação de identidade da AC. Ela poderá solicitar documento oficial, comprovante de endereço, telefone verificável, confirmação por chamada ou outros documentos.
3. Caso receba um token USB, siga as instruções oficiais de instalação e proteja o PIN. Não formate, descarte ou envie o token para terceiros.
4. Quando o certificado for emitido, peça à AC o nome exato do titular/editor que aparecerá na assinatura do Windows.
5. Envie ao mantenedor do Floatly somente:
   - o nome exato do titular/editor exibido pelo certificado;
   - o nome da AC e do produto contratado;
   - a modalidade de proteção da chave: token USB, HSM local ou assinatura remota em nuvem.

**Não envie a chave privada, PIN, senha, arquivo PFX ou acesso à conta da AC.** O mantenedor não precisa receber esses segredos para configurar o nome do publisher.

O campo \`bundle.publisher\` será ajustado no repositório para corresponder à identidade confirmada. Não edite Rust ou JSON manualmente.

## 2. Alternativa caso não seja viável obter o certificado

A Microsoft também aceita aplicativos empacotados como **MSIX**. Nesse formato, quando o aplicativo é distribuído pela Store, a própria Microsoft assina novamente o pacote depois da certificação. Assim, não é necessário comprar um certificado Code Signing de uma AC para a publicação na Store.

Documentação oficial:
https://learn.microsoft.com/pt-br/windows/apps/package-and-deploy/code-signing-options

Há uma documentação da Microsoft que ensina a empacotar aplicativos Tauri como MSIX usando o utilitário WinApp CLI:
https://learn.microsoft.com/en-us/windows/apps/dev-tools/winapp-cli/guides/tauri

Essa alternativa exige uma migração técnica do processo de empacotamento. Além disso, o Floatly já foi criado no Partner Center como produto EXE/MSI. Não sabemos se a Microsoft permitirá mudar o tipo desse produto em vez de abrir uma nova ficha. Portanto, não altere nem exclua o produto atual. Se nenhuma AC confirmar a emissão adequada para sua situação, podemos consultar o suporte do Partner Center sobre a conversão do produto existente para MSIX antes de decidir o caminho.

## 3. Preparar a chave do atualizador

O instalador EXE/MSI da Store não atualiza automaticamente as instalações existentes. Para que o Floatly consiga receber versões futuras, precisamos de atualização dentro do próprio aplicativo.

A opção planejada é o **Tauri Updater**, com um feed estático publicado em GitHub Releases. A integração do plugin, as permissões, a verificação de atualizações e o workflow são trabalho técnico no repositório; não edite esses arquivos manualmente.

A ação humana é gerar e guardar o par de chaves:

1. No PowerShell, abra a pasta do repositório e instale as dependências:

~~~powershell
npm ci
New-Item -ItemType Directory -Force "$HOME\.tauri" | Out-Null
npm run tauri signer generate -- -w "$HOME\.tauri\floatly.key"
~~~

2. Guarde os arquivos gerados em local seguro. A chave privada é \`floatly.key\`; a chave pública correspondente é \`floatly.key.pub\`.
3. No GitHub, abra **Settings → Secrets and variables → Actions** e crie:
   - \`TAURI_SIGNING_PRIVATE_KEY\`: conteúdo da chave privada.
   - \`TAURI_SIGNING_PRIVATE_KEY_PASSWORD\`: senha escolhida na geração, se houver.
4. Mantenha uma cópia segura da chave privada e da senha fora do computador. Se perder a chave, não será possível assinar atualizações futuras para as instalações existentes.
5. Nunca publique a chave privada. A chave pública pode ser incluída na configuração do aplicativo; a privada deve existir somente no computador seguro e no segredo protegido do GitHub Actions.

Não cadastre endpoint agora. A integração do repositório definirá o endereço do feed e o formato do arquivo \`latest.json\`. O endereço planejado é:

~~~text
https://github.com/pablopcdossantos2-sys/floatly-website-widget/releases/latest/download/latest.json
~~~

Esse endereço só deverá ser usado depois que a integração estiver implementada e um release real publicar o arquivo e os artefatos assinados correspondentes.

## 4. Gerar o pacote assinado da Microsoft Store

Esta etapa depende da conclusão de duas tarefas técnicas no repositório: integração do atualizador e adaptação do processo Authenticode ao tipo de certificado contratado.

**Não execute o workflow atual de Store esperando que ele funcione com um token USB ou HSM.** O script ainda está preparado para receber um PFX, e essa compatibilidade precisa ser resolvida antes.

Quando o mantenedor confirmar que o processo está pronto:

1. Abra a aba **Actions** do GitHub.
2. Selecione **Build Microsoft Store package**.
3. Clique em **Run workflow** e escolha a branch \`main\`.
4. Aguarde o workflow terminar. Só considere o pacote utilizável quando compilação, testes e validação das assinaturas forem concluídos com sucesso.
5. Baixe o artefato \`widget-website-microsoft-store-x64\`.
6. Guarde o EXE e o \`SHA256SUMS.txt\` juntos, identificando a versão e o commit que os produziram.

O ZIP do Actions é temporário; não use a URL do artefato como endereço permanente do pacote na Store. Se o workflow falhar, abra a execução, identifique o primeiro passo com falha e corrija-o antes de prosseguir.

## 5. Testar o instalador em um Windows limpo

Use uma máquina virtual ou um computador Windows de teste que não tenha uma instalação anterior do Floatly. Preserve uma cópia exata do EXE e do manifesto de hashes.

### Verificar assinatura e pacote

1. No PowerShell, execute \`Get-AuthenticodeSignature\` para o EXE. O campo \`Status\` deve ser \`Valid\`.
2. Confira o nome do certificado e a cadeia de confiança.
3. Revise \`SHA256SUMS.txt\` e confirme que todos os executáveis e DLLs extraídos do instalador foram validados pelo processo de build.
4. Calcule o hash do EXE que será publicado:

~~~powershell
Get-FileHash ".\Floatly-Installer.exe" -Algorithm SHA256
~~~

Use o nome real do arquivo gerado pelo workflow.

### Instalação e funcionamento

1. Instale o pacote com a opção silenciosa exigida pelo Partner Center:

~~~powershell
Start-Process -FilePath ".\Floatly-Installer.exe" -ArgumentList "/S" -Wait -PassThru
~~~

2. Confirme que o aplicativo aparece em **Configurações do Windows → Aplicativos → Aplicativos instalados**, com nome, publisher e versão corretos.
3. Abra o Floatly. A primeira execução deve pedir que você escolha um site; não deve carregar um endereço de terceiros automaticamente.
4. Configure uma URL HTTPS de teste que você controle.
5. Teste janela flutuante, movimentação, redimensionamento, sempre no topo, bandeja do sistema, atalho global, zoom, idioma e persistência da sessão.
6. Reinicie o aplicativo e o Windows. Confirme o comportamento da inicialização automática, se ativada.
7. Teste a atualização in-app: publique uma versão posterior de teste no feed e confirme que o Floatly detecta, baixa, valida a assinatura e instala a atualização.
8. Desinstale com a opção de excluir os dados marcada e confirme a remoção das pastas de dados. Reinstale e teste novamente a primeira execução.

### Conferir o WebView2

No computador de teste, confira a versão instalada do Runtime:

~~~powershell
$programFilesX86 = [Environment]::GetEnvironmentVariable("ProgramFiles(x86)")
$runtimeRoot = Join-Path $programFilesX86 "Microsoft\EdgeWebView\Application"
$runtime = Get-ChildItem $runtimeRoot -Filter "msedgewebview2.exe" -Recurse -File -ErrorAction SilentlyContinue | Select-Object -First 1
if ($runtime) {
    $runtime.VersionInfo.FileVersion
} else {
    Write-Warning "WebView2 Runtime não encontrado no caminho padrão."
}
~~~

Compare a versão efetivamente instalada com as versões atuais documentadas pela Microsoft e confirme que o aplicativo adota a versão atualizada depois de ser reiniciado. Registre a versão encontrada, a edição do Windows e o resultado do teste. A regra 10.2.1 da Store exige que o mecanismo de navegação permaneça dentro do limite de duas versões principais.

Não publique se a assinatura, a atualização, a instalação offline do WebView2 ou qualquer função essencial falhar.

## 6. Hospedar o instalador em uma URL permanente

O Partner Center precisa de uma URL HTTPS pública que baixe diretamente o EXE, sem login, cookies ou etapas intermediárias.

1. Publique o instalador em um endereço permanente e versionado. Uma opção é anexá-lo a um **GitHub Release** do repositório; use um arquivo de release, não o artefato temporário de Actions.
2. Use um caminho específico para a versão, por exemplo:

~~~text
https://github.com/pablopcdossantos2-sys/floatly-website-widget/releases/download/v0.1.0/Floatly-Installer.exe
~~~

Substitua a versão e o nome do arquivo pelos valores reais.

3. Abra a URL em uma janela anônima, sem estar autenticado no GitHub. Confirme que o download do EXE começa sem exigir login.
4. Compare o SHA-256 do arquivo baixado com o resultado obtido na etapa anterior.
5. Não substitua nem sobrescreva esse arquivo depois de submetê-lo. Uma versão nova deve ter uma URL própria.

Caso o Partner Center não aceite o redirecionamento do link de release, use uma hospedagem HTTPS que forneça um download direto.

## 7. Finalizar a submissão existente no Partner Center

O produto Floatly já existe e suas propriedades básicas estão preenchidas. Não volte a configurar disponibilidade, preço, categoria, classificação etária, requisitos do sistema, notas de certificação ou política de privacidade.

1. Abra **Partner Center → Apps and Games → Floatly**.
2. Inicie a submissão inicial do produto.
3. Na seção **Packages**, informe a URL HTTPS permanente do EXE assinado, arquitetura x64 e os dados de pacote solicitados.
4. Informe \`/S\` como argumento de instalação silenciosa do NSIS.
5. Na listagem, use os textos de \`MICROSOFT_STORE_LISTING_DRAFT.md\`.
6. Carregue screenshots reais capturadas no Windows com a versão testada. Mostre a primeira execução, a janela flutuante, as configurações e os controles da bandeja. Não use mockups nem conteúdo de sites de terceiros sem autorização.
7. Salve as seções e execute **Package validation** em **Manage Packages**.
8. Se houver falhas, corrija o pacote, gere outro EXE, publique-o em uma nova URL versionada e execute a validação novamente.
9. Quando a validação estiver aprovada, revise a submissão e clique em **Submit to the Store**.
10. Acompanhe o resultado no Partner Center. Só anuncie a disponibilidade depois que o produto estiver publicado.

## 8. Para futuras versões

Depois da primeira publicação:

1. A atualização in-app deve ser publicada com os artefatos assinados pelo Tauri Updater e o arquivo \`latest.json\` no GitHub Release.
2. Publique também o novo instalador Authenticode em uma URL versionada.
3. Crie uma atualização da submissão existente no Partner Center e envie o novo pacote.
4. Não substitua arquivos de versões anteriores que já tenham sido publicados.

## Referências oficiais

- [Microsoft Learn — opções de assinatura de código](https://learn.microsoft.com/pt-br/windows/apps/package-and-deploy/code-signing-options).
- [Microsoft Learn — requisitos de pacote EXE/MSI](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/app-package-requirements).
- [Microsoft Trusted Root Program — participantes](https://ccadb.my.salesforce-sites.com/microsoft/IncludedCACertificateReportForMSFT).
- [Sectigo — validação Code Signing OV para organizações e indivíduos](https://www.sectigo.com/faqs/detail/OV-Code-Signing-Validation-for-Organizations-and-Individuals).
- [Microsoft Learn — empacotar aplicativo Tauri como MSIX](https://learn.microsoft.com/en-us/windows/apps/dev-tools/winapp-cli/guides/tauri).
- [Tauri — Microsoft Store](https://tauri.app/distribute/microsoft-store/).
- [Tauri — Updater plugin](https://v2.tauri.app/plugin/updater/).
