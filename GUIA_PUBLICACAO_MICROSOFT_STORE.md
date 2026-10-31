# Guia das tarefas do desenvolvedor — publicação do Floatly na Microsoft Store

**Revisado em:** 30 de outubro de 2026  
**Situação:** o produto e as propriedades básicas já estão configurados no Partner Center. Ainda não enviar para certificação.

Este guia contém somente as ações que ainda dependem do desenvolvedor humano. As etapas já concluídas não precisam ser repetidas. O trabalho técnico que pode ser feito no próprio repositório — como integrar o atualizador Tauri e ajustar o workflow — não exige edição manual de código pelo desenvolvedor.

## Antes de começar

Use o repositório https://github.com/pablopcdossantos2-sys/floatly-website-widget, sempre na branch main, e o produto **Floatly** que já existe na conta Partner Center.

O produto já tem disponibilidade, preço, categoria, classificação etária, notas de certificação, requisitos de sistema e política de privacidade configurados. Não crie outro produto nem refaça essas etapas.

## 1. Atualizar o site de suporte no Partner Center

A página dedicada de suporte já está pronta:

https://github.com/pablopcdossantos2-sys/floatly-website-widget/blob/main/SUPPORT.md

No Partner Center:

1. Abra **Apps and Games → Floatly**.
2. Entre em **Properties**.
3. Na área de informações de suporte, localize o campo **Website**.
4. Substitua a URL da raiz do repositório pela URL direta acima.
5. Salve as alterações.

A página é pública e contém o e-mail de suporte, instruções de primeira utilização e orientações de privacidade para envio de logs.

## 2. Obter o certificado Authenticode e confirmar o publisher

O pacote EXE/MSI precisa ser assinado com um certificado de assinatura de código confiável. O nome do publisher configurado no Tauri deve corresponder à identidade que aparece no certificado e ser coerente com o publisher da conta Partner Center.

1. Obtenha um certificado **Code Signing** de uma autoridade certificadora aceita pelo Microsoft Trusted Root Program.
2. Peça à autoridade certificadora que confirme o nome exato exibido como sujeito/Publisher do certificado.
3. Compare esse nome com a identidade de publicador da conta Partner Center. Não use automaticamente o nome que aparece na política de privacidade: ele identifica o responsável pelo tratamento de dados e pode não ser a grafia exigida pelo certificado.
4. Exporte o certificado com sua chave privada em um arquivo PFX e guarde-o em local seguro.

Depois de confirmar o nome exato, atualize o arquivo src-tauri/tauri.conf.json. Dentro de bundle, acrescente publisher, preservando os outros campos:

~~~json
"bundle": {
  "publisher": "NOME EXATO DO PUBLISHER NO CERTIFICADO"
}
~~~

Não substitua o objeto bundle inteiro por esse exemplo. Faça a alteração na branch main e confirme que npm run check:store-config não apresenta mais o aviso de publisher ausente.

### Cadastrar os segredos de assinatura no GitHub

1. Abra o PFX no computador Windows e converta seu conteúdo para Base64 no PowerShell:

~~~powershell
[Convert]::ToBase64String([IO.File]::ReadAllBytes("C:\caminho\publisher.pfx"))
~~~

2. No GitHub, abra **Settings → Secrets and variables → Actions**.
3. Crie os segredos do repositório:
   - WINDOWS_PFX_BASE64: saída Base64 do PFX.
   - WINDOWS_PFX_PASSWORD: senha do PFX.

Nunca envie o PFX, sua senha ou a chave privada por mensagem, issue, commit ou arquivo do repositório. Não coloque esses valores em README ou em arquivos .env.

## 3. Preparar a chave do atualizador

O instalador EXE/MSI da Store não recebe atualizações automaticamente para quem já instalou o aplicativo. A Microsoft permite atualizações dentro do próprio aplicativo quando o instalador dá suporte a elas. Por isso, o Floatly precisa de um mecanismo in-app antes de ser considerado pronto para distribuição estável.

Para este repositório, a opção padrão recomendada é usar o **Tauri Updater com um feed estático publicado em GitHub Releases**. A integração do plugin, as permissões, a verificação de atualizações e o workflow de release são alterações de código que serão feitas no repositório; não edite esses arquivos manualmente.

A ação do desenvolvedor é gerar e guardar o par de chaves:

1. No PowerShell, abra a pasta do repositório e instale as dependências:

~~~powershell
npm ci
New-Item -ItemType Directory -Force "$HOME\.tauri" | Out-Null
npm run tauri signer generate -- -w "$HOME\.tauri\floatly.key"
~~~

2. Guarde os arquivos gerados em local seguro. A chave privada é o arquivo floatly.key; a chave pública correspondente é floatly.key.pub.
3. No GitHub, em **Settings → Secrets and variables → Actions**, crie:
   - TAURI_SIGNING_PRIVATE_KEY: conteúdo da chave privada.
   - TAURI_SIGNING_PRIVATE_KEY_PASSWORD: senha escolhida na geração, se houver.
4. Mantenha uma cópia segura da chave privada e da senha fora do computador. Se perder a chave, não será possível assinar atualizações futuras para as instalações existentes.
5. Nunca publique a chave privada. A chave pública pode ser incluída na configuração do aplicativo; a privada deve existir somente no computador seguro e no segredo protegido do GitHub Actions.

Não é necessário cadastrar um endpoint agora. A integração do repositório definirá o endereço do feed e o formato do arquivo latest.json. O endereço planejado para o feed estático é:

~~~text
https://github.com/pablopcdossantos2-sys/floatly-website-widget/releases/latest/download/latest.json
~~~

Esse endereço só deverá ser usado depois que a integração estiver implementada e um release real publicar o arquivo latest.json e os artefatos assinados correspondentes.

## 4. Gerar o pacote assinado da Microsoft Store

Esta etapa depende das anteriores e da integração do atualizador no repositório.

1. Confirme que bundle.publisher, os segredos Authenticode e os segredos do atualizador estão configurados.
2. Abra a aba **Actions** do GitHub.
3. Selecione **Build Microsoft Store package**.
4. Clique em **Run workflow** e escolha a branch main.
5. Aguarde o workflow terminar. Só considere o pacote utilizável quando todos os testes, a compilação e a validação das assinaturas forem concluídos com sucesso.
6. Baixe o artefato widget-website-microsoft-store-x64.
7. Guarde o EXE e o SHA256SUMS.txt juntos, identificando a versão e o commit que os produziram.

O ZIP do Actions é temporário; não use a URL do artefato como endereço permanente do pacote na Store. Se o workflow falhar, abra a execução, identifique o primeiro passo com falha e corrija-o antes de prosseguir.

## 5. Testar o instalador em um Windows limpo

Use uma máquina virtual ou um computador Windows de teste que não tenha uma instalação anterior do Floatly. Preserve uma cópia exata do EXE e do manifesto de hashes.

### Verificar assinatura e pacote

1. No PowerShell, execute Get-AuthenticodeSignature para o EXE. O campo Status deve ser Valid.
2. Confira o nome do certificado e a cadeia de confiança.
3. Revise SHA256SUMS.txt e confirme que todos os executáveis e DLLs extraídos do instalador foram validados pelo script.
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
7. Teste o mecanismo de atualização in-app: publique uma versão posterior de teste no feed e confirme que o Floatly detecta, baixa, valida a assinatura e instala a atualização.
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
4. Informe /S como argumento de instalação silenciosa do NSIS.
5. Na listagem, use os textos de MICROSOFT_STORE_LISTING_DRAFT.md.
6. Carregue screenshots reais capturadas no Windows com a versão testada. Mostre a primeira execução, a janela flutuante, as configurações e os controles da bandeja. Não use mockups nem conteúdo de sites de terceiros sem autorização.
7. Confirme que o campo de suporte aponta para SUPPORT.md.
8. Salve as seções e execute **Package validation** em **Manage Packages**.
9. Se houver falhas, corrija o pacote, gere outro EXE, publique-o em uma nova URL versionada e execute a validação novamente.
10. Quando a validação estiver aprovada, revise a submissão e clique em **Submit to the Store**.
11. Acompanhe o resultado no Partner Center. Só anuncie a disponibilidade depois que o produto estiver publicado.

## 8. Para futuras versões

Depois da primeira publicação:

1. A atualização in-app deve ser publicada com os artefatos assinados pelo Tauri Updater e o arquivo latest.json no GitHub Release.
2. Publique também o novo instalador Authenticode em uma URL versionada.
3. Crie uma atualização da submissão existente no Partner Center e envie o novo pacote.
4. Não substitua arquivos de versões anteriores que já tenham sido publicados.

## Referências oficiais

- [Microsoft Learn — atualizações de aplicativos MSI/EXE na Store](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/publish-update-to-your-app-on-store).
- [Microsoft Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies), especialmente a regra 10.2.1.
- [Tauri — Microsoft Store](https://tauri.app/distribute/microsoft-store/).
- [Tauri — Updater plugin](https://v2.tauri.app/plugin/updater/).
- [Tauri — assinatura de aplicativos Windows](https://tauri.app/distribute/sign/windows/).
