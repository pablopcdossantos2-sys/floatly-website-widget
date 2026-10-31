# Microsoft Store — pendências do desenvolvedor

**Última revisão:** 30 de outubro de 2026  
**Status:** não enviar para certificação ainda.

O produto **Floatly** e as propriedades básicas da Store já estão configurados. Este documento lista somente as ações humanas que ainda faltam. Siga o [guia passo a passo](GUIA_PUBLICACAO_MICROSOFT_STORE.md) para executá-las.

## Ações pendentes

- [ ] Confirmar com uma autoridade certificadora se ela emite Code Signing OV/Authenticode para a situação do desenvolvedor no Brasil (pessoa física/CPF ou empresa/CNPJ), se a cadeia atende ao Microsoft Trusted Root Program e qual é a forma de proteção da chave.
- [ ] Depois da confirmação, contratar o certificado adequado e informar ao mantenedor apenas o nome exato do publisher, a AC/produto e a modalidade de assinatura. Não enviar chave privada, PIN, senha ou PFX.
- [ ] Gerar a chave do Tauri Updater e cadastrar \`TAURI_SIGNING_PRIVATE_KEY\` e, se aplicável, \`TAURI_SIGNING_PRIVATE_KEY_PASSWORD\`.
- [ ] Aguardar a adaptação do script/workflow de assinatura Authenticode ao token, HSM ou serviço remoto escolhido; o workflow atual ainda pressupõe PFX e não deve ser executado com um token USB sem essa adaptação.
- [ ] Executar o workflow de Store depois que a integração estiver pronta e guardar o EXE assinado e \`SHA256SUMS.txt\`.
- [ ] Testar instalação, atualização in-app, desinstalação, WebView2, DPI e monitores múltiplos em um Windows limpo.
- [ ] Publicar o EXE em uma URL HTTPS direta, pública, versionada e imutável.
- [ ] Capturar screenshots reais da versão testada e revisar os textos finais da listagem.
- [ ] Executar Package Validation no Partner Center, corrigir eventuais falhas e submeter o produto.

## Trabalho que será feito no repositório

A integração do Tauri Updater, as permissões necessárias, a criação do feed estático, a geração de artefatos assinados e a adaptação do workflow ao método de assinatura contratado são trabalho técnico no código. O desenvolvedor humano não precisa editar Rust, TypeScript, YAML ou PowerShell manualmente.

Não cadastre \`WINDOWS_PFX_BASE64\` ou \`WINDOWS_PFX_PASSWORD\` até que o método de emissão e armazenamento do certificado seja confirmado e o workflow tenha sido adaptado. Certificados Code Signing atuais normalmente mantêm a chave privada protegida em token ou HSM; não presuma que ela poderá ser exportada para PFX.

## Alternativa técnica

Se não houver uma opção viável de certificado para a situação do desenvolvedor no Brasil, avaliar a migração do produto para MSIX. A Microsoft assina pacotes MSIX submetidos à Store sem exigir que o desenvolvedor compre certificado próprio. Como o Floatly já foi registrado como produto EXE/MSI, qualquer migração deve ser avaliada com o Partner Center antes de alterar ou substituir o produto atual.

Não marque uma etapa como concluída apenas porque o arquivo de configuração foi alterado: a instalação e a atualização precisam ser testadas em Windows.
