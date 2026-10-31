# Guia passo a passo — publicar o Floatly como MSIX na Microsoft Store

**Revisado em:** 31 de outubro de 2026  
**Formato:** MSIX para Windows x64  
**Situação:** nova ficha do Floatly no Partner Center. Não use o fluxo EXE/MSI.

O MSIX é a rota adequada para publicar o Floatly sem comprar certificado Code Signing. O pacote enviado à Store não precisa ser assinado por você: depois da aprovação, a Microsoft assina o pacote e hospeda a distribuição. A Store também gerencia as atualizações das instalações feitas por ela.

## 1. Abrir a ficha já criada

A ficha **Floatly** já foi criada no Partner Center e a identidade do pacote foi confirmada. Não crie outro produto e não reserve o nome novamente.

1. Entre no [Microsoft Partner Center](https://partner.microsoft.com/dashboard).
2. Abra **Apps and games → Floatly**.
3. Abra a submissão existente ou clique em **Start submission** para começar a submissão inicial.

Confirme que o tipo de produto escolhido foi **MSIX or PWA app**. Não use **EXE or MSI app**, nem **Game**. O Floatly é um aplicativo desktop Tauri que será entregue como pacote MSIX, não um site PWA.

Referência oficial: https://learn.microsoft.com/en-us/windows/apps/publish/get-started

## 2. Identidade do pacote confirmada

Os valores abaixo foram copiados da seção **Exibir a identidade do produto** do Partner Center e já estão configurados como padrões do workflow MSIX:

| Campo do Partner Center | Valor |
|---|---|
| Package/Identity/Name | `pablopcdsantos.Floatly` |
| Package/Identity/Publisher | `CN=A3E973B8-1411-4314-B754-F3DADD99C070` |
| Package/Properties/PublisherDisplayName | `pablopcdsantos` |

O build preenche o manifesto com esses valores exatos. Não altere maiúsculas, minúsculas, pontuação ou espaços. Se algum dado mudar no Partner Center, atualize os padrões do workflow antes de gerar uma nova versão.

Esses valores são dados públicos de identidade do pacote, não são senhas nem chaves privadas.

## 3. Gerar o pacote MSIX pelo GitHub Actions

O workflow **Build MSIX for Microsoft Store** é acionado automaticamente a cada atualização enviada à branch `main`. Ele utiliza os três valores confirmados acima e gera o pacote sem certificado pago.

1. Abra o repositório: https://github.com/pablopcdossantos2-sys/floatly-website-widget
2. Clique em **Actions**.
3. Abra a execução mais recente de **Build MSIX for Microsoft Store**, correspondente ao commit que deseja publicar.
4. Aguarde o workflow terminar com sucesso.
5. Na seção **Artifacts** da execução, baixe **floatly-msix-x64**.

Para gerar manualmente um novo pacote sem fazer outro commit, use **Run workflow** na mesma tela. Os três campos já aparecem preenchidos com a identidade oficial; mantenha os valores exatamente como estão.

Dentro do ZIP você encontrará:
- um arquivo Floatly com extensão .msix;
- SHA256SUMS.txt, com a versão, identidade do pacote e hash SHA-256.

O artefato é de arquitetura **x64**. Não selecione ARM64 no Partner Center enquanto não existir um pacote ARM64 correspondente.

### Por que o pacote não vem assinado?

O workflow usa a opção --no-sign do WinApp CLI de propósito. Esse é o fluxo esperado para submissão MSIX à Microsoft Store: a Microsoft assina o pacote após a aprovação. O pacote sem assinatura do artefato não é destinado à instalação direta por duplo clique antes da certificação.

Você não precisa:
- comprar certificado Code Signing;
- gerar ou exportar arquivo PFX;
- cadastrar WINDOWS_PFX_BASE64 ou WINDOWS_PFX_PASSWORD;
- cadastrar chave do Tauri Updater;
- hospedar o MSIX em um site ou GitHub Release.

Referências oficiais:
- https://learn.microsoft.com/en-us/windows/apps/dev-tools/winapp-cli/guides/tauri
- https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements

## 4. Preencher as informações da nova submissão

Na submissão iniciada no Partner Center, complete as seções apresentadas. A ordem visual pode mudar; use os nomes das seções da sua conta.

### Pricing and availability

1. Defina o preço como **Free**.
2. Se pretende disponibilizar o Floatly internacionalmente, selecione todos os mercados nos quais deseja publicá-lo.
3. Salve a seção.

### Properties

1. Escolha **Productivity** como categoria principal.
2. Preencha o site de suporte com a página dedicada:
   https://github.com/pablopcdossantos2-sys/floatly-website-widget/blob/main/SUPPORT.md
3. Preencha a política de privacidade:
   https://github.com/pablopcdossantos2-sys/floatly-website-widget/blob/main/PRIVACY_POLICY.md
4. Use o e-mail de suporte divulgado em SUPPORT.md.
5. Preencha somente os demais campos obrigatórios. Não invente endereço comercial, telefone empresarial ou dados que você não possui.

### Age ratings

1. Abra o questionário de classificação etária (IARC).
2. Responda com base no comportamento real do Floatly.
3. Considere que o Floatly abre endereços HTTP/HTTPS escolhidos pelo usuário e pode exibir conteúdo de websites externos. Não declare que o aplicativo controla ou modera o conteúdo desses websites.
4. Revise as respostas antes de enviar.

Não reutilize respostas de outro produto se o questionário ou as funcionalidades forem diferentes.

## 5. Enviar o pacote MSIX

1. Abra a seção **Packages** da submissão.
2. Selecione **Upload package** ou arraste o arquivo .msix extraído do artefato floatly-msix-x64.
3. Aguarde o Partner Center processar o arquivo.
4. Confira se a arquitetura identificada é **x64** e se a versão corresponde ao build.
5. Se o Partner Center indicar erro de identidade, não tente editar o manifesto às cegas. Compare os três valores digitados no workflow com **Product identity** e gere o pacote novamente.
6. Salve a seção.

O Partner Center aceita pacotes .msix, .msixbundle, .msixupload e outros formatos MSIX documentados. Neste primeiro build, o Floatly entrega um .msix x64.

Referência oficial: https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/upload-app-packages

## 6. Preparar a Store listing

Use o conteúdo de [MICROSOFT_STORE_LISTING_DRAFT.md](MICROSOFT_STORE_LISTING_DRAFT.md).

1. Cadastre o nome Floatly e a descrição em português.
2. Adicione inglês e espanhol como idiomas adicionais, usando as traduções do rascunho.
3. Revise os termos de pesquisa para descrever apenas o Floatly; não use nomes de websites de terceiros como se fossem marcas do aplicativo.
4. Capture screenshots reais do Floatly executando no Windows. Não use mockups nem telas que sugiram funcionalidades inexistentes.
5. Mostre, de preferência:
   - a primeira execução e a escolha de um website;
   - a janela flutuante sobre a área de trabalho;
   - o painel de configurações;
   - o menu da bandeja do sistema.
6. Salve a listagem.

## 7. Informações para a certificação

Na seção **Submission options** ou nas notas destinadas à certificação, explique de forma objetiva:

- O Floatly é um aplicativo desktop Windows construído com Tauri e WebView2.
- Na primeira execução, o usuário precisa escolher o endereço HTTP/HTTPS que deseja exibir; não existe website de terceiros predefinido.
- O aplicativo usa a capacidade restrita runFullTrust porque é um processo desktop Win32 que precisa criar uma janela flutuante sem moldura, integrar-se à bandeja do sistema, registrar um atalho global e hospedar o WebView2.
- A extensão windows.startupTask declara a opção de iniciar com o Windows. O usuário controla essa permissão nas configurações do aplicativo e nos controles de inicialização do Windows.
- O aplicativo não instala drivers, não solicita elevação administrativa e não executa tarefas em segundo plano sem interação do usuário.

Peça ao avaliador que, na primeira execução, escolha um endereço HTTPS de demonstração. Não forneça credenciais pessoais nem dependa de acesso a uma conta privada para a avaliação.

## 8. Validar e enviar

1. Revise todas as seções e confirme que o pacote enviado corresponde à última execução bem-sucedida do workflow.
2. Salve o rascunho.
3. Execute as validações disponibilizadas pelo Partner Center.
4. Corrija os erros indicados e gere um novo pacote quando necessário.
5. Quando todas as seções estiverem completas, clique em **Submit to the Store**.
6. Acompanhe o resultado da certificação no Partner Center. Só anuncie o aplicativo depois que o status indicar que ele está publicado.

A certificação MSIX pode levar alguns dias úteis. O Partner Center informa o resultado e os problemas encontrados; não interprete o simples upload do pacote como aprovação.

## 9. Testar o aplicativo depois da publicação

Quando o Floatly estiver disponível na Store:

1. Instale-o pela Microsoft Store em um Windows 10/11 x64 de teste.
2. Abra o aplicativo e confirme que a primeira execução solicita um website.
3. Configure um endereço HTTPS de teste.
4. Teste janela flutuante, movimentação, redimensionamento, sempre no topo, bandeja, atalho global, zoom, idiomas e sessão WebView2.
5. Ative e desative a opção de iniciar com o Windows. Confirme também como ela aparece em **Configurações do Windows → Aplicativos → Inicialização** ou no Gerenciador de Tarefas.
6. Feche e abra novamente o aplicativo, reinicie o Windows e repita os testes relevantes.
7. Teste atualização e desinstalação pela Store.

A Microsoft Store distribui as atualizações das instalações feitas por ela. Para esse canal não é necessário configurar um feed do Tauri Updater.

## 10. Publicar uma versão futura

1. Atualize a versão de forma consistente em package.json, src-tauri/tauri.conf.json e src-tauri/Cargo.toml.
2. Execute novamente **Build MSIX for Microsoft Store**, usando os mesmos dados de identidade do produto.
3. Envie o novo .msix como atualização da submissão existente no Partner Center.
4. Não reutilize uma versão numérica já publicada. O MSIX exige uma versão superior para atualizar uma instalação existente.

## Referências oficiais

- [Reservar o nome de um aplicativo MSIX](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/reserve-your-apps-name).
- [Iniciar publicação no Microsoft Store Partner Center](https://learn.microsoft.com/en-us/windows/apps/publish/get-started).
- [Guia oficial Tauri + WinApp CLI](https://learn.microsoft.com/pt-br/windows/apps/dev-tools/winapp-cli/guides/tauri).
- [Uso da WinApp CLI e opções do comando pack](https://learn.microsoft.com/pt-br/windows/apps/dev-tools/winapp-cli/usage).
- [Requisitos de pacote MSIX](https://learn.microsoft.com/pt-br/windows/apps/publish/publish-your-app/msix/app-package-requirements).
- [Upload de pacotes MSIX no Partner Center](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/upload-app-packages).
- [StartupTask para aplicativos desktop empacotados](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/applifecycle/applaunch-startup-task).
