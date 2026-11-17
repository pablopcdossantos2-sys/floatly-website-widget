# Floatly 1.0.1 — atualização da Microsoft Store

## Estado e origem

- Versão do aplicativo: **1.0.1** (Tauri, Cargo e npm).
- Versão esperada no manifesto MSIX: **1.0.1.0** (4º componente = 0).
- Arquitetura: Windows x64. Mínimo: Windows 10 versão 2004 / build 19041.
- Workflow oficial: [Build MSIX for Microsoft Store](https://github.com/pablopcdossantos2-sys/floatly-website-widget/actions/workflows/build-msix.yml).
- O arquivo **`floatly-msix-x64`** é um artefato temporário do Actions; baixe-o antes de expirar.
- **Atenção:** verificar no Partner Center se a versão mais alta publicada é inferior a **1.0.1.0**. Se não for, atualizar a versão em todas as fontes e reconstruir antes do envio.
- Conferir **Package identity** (Name, Publisher, PublisherDisplayName) com os dados atuais do Partner Center antes de enviar. O workflow possui valores padrão de uma submissão anterior; não presumir que o cadastro não mudou.

## Alterações da versão 1.0.1

### Português (Brasil)
- As URLs usadas recentemente ficam salvas no próprio dispositivo (até 20) e podem ser reutilizadas nas Configurações depois de reiniciar.
- Reforçada a segurança da janela auxiliar de autenticação: a barra de endereços não altera a URL principal.
- Melhorias na navegação, mensagens de erro e recuperação após falha de conexão.
- Suporte ao botão primário do mouse mesmo com o mapeamento invertido no Windows.
- Correções de desempenho, posicionamento inicial e textos da interface.

### English
- Remember up to 20 recently used websites locally and select them again after restarting.
- Hardened the sign-in window so it cannot change the primary configured website.
- Improved navigation failures, retry behavior and connection recovery.
- Fixed dragging/resizing for swapped mouse buttons and improved startup window positioning.

### Español
- Se recuerdan localmente hasta 20 sitios web recientes para reutilizarlos tras reiniciar.
- Mayor seguridad en la ventana auxiliar de inicio de sesión.
- Mejoras en navegación, reintentos y recuperación de conexión.
- Corrección del arrastre/redimensionamiento con botones de ratón intercambiados y posición inicial.

## Ações automatizadas no GitHub

1. Integrar código aprovado à `main` e atualizar versão de `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` e `src-tauri/tauri.conf.json`.
2. Confirmar workflow **Build Windows** e workflow **Build MSIX for Microsoft Store** com status **success** no commit da versão.
3. Em **Actions → Build MSIX for Microsoft Store → execução mais recente → Artifacts**, baixar `floatly-msix-x64`.
4. Extrair o ZIP, conferindo **`Floatly_1.0.1.0_x64.msix`** e **`SHA256SUMS.txt`**. Conferir hash SHA-256 quando possível. **Não** enviar o próprio ZIP do Actions para a Store.
5. O pacote MSIX é **sem assinatura de desenvolvedor** por projeto; a Microsoft assina o pacote após a certificação. O instalador NSIS é separado e não deve ser enviado no lugar deste MSIX.

## Ações que exigem acesso ao Partner Center

1. Acessar [Partner Center](https://partner.microsoft.com/dashboard) e abrir a ficha do Floatly.
2. Verificar a versão mais alta dos pacotes **atualmente publicados**, a identidade do aplicativo e se já existe uma submissão de atualização em andamento.
3. Selecionar **Start update / Update** na área **Product release** (ou continuar o rascunho existente).
4. Em **Packages**, enviar **`Floatly_1.0.1.0_x64.msix`**; não remover pacotes de outras arquiteturas se estiverem em produção sem uma decisão explícita.
5. Revisar **Store listing**, capturas, descrição, política de privacidade, informações de contato e todos os alertas do Partner Center. Texto curto de novidades disponível acima, em três idiomas.
6. Executar o teste do aplicativo instalado pela Store e testar persistência das URLs, primeiro acesso, conexão e autenticação.
7. **Submit for certification** e acompanhar a certificação. Após aprovar, verificar versão disponível e atualização de instalação anterior.

## Limitações e cuidados

- O CI valida código, testes Rust, frontend, instalador NSIS e estrutura MSIX, mas **não substitui** teste manual da experiência real no Windows, a verificação do Partner Center ou a certificação da Microsoft.
- As URLs são salvas em `config.json`. Como URLs podem conter dados privados em caminhos, queries e fragmentos, evite usar links com segredos e não envie o arquivo de configuração a terceiros.
- Os dados existentes ficam na pasta do perfil anterior do aplicativo para evitar perda durante a atualização, mas é prudente fazer um backup de `%APPDATA%\WidgetWebsite` antes de testar a atualização.
- No relatório original, ainda há melhorias de baixa prioridade e verificações específicas do Windows não realizadas por teste humano.
- Referências Microsoft: [Publish an update to your MSIX app](https://learn.microsoft.com/windows/apps/publish/publish-your-app/msix/publish-update-to-your-app-on-store) e [MSIX Store readiness](https://learn.microsoft.com/en-us/windows/apps/publish/store-readiness-checklist).
