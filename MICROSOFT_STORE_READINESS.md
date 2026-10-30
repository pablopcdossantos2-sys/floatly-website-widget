# Preparação para Microsoft Store

**Estado:** preparação inicial, ainda não pronto para submissão.

O Widget Website é um widget para desktop que mantém visível um site escolhido pela pessoa usuária em uma janela compacta. O Investing.com é apenas um exemplo de endereço que pode ser configurado; o produto não é um aplicativo oficial desse serviço, não fornece o conteúdo dele e não deve usar sua marca como identidade do produto. Não há URL de terceiros predefinida no código para novas instalações.

## Caminho de distribuição considerado

A configuração Tauri atual produz um instalador NSIS para Windows. O guia oficial atual do Tauri informa que o fluxo direto para a Microsoft Store usa o instalador EXE/MSI publicado por um link, e exige instalador offline, assinatura de código e suporte à instalação silenciosa. A documentação também exige o modo de instalação offline do WebView2 para o instalador distribuído pela Store.

Foi criado o arquivo complementar `src-tauri/tauri.microsoftstore.conf.json`, que seleciona `webviewInstallMode.type = "offlineInstaller"` sem alterar o modo de instalação usado para as distribuições normais do projeto.

Para gerar o pacote local de preparação, a partir do PowerShell:

```powershell
npm ci
npm run icons
npm run tauri build -- --no-bundle
npm run tauri bundle -- --config src-tauri/tauri.microsoftstore.conf.json
```

O resultado ainda precisa ser inspecionado e testado em Windows limpo antes de qualquer submissão.

## Pendências antes da submissão

- [ ] Criar ou acessar uma conta de desenvolvedor no Partner Center e reservar um nome exclusivo.
- [ ] Definir o nome de editor (publisher) no Tauri e confirmar que ele é diferente de `productName`, como exige o guia atual do Tauri.
- [ ] Decidir o mecanismo de atualização compatível com o caminho de distribuição. O guia de distribuição do Tauri para EXE/MSI pede que o instalador trate atualizações; o projeto ainda precisa definir e validar esse fluxo.
- [ ] Corrigir a assinatura do instalador para a distribuição Store. A assinatura Authenticode atual do workflow é opcional, e a cópia do executável que fica dentro do NSIS pode continuar sem assinatura. Confirmar e implementar a assinatura de todos os arquivos PE exigidos antes de empacotar o instalador de Store, além de assinar o instalador final.
- [ ] Confirmar instalação silenciosa (o guia do Tauri documenta `/S` para NSIS), desinstalação, preservação/remoção de dados e comportamento de atualização.
- [ ] Testar instalação e execução em máquinas Windows 10/11 limpas, incluindo ausência do WebView2 Runtime, rede indisponível, escala de tela, múltiplos monitores e login em sites que exijam autenticação.
- [ ] Preparar ícone, capturas de tela, descrição, categoria, termos de pesquisa, classificação etária e endereço de política de privacidade.
- [ ] Descrever com precisão o valor do produto: janela flutuante, dimensionamento, posicionamento, sempre no topo, bandeja do sistema, atalho e zoom. O valor da ferramenta deve estar claro na primeira execução.
- [ ] Avaliar o risco de certificação de um widget que exibe sites de terceiros. As políticas exigem valor distinto e experiência funcional; destacar os recursos próprios de desktop e verificar se o uso genérico não entra na categoria de web app que deve ser publicado pelo proprietário do domínio.
- [ ] Explicar que a pessoa usuária escolhe o site e que o funcionamento de recursos específicos depende do próprio site e de seus termos. Não sugerir afiliação com Investing.com ou outros sites.
- [ ] Revisar a política de segurança e as permissões remotas. O conteúdo remoto pode invocar somente o comando autenticado de sinal de atividade; não ampliar suas permissões sem uma análise de segurança.
- [ ] Executar o Windows App Certification Kit, validar o pacote e testar o fluxo completo de submissão antes de marcar o projeto como pronto.

## Critério de comunicação

A documentação e a página da Store devem apresentar o Widget Website como um utilitário genérico para exibir um site escolhido pela pessoa usuária em uma janela de desktop, sem prometer que todos os sites ou fluxos de login serão compatíveis. O Investing.com pode aparecer apenas em um exemplo de configuração de URL.

## Referências oficiais

- [Microsoft Store Policies — versão 7.20](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies) (publicada em 15 de setembro de 2026; efetiva em 22 de outubro de 2026).
- [Guia do Tauri para Microsoft Store](https://tauri.app/distribute/microsoft-store/).
- [Como distribuir aplicativos Win32 pela Microsoft Store](https://learn.microsoft.com/en-us/windows/apps/distribute-through-store/how-to-distribute-your-win32-app-through-microsoft-store).
- [Requisitos de pacote para aplicativos MSI/EXE](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/app-package-requirements).
- [Publicar o primeiro aplicativo Windows](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/publish-first-app).
