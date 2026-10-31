# Microsoft Store — pendências do desenvolvedor

**Última revisão:** 30 de outubro de 2026  
**Status:** não enviar para certificação ainda.

O produto **Floatly** e as propriedades básicas da Store já estão configurados. Este documento lista somente as ações humanas que ainda faltam. Siga o [guia passo a passo](GUIA_PUBLICACAO_MICROSOFT_STORE.md) para executá-las.

## Ações pendentes

- [ ] Atualizar o campo Website de suporte no Partner Center para a URL direta de [SUPPORT.md](https://github.com/pablopcdossantos2-sys/floatly-website-widget/blob/main/SUPPORT.md).
- [ ] Obter um certificado Authenticode confiável e confirmar o nome exato do publisher.
- [ ] Configurar bundle.publisher e os segredos WINDOWS_PFX_BASE64 e WINDOWS_PFX_PASSWORD no GitHub.
- [ ] Gerar a chave do Tauri Updater e cadastrar TAURI_SIGNING_PRIVATE_KEY e, se aplicável, TAURI_SIGNING_PRIVATE_KEY_PASSWORD.
- [ ] Aguardar a integração do Tauri Updater e do feed de releases no repositório; essa é uma alteração de código, não uma tarefa de edição manual do Partner Center.
- [ ] Executar o workflow de Store depois que a integração estiver pronta e guardar o EXE assinado e SHA256SUMS.txt.
- [ ] Testar instalação, atualização in-app, desinstalação, WebView2, DPI e monitores múltiplos em um Windows limpo.
- [ ] Publicar o EXE em uma URL HTTPS direta, pública, versionada e imutável.
- [ ] Capturar screenshots reais da versão testada e revisar os textos finais da listagem.
- [ ] Executar Package Validation no Partner Center, corrigir eventuais falhas e submeter o produto.

## Trabalho que será feito no repositório

A integração do Tauri Updater, as permissões necessárias, a criação do feed estático, a geração de artefatos assinados e o workflow de release são trabalho técnico no código. O desenvolvedor humano não precisa editar Rust ou TypeScript para realizá-los; precisa somente definir a hospedagem, gerar a chave e cadastrar os segredos conforme o guia.

Não marque uma etapa como concluída apenas porque o arquivo de configuração foi alterado: a instalação e a atualização precisam ser testadas em Windows.
