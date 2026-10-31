# Prompt — Auditoria do repositório Floatly (`floatly-website-widget-main`) em 4 partes

> **Como usar:** anexe o ZIP `floatly-website-widget-main.zip` (ou o repositório) à outra IA e cole este arquivo inteiro como instrução.
> Você pode pedir tudo de uma vez ou **uma parte por vez** (ex.: "Execute a Parte 2"). Veja a seção 9.

---

## 1. Papel e objetivo

Você é um(a) engenheiro(a) sênior de software especializado(a) em **Rust, Tauri 2, TypeScript, WebView2/Windows e CI/CD**, atuando como auditor(a) de código.

Sua tarefa:

1. Ler e entender o repositório anexado (Floatly).
2. Dividi-lo nas **4 partes** definidas na seção 4 e analisar **cada parte individualmente**, em busca de erros, falhas, vulnerabilidades, inconsistências e riscos.
3. Produzir **um relatório em Markdown (`.md`)** descrevendo cada problema encontrado e propondo uma solução concreta.

**Ponto crítico:** o relatório **não será lido por uma pessoa, e sim por outra IA que irá corrigir os problemas**. Portanto, cada achado precisa ser **autossuficiente, inequívoco e executável**: localização exata, evidência, causa raiz, passos de correção, critérios de aceite e forma de verificação. Evite frases vagas como "melhorar o tratamento de erros" sem dizer onde, como e como validar.

---

## 2. Regras de conduta (obrigatórias)

1. **Somente leitura.** Não altere, apague nem reescreva arquivos do projeto. Você apenas analisa e escreve o relatório. Propostas de código vão **dentro do relatório**, como trechos/diffs.
2. **Evidência antes de afirmação.** Todo achado deve citar `arquivo:linha(s)` e um trecho real do código. Se não leu o arquivo inteiro, **não** conclua nada sobre ele.
3. **Classifique a confiança** de cada achado: `Confirmado` (reproduzível ou comprovado pelo código), `Provável` (forte indício, depende de contexto externo) ou `Hipótese` (precisa de teste/execução para confirmar). Nunca apresente hipótese como fato.
4. **Não invente.** Não cite funções, versões, APIs, flags ou comportamentos que você não verificou no código ou na documentação oficial. Se for incerto, diga "não verificado".
5. **Sem falsos positivos por checklist.** Antes de reportar, tente refutar o seu próprio achado (há teste que cobre? há `cfg` que isola? há validação em outra camada?). Registre o que foi refutado na seção "Itens descartados" do relatório.
6. **Respeite o escopo do produto.** O Floatly é um widget Windows que exibe **um site escolhido pelo usuário** em uma janela flutuante. Não proponha mudar a arquitetura, trocar de framework, adicionar suporte a macOS/Linux ou reescrever módulos inteiros, salvo se for a **única** forma de corrigir uma falha grave (justifique).
7. **Preserve convenções do repositório:** comentários e logs em português (pt-BR), textos de interface nos 3 idiomas (`pt-BR`, `en`, `es`), scripts `check:*` existentes, testes Rust inline (`#[cfg(test)]`). Toda correção proposta deve manter isso.
8. **Plataforma-alvo:** Windows x64 (`x86_64-pc-windows-msvc`). Partes do código usam `#[cfg(windows)]`, `windows-sys`, `windows` e WinRT. Se você estiver em outro SO, **não** tente rodar `tauri build`; analise estaticamente e diga o que não pôde executar.
9. **Não exponha segredos.** Se encontrar algo que pareça credencial, informe apenas o local, não repita o valor.
10. **Seja proporcional.** Prefira poucos achados bem fundamentados a uma lista longa de apontamentos triviais. Agrupe estilo/nitpicks em um único achado de severidade Baixa por parte.

---

## 3. Contexto do projeto

| Item | Detalhe |
|---|---|
| Produto | **Floatly** — navegador compacto/janela flutuante que mantém um site escolhido pelo usuário visível |
| Stack | Tauri 2 (Rust 1.90, edition 2021) + frontend TypeScript/Vite (sem framework) + WebView2 |
| Pacote npm / crate | `widget-website` v1.0.0 (nome interno antigo; marca atual: Floatly) |
| Identificador | `com.pablopcdossantos.widgetwebsite` |
| Diretório de dados | `%APPDATA%\WidgetWebsite` (`config.json`, logs, `webview-data/`) |
| Distribuição | Instalador NSIS (PT-BR/EN/ES) e pacote **MSIX** para Microsoft Store (StartupTask `FloatlyStartup`) |
| Plugins Tauri | `autostart`, `single-instance`, `global-shortcut`, `tray-icon` |
| Janelas | `main` (site remoto + script injetado), `settings` (UI local), janela de `login` (popup de autenticação) |
| IPC | 5 comandos: `get_config`, `save_config`, `open_settings`, `close_settings`, `heartbeat` |
| Segurança declarada | CSP restrita nas páginas locais; capacidades Tauri separadas (`main-local`, `settings-local`, `main-remote-heartbeat`); token por processo para `heartbeat` e para ações `widget-action://` |

**Fluxo resumido:** `main.rs` carrega a configuração, registra o atalho global (padrão `CTRL+ALT+C`), cria a janela principal (`window::create_main`), a bandeja (`tray`), o auto-reload e o *watchdog* de diagnóstico. A página local `index.html`/`src/main.ts` testa a conexão e redireciona (`location.replace`) para o site configurado. Dentro do site, o script `INIT` injetado por `window.rs` adiciona alça de arrastar/redimensionar, barra de rolagem discreta, tela offline e envia *heartbeat*. Navegações são classificadas por `decide_main`/`decide_login` (permitir, abrir popup de login, abrir externamente ou bloquear).

**Árvore relevante (≈ 5,5 mil linhas):**

```
src/                 main.ts (151) · settings.ts (374) · styles.css (53)
index.html · settings.html · vite.config.ts · tsconfig.json · package.json
src-tauri/src/       main.rs (305) · config.rs (703) · commands.rs (439) · diagnostics.rs (571)
                     window.rs (1768) · tray.rs (88) · autostart.rs (85) · i18n.rs (105)
src-tauri/           Cargo.toml · build.rs · tauri.conf.json · capabilities/*.json · windows/hooks.nsh
scripts/             check-acl · check-drag-overlay · check-msix-config · check-tauri-versions
                     check-theme-contrast (.mjs) · build-msix.ps1
packaging/msix/      Package.appxmanifest.template.xml
.github/             workflows/build-windows.yml (243) · workflows/build-msix.yml (144) · dependabot.yml
docs                 README.md · README-en.md · README.es.md · PRIVACY_POLICY.md · SUPPORT.md
                     GUIA_PUBLICACAO_MICROSOFT_STORE.md · MICROSOFT_STORE_LISTING_DRAFT.md · MICROSOFT_STORE_READINESS.md
```

---

## 4. Divisão em 4 partes

Cada parte deve ser analisada **por inteiro e isoladamente**, e depois você deve registrar as **interfaces entre partes** (achados que dependem de mais de uma parte).

### Parte 1 — Núcleo Rust: ciclo de vida, configuração, comandos IPC e diagnóstico

**Arquivos:** `src-tauri/src/main.rs`, `config.rs`, `commands.rs`, `diagnostics.rs`, `autostart.rs`, `src-tauri/Cargo.toml`, `build.rs`.

**O que investigar:**
- **Inicialização (`main.rs`):** ordem do `setup`, tratamento de erros (`?` vs `log`), registro do atalho global e *fallback* para `CTRL+ALT+C`, detecção do WebView2, tratamento de `WindowEvent::Destroyed` (encerra com `exit(1)`) e `CloseRequested`; risco de falsos positivos de "janela destruída inesperadamente".
- **Configuração (`config.rs`):** `normalize()` (URL, `http`/`https`, credenciais embutidas, limites de largura/altura/zoom/auto-reload/raio, parsing do atalho), `load_with_warnings` (JSON corrompido, campos ausentes, preservação do arquivo original, `safe_to_persist`), `save` (escrita atômica, arquivo temporário, `rename` no Windows, antivírus/arquivo em uso, limpeza de temporários), `app_directory()` (fallback para `current_dir()` quando `APPDATA` não existe; comportamento sob MSIX).
- **Concorrência:** `Mutex` com recuperação de *poison* (`lock_config`) vs. `registered_shortcut.lock()` sem essa recuperação; `SAVE_LOCK`; ordem de aquisição de locks (risco de *deadlock*); salvamento com *debounce* de geometria vs. `save_config`; `AtomicU64`/`AtomicBool` e ordenações (`Relaxed` vs `SeqCst`).
- **Comandos (`commands.rs`):** `ConfigPatch::apply_to`, `save_config` (rollback em falha de disco, troca de atalho com `register`/`unregister` e consistência se uma das duas falhar, efeito em autostart, `force_resize`), `heartbeat` (comparação do token), `get_config` (sobrescreve `autostart` com o estado real do SO), `reload_site`/`quit_app`/`toggle_*` (quais são realmente expostos?).
- **Autostart (`autostart.rs`):** divergência entre o plugin `autostart` (NSIS, entrada `Run` nomeada pelo crate) e o `StartupTask` WinRT (MSIX); estados `Disabled/DisabledByUser/EnabledByPolicy`; erros assíncronos bloqueando thread.
- **Diagnóstico (`diagnostics.rs`):** sanitização de logs (`sanitize_log_message`, `sanitize_url` — vazamento de query string/token/credenciais), rotação e poda de backups, limite de tamanho, *rate limiting* de linhas, *panic hook*, **watchdog** (critérios de "janela travada", ações tomadas, falsos positivos quando o site não envia *heartbeat*, custo de CPU, consulta a processos filhos do WebView2 via ToolHelp).
- **Dependências do crate:** versões mistas `windows` 0.62 / `windows-future` 0.3 / `windows-sys` 0.61, features realmente usadas, duplicidade de versões, `unsafe`.

### Parte 2 — Janela, navegação, login e script injetado (`window.rs`)

**Arquivos:** `src-tauri/src/window.rs` (1.768 linhas — **leia por inteiro, em blocos**), `tray.rs`, `i18n.rs`.

**O que investigar:**
- **Política de navegação:** `normalized_host`, `host_matches` (subdomínios, IPs, ponto final, *punycode*/IDN, `localhost`), `is_auth_provider`, `is_login_path` (lista `LOGIN_SEGMENTS` pode pegar caminhos legítimos como `/account`, `/accounts`, `/auth` do próprio site → abrir popup indevido), `decide_main`, `decide_login`, `should_close_login`, redirecionamentos em cadeia, esquemas não-http (`javascript:`, `data:`, `blob:`, `file:`, `about:`), `is_internal_app_url` (`cfg!(debug_assertions)` liberando `127.0.0.1` em *release*?).
- **Script `INIT` e `LOGIN_INIT` (strings JS dentro do Rust):**
  - Os tokens `__WIDGET_ACTION_TOKEN__` e `__WIDGET_HEARTBEAT_TOKEN__` são **injetados no contexto da página remota**: qualquer script do site (ou de terceiros embutidos) consegue lê-los. Avalie o que um site malicioso/comprometido consegue fazer com eles (disparar `widget-action://…/drag|resize/*`, chamar `heartbeat`, forjar atividade).
  - Substituição de *placeholders* por `replace`: escape correto de idioma/valores; injeção em `UI_LANGUAGE`.
  - Execução em `window.top !== window` (iframes), `DOMContentLoaded`, `MutationObserver` (custo/loops), colisão de IDs `__widget_*` e CSS global (`html{overflow-y:auto!important}`, `*{scrollbar-…}`) quebrando layout/acessibilidade de sites de terceiros.
  - Textos da UI injetada: há resquícios de um produto anterior específico (ex.: mensagem *"Sem conexão com o calendário"* / *"Calendar connection unavailable"*) em um widget agora genérico.
  - `location.href='widget-action://…'` como canal de comando: interceptação, *race conditions*, bloqueio por CSP do site, falha silenciosa.
- **Janela principal (`create_main`):** `on_navigation`, `on_new_window`/popups, `initialization_script`, `data_directory`, zoom, `always_on_top`, `skip_taskbar`, `decorations`/`shadow`, restauração de posição (`restore_position`, `is_placeable`, monitores desconectados, DPI misto, coordenadas negativas).
- **Win32 / `unsafe`:** `begin_native_drag`, `begin_native_resize`, `pack_lparam`, `apply_native_corners` (`CreateRoundRectRgn`/`SetWindowRgn`, **vazamento de HRGN/GDI**, leitura de DPI), `MonitorBounds`, `corner_region_dims`; sempre verificar `#[cfg(windows)]` vs. *stubs* e invariantes de `unsafe`.
- **Login popup:** `open_login`, `LOGIN_WINDOW_LOCK`, `LOGIN_LAST_OPEN_MS`, `LOGIN_POPUP_LAST_MS`, `login_window_action`, `close_login_and_reload` (limite de janelas, *race* entre fechar e recarregar, compartilhamento de cookies/sessão com a janela principal).
- **Abertura externa:** `open_external_limited` / `ExternalOpenState` (limitador, `open::that` com URL não confiável, apenas `http/https`).
- **Visibilidade e foco:** `show_main`, `toggle_main_visibility`, `toggle_action` (minimizada/oculta/sem foco), `open_settings` e `SETTINGS_WINDOW_CREATING` (condição de corrida e **flag que pode ficar presa em `true`** se a criação falhar).
- **Persistência de geometria:** `schedule_debounced_config_save`, `flush_pending_config`, `save_window_state`, `is_valid_saved_position/size`, janela minimizada (coordenadas `-32000`), salvamento durante o encerramento.
- **Auto-reload:** `ReloadState`, `next_reload_state`, `start_auto_reload` (thread em laço, intervalo mínimo, reload com janela oculta, perda de sessão/formulários, parada no encerramento).
- **Tray (`tray.rs`) e i18n (`i18n.rs`):** itens do menu, atualização de idioma em tempo de execução, rótulos de bloqueado/desbloqueado, tratamento de erros de ícone, cobertura dos 3 idiomas.
- **Testes inline:** módulos `toggle_action_tests`, `navigation_policy_tests`, `internal_app_navigation_tests`, `corner_geometry_tests`, `geometry_tests` — lacunas (IDN, IPv6, `//`, `%2e`, portas, `userinfo@`, subdomínio parecido como `evilexample.com`).

### Parte 3 — Interface local (frontend), IPC e superfície de segurança

**Arquivos:** `index.html`, `settings.html`, `src/main.ts`, `src/settings.ts`, `src/styles.css`, `vite.config.ts`, `tsconfig.json`, `package.json` (scripts/dependências de runtime), `src-tauri/tauri.conf.json` (seção `app`/`security`), `src-tauri/capabilities/*.json`.

**O que investigar:**
- **`src/main.ts`:** `fetch(new URL("/favicon.ico", target), { mode: "no-cors" })` como teste de conectividade — compatibilidade com a **CSP** (`connect-src … https:` apenas) e com sites em `http://`; `timer` do `setInterval` de 30 s nunca limpo; `navigating` preso em `true`; `location.replace(target)` sem revalidar o esquema; sites que bloqueiam `/favicon.ico` (403/redirect/HSTS) levando a falso "sem conexão"; internacionalização e acessibilidade (`aria-live`, foco do botão).
- **`src/settings.ts` (374 linhas):** leitura/validação do formulário (`readPatch`), números inválidos/`NaN`/vazios, `save_config` com `forceResize`, tratamento de `warning` e de erros localizados (chaves como `config.site_url_invalid`), captura de atalho (`keydown`: teclas mortas, `Meta`, `AltGr`, layouts não-US), botão "restaurar padrões", `listen`/`unlisten` de eventos (vazamento de *listeners*), tema claro/escuro persistido, sincronização do idioma com a bandeja, uso de `innerHTML`/`textContent`.
- **`settings.html` / `styles.css`:** `data-i18n*` sem tradução correspondente, rótulos/`aria`, foco visível, contraste (cruzar com `check-theme-contrast.mjs`), responsividade em janela pequena, CSS morto.
- **CSP e Tauri (`tauri.conf.json`):** `style-src 'unsafe-inline'`, `connect-src` com `https:` amplo, `img-src`, `form-action 'self' https:`; **`withGlobalTauri: true`** (expõe `window.__TAURI__` a qualquer página que carregue na janela principal, inclusive o site remoto); coerência entre o `csp` e o que as páginas locais realmente usam.
- **Capabilities:** `main-remote-heartbeat.json` concede `allow-heartbeat` a **`http://*` e `https://*`** na janela `main` (inclui HTTP sem TLS e qualquer domínio) — avaliar se deveria ser restrito ao host configurado/dinâmico; mistura de idiomas nas descrições (espanhol/português); `$schema` apontando para `../gen/schemas/…` (diretório ignorado pelo git); permissões mínimas por janela; risco de o site remoto invocar comandos além de `heartbeat`.
- **Tipagem:** `type Cfg` frouxo (`[key: string]: unknown`), `!` não-nulo em seletores do DOM sem verificação, `tsconfig` (`strict` ok, mas sem `noUncheckedIndexedAccess`), ausência de ESLint/Prettier e de testes de frontend.
- **Build do frontend:** `vite.config.ts` (`strictPort`, `rollupOptions.input` com duas páginas), versões declaradas em `package.json` (`typescript ~7.0.2`, `vite ^8.3.5`) — conferir se existem, são compatíveis entre si e com Node `>=22.12`.

### Parte 4 — Build, CI/CD, empacotamento, dependências, documentação e distribuição

**Arquivos:** `.github/workflows/*.yml`, `.github/dependabot.yml`, `scripts/*` (todos), `packaging/msix/Package.appxmanifest.template.xml`, `src-tauri/windows/hooks.nsh`, `src-tauri/tauri.conf.json` (seção `bundle`), `src-tauri/Cargo.toml` e `Cargo.lock`, `package.json`/`package-lock.json`, `rust-toolchain.toml`, `.gitignore`, `README*.md`, `PRIVACY_POLICY.md`, `SUPPORT.md`, `GUIA_PUBLICACAO_MICROSOFT_STORE.md`, `MICROSOFT_STORE_*.md`.

**O que investigar:**
- **Workflows:** ações fixadas por SHA (verifique se os SHAs/comentários de versão são coerentes e se algum `uses:` está sem versão, ex.: `dtolnay/rust-toolchain` com comentário genérico), `permissions` por *job* (o job `sign-and-publish` herda permissões?), segredos `WINDOWS_PFX_BASE64`/`WINDOWS_PFX_PASSWORD` (execução em PRs, *forks*, vazamento em logs, limpeza do certificado), `signtool` com `/tr http://…` (timestamp sem TLS), `cargo install cargo-audit` a cada execução (lento/instável), cache apenas do registry (sem `target`), `concurrency` com `cancel-in-progress` em `main`, filtros `paths:`, `runs-on: windows-2025`, passo `npm run tauri build -- -- --locked`, verificação do subsistema PE, `upload-artifact` e retenção, coerência entre `build-windows.yml` e `build-msix.yml` (leia o segundo por inteiro).
- **Scripts `check-*.mjs`:** são verificações baseadas em **regex sobre o código-fonte** (frágeis: quebram com reformatação ou renomeação e podem passar quando não deveriam); caminhos relativos; mensagens de erro; falsos negativos; `check-acl.mjs` compara `generate_handler!` ↔ `build.rs` ↔ capabilities.
- **`build-msix.ps1` e manifesto MSIX:** parâmetros de identidade/publisher, `Package.appxmanifest.template.xml` (capabilities declaradas, `runFullTrust`, `StartupTask`, versões mínimas do Windows, *placeholders* sem substituição), assinatura, escape de variáveis PowerShell, tratamento de `$LASTEXITCODE`, limpeza de diretórios.
- **NSIS (`hooks.nsh`):** remoção de `RunKey` (`widget-website`/`Widget Website`), `RmDir /r` em `$APPDATA\WidgetWebsite` e `$LOCALAPPDATA\WidgetWebsite` (confirme que cobre `webview-data` e que não apaga dados fora do escopo; WebView2 também pode criar pasta com o *identifier*), comportamento em `UpdateMode`, idiomas.
- **Dependências:** `npm audit`/`cargo audit` (não há rede? então apenas analise a árvore e as versões), versões muito recentes ou inexistentes, `^` vs `~`, *lockfiles* coerentes, `Cargo.toml` (features mínimas, `rust-version`), `dependabot.yml` agrupando *minor/patch* em grupos **separados para npm e cargo** — isso pode gerar PRs que falham em `check:tauri-versions` (API/CLI/crate do Tauri precisam ficar na mesma versão *major.minor*).
- **`.gitignore` e reprodutibilidade:** `src-tauri/icons/` e `src-tauri/gen/` ignorados, mas `tauri.conf.json` referencia `icons/32x32.png`, `128x128.png` e `icon.ico` — um *clone* limpo compila sem rodar `npm run icons`? O README documenta isso?
- **Nomes inconsistentes (marca vs. nome interno):** `Floatly` (produto) × `widget-website` (crate, npm, executável, artefatos, `concurrency`) × `WidgetWebsite` (pasta de dados) × `com.pablopcdossantos.widgetwebsite` (identifier). Avalie riscos de migração/atualização e de submissão à Store, **sem** propor renomeação ampla a menos que seja necessária.
- **Documentação e conformidade:** `README.md` / `README-en.md` / `README.es.md` (nomes de arquivo inconsistentes, instruções desatualizadas, comandos que não existem), `PRIVACY_POLICY.md` (38 KB) **versus o comportamento real do código** (logs locais, *heartbeat*, abertura de links externos, autostart, dados do WebView2, ausência de telemetria), `SUPPORT.md`, guia e *listing* da Microsoft Store (capabilities declaradas × usadas, classificação etária, textos que prometem algo que o código não faz), links quebrados e dados pessoais desnecessários.

---

## 5. Metodologia (aplicar a **cada** parte)

1. **Inventário:** liste os arquivos da parte, tamanho, responsabilidade e dependências com as outras partes.
2. **Leitura integral:** leia o código por completo (em blocos, para `window.rs`). Registre funções/estruturas centrais e invariantes esperados.
3. **Análise por categorias:** segurança · corretude/lógica · concorrência · tratamento de erros · robustez (entradas inválidas, falhas de disco/rede/SO) · desempenho/recursos (threads, laços, *handles*) · UX/acessibilidade · i18n · consistência de nomes/configuração · testes ausentes · manutenibilidade.
4. **Tentativa de refutação** de cada hipótese (regra 5 da seção 2).
5. **Verificação executável (quando possível):** rode os comandos da seção 7 relevantes à parte e anexe o resultado (ou diga por que não rodou).
6. **Priorização:** atribua severidade e esforço (escalas na seção 8).
7. **Proposta de correção** completa para cada achado, já pensada para ser aplicada por outra IA.

---

## 6. Pontos de atenção iniciais (hipóteses a confirmar — **não** são conclusões)

Estes pontos vieram de uma leitura rápida e **precisam ser verificados por você**. Confirme, refute ou refine cada um e registre o resultado no relatório (confirmado → achado; refutado → "Itens descartados").

| # | Parte | Hipótese |
|---|---|---|
| H1 | 3 | A CSP (`connect-src ipc: http://ipc.localhost https:`) bloqueia o `fetch` de `main.ts` para sites `http://`, apesar de `normalize()` aceitar `http`. Resultado esperado: sites HTTP sempre caem em "não foi possível acessar". |
| H2 | 2/3 | Os tokens `ACTION_TOKEN` e `HEARTBEAT_TOKEN` ficam legíveis por scripts do site remoto; `withGlobalTauri: true` e a capability `main-remote-heartbeat` (`http://*`, `https://*`) ampliam a superfície. Avaliar impacto real e mitigação proporcional. |
| H3 | 2 | `LOGIN_SEGMENTS` inclui `account`, `accounts`, `auth`, `callback`; páginas legítimas do site configurado com esses segmentos podem ser desviadas para o popup de login (exceto a URL exata configurada). |
| H4 | 2 | `SETTINGS_WINDOW_CREATING` (e travas similares de login) podem ficar presas se a criação da janela falhar ou entrar em *panic*, impedindo reabrir as Configurações. |
| H5 | 2 | O texto da tela offline injetada menciona "calendário", resquício de uma versão anterior específica; incoerente com o produto genérico. |
| H6 | 1/2 | O *watchdog* e o *heartbeat* dependem de o site executar o script injetado e de `window.__TAURI__` existir; em sites com CSP/`Trusted Types` rígidos ou páginas sem JS, pode haver diagnóstico/ações indevidas. |
| H7 | 1 | `registered_shortcut.lock().map_err(...)` em `save_config` não usa a recuperação de *poison* aplicada a `config`; um *panic* anterior faria todas as gravações de configuração falharem. |
| H8 | 1 | `app_directory()` recorre a `current_dir()` quando `APPDATA` não existe; sob MSIX/ambientes restritos isso pode gravar em local inesperado ou sem permissão. |
| H9 | 4 | Grupos do Dependabot separados para npm e cargo podem produzir PRs que quebram `check:tauri-versions`. |
| H10 | 4 | `.gitignore` ignora `src-tauri/icons/`, mas `tauri.conf.json` os exige; *clone* limpo pode falhar em `cargo test`/`tauri dev` sem `npm run icons`. |
| H11 | 4 | Os `check-*.mjs` validam por regex no código-fonte; mudanças de formatação podem gerar falsos erros ou falsos passes. |
| H12 | 4 | A `PRIVACY_POLICY.md` e os textos da Store podem divergir do comportamento real (logs, heartbeat, dados do WebView2). |
| H13 | 4 | Versões `typescript ~7.0.2`, `vite ^8.3.5` e `windows 0.62.2`/`windows-future 0.3.2`: confirmar existência, compatibilidade e *lockfile* coerente. |
| H14 | 1/4 | Nome da pasta de dados (`WidgetWebsite`), *Run key* (`widget-website`) e marca (`Floatly`) divergem: risco em atualização, desinstalação e migração de configuração. |

---

## 7. Comandos de verificação (execute o que for possível e registre a saída)

```bash
# Frontend / Node (>= 22.12)
npm ci
npm run typecheck
npm run check:tauri-versions
npm run check:acl
npm run check:drag-overlay
npm run check:contrast
npm run check:msix-config
npm run build            # typecheck + vite build

# Rust (toolchain 1.90.0, alvo x86_64-pc-windows-msvc)
cargo fmt   --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test  --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings

# Auditoria de dependências (requer rede)
npm audit --audit-level=high
cargo audit --file src-tauri/Cargo.lock
```

Observações: `npm run icons` precisa rodar antes de testes/build Rust em *clone* limpo. `tauri build`, MSIX e assinatura só funcionam em Windows com as ferramentas instaladas. Se algo não puder ser executado, **escreva "não executado" e o motivo**; não presuma que passou.

---

## 8. Formato do relatório

**Arquivo de saída:** `RELATORIO_ANALISE_FLOATLY.md` (um único `.md`, em português do Brasil).
Se você executar uma parte por vez, gere `RELATORIO_ANALISE_FLOATLY_PARTE_N.md` (N = 1 a 4) e, ao final, a versão consolidada.

### 8.1 Estrutura obrigatória

1. **Resumo executivo** — visão geral, nota de risco (Baixo/Médio/Alto/Crítico), contagem de achados por severidade e por parte (tabela) e os 5 achados mais importantes.
2. **Escopo, premissas e limitações** — o que foi lido, o que foi executado, o que **não** foi verificado e por quê.
3. **Mapa das 4 partes** — arquivos analisados e interfaces entre partes.
4. **Índice de achados** — tabela: `ID | Título | Parte | Severidade | Confiança | Esforço | Depende de`.
5. **Achados detalhados por parte** — Parte 1, 2, 3 e 4, cada uma no formato 8.2.
6. **Achados transversais** — problemas que envolvem mais de uma parte (indique todas as partes e a ordem de correção).
7. **Plano de correção ordenado** — ver 8.3.
8. **Testes a adicionar** — lista de testes (Rust inline, scripts `check:*`, testes de frontend se houver), com nome sugerido e o que cada um cobre.
9. **Itens descartados** — hipóteses refutadas, com a evidência da refutação (evita retrabalho).
10. **Pontos não verificados / perguntas em aberto** — o que depende de decisão do autor ou de execução em Windows.
11. **Instruções operacionais para a IA corretora** — ver 8.4.
12. **Prompt de execução sugerido** — texto curto, pronto para colar, que mande a IA corretora seguir o relatório.

### 8.2 Modelo de cada achado (siga exatamente)

```markdown
### FLY-P{parte}-{nnn} — {título curto e específico}

| Campo | Valor |
|---|---|
| Parte | 1 / 2 / 3 / 4 (e outras afetadas) |
| Severidade | Crítica · Alta · Média · Baixa · Info |
| Categoria | Segurança · Corretude · Concorrência · Robustez · Desempenho · UX/A11y · i18n · Build/CI · Dependência · Documentação · Testes · Manutenibilidade |
| Confiança | Confirmado · Provável · Hipótese |
| Esforço | P (<30 min) · M (até 2 h) · G (>2 h) |
| Risco de regressão | Baixo · Médio · Alto |
| Local | `caminho/arquivo.ext:linha-inicial-linha-final` (liste todos os pontos) |
| Depende de | IDs de outros achados que devem ser corrigidos antes (ou "—") |

**Evidência** — trecho real do código (máx. ~15 linhas) em bloco de código com a linguagem correta.

**Descrição e causa raiz** — o que está errado e por quê, em termos técnicos.

**Impacto / cenário de reprodução** — quem é afetado, em que condições, passos para reproduzir ou raciocínio que prova o problema.

**Solução proposta**
1. Passo objetivo (arquivo, função, o que mudar).
2. …
```diff
- código atual
+ código proposto
```
(Use diff/patch concreto sempre que possível. Se houver mais de uma abordagem, indique a **recomendada** e o porquê, e cite as alternativas descartadas.)

**Arquivos a alterar** — lista exata.

**Critérios de aceite** — condições verificáveis (ex.: "`cargo test` passa com o novo teste `host_matches_rejects_lookalike_suffix`", "sites `http://` abrem na janela principal").

**Como verificar** — comandos e/ou teste a escrever (nome sugerido e esqueleto).

**Cuidados** — o que **não** mudar, efeitos colaterais, compatibilidade com configuração já gravada pelos usuários, i18n obrigatória (pt-BR/en/es), código `#[cfg(windows)]`.
```

**Escalas**
- **Crítica:** execução remota, escalonamento de privilégio, perda de dados do usuário, impossibilidade de iniciar/instalar.
- **Alta:** falha de segurança explorável, travamento/encerramento inesperado, quebra de funcionalidade principal, risco de rejeição na Store.
- **Média:** falha em cenários específicos, inconsistência relevante, CI frágil.
- **Baixa:** melhoria de robustez, clareza, estilo, documentação.
- **Info:** observação sem ação necessária.

### 8.3 Plano de correção ordenado (seção 7 do relatório)

- Organize em **ondas** (Onda 1 = críticas/altas e pré-requisitos; Onda 2 = médias; Onda 3 = baixas/documentação).
- Dentro de cada onda, **ordene por dependência** (ordem topológica de "Depende de") e **agrupe por arquivo** para evitar conflitos de edição (ex.: todos os achados de `window.rs` em sequência, do final do arquivo para o início, ou indique que as linhas citadas mudam após cada edição).
- Para cada item do plano: `ID · arquivo(s) · esforço · como validar`.
- Marque o que pode ser feito **em paralelo** e o que **exige decisão humana** (nesse caso, apresente as opções e a recomendação).

### 8.4 Instruções operacionais para a IA corretora (inclua no relatório)

- Aplicar **um achado por vez**, na ordem do plano; após cada um, rodar os comandos da seção 7 pertinentes e só avançar se passarem.
- Mudar **somente** o que o achado pede; não refatorar por conta própria.
- Antes de editar, **reler o trecho atual** (os números de linha podem ter mudado).
- Adicionar o teste sugerido junto com a correção.
- Manter comentários/logs em pt-BR e atualizar os três idiomas quando alterar texto de interface; atualizar `i18n.rs`, `main.ts`, `settings.ts` e o script injetado de forma consistente.
- Se uma correção exigir alterar `generate_handler!`, `build.rs` ou capabilities, atualizar os três **juntos** e conferir `npm run check:acl`.
- Nunca incluir segredos; nunca desativar verificações (`check:*`, `clippy -D warnings`, `cargo audit`) para "fazer passar".
- Ao terminar cada achado, registrar: ID, arquivos alterados, resultado dos comandos e eventual desvio em relação ao proposto.

---

## 9. Modo de execução

- **Completo:** execute as Partes 1 → 4 em sequência e entregue o relatório consolidado.
- **Por parte:** quando o usuário disser "**Execute a Parte N**", analise **somente** a Parte N (leia as demais apenas o necessário para entender as interfaces), entregue `RELATORIO_ANALISE_FLOATLY_PARTE_N.md` e, ao final, informe quais interfaces com outras partes ficaram pendentes.
- **Consolidação:** quando o usuário pedir, una os relatórios parciais, renumerando nada (os IDs `FLY-P{parte}-{nnn}` são estáveis), removendo duplicatas e gerando o plano de correção global.

---

## 10. Checklist final (confira antes de entregar)

- [ ] Li integralmente todos os arquivos das 4 partes (incluindo `window.rs` e `build-msix.yml`).
- [ ] Cada achado tem `arquivo:linha`, evidência, causa raiz, solução com passos/diff, critérios de aceite e forma de verificação.
- [ ] Cada achado traz **confiança** e **esforço**; nenhuma hipótese foi tratada como fato.
- [ ] Tentei refutar cada achado; os refutados estão em "Itens descartados".
- [ ] As hipóteses H1–H14 foram confirmadas, refutadas ou marcadas como "não verificadas".
- [ ] O plano de correção está ordenado por dependência e agrupado por arquivo.
- [ ] Informei o que executei, o que não pude executar e por quê.
- [ ] O relatório está em `.md`, em português do Brasil, sem alterar nenhum arquivo do projeto.
- [ ] Incluí a seção "Instruções operacionais para a IA corretora" e o "Prompt de execução sugerido".

**Comece pelo inventário (seção 5, passo 1) e, em seguida, execute a Parte 1.**
