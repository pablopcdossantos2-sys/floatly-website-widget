# Microsoft Store — pendências do desenvolvedor

**Última revisão:** 30 de outubro de 2026  
**Status:** aguardar definição do formato de publicação antes de comprar certificados ou gerar chaves.

O produto **Floatly** e as propriedades básicas da Store já estão configurados. O site de suporte também já foi atualizado. Não é necessário repetir essas tarefas.

## Próxima ação humana

- [ ] Abrir um chamado no Partner Center perguntando se a ficha Floatly, criada como **EXE or MSI app** e ainda não publicada, pode ser convertida para **MSIX**, preservando o nome e a identidade do produto. Use a mensagem pronta do [guia passo a passo](GUIA_PUBLICACAO_MICROSOFT_STORE.md).

**Não compre um certificado Code Signing, não gere chave do Tauri Updater e não exclua a ficha atual antes da resposta do suporte.**

## Depois da resposta do Partner Center

### Se o suporte permitir MSIX

A Microsoft assina o pacote MSIX depois da certificação e gerencia as atualizações da distribuição pela Store. Não será necessário comprar certificado de uma AC, enviar PFX ou usar token USB/HSM.

O trabalho no repositório será migrar o empacotamento Tauri para MSIX, configurar o manifesto/identidade e testar instalação, persistência de dados, bandeja, atalhos, janela flutuante e atualizações pela Store. O desenvolvedor humano fará os testes finais e a submissão no Partner Center.

### Se for necessário manter EXE/MSI

Aí sim será necessário adquirir um certificado Code Signing compatível com Authenticode e com o Microsoft Trusted Root Program. O guia contém um procedimento para consultar a certificadora, verificar emissão para CPF/CNPJ e confirmar token/HSM antes de pagar.

Nesse modelo, o Tauri Updater e o processo de assinatura do instalador precisarão ser preparados no repositório. O workflow atual pressupõe PFX e não deve ser executado com token USB/HSM sem adaptação.

## Trabalho que será feito no repositório

Depois de definida a rota, a integração técnica correspondente — empacotamento MSIX ou assinatura EXE/MSI, permissões, atualizações e workflow — será realizada diretamente na branch main. O desenvolvedor humano não precisa editar Rust, TypeScript, YAML ou PowerShell manualmente.

Não marque uma etapa como concluída apenas porque o arquivo de configuração foi alterado: a instalação e o fluxo de atualização precisam ser testados em Windows.