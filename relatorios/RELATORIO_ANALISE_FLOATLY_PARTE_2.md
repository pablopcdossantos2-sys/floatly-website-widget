# Relatório de Análise — Floatly — **PARTE 2 de 4**

> **Parte analisada neste relatório: PARTE 2 — Janela, navegação, login e script injetado (`window.rs`, `tray.rs`, `i18n.rs`).**
> As Partes 1, 3 e 4 **não foram analisadas** nesta execução (foram lidas apenas as linhas necessárias para entender as interfaces). Os IDs `FLY-P2-nnn` são estáveis e podem ser consolidados depois com os demais relatórios parciais.
>
> Público-alvo: **outra IA que irá corrigir os problemas**. Cada achado é autossuficiente (local, evidência, causa, passos, critérios de aceite, verificação).
> Repositório analisado: `floatly-website-widget-main.zip` (Floatly / crate `widget-website` v1.0.0, Tauri 2.12.5, tao 0.37.2, wry 0.57.1, `open` 5.4.5, `windows-sys` 0.61.2 — versões lidas do `Cargo.lock`).
> Data da análise: 2026-10-31. Idioma: pt-BR.

---

## 1. Resumo executivo

**Nota de risco da Parte 2: MÉDIO.** Não há achado Crítico nem Alto. O núcleo de segurança da navegação está bem construído (limite por dot-boundary em `host_matches`, bloqueio de esquemas não-HTTP(S), token por processo para `widget-action://`, escrita defensiva de HRGN, aritmética em `i64`). Os problemas reais estão em **corretude de política de navegação**, **robustez do script injetado em sites de terceiros** e **UX/i18n**.

### Contagem de achados por severidade e por parte

| Parte | Escopo | Crítica | Alta | Média | Baixa | Info | Total |
|---|---|---:|---:|---:|---:|---:|---:|
| 1 | Núcleo Rust (main/config/commands/diagnostics/autostart) | — | — | — | — | — | **não analisada** |
| **2** | **Janela, navegação, login, INIT (`window.rs`, `tray.rs`, `i18n.rs`)** | 0 | 0 | **9** | **12** | 0 | **21** |
| 3 | Frontend, IPC, CSP e capabilities | — | — | — | — | — | **não analisada** |
| 4 | Build, CI/CD, empacotamento, docs | — | — | — | — | — | **não analisada** |

### Os 5 achados mais importantes

1. **FLY-P2-001** — Segmentos ambíguos (`account`, `accounts`, `auth`, `callback`) desviam **páginas legítimas do próprio site** para o popup de login (confirmado por port fiel + exemplos; seção 5).
2. **FLY-P2-003** — Arraste/redimensionamento nativo usa o botão **físico** esquerdo (`GetAsyncKeyState(VK_LBUTTON)`); com botões do mouse trocados (canhotos) o redimensionamento nunca funciona e o arraste cai sempre no *fallback*.
3. **FLY-P2-005** — Os limitadores de abertura externa/popup são **globais e incompletos**: um site/anúncio pode abrir o navegador padrão a cada 2 s e roubar foco do popup de login a cada 1 s.
4. **FLY-P2-007** — Provedores de identidade comuns (GitHub, Auth0, Okta, Cognito, B2C…) caem em "navegador externo", onde a sessão **não** é compartilhada com o widget; popups `window.open` perdem o `window.opener`.
5. **FLY-P2-010** — CSS global injetado (`html{overflow-y:auto!important}` + regras `*`) pode quebrar *scroll lock* de modais e layouts de sites de terceiros.

---

## 2. Escopo, premissas e limitações

**Lido integralmente:** `src-tauri/src/window.rs` (1.768 linhas, em 4 blocos de ≈450 linhas), `tray.rs` (88), `i18n.rs` (105).
**Lido apenas para interfaces:** `main.rs` (305, integral), `commands.rs` (439, integral), `tauri.conf.json`, `capabilities/*.json`, trechos de `diagnostics.rs` (`log_navigation`, `sanitize_url`, `record_heartbeat`, `set_window_visible`), `config.rs` (`normalize`, `snapshot`, `lock_config`, `save` — retry de `rename`), `scripts/check-drag-overlay.mjs`, `Cargo.toml`/`Cargo.lock` (versões).

**Executado (resultado real):**

| Verificação | Resultado |
|---|---|
| `node scripts/check-drag-overlay.mjs` | ✅ passou |
| `node scripts/check-acl.mjs` | ✅ passou |
| `node scripts/check-tauri-versions.mjs` | ✅ passou (`api 2.12.4`, `cli 2.12.4`, crate `2.12.5`, build `2.7.4`) |
| Port fiel em Python de `is_login_path`/`host_matches`/`decide_main` (`window.rs:216-322`) com 12 casos | ✅ executado — evidência em FLY-P2-001, -007, -008 |
| Parsing WHATWG de 22 URLs de borda (Node `URL`; mesma especificação que a crate `url` 2.5.x) | ✅ executado — evidência em FLY-P2-006 e seção 8 |

**NÃO executado (e por quê):**

| Comando | Motivo |
|---|---|
| `cargo test`, `cargo clippy`, `cargo fmt --check` | Ambiente Linux sem acesso a `index.crates.io` (proxy retorna 403) e `rustup` não conseguiu baixar a toolchain 1.90.0; além disso o crate depende de `windows*`/WebView2. **Nenhum teste Rust foi compilado ou rodado.** As propostas de código abaixo **não foram compiladas**; a IA corretora deve compilá-las no Windows. |
| `tauri build`, MSIX, `cargo audit` | Windows/rede necessários. |
| `npm ci`, `typecheck`, `check:contrast`, `check:msix-config`, `build` | Pertencem às Partes 3/4; fora do escopo desta execução. |

**Convenções adotadas nos achados:** "Confirmado" = comprovado pelo código ou por execução; "Provável" = forte indício dependente de comportamento externo (WebView2/Tauri/Win32) documentado; "Hipótese" = precisa de teste em Windows/WebView2.

---

## 3. Mapa das 4 partes (delimitações)

> ▶ **Parte trabalhada neste relatório: a PARTE 2.**

| # | Parte | Arquivos | Situação neste relatório |
|---|---|---|---|
| 1 | Núcleo Rust: ciclo de vida, configuração, comandos IPC, diagnóstico | `src-tauri/src/main.rs`, `config.rs`, `commands.rs`, `diagnostics.rs`, `autostart.rs`, `src-tauri/Cargo.toml`, `build.rs` | ⏸ Não analisada (apenas interfaces lidas) |
| **2** | **Janela, navegação, login e script injetado** | **`src-tauri/src/window.rs` (1.768 l.), `tray.rs` (88 l.), `i18n.rs` (105 l.)** | **✅ ANALISADA — 21 achados** |
| 3 | Interface local (frontend), IPC e superfície de segurança | `index.html`, `settings.html`, `src/main.ts`, `src/settings.ts`, `src/styles.css`, `vite.config.ts`, `tsconfig.json`, `package.json` (runtime), `tauri.conf.json` (`app`/`security`), `capabilities/*.json` | ⏸ Não analisada |
| 4 | Build, CI/CD, empacotamento, dependências, documentação | `.github/**`, `scripts/*`, `packaging/msix/*`, `windows/hooks.nsh`, `tauri.conf.json` (`bundle`), `Cargo.toml`/`Cargo.lock`, `package*.json`, `rust-toolchain.toml`, `.gitignore`, `README*.md`, `PRIVACY_POLICY.md`, `SUPPORT.md`, `GUIA_*`, `MICROSOFT_STORE_*` | ⏸ Não analisada |

### Inventário da Parte 2

| Arquivo | Linhas | Responsabilidade | Depende de |
|---|---:|---|---|
| `window.rs` | 1.768 | Script `INIT`/`LOGIN_INIT` (JS em strings Rust), política de navegação, `create_main`, popup de login, drag/resize Win32, cantos nativos, posição/tamanho, visibilidade, configurações, auto-reload, testes inline | `config.rs` (`AppConfig`, `AppState`, `snapshot`, `lock_config`, `persist_current`), `diagnostics.rs` (`log`, `log_navigation`, `sanitize_url`, `set_window_visible`), `i18n.rs`, Tauri/wry, `windows-sys`, `open`, `uuid` |
| `tray.rs` | 88 | Ícone/menu da bandeja, idioma do menu | `commands.rs`, `i18n.rs`, `AppState` |
| `i18n.rs` | 105 | Rótulos da bandeja, títulos de janelas | — |

### Interfaces com as demais partes

| De → Para | Interface | Observação |
|---|---|---|
| P2 → P1 | `AppState.{heartbeat_token, geometry_save_revision, geometry_save_worker_running, shutdown_requested, app_dir}`, `persist_current()`, `lock_config()` | Ver FLY-P2-016/-017 |
| P1 → P2 | `main.rs` registra `WindowEvent::{Moved,Resized,ScaleFactorChanged,CloseRequested,Destroyed}` e chama `window::save_window_state/refresh_native_corners/flush_pending_config/start_auto_reload`; `tray::setup(...)?` | Ver FLY-P2-020 |
| P1 → P2 | `commands.rs` chama `window::apply_config/open_settings/open_login_from_tray/toggle_main_visibility` e `tray::update_language` | — |
| P2 ↔ P3 | `INIT` roda também em `http://tauri.localhost` (página local); CSP do `tauri.conf.json` afeta o `fetch` injetado; `withGlobalTauri`+capability `main-remote-heartbeat` definem o que o site remoto pode invocar | Ver FLY-P2-004 e -014 |
| P2 ↔ P4 | `scripts/check-drag-overlay.mjs` extrai `INIT` por regex de `window.rs` e exige substrings literais | **Qualquer edição de `INIT` deve preservar essas substrings** (ver seção 11) |

---

## 4. Índice de achados

| ID | Título | Parte | Severidade | Confiança | Esforço | Depende de |
|---|---|---|---|---|---|---|
| FLY-P2-001 | Segmentos ambíguos de login desviam páginas legítimas do site para o popup | 2 | Média | Confirmado | M | — |
| FLY-P2-002 | *Fallback* `start_dragging()` é acionado após o cancelamento intencional "botão já solto" | 2 | Média | Provável | P | — |
| FLY-P2-003 | Arraste/resize nativo usa botão físico esquerdo (mouse com botões trocados) e não valida o cursor | 2 | Média | Provável | M | FLY-P2-002 |
| FLY-P2-005 | Limitadores de abertura externa/popup globais e incompletos permitem abuso | 2 | Média | Confirmado | M | — |
| FLY-P2-007 | IdPs comuns fora da lista vão ao navegador externo; popups perdem `window.opener` | 2 | Média | Provável | M | FLY-P2-001 |
| FLY-P2-008 | Escopo de host estrito: apex↔`www` do site configurado cai no navegador externo | 2 | Média | Provável | P | FLY-P2-001 |
| FLY-P2-009 | Textos do `INIT` incoerentes ("calendário"), `aria-label` fixo em pt-BR e rótulos divergentes | 2 | Média | Confirmado | P | — |
| FLY-P2-010 | CSS global injetado (`overflow-y:auto!important`, `*`) quebra layouts/scroll lock de sites | 2 | Média | Provável | M | — |
| FLY-P2-014 | Verificação de conectividade injetada: falha sob Trusted Types/CSP, sem `online`, roda na página local | 2 | Média | Provável | M | FLY-P2-009 |
| FLY-P2-004 | Tokens do `INIT` não são globais, mas são observáveis pela página (hardening) | 2 (e 3) | Baixa | Hipótese | M | FLY-P2-003 |
| FLY-P2-006 | `is_internal_app_url` não valida porta nem host do esquema `tauri:` | 2 | Baixa | Confirmado | P | — |
| FLY-P2-011 | `MutationObserver` com `subtree` reexecuta `install()` a cada mutação do DOM | 2 | Baixa | Provável | P | — |
| FLY-P2-012 | Placeholders do `INIT`/`LOGIN_INIT` substituídos sem escape/allow-list | 2 | Baixa | Provável | P | — |
| FLY-P2-013 | Atalhos Ctrl+L/T/N/W/P/J/U/S são engolidos com `stopImmediatePropagation` | 2 | Baixa | Confirmado | P | — |
| FLY-P2-015 | Canal `location.href='widget-action://…'` pode disparar `beforeunload` do site | 2 | Baixa | Hipótese | M | FLY-P2-004 |
| FLY-P2-016 | Travas `SETTINGS_WINDOW_CREATING`/`geometry_save_worker_running` sem guard RAII | 2 (e 1) | Baixa | Provável | P | — |
| FLY-P2-017 | Persistência de geometria: `remember_position=false` descarta também o tamanho; custo por evento | 2 (e 1) | Baixa | Confirmado | P | — |
| FLY-P2-018 | Auto-reload dispara com janela minimizada e a *thread* nunca encerra | 2 | Baixa | Provável | P | — |
| FLY-P2-019 | Título da janela de login fixo no host inicial (enganoso após redirecionamentos) | 2 | Baixa | Confirmado | P | — |
| FLY-P2-020 | Bandeja: `DoubleClick` possivelmente inalcançável; ícone ausente sem log; falha de `tray::setup` fatal | 2 (e 1) | Baixa | Hipótese | P | — |
| FLY-P2-021 | Melhorias menores consolidadas (nitpicks, robustez, i18n) | 2 | Baixa | Confirmado | M | — |

---

## 5. Achados detalhados — Parte 2

### FLY-P2-001 — Segmentos ambíguos de login desviam páginas legítimas do site para o popup

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Média |
| Categoria | Corretude |
| Confiança | Confirmado |
| Esforço | M |
| Risco de regressão | Médio |
| Local | `src-tauri/src/window.rs:216-218`, `271-278`, `305-322`, `1531-1547` |
| Depende de | — |

**Evidência**
```rust
// window.rs:216
const LOGIN_SEGMENTS: &[&str] = &[
    "login", "log-in", "signin", "sign-in", "account", "accounts", "auth", "oauth", "callback",
];
// window.rs:305-318
fn decide_main(url: &Url, c: &AppConfig) -> NavAction {
    ...
    if let Some(target) = configured_host(c) {
        if host_matches(host, &target) {
            if is_login_path(url.path()) && !is_exact_configured_page(url, c) {
                return NavAction::OpenLogin;
            }
            return NavAction::Allow;
```
Saída da port fiel (Python) executada:
```
site                                   url                                              -> decisão
https://example.com/                   https://example.com/account/settings             -> OpenLogin(popup)
https://example.com/                   https://example.com/accounts/billing             -> OpenLogin(popup)
https://example.com/                   https://example.com/docs/auth/overview           -> OpenLogin(popup)
https://example.com/                   https://example.com/api/callback                 -> OpenLogin(popup)
https://app.example.com/account/home   https://app.example.com/account/orders           -> OpenLogin(popup)
https://app.example.com/account/home   https://app.example.com/account/home             -> Allow
```

**Descrição e causa raiz.** A mesma lista `LOGIN_SEGMENTS` serve a duas decisões diferentes: (a) *desviar uma navegação da janela principal para o popup* (`decide_main`) e (b) *reconhecer que o popup ainda está em tela de autenticação* (`is_auth_page`, usada por `should_close_login`). Para (b) a lista ampla é necessária (um `/auth/callback?code=…` precisa carregar dentro do popup — se fosse tratado como página normal o popup fecharia e **cancelaria** a navegação antes de o site processar o `code`). Para (a), porém, `account`, `accounts`, `auth` e `callback` são nomes comuns de páginas legítimas (`/account/orders`, `/docs/auth`, `/api/callback`).

**Impacto / cenário.** Qualquer link do site configurado para `/account/...`, `/accounts/...` ou `/<qualquer>/auth/...` abre o popup em vez de navegar na janela principal. Se a URL inicial está dentro de `/account/` (apps SaaS), **toda** navegação interna diferente da URL exata abre popup. Páginas de conta só podem ser vistas no popup (que não fecha sozinho em páginas "de auth").

**Solução proposta** (recomendada: separar as duas listas; alternativas descartadas: remover os segmentos ambíguos globalmente — quebraria o fechamento correto do popup; configuração por usuário — excesso de escopo).
1. Manter `LOGIN_SEGMENTS`/`is_login_path` **inalterados** para `is_auth_page` (popup).
2. Criar um classificador estrito para a decisão da janela principal: segmentos fortes sempre valem; os ambíguos só valem quando a URL traz parâmetros típicos de OAuth/OIDC.
3. Trocar a chamada em `decide_main`.
```diff
+const STRONG_LOGIN_SEGMENTS: &[&str] = &["login", "log-in", "signin", "sign-in", "oauth"];
+const AMBIGUOUS_LOGIN_SEGMENTS: &[&str] = &["account", "accounts", "auth", "callback"];
+const OAUTH_QUERY_KEYS: &[&str] = &["client_id", "redirect_uri", "response_type", "code_challenge", "id_token"];
+
+/// Decide se uma navegação da janela principal deve ir para o popup de login.
+/// Segmentos ambíguos (`account`, `auth`…) só contam com parâmetros típicos de OAuth.
+fn is_login_navigation(url: &Url) -> bool {
+    let segments: Vec<String> = url
+        .path()
+        .split('/')
+        .filter(|segment| !segment.is_empty())
+        .map(|segment| segment.to_ascii_lowercase())
+        .collect();
+    if segments.iter().any(|s| STRONG_LOGIN_SEGMENTS.contains(&s.as_str())) {
+        return true;
+    }
+    let has_oauth_params = url
+        .query_pairs()
+        .any(|(key, _)| OAUTH_QUERY_KEYS.contains(&key.as_ref()));
+    has_oauth_params
+        && segments.iter().any(|s| AMBIGUOUS_LOGIN_SEGMENTS.contains(&s.as_str()))
+}
@@ fn decide_main
-            if is_login_path(url.path()) && !is_exact_configured_page(url, c) {
+            if is_login_navigation(url) && !is_exact_configured_page(url, c) {
                 return NavAction::OpenLogin;
             }
```

**Arquivos a alterar:** `src-tauri/src/window.rs` (constantes, nova função, `decide_main`, testes).

**Critérios de aceite**
- `decide_main("https://example.com/account/settings")` com site `https://example.com/` → `Allow`; idem `/accounts/billing`, `/docs/auth/overview`, `/api/callback`.
- `/login`, `/signin`, `/sign-in`, `/oauth/authorize` continuam → `OpenLogin`.
- `/auth/start?client_id=x&redirect_uri=y` → `OpenLogin`.
- Os testes existentes (`is_auth_page_matches_path_segments_not_substrings`, etc.) continuam passando sem alteração.

**Como verificar** — adicionar em `mod navigation_policy_tests`:
```rust
#[test]
fn main_window_does_not_divert_ambiguous_site_pages_to_login() {
    let cfg = config("https://example.com/");
    for path in ["/account/settings", "/accounts/billing", "/docs/auth/overview", "/api/callback"] {
        let url = Url::parse(&format!("https://example.com{path}")).unwrap();
        assert_eq!(decide_main(&url, &cfg), NavAction::Allow, "{path}");
    }
}
#[test]
fn main_window_still_diverts_real_login_pages() {
    let cfg = config("https://example.com/");
    for path in ["/login", "/signin", "/sign-in", "/oauth/authorize", "/auth/start?client_id=x&redirect_uri=y"] {
        let url = Url::parse(&format!("https://example.com{path}")).unwrap();
        assert_eq!(decide_main(&url, &cfg), NavAction::OpenLogin, "{path}");
    }
}
```
Comando: `cargo test --manifest-path src-tauri/Cargo.toml --locked navigation_policy`.

**Cuidados.** Não alterar `is_login_path`/`is_auth_page`/`should_close_login` (popup). Manter a exceção `is_exact_configured_page`. Comentários em pt-BR.

---

### FLY-P2-002 — *Fallback* `start_dragging()` é acionado após o cancelamento intencional "botão já solto"

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Média |
| Categoria | Corretude |
| Confiança | Provável (fluxo confirmado no código; efeito no `tao` 0.37.2 não verificado) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `window.rs:549-557`, `627-629`, `677-681` |
| Depende de | — |

**Evidência**
```rust
// 627
if unsafe { GetAsyncKeyState(VK_LBUTTON as i32) } as u16 & 0x8000 == 0 {
    return Err("Arraste cancelado porque o botão esquerdo do mouse já foi solto.".into());
}
// 552-557
if let Err(error)=begin_native_drag(&window) {
    crate::diagnostics::log(&diag_dir,"ERRO",format!("O arraste nativo falhou: {error}"));
    if let Err(fallback_error) = window.start_dragging() { ... }
}
```
**Descrição.** O cancelamento deliberado (clique rápido: o botão foi solto antes de a navegação `widget-action://` chegar ao Rust) é devolvido como `Err`, e o chamador trata **qualquer** `Err` como falha do arraste nativo e executa `start_dragging()`. Na prática, o *guard* de "botão solto" é anulado: se `start_dragging()` enviar `WM_NCLBUTTONDOWN/HTCAPTION` com o botão já solto, o Windows entra em laço modal de movimento que "gruda" a janela no cursor até o próximo clique. **Não verificado** se o `tao` 0.37.2 faz checagem própria do botão (sem acesso à crate neste ambiente) — por isso "Provável".

**Impacto.** Cliques rápidos sobre a alça/bordas podem iniciar movimento "preso" ao mouse; além disso o log registra `ERRO` para algo que não é erro.

**Solução proposta.** Tratar cancelamento como "nada a fazer" (`Ok(())`), reservando `Err` para falha real. Aplicar em `begin_native_drag` e `begin_native_resize`.
```diff
     if unsafe { GetAsyncKeyState(VK_LBUTTON as i32) } as u16 & 0x8000 == 0 {
-        return Err("Arraste cancelado porque o botão esquerdo do mouse já foi solto.".into());
+        // Cancelamento intencional: o botão já foi solto. Não é falha e NÃO deve acionar o fallback.
+        return Ok(());
     }
```
(e o equivalente em `begin_native_resize`, linhas 677-681).

**Arquivos a alterar:** `window.rs`.

**Critérios de aceite:** com o botão solto antes do processamento, nenhum `ERRO` é logado e `start_dragging()` não é chamado; falhas reais de `hwnd()`/`GetCursorPos`/`PostMessageW` continuam acionando o *fallback*.

**Como verificar:** manual no Windows (clique rápido na alça `⋮⋮` e nas bordas; a janela não deve "grudar"). Opcional: extrair `fn should_start_native_move(button_down: bool) -> bool` e testá-la.

**Cuidados.** FLY-P2-003 edita as mesmas linhas — aplicar este primeiro.

---

### FLY-P2-003 — Arraste/resize nativo usa o botão físico esquerdo e não valida o cursor

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Média |
| Categoria | Corretude · Robustez |
| Confiança | Provável (comportamento documentado de `GetAsyncKeyState`; não executado em Windows) |
| Esforço | M |
| Risco de regressão | Médio |
| Local | `window.rs:617-649`, `656-701` |
| Depende de | FLY-P2-002 |

**Evidência**
```rust
if unsafe { GetAsyncKeyState(VK_LBUTTON as i32) } as u16 & 0x8000 == 0 { ... }
```
`GetAsyncKeyState(VK_LBUTTON)` retorna o estado do botão **físico** esquerdo, independentemente do mapeamento lógico (`SM_SWAPBUTTON`). O JavaScript (`event.button !== 0` em `dragWindow`, linha 83) usa o botão **lógico** primário.

**Impacto.** Usuários com botões trocados (canhotos): o clique primário é o botão físico direito → `VK_LBUTTON` está solto → o arraste nativo é sempre cancelado (e, hoje, cai no *fallback*, FLY-P2-002) e o **redimensionamento nunca funciona** (não há *fallback*). Além disso, o código não confirma que o cursor está sobre a janela do widget ao processar a ação (relevante para FLY-P2-004).

**Solução proposta.** Extrair um *helper* único (também elimina ~30 linhas duplicadas entre drag e resize) que respeite `SM_SWAPBUTTON` e exija cursor dentro do retângulo da janela. As *features* `Win32_UI_WindowsAndMessaging`, `Win32_UI_Input_KeyboardAndMouse` e `Win32_Foundation` de `windows-sys` já estão habilitadas no `Cargo.toml`.
```rust
#[cfg(windows)]
fn primary_button_is_down() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SWAPBUTTON};
    // GetAsyncKeyState informa o botão FÍSICO; com botões trocados o primário lógico é o direito físico.
    let vk = if unsafe { GetSystemMetrics(SM_SWAPBUTTON) } != 0 { VK_RBUTTON } else { VK_LBUTTON };
    (unsafe { GetAsyncKeyState(vk as i32) } as u16) & 0x8000 != 0
}

#[cfg(windows)]
fn cursor_is_over_window(hwnd: windows_sys::Win32::Foundation::HWND, cursor: &windows_sys::Win32::Foundation::POINT) -> bool {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect;
    let mut rect: RECT = unsafe { std::mem::zeroed() };
    unsafe { GetWindowRect(hwnd, &mut rect) != 0 }
        && cursor.x >= rect.left && cursor.x < rect.right && cursor.y >= rect.top && cursor.y < rect.bottom
}
```
Em `begin_native_drag`/`begin_native_resize`: substituir o teste de `GetAsyncKeyState` por `if !primary_button_is_down() { return Ok(()); }` e, após `GetCursorPos`, adicionar `if !cursor_is_over_window(raw_hwnd.0, &cursor) { return Ok(()); }` (cancelamento silencioso, como em FLY-P2-002).

**Arquivos a alterar:** `window.rs` (e testes de `pack_lparam` permanecem).

**Critérios de aceite:** `cargo clippy ... -D warnings` limpo no Windows; arrasto e resize funcionam com botões padrão **e** com "Alternar botões primário e secundário" ativado em *Configurações → Mouse*; ação com cursor fora da janela é ignorada.

**Como verificar:** manual (ambas as configurações de mouse; monitores com DPI misto — `GetCursorPos` e `GetWindowRect` usam coordenadas de tela do mesmo espaço, mas confirmar com 125%/150%).

**Cuidados.** Manter `#[cfg(not(windows))]` stubs. Não mudar `pack_lparam`.

---

### FLY-P2-005 — Limitadores de abertura externa/popup globais e incompletos permitem abuso

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Média |
| Categoria | Segurança · Robustez |
| Confiança | Confirmado (mecanismo); impacto depende de site/anúncios hostis |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | `window.rs:378-395`, `444-474`, `576-590`, `593-607`, `740-781`, `906-936` |
| Depende de | — |

**Evidência**
```rust
fn external_open_allowed(state: &mut ExternalOpenState, url: &str, now_ms: u64) -> bool {
    let elapsed = now_ms.saturating_sub(state.last_open_ms);
    if state.last_open_ms != 0 && elapsed < 2_000 { return false; }
    if state.last_url == url && state.last_open_ms != 0 && elapsed < 10_000 { return false; }
```
```rust
// on_navigation (main), 577-581 — NÃO chama allow_login_popup()
NavAction::OpenLogin => { ... open_login_limited(nav_app.clone(),u.clone(),diag_dir.clone()); false }
```
**Descrição.** (1) `on_navigation` e `on_new_window` roteiam para o **navegador padrão** qualquer navegação/`window.open` de topo para fora do host configurado, sem noção de gesto do usuário; o único freio é "1 abertura a cada 2 s" com URLs diferentes — ou seja, 30 abas/min no navegador padrão. (2) O desvio `OpenLogin` da janela principal (`on_navigation`) **não** usa `allow_login_popup()` (só o `on_new_window` usa); no caminho `Reuse` (`740-781`) o popup é re-navegado e recebe `show()`+`set_focus()` a cada chamada, sem limite.

**Impacto.** Site (ou anúncio de terceiros embutido) pode: abrir abas no navegador padrão a cada 2 s (`setInterval(()=>window.open(url_i))` ou `location.href`); e, navegando repetidamente para `https://accounts.google.com/…`, roubar o foco do usuário (≈1×/s pelo `on_new_window`; sem limite próprio no desvio feito por `on_navigation`).

**Solução proposta.**
1. Acrescentar um orçamento em janela deslizante ao limitador externo (p. ex. 5 aberturas/60 s), mantendo as regras atuais.
2. Aplicar `allow_login_popup()` também ao desvio `OpenLogin` de `on_navigation`.
```diff
 #[derive(Debug, Default)]
 struct ExternalOpenState {
     last_open_ms: u64,
     last_url: String,
+    recent_ms: std::collections::VecDeque<u64>,
 }
+const EXTERNAL_OPEN_BUDGET: usize = 5;
+const EXTERNAL_OPEN_WINDOW_MS: u64 = 60_000;
@@ fn external_open_allowed(...)
     if state.last_url == url && state.last_open_ms != 0 && elapsed < 10_000 {
         return false;
     }
+    while state.recent_ms.front().is_some_and(|t| now_ms.saturating_sub(*t) >= EXTERNAL_OPEN_WINDOW_MS) {
+        state.recent_ms.pop_front();
+    }
+    if state.recent_ms.len() >= EXTERNAL_OPEN_BUDGET {
+        return false;
+    }
+    state.recent_ms.push_back(now_ms);
     state.last_open_ms = now_ms;
```
```diff
 // on_navigation, NavAction::OpenLogin
-        open_login_limited(nav_app.clone(),u.clone(),diag_dir.clone());
+        if allow_login_popup() {
+            open_login_limited(nav_app.clone(),u.clone(),diag_dir.clone());
+        } else {
+            crate::diagnostics::log(&diag_dir,"WARN","Desvio para autenticação descartado pelo limite de uma abertura por segundo.");
+        }
         false
```
**Arquivos a alterar:** `window.rs` (+ testes).

**Critérios de aceite:** o teste existente `external_open_rate_limiter_drops_burst_and_duplicates` continua passando; novo teste: 6 URLs distintas espaçadas de 2,1 s → a 6ª é negada; após 60 s volta a permitir.

**Como verificar**
```rust
#[test]
fn external_open_budget_caps_openings_per_minute() {
    let mut s = ExternalOpenState::default();
    for i in 0..5u64 { assert!(external_open_allowed(&mut s, &format!("https://e.com/{i}"), 1_000 + i * 2_100)); }
    assert!(!external_open_allowed(&mut s, "https://e.com/x", 1_000 + 5 * 2_100));
    assert!(external_open_allowed(&mut s, "https://e.com/y", 1_000 + 61_000 + 5 * 2_100));
}
```
**Cuidados.** Valores (5/60 s) são **decisão do autor** — ver plano. Alternativa descartada: exigir gesto do usuário (precisaria de novo comando IPC + ACL nas Partes 1/3; custo alto para o ganho). Residual aceito: 1 foco/s no caminho `Reuse`.

---

### FLY-P2-007 — IdPs comuns fora da lista vão ao navegador externo; popups perdem `window.opener`

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Média |
| Categoria | Corretude · UX |
| Confiança | Provável |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | `window.rs:247-269`, `324-336`, `593-607`, `906-936` |
| Depende de | FLY-P2-001 |

**Evidência**
```rust
match host.as_str() {
    "accounts.google.com" | "appleid.apple.com" | "login.microsoftonline.com" | "login.live.com" => true,
    "facebook.com" | "www.facebook.com" | "m.facebook.com" => { /* só caminhos de login */ }
    _ => false,
}
```
Port executada: `https://github.com/login/oauth/authorize` → `External`; `https://example.auth0.com/authorize` → `External`; `https://login.example.org/sso` → `External`.

**Descrição.** Fora de 4 hosts + Facebook, qualquer provedor de identidade é tratado como "link externo": abre no navegador padrão, **cuja sessão não é compartilhada** com o WebView2 do widget — o usuário conclui o login lá e o widget continua deslogado. Adicionalmente, `on_new_window` responde sempre `NewWindowResponse::Deny` e recria a página via `open_login`/`external`; fluxos OAuth em *popup* que dependem de `window.opener.postMessage` (ex.: Google Identity Services em modo popup) não completam, porque o popup criado pelo app não tem relação com o *opener*. Links `target=_blank` do **próprio site** (`Allow`) também vão ao navegador padrão (`604`).

**Impacto.** Login impossível em sites que usam Auth0/Okta/Cognito/B2C/GitHub/Atlassian etc.; comportamento "abre no Edge/Chrome e nada acontece" no widget.

**Solução proposta.** Ampliar a lista de provedores por host exato e sufixo (usando `host_matches`, que já respeita fronteira de ponto) e **documentar** a limitação de `window.opener` (Info para a IA da Parte 4/README).
```diff
+const AUTH_PROVIDER_HOSTS: &[&str] = &[
+    "accounts.google.com", "appleid.apple.com", "login.microsoftonline.com", "login.microsoft.com",
+    "login.live.com", "account.live.com", "login.windows.net", "id.atlassian.com",
+];
+// Tenants de IdPs de terceiros (qualquer subdomínio).
+const AUTH_PROVIDER_SUFFIXES: &[&str] = &["auth0.com", "okta.com", "oktapreview.com", "b2clogin.com", "amazoncognito.com"];
@@ fn is_auth_provider
-    match host.as_str() {
-        "accounts.google.com" | "appleid.apple.com" | "login.microsoftonline.com" | "login.live.com" => true,
+    if AUTH_PROVIDER_HOSTS.contains(&host.as_str())
+        || AUTH_PROVIDER_SUFFIXES.iter().any(|suffix| host_matches(&host, suffix))
+    {
+        return true;
+    }
+    match host.as_str() {
+        "github.com" => path.split('/').any(|s| matches!(s.to_ascii_lowercase().as_str(), "login" | "oauth" | "sessions")),
         "facebook.com" | "www.facebook.com" | "m.facebook.com" => { ... }
```
(mantendo o ramo do Facebook como está).

**Arquivos a alterar:** `window.rs` (+ testes).

**Critérios de aceite:** `decide_main` → `OpenLogin` para `https://github.com/login/oauth/authorize`, `https://t.auth0.com/authorize`, `https://x.b2clogin.com/…`; `https://github.com/user/repo` continua `External`; `https://evilauth0.com/` → `External`.

**Como verificar:** testes em `navigation_policy_tests` (exemplos acima) + manual com um site real que use Auth0.

**Cuidados / decisão humana.** A lista de sufixos é decisão de produto: tenants `*.auth0.com` são criáveis por qualquer pessoa, mas o popup não tem privilégios Tauri e só abre por navegação do site configurado. Não reintroduzir nomes de sites específicos no código (há teste que exige genericidade).

---

### FLY-P2-008 — Escopo de host estrito: apex↔`www` do site configurado cai no navegador externo

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Média |
| Categoria | Corretude · UX |
| Confiança | Provável (redirect `www`→apex depende do site) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `window.rs:225-239`, `284`, `292`, `299`, `314`, `330` |
| Depende de | FLY-P2-001 |

**Evidência.** Port executada: site `https://www.example.com/` + `https://example.com/` → `External(browser padrão)`; `https://br.example.com/` + `https://www.example.com/` → `External`.
```rust
host.ends_with(&format!(".{target}"))   // só o próprio host ou subdomínios do host configurado
```
**Descrição.** `host_matches(host, target)` aceita apenas igualdade ou subdomínio **do host configurado**. Se o usuário configura `https://www.site.com/` e o servidor canonicaliza para `https://site.com/` (ou um link aponta para o apex), a navegação é enviada ao navegador padrão e a janela do widget fica sem conteúdo. O sentido inverso (configurar o apex) já funciona.

**Solução proposta.** Adicionar um *wrapper* que aceita também o apex correspondente a um host configurado com prefixo `www.`, sem alargar para irmãos (`api.site.com`).
```diff
+/// Host do site configurado: igual, subdomínio, ou apex de um host `www.`.
+fn host_matches_site(host: &str, target: &str) -> bool {
+    if host_matches(host, target) {
+        return true;
+    }
+    normalized_host(target)
+        .strip_prefix("www.")
+        .is_some_and(|apex| normalized_host(host) == apex)
+}
```
Substituir `host_matches(...)` por `host_matches_site(...)` em `is_auth_page`, `is_configured_page`, `is_exact_configured_page`, `decide_main` e `decide_login` (não em `is_auth_provider`).

**Critérios de aceite:** site `https://www.example.com/` → `https://example.com/x` = `Allow`; `https://api.example.com/` = `External`; `https://evilexample.com/` = `External`.

**Como verificar** (novo teste):
```rust
#[test]
fn www_configured_site_also_accepts_its_apex_but_not_siblings() {
    let cfg = config("https://www.example.com/");
    assert_eq!(decide_main(&Url::parse("https://example.com/x").unwrap(), &cfg), NavAction::Allow);
    assert_eq!(decide_main(&Url::parse("https://api.example.com/").unwrap(), &cfg), NavAction::External);
    assert_eq!(decide_main(&Url::parse("https://evilexample.com/").unwrap(), &cfg), NavAction::External);
}
```
**Cuidados.** Não usar lista pública de sufixos nem heurística de "domínio registrável" (escopo).

---

### FLY-P2-009 — Textos do `INIT` incoerentes ("calendário"), `aria-label` fixo e rótulos divergentes

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Média |
| Categoria | i18n · UX/A11y |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `window.rs:26-28`, `30`, `94`, `184`, `i18n.rs:68-74` |
| Depende de | — |

**Evidência**
```js
'pt-BR':{... offlineTitle:'Sem conexão com o calendário', ...}
en:{... offlineTitle:'Calendar connection unavailable', ...}
es:{... offlineTitle:'Sin conexión con el calendario', ...}
frame.setAttribute('aria-label','Borda para mover o widget');   // linha 94: fixo em pt-BR
```
`rg -i calend` no repositório: as únicas ocorrências em código são essas 3 (e URLs de exemplo `economic-calendar` em README/testes, legítimas).

**Descrição.** (H5 confirmada) O widget é genérico, mas a tela offline fala em "calendário" nos 3 idiomas. Além disso, o `aria-label` do frame (`__widget_drag_frame`) é fixo em pt-BR e `setUiLanguage` não o atualiza; no popup, a barra usa `login:'Login'` (pt) enquanto o título da janela usa "Entrar" (`i18n::login_title`).

**Solução proposta.**
```diff
- offlineTitle:'Sem conexão com o calendário'
+ offlineTitle:'Sem conexão com o site'
- offlineTitle:'Calendar connection unavailable'
+ offlineTitle:'Website connection unavailable'
- offlineTitle:'Sin conexión con el calendario'
+ offlineTitle:'Sin conexión con el sitio'
```
Adicionar `frameLabel` em `UI_COPY` (`'Borda para mover o widget'` / `'Border to move the widget'` / `'Borde para mover el widget'`), usar em `install()` (linha 94) e atualizar em `setUiLanguage` (junto aos `.__widget_drag_edge`). Em `LOGIN_INIT`, `login:'Login'` → `login:'Entrar'`.

**Arquivos a alterar:** `window.rs` (`INIT`, `LOGIN_INIT`).

**Critérios de aceite:** `rg -i "calend" src-tauri/src` não encontra mais texto de interface; `npm run check:drag-overlay` passa; o `aria-label` muda ao trocar o idioma.

**Como verificar:** `node scripts/check-drag-overlay.mjs`; manual: alternar idioma nas Configurações e inspecionar o `aria-label` (DevTools em build *debug*).

**Cuidados.** Manter as 3 línguas sincronizadas e não quebrar as substrings exigidas pelo `check-drag-overlay.mjs` (seção 11).

---

### FLY-P2-010 — CSS global injetado quebra layouts/scroll lock de sites de terceiros

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Média |
| Categoria | Corretude · Desempenho · UX/A11y |
| Confiança | Provável |
| Esforço | M |
| Risco de regressão | Médio |
| Local | `window.rs:49-55` |
| Depende de | — |

**Evidência**
```css
html{overflow-y:auto!important}
html,body,*{scrollbar-width:thin;scrollbar-color:transparent transparent}
html::-webkit-scrollbar,body::-webkit-scrollbar,*::-webkit-scrollbar{width:9px;height:9px}
...
html.__widget_scroll_active,html.__widget_scroll_active body,html.__widget_scroll_active *{scrollbar-color:rgba(100,116,139,.82) transparent!important}
```
**Descrição.** (a) `html{overflow-y:auto!important}` anula o padrão comum de *scroll lock* de modais/menus (`html{overflow:hidden}`) e de apps de altura fixa → rolagem do fundo com modal aberto, barras duplas. (b) Regras com seletor universal `*` (e `*::-webkit-scrollbar*`) forçam recálculo de estilo para todos os elementos e tornam **invisíveis** as barras de rolagem de *todos* os contêineres internos (tabelas, caixas de código, listas) — a revelação só altera a classe no `html` e depende de roda/scroll/`pointermove` perto da borda direita. (c) `scrollbar-width/-color` e `::-webkit-scrollbar` são redundantes no Chromium atual (as propriedades padrão têm precedência).

**Impacto.** Sites com modais, *drawers* ou layouts `height:100%` podem se comportar mal; barras horizontais/internas "somem" (A11y para quem arrasta a barra).

**Solução proposta** (recomendada: limitar ao *viewport* e remover o `!important` de overflow; alternativa: manter e só documentar — descartada).
```diff
-html{overflow-y:auto!important}
-html,body,*{scrollbar-width:thin;scrollbar-color:transparent transparent}
-html::-webkit-scrollbar,body::-webkit-scrollbar,*::-webkit-scrollbar{width:9px;height:9px}
-html::-webkit-scrollbar-track,...
-html.__widget_scroll_active,html.__widget_scroll_active body,html.__widget_scroll_active *{scrollbar-color:rgba(100,116,139,.82) transparent!important}
-html.__widget_scroll_active::-webkit-scrollbar-thumb,...{background-color:rgba(100,116,139,.78)!important}
+html{scrollbar-width:thin;scrollbar-color:transparent transparent}
+html.__widget_scroll_active{scrollbar-color:rgba(100,116,139,.82) transparent}
```
**Arquivos a alterar:** `window.rs` (`INIT`, bloco CSS).

**Critérios de aceite:** barra do documento continua discreta e reaparece ao rolar; páginas com `html{overflow:hidden}` (modal aberto) deixam de rolar ao fundo; barras internas ficam visíveis.

**Como verificar:** manual no site de exemplo do README (`https://br.investing.com/economic-calendar`) e em um site com modal/scroll lock; `node scripts/check-drag-overlay.mjs`.

**Cuidados.** O `!important` pode ter sido adicionado para um caso observado pelo autor — **validar a rolagem do site de exemplo antes de remover** e, se regredir, reintroduzir só como `html:not([style*="overflow"])`. Não tocar nas regras do `#__widget_drag*` (cobertas por `check-drag-overlay`).

---

### FLY-P2-014 — Verificação de conectividade injetada: falha sob Trusted Types/CSP, sem `online`, roda na página local

| Campo | Valor |
|---|---|
| Parte | 2 (interface com 3) |
| Severidade | Média |
| Categoria | Robustez · UX/A11y |
| Confiança | Provável |
| Esforço | M |
| Risco de regressão | Médio |
| Local | `window.rs:18-19`, `33`, `47-48`, `168-171`; `tauri.conf.json` (`csp`) |
| Depende de | FLY-P2-009 |

**Evidência**
```js
b.innerHTML='<div style="font-size:30px">⌁</div><h2></h2><p></p><button type="button"></button>';   // 168
addEventListener('offline',check);  setInterval(check,30000);                                      // 170-171
```
```json
"csp": "... connect-src ipc: http://ipc.localhost https:; ..."   // sem 'self' em connect-src
```
**Descrição.** (1) Em páginas com `require-trusted-types-for 'script'` (comum em sites modernos), atribuir *string* a `innerHTML` lança `TypeError` **no `catch`** → o overlay nunca aparece e o erro vira rejeição não tratada. (2) O estilo do overlay (`#__widget_offline{…}`) vive em um `<style>` criado por script, que um `style-src` sem `'unsafe-inline'` bloqueia; a alça e as bordas foram feitas com CSSOM inline justamente para evitar isso — o overlay não. (3) Não há listener `online`: após a reconexão o overlay permanece até 30 s. (4) A guarda `localTauriOrigin` (linha 18) faz o script rodar também na **página local** `http://tauri.localhost`; ali o `fetch(new URL('/favicon.ico', location.origin))` é regido pela CSP local, cujo `connect-src` **não** lista `'self'` → o `fetch` falharia e, após 30 s na tela local (p. ex. com o site inacessível), o overlay "Sem conexão…" seria exibido por cima da UI local (**Hipótese** a confirmar em runtime; ligada à H1 da Parte 3). (5) O overlay não tem `role="alert"`.

**Solução proposta.** Construir o overlay com `createElement` + estilos via `setInlineStyles` (já existente), adicionar `online`, exigir origem remota para a checagem e `role="alert"`.
```diff
-function check... catch(_){ if(!document.getElementById('__widget_offline')){ const b=...; b.innerHTML='<div ...>'; ... } }
+const isAppOrigin=location.protocol==='tauri:'||location.host==='tauri.localhost';
+function showOffline(){
+ if(document.getElementById('__widget_offline'))return;
+ const copy=UI_COPY[UI_LANGUAGE]||UI_COPY['pt-BR'];
+ const b=document.createElement('div');b.id='__widget_offline';b.setAttribute('role','alert');
+ setInlineStyles(b,{position:'fixed','z-index':'2147483646',inset:'0',background:'rgba(248,250,253,.98)',color:'#203247',display:'flex','flex-direction':'column','align-items':'center','justify-content':'center','text-align':'center',font:'14px Segoe UI,Arial,sans-serif',padding:'24px'});
+ const icon=document.createElement('div');icon.textContent='⌁';icon.style.setProperty('font-size','30px');
+ const h=document.createElement('h2');h.textContent=copy.offlineTitle;
+ const p=document.createElement('p');p.textContent=copy.offlineMessage;
+ const btn=document.createElement('button');btn.type='button';btn.textContent=copy.retry;
+ setInlineStyles(btn,{border:'0',background:'#1769d2',color:'#fff','border-radius':'8px',padding:'10px 14px'});
+ btn.addEventListener('click',()=>location.reload());
+ b.append(icon,h,p,btn);(document.body||document.documentElement).appendChild(b);
+}
 ...
-addEventListener('offline',check);
-setInterval(check,30000);
+if(!isAppOrigin){addEventListener('offline',check);addEventListener('online',check);setInterval(check,30000);}
```
(com `check()` chamando `showOffline()` no `catch`). `setUiLanguage` já localiza `h2/p/button` por seletor — continua válido.

**Arquivos a alterar:** `window.rs` (`INIT`, remover regras `#__widget_offline*` do `<style>` se desejar).

**Critérios de aceite:** em página com Trusted Types e em página com `style-src` estrito o overlay aparece estilizado; após `online` o overlay some em segundos; na página local não há overlay.

**Como verificar:** manual (WebView2) — carregar uma página de teste com `Content-Security-Policy: require-trusted-types-for 'script'; style-src 'self'`, desconectar a rede, reconectar; `node scripts/check-drag-overlay.mjs`.

**Cuidados.** Revalidar com a correção de H1 na Parte 3 (a CSP pode mudar). Textos: pt-BR/en/es via `UI_COPY` (ver FLY-P2-009).

---

### FLY-P2-004 — Tokens do `INIT` não são globais, mas são observáveis pela página (hardening)

| Campo | Valor |
|---|---|
| Parte | 2 (e 3) |
| Severidade | Baixa |
| Categoria | Segurança |
| Confiança | Hipótese (exposição por Navigation API não testada; hook de `invoke` confirmado por leitura) |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | `window.rs:23-24`, `58`, `172-174`, `507-513`, `617-701`; `capabilities/main-remote-heartbeat.json` (Parte 3) |
| Depende de | FLY-P2-003 |

**Evidência**
```js
(function boot(){ ... const ACTION_TOKEN='__WIDGET_ACTION_TOKEN__'; const HEARTBEAT_TOKEN='__WIDGET_HEARTBEAT_TOKEN__'; ...
function action(p){location.href='widget-action://'+ACTION_TOKEN+'/'+p}
function heartbeat(){ ... core.invoke('heartbeat',{token:HEARTBEAT_TOKEN}) ...}   // lê window.__TAURI__.core.invoke a cada chamada
```
**Descrição e refinamento da H2.** A hipótese "qualquer script do site lê os tokens" é **parcialmente refutada**: ambos são `const` dentro da IIFE `boot()`, **não** são propriedades de `window`. Há, porém, vazamentos indiretos: (1) `heartbeat()` resolve `window.__TAURI__.core.invoke` **a cada 5 s**; um script do site pode substituir essa função por um *wrapper* e capturar `HEARTBEAT_TOKEN` (confirmado por leitura; vale também para `__TAURI_INTERNALS__`). (2) O token de ação trafega na URL de uma navegação iniciada pela página; a **Navigation API** (`navigation.addEventListener('navigate', e => e.destination.url)`) pode expô-la a scripts do site (**Hipótese**: não testado se o evento dispara para esquemas customizados em WebView2).

**Impacto real (proporcional).** Com `heartbeat`: só forjar sinal de vida (afeta o *watchdog* — Parte 1). Com `widget-action`: quem tem o token só consegue iniciar arraste/resize **enquanto o botão primário estiver pressionado** (checagem Win32) e com a janela destravada — incômodo, sem execução de código. Não existe forma de manter um segredo inacessível à página no mesmo *world* do JS; a mitigação correta é **reduzir a capacidade**, não esconder o token.

**Solução proposta.**
1. Já proposto em FLY-P2-003: ignorar ação se o cursor não estiver sobre a janela do widget (reduz o abuso a cliques sobre o próprio widget).
2. (Opcional) capturar a referência de `invoke` no *boot*, antes de scripts do site: `const invoke=window.__TAURI__&&window.__TAURI__.core&&window.__TAURI__.core.invoke;` e usá-la em `heartbeat()`. Dificulta, mas não impede, a captura (o site pode envolver `fetch`/`postMessage`).
3. Registrar no relatório/README (Parte 4) que os tokens protegem contra *navegações forjadas sem conhecimento do token*, não contra o script do próprio site.

**Arquivos a alterar:** `window.rs` (`INIT`); nenhuma mudança de ACL nesta parte. Para a restrição de `http://*`/`https://*` na capability, ver Parte 3.

**Critérios de aceite:** item 1 coberto por FLY-P2-003; teste manual descrito abaixo.

**Como verificar (teste da hipótese 2).** Em página de teste servida pelo site configurado:
```js
navigation.addEventListener('navigate', e => console.log('NAV', e.destination.url));
```
clicar na alça `⋮⋮` e ver se `widget-action://<token>/drag` aparece. Registrar o resultado (confirmado/refutado).

**Cuidados.** Não remover `event.isTrusted`, a checagem de `locked` nem o token (camada de defesa existente).

---

### FLY-P2-006 — `is_internal_app_url` não valida porta nem host do esquema `tauri:`

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Baixa |
| Categoria | Segurança |
| Confiança | Confirmado (código e parsing); impacto Hipótese |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `window.rs:342-349`, `528-534` |
| Depende de | — |

**Evidência**
```rust
fn is_internal_app_url(url: &Url) -> bool {
    if url.scheme() == "tauri" { return true; }
    let host = url.host_str().unwrap_or("");
    (url.scheme() == "http" && host == "tauri.localhost")
        || (cfg!(debug_assertions) && host == "127.0.0.1")
}
```
Parsing WHATWG executado: `tauri://evil/x` → protocolo `tauri:` (aceito); `http://tauri.localhost:8080/x` → host `tauri.localhost`, porta 8080 (aceito). Em *debug*, `127.0.0.1` é aceito com **qualquer** esquema.

**Descrição.** H "`cfg!(debug_assertions)` libera `127.0.0.1` em *release*" foi **refutada** (em *release* é `false`). O que resta: o teste ignora porta e host do esquema `tauri:`. Um site remoto que navegue o topo para `http://tauri.localhost:PORTA/…` contorna a política "externo → navegador padrão" (o Chromium trata `*.localhost` como loopback), levando a janela principal a conteúdo de um serviço local qualquer. Se esse conteúdo seria tratado como "local" pelo ACL do Tauri (`main-local`) **não foi verificado**.

**Solução proposta**
```diff
 fn is_internal_app_url(url: &Url) -> bool {
-    if url.scheme() == "tauri" {
-        return true;
-    }
-    let host = url.host_str().unwrap_or("");
-    (url.scheme() == "http" && host == "tauri.localhost")
-        || (cfg!(debug_assertions) && host == "127.0.0.1")
+    let host = url.host_str().unwrap_or("");
+    match url.scheme() {
+        "tauri" => host == "localhost" && url.port().is_none(),
+        "http" => {
+            (host == "tauri.localhost" && url.port().is_none())
+                || (cfg!(debug_assertions) && host == "127.0.0.1")
+        }
+        _ => false,
+    }
 }
```
**Critérios de aceite:** testes existentes (`allows_tauri_windows_asset_origin`, `allows_tauri_protocol`) passam; novos: `tauri://evil/` e `http://tauri.localhost:8080/` → `false`.

**Como verificar**
```rust
#[test]
fn internal_origin_rejects_other_hosts_and_ports() {
    assert!(!is_internal_app_url(&Url::parse("tauri://evil/").unwrap()));
    assert!(!is_internal_app_url(&Url::parse("http://tauri.localhost:8080/").unwrap()));
    assert!(!is_internal_app_url(&Url::parse("https://tauri.localhost/").unwrap()));
}
```
**Cuidados.** O build de desenvolvimento usa `http://127.0.0.1:1420` (Vite) — mantido para `debug_assertions`. Se um dia `useHttpsScheme` for ativado no Tauri, incluir `https://tauri.localhost`.

---

### FLY-P2-011 — `MutationObserver` com `subtree` reexecuta `install()` a cada mutação do DOM

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Baixa |
| Categoria | Desempenho |
| Confiança | Provável |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `window.rs:89-139` |
| Depende de | — |

**Evidência**
```js
const dragObserver=new MutationObserver(()=>install());
dragObserver.observe(document.documentElement,{childList:true,subtree:true});
```
`install()` sempre termina em `applyDragLockedUi()` (4 `querySelectorAll`-iterações + ~8 `setProperty`).

**Descrição.** Em sites com DOM muito dinâmico (cotações, tabelas ao vivo — o site de exemplo é um calendário econômico) o callback roda dezenas de vezes por segundo. Só é preciso reagir à **remoção** do frame (filho de `<html>`) e da alça (filho de `<body>`).

**Solução proposta**
```diff
+function install(){ if(!document.documentElement)return;
+ if(document.getElementById('__widget_drag_frame')&&document.getElementById('__widget_drag'))return;
 ...
-const dragObserver=new MutationObserver(()=>install());
-dragObserver.observe(document.documentElement,{childList:true,subtree:true});
+let observedBody=null;
+const dragObserver=new MutationObserver(()=>{install();const b=document.body;if(b&&b!==observedBody){observedBody=b;dragObserver.observe(b,{childList:true})}});
+dragObserver.observe(document.documentElement,{childList:true});
+if(document.body){observedBody=document.body;dragObserver.observe(document.body,{childList:true})}
```
**Critérios de aceite:** a alça e a borda reaparecem se o site remover/substituir `<body>`; `check-drag-overlay` passa; em página com mutações contínuas o `install()` não executa `applyDragLockedUi()` repetidamente.

**Como verificar:** manual + DevTools Performance em build *debug*; `node scripts/check-drag-overlay.mjs`.

**Cuidados.** `setUiLocked` chama `applyDragLockedUi()` diretamente — o *early return* não afeta a troca de travamento. Manter as substrings exigidas pelo check (seção 11).

---

### FLY-P2-012 — Placeholders do `INIT`/`LOGIN_INIT` substituídos sem escape/allow-list

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Baixa |
| Categoria | Segurança (defesa em profundidade) · Manutenibilidade |
| Confiança | Provável (mitigado por `normalize()` — Parte 1 — que limita `language` a `pt-BR`/`en`/`es`) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `window.rs:21`, `178`, `509-513`, `839-843`; `config.rs:148-149` |
| Depende de | — |

**Evidência**
```rust
INIT.replace("__WIDGET_LANGUAGE__", &c.language)   // JS: let UI_LANGUAGE='__WIDGET_LANGUAGE__';
```
**Descrição.** O valor entra num literal JS entre aspas simples sem escape. Hoje é seguro porque `normalize()` valida `language`; mas `on_page_load`/`apply_config` usam `serde_json::to_string` (forma segura), tornando a abordagem inconsistente. Se uma futura via de entrada esquecer `normalize()`, haveria injeção de JS no contexto do site.

**Solução proposta.** Função de allow-list em `i18n.rs` e uso nas duas substituições.
```diff
+// i18n.rs
+/// Devolve sempre um idioma suportado, nunca texto arbitrário.
+pub fn normalize_language(language: &str) -> &'static str {
+    match language { "en" => "en", "es" => "es", _ => "pt-BR" }
+}
```
```diff
-        .replace("__WIDGET_LANGUAGE__", &c.language)
+        .replace("__WIDGET_LANGUAGE__", crate::i18n::normalize_language(&c.language))
```
(e em `open_login`: `LOGIN_INIT.replace("__WIDGET_LANGUAGE__", crate::i18n::normalize_language(&language))`).

**Critérios de aceite / verificação**
```rust
#[test]
fn normalize_language_never_returns_untrusted_text() {
    assert_eq!(normalize_language("en'};alert(1)//"), "pt-BR");
    assert_eq!(normalize_language("es"), "es");
}
```
**Cuidados.** Não mudar os literais do JS (o `check-drag-overlay` não os testa, mas manter o formato).

---

### FLY-P2-013 — Atalhos Ctrl+L/T/N/W/P/J/U/S são engolidos com `stopImmediatePropagation`

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Baixa |
| Categoria | UX · Corretude |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `window.rs:157-162` |
| Depende de | — |

**Evidência**
```js
if((e.ctrlKey||e.metaKey)&&['l','t','n','w','p','j','u','s'].includes(e.key.toLowerCase())){e.preventDefault();e.stopImmediatePropagation()}
```
**Descrição.** O *listener* em captura no `document` impede que a **página** receba Ctrl+S/P/etc. (apps web que usam Ctrl+S para salvar, editores, planilhas). Bloquear a ação padrão do navegador é a intenção; privar o site do evento não é necessário.

**Solução proposta (recomendada):** manter `preventDefault()` e remover `stopImmediatePropagation()`.
```diff
-...includes(e.key.toLowerCase())){e.preventDefault();e.stopImmediatePropagation()}
+...includes(e.key.toLowerCase())){e.preventDefault()}
```
**Critérios de aceite:** Ctrl+P/S não abre diálogos do navegador, mas um site com `keydown` próprio ainda os recebe.

**Como verificar:** manual com uma página de teste que escuta `keydown`.

**Cuidados / decisão humana.** Se o autor **quer** isolar o site desses atalhos, manter como está e documentar. `F5` → `location.reload()` não muda.

---

### FLY-P2-015 — Canal `location.href='widget-action://…'` pode disparar `beforeunload` do site

| Campo | Valor |
|---|---|
| Parte | 2 (e 1/3 se houver troca de canal) |
| Severidade | Baixa |
| Categoria | Robustez · UX |
| Confiança | Hipótese |
| Esforço | M |
| Risco de regressão | Médio |
| Local | `window.rs:58`, `82-88`, `140-150`, `535-571` |
| Depende de | FLY-P2-004 |

**Descrição.** Comandos da UI injetada são enviados atribuindo `location.href`. Em navegações de topo iniciadas pelo documento, o Chromium executa `beforeunload` antes de a navegação chegar ao `NavigationStarting` (onde o app a cancela). Sites com `beforeunload` (formulários/editores com alterações não salvas) podem exibir "Sair do site?" ao clicar na alça ou nas bordas. **Não testado.**

**Como verificar (obrigatório antes de qualquer mudança):** página de teste com `addEventListener('beforeunload', e => { e.preventDefault(); e.returnValue=''; })`, interagir com o campo e arrastar pela alça.

**Solução proposta (somente se confirmada).** Preferida: manter o canal e registrar, no *boot*, um listener de captura em `window` que suprime `beforeunload` do site **apenas durante uma ação do widget** (flag com janela curta, p. ex. 1 s; o valor deve ser validado empiricamente porque o disparo pode ser assíncrono):
```js
let actionUntil=0;
addEventListener('beforeunload',e=>{if(Date.now()<actionUntil)e.stopImmediatePropagation()},true);
function action(p){actionUntil=Date.now()+1000;location.href='widget-action://'+ACTION_TOKEN+'/'+p}
```
Alternativa (mais invasiva, descartada por ora): trocar o canal por um comando IPC (`window_action`) — exige alterar `commands.rs`, `build.rs` e capabilities **juntos** (`npm run check:acl`) e **ampliar** a superfície remota além de `heartbeat`.

**Critérios de aceite:** com a página de teste, nenhuma caixa de `beforeunload` ao arrastar; `check-drag-overlay` passa (a substring `widget-action://'+ACTION_TOKEN` deve permanecer).

**Cuidados.** Se a hipótese for refutada, fechar o achado sem alterações.

---

### FLY-P2-016 — Travas `SETTINGS_WINDOW_CREATING`/`geometry_save_worker_running` sem guard RAII

| Campo | Valor |
|---|---|
| Parte | 2 (e 1: `AppState`) |
| Severidade | Baixa |
| Categoria | Concorrência · Robustez |
| Confiança | Provável |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `window.rs:1199-1261`, `1263-1302` |
| Depende de | — |

**Evidência**
```rust
if SETTINGS_WINDOW_CREATING.compare_exchange(false, true, ...).is_err() { return; }
thread::spawn(move || {
    ...
    let result = match std::fs::create_dir_all(&dir) { Ok(()) => WebviewWindowBuilder::new(...).build()... };
    SETTINGS_WINDOW_CREATING.store(false, Ordering::SeqCst);   // 1252
```
**Descrição (H4: parcialmente confirmada).** Nos caminhos de **erro** (`Err` de `build`/`create_dir_all`) a flag é liberada — esse risco foi refutado. Mas um *panic* entre o `compare_exchange` e o `store(false)` (inclusive dentro de `build()` ou de `app.state::<AppState>()`) deixa a flag `true` para sempre e `open_settings` passa a retornar em silêncio (o usuário não consegue reabrir Configurações até reiniciar). O mesmo vale para `geometry_save_worker_running` no *worker* de debounce (um *panic* em `persist_current()` interrompe o salvamento de posição/tamanho pelo resto da sessão). Em ambos, cliques durante a criação são ignorados sem feedback.

**Solução proposta**
```rust
struct ResetOnPanic<'a>(&'a AtomicBool);
impl Drop for ResetOnPanic<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() { self.0.store(false, Ordering::SeqCst); }
    }
}
```
- `open_settings`: `let _guard = ResetOnPanic(&SETTINGS_WINDOW_CREATING);` no início da *thread* (mantém o `store(false)` existente).
- Worker de debounce: guardar um `AppHandle` e liberar a flag **apenas se `panicking()`** (um `store(false)` incondicional no `Drop` quebraria a invariante quando o worker reagenda a si mesmo, linhas 1297-1299):
```rust
struct GeometryWorkerGuard(AppHandle);
impl Drop for GeometryWorkerGuard {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.state::<AppState>().geometry_save_worker_running.store(false, Ordering::SeqCst);
        }
    }
}
// no início do loop: let _guard = GeometryWorkerGuard(app.clone());
```
**Critérios de aceite:** comportamento normal inalterado; teste de unidade do guard (`catch_unwind` + verificar flag).

**Como verificar**
```rust
#[test]
fn reset_on_panic_clears_flag_only_when_panicking() {
    static FLAG: AtomicBool = AtomicBool::new(true);
    let _ = std::panic::catch_unwind(|| { let _g = ResetOnPanic(&FLAG); panic!("x"); });
    assert!(!FLAG.load(Ordering::SeqCst));
}
```
**Cuidados.** O `AppState` é da Parte 1 — não alterar sua definição.

---

### FLY-P2-017 — Persistência de geometria: `remember_position=false` descarta também o tamanho; custo por evento

| Campo | Valor |
|---|---|
| Parte | 2 (e 1/3) |
| Severidade | Baixa |
| Categoria | Corretude · Desempenho |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Médio |
| Local | `window.rs:1324-1359`; `config.rs` (`save` com *retry* de `rename`, 3×50 ms) |
| Depende de | — |

**Evidência**
```rust
let mut c = state.lock_config();
if !c.remember_position { return; }          // 1341: também impede salvar width/height
...
let copy = c.clone(); drop(c);
schedule_debounced_config_save(app);
let _ = app.emit("config-changed", &copy);    // 1358: a CADA Moved/Resized
```
**Descrição.** (a) `remember_position=false` impede que o **tamanho** redimensionado pelo usuário seja salvo (o nome da opção sugere apenas posição). (b) A cada evento `Moved`/`Resized` (dezenas/s durante o arraste) o código clona a configuração e a emite; só a janela Configurações costuma ouvir. (c) `lock_config()` é chamado na *thread* de UI e o mesmo *mutex* é mantido por `save_config`/`toggle_*` durante `config::save` (que pode dormir até ≈100 ms em *retry*), bloqueando eventos de janela nesse intervalo.

**Solução proposta.** Emitir apenas se a janela `settings` existir (mudança segura); a separação posição/tamanho é **decisão do autor** (ver plano).
```diff
-    let _ = app.emit("config-changed", &copy);
+    if app.get_webview_window("settings").is_some() {
+        let _ = app.emit("config-changed", &copy);
+    }
```
Opcional (se o autor decidir que tamanho deve ser lembrado sempre): aplicar `remember_position` apenas ao ramo `WindowEvent::Moved`.

**Critérios de aceite:** com Configurações aberta, os campos continuam acompanhando a janela; sem ela, nenhum `emit` durante o arraste.

**Como verificar:** manual + log; teste existente `window_state_*` continua passando.

**Cuidados.** Não alterar `config::save` (Parte 1). Verificar em `settings.ts` (Parte 3) como `config-changed` é consumido.

---

### FLY-P2-018 — Auto-reload dispara com janela minimizada e a *thread* nunca encerra

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Baixa |
| Categoria | Robustez · Desempenho |
| Confiança | Provável |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `window.rs:1387-1412` |
| Depende de | — |

**Evidência**
```rust
let visible = window.as_ref().is_some_and(|window| window.is_visible().unwrap_or(false));
```
**Descrição.** No Windows, `IsWindowVisible` é verdadeiro para janelas **minimizadas**; o reload ocorre sem ninguém ver, gerando tráfego e perda de estado. A *thread* é um laço infinito sem verificação de `shutdown_requested`. (Testado e correto: a aritmética de `next_reload_state` usa saturação; reload oculto é adiado.)

**Solução proposta**
```diff
-            let visible = window
-                .as_ref()
-                .is_some_and(|window| window.is_visible().unwrap_or(false));
+            if app.state::<AppState>().shutdown_requested.load(Ordering::SeqCst) { break; }
+            let visible = window.as_ref().is_some_and(|window| {
+                window.is_visible().unwrap_or(false) && !window.is_minimized().unwrap_or(false)
+            });
```
**Critérios de aceite:** minimizar o widget com auto-reload de 1 min não recarrega; ao restaurar, recarrega no próximo ciclo (comportamento já testado para "oculto"). **Cuidados:** manter os testes de `next_reload_state`.

---

### FLY-P2-019 — Título da janela de login fixo no host inicial

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Baixa |
| Categoria | Segurança (sinal de confiança) · UX |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `window.rs:826-830`, `740-781`, `185-203` |
| Depende de | — |

**Evidência**
```rust
let host = url.host_str().unwrap_or("Login").to_owned();
...
.title(format!("{login_title} — {host}"))     // definido uma única vez, no build
```
**Descrição.** Após redirecionamentos (site → IdP → site) e após `Reuse` (`existing.navigate(url)`), o título continua mostrando o primeiro host. Como a barra de endereço injetada é DOM da página (falsificável), o título nativo é o único indicador confiável do destino.

**Solução proposta.** Atualizar o título a cada carga.
```diff
         .zoom_hotkeys_enabled(false)
+        .on_page_load({
+            let title_prefix = login_title;
+            move |window, payload| {
+                if payload.event() == PageLoadEvent::Finished {
+                    if let Some(host) = payload.url().host_str() {
+                        let _ = window.set_title(&format!("{title_prefix} — {host}"));
+                    }
+                }
+            }
+        })
```
**Critérios de aceite:** o título reflete o host atual após redirecionamento e após `Reuse`. **Cuidados:** a assinatura exata do parâmetro de `WebviewWindowBuilder::on_page_load` em Tauri 2.12.5 **não foi verificada** (o código existente usa `window.eval`, comum a `Webview`/`WebviewWindow`); se for `Webview`, usar `window.window().set_title(..)`.

---

### FLY-P2-020 — Bandeja: `DoubleClick` possivelmente inalcançável; ícone ausente sem log; falha de `tray::setup` é fatal

| Campo | Valor |
|---|---|
| Parte | 2 (e 1: `main.rs`, chamada `tray::setup`) |
| Severidade | Baixa |
| Categoria | UX · Robustez |
| Confiança | Hipótese |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `tray.rs:56-88`, `63-65`, `77-85`; `main.rs` (`tray::setup(app.handle())?`) |
| Depende de | — |

**Descrição.** (1) Por padrão o Tauri 2 abre o menu no clique esquerdo; o primeiro clique de um duplo-clique abre o menu modal e o segundo tende a fechá-lo, de modo que `TrayIconEvent::DoubleClick` possivelmente nunca chega (**Hipótese** — testar no Windows). (2) Se `app.default_window_icon()` é `None` o ícone fica vazio e nada é registrado. (3) `tray::setup(...)?` propaga o erro e aborta a inicialização (ex.: Explorer ainda não pronto no início da sessão); como a janela só oculta ao fechar, sem bandeja o usuário não tem como sair — abortar pode ser deliberado (**decisão do autor**).

**Solução proposta (mínima, sem mudar UX):** registrar ausência de ícone.
```diff
-    if let Some(icon) = app.default_window_icon().cloned() {
-        builder = builder.icon(icon);
-    }
+    match app.default_window_icon().cloned() {
+        Some(icon) => builder = builder.icon(icon),
+        None => diagnostics::log(
+            &app.state::<AppState>().app_dir,
+            "WARN",
+            "A bandeja foi criada sem ícone: nenhum ícone padrão da janela.",
+        ),
+    }
``` Se (1) for confirmada: `.show_menu_on_left_click(false)` e tratar `TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }` para alternar a janela — **muda o comportamento** (decisão humana).

**Critérios de aceite / verificação:** manual no Windows (duplo-clique alterna? menu abre?). **Cuidados:** `tray::update_language` reconstrói o menu a cada `save_config` (inclusive sem mudança de idioma); aceitável, só registrar.

---

### FLY-P2-021 — Melhorias menores consolidadas

| Campo | Valor |
|---|---|
| Parte | 2 |
| Severidade | Baixa |
| Categoria | Manutenibilidade · Robustez · i18n · UX |
| Confiança | Confirmado |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | ver lista |
| Depende de | — |

Cada item é independente; aplicar somente se o autor concordar.

| # | Local | Observação | Sugestão |
|---|---|---|---|
| a | `window.rs:516`, `609` | Janela criada `visible(true)` **antes** de `restore_position` → "salto" visual na inicialização | `.visible(false)` + `show()` após `restore_position` (manter `set_window_visible(true)` em `main.rs`) |
| b | `window.rs:324-336`, `883`, `910` | `decide_login` nunca devolve `OpenLogin`; os *arms* `Allow \| OpenLogin` são código morto | Simplificar ou documentar |
| c | `window.rs:271-278` | `is_login_path` não decodifica percent-encoding (`/%61ccount` ≠ `account`; verificado com WHATWG) e `host_matches` aceita `example.com..` (todos os pontos finais são removidos) | Decodificar segmentos (`percent_encoding`) ou aceitar a limitação; teste |
| d | `window.rs:829-838`, `1238-1244` | Login/Configurações não herdam `always_on_top`; podem abrir **atrás** do widget "sempre no topo" | `.always_on_top(true)` apenas quando o principal estiver no topo |
| e | `window.rs:1101-1106` | `apply_native_corners` recria HRGN e redesenha (`bRedraw=1`) em **cada** `Resized` | Debounce ou `bRedraw=0` + `RedrawWindow` |
| f | `window.rs:26-28`, `181-184`, `i18n.rs` | Textos em 3 lugares (JS principal, JS do login, `i18n.rs`) | Centralizar em `i18n.rs` e injetar JSON via `serde_json` |
| g | `window.rs:1183-1196` | `let _ =` descarta erros de `set_always_on_top/skip_taskbar/resizable/size/zoom/shadow/eval` | Usar `diagnostics::log_err` |
| h | `window.rs:342-349`, `306-312` | A política ignora **esquema e porta** do host configurado (`http://` aceito se o site é `https://`) | Documentar (aceitável) ou restringir esquema |
| i | `tray.rs:28`, `60` | ID legado `widget-website-tray` (H14: nomes internos antigos) | Manter (renomear exige migração; sem ganho) |
| j | `window.rs:457-463` | Aberturas externas descartadas sem aviso ao usuário (só log) | Aceitável; opcional toast |
| k | `i18n.rs:76-105` | Testes não garantem que **todos** os campos de `TrayLabels` sejam não vazios em cada idioma | Acrescentar teste (seção 8) |

---

## 6. Achados transversais (mais de uma parte)

| ID relacionado | Partes | Ordem de correção | Observação |
|---|---|---|---|
| FLY-P2-014 ↔ **H1** (Parte 3) | 2 + 3 | Corrigir H1 (CSP/`connect-src`) **antes** de validar FLY-P2-014 | O `fetch` injetado e o `fetch` de `main.ts` compartilham a CSP local; a correção da CSP pode tornar desnecessária a guarda `isAppOrigin`. |
| FLY-P2-004 ↔ capability `main-remote-heartbeat` (`http://*`, `https://*`) | 2 + 3 | Parte 3 decide se restringe ao host configurado; Parte 2 só reduz impacto | Capabilities são estáticas; restringir ao host dinâmico pode exigir `Window::add_capability` em runtime (Parte 1/3). |
| FLY-P2-016 / -017 ↔ `AppState` (`config.rs`) e `main.rs` | 1 + 2 | Parte 1 não precisa mudar `AppState` para corrigir -016; -017 depende de decisão de produto | `lock_config` mantido durante `config::save` bloqueia a *thread* de UI por ≤ ≈100 ms. |
| FLY-P2-020 ↔ `main.rs` (`tray::setup(...)?`, `WindowEvent::Destroyed → exit(1)`) | 1 + 2 | Avaliar na Parte 1 | Política "falhar rápido" vs. "continuar sem bandeja". |
| FLY-P2-007 ↔ README/Store (Parte 4) | 2 + 4 | Documentar a limitação de `window.opener` | Evitar promessa de "qualquer login funciona". |
| `check-drag-overlay.mjs` ↔ `INIT` (Parte 4) | 2 + 4 | Qualquer achado que edite `INIT` deve rodar `npm run check:drag-overlay` | Script é regex sobre `window.rs` (H11). |

---

## 7. Plano de correção ordenado

Não há achados Críticos/Altos; a **Onda 1** contém os pré-requisitos e o desvio de login (mais visível). Como (quase) tudo está em `window.rs`, **aplicar sequencialmente** (os números de linha mudam após cada edição — reler o trecho). Itens em `tray.rs`/`i18n.rs` podem ser feitos em paralelo.

### Onda 1 — pré-requisitos e correção mais visível
| Ordem | ID | Arquivo(s) | Esforço | Como validar |
|---:|---|---|---|---|
| 1 | FLY-P2-001 | `window.rs` | M | `cargo test … navigation_policy` + 2 testes novos |
| 2 | FLY-P2-002 | `window.rs` | P | `cargo test`; manual (clique rápido na alça) |

### Onda 2 — médias
| Ordem | ID | Arquivo(s) | Esforço | Como validar | Notas |
|---:|---|---|---|---|---|
| 3 | FLY-P2-008 | `window.rs` | P | teste `www_configured_site_also_accepts_its_apex_but_not_siblings` | depende de 001 |
| 4 | FLY-P2-007 | `window.rs` | M | testes de IdP; manual com Auth0 | **decisão humana** (lista de sufixos); depende de 001 |
| 5 | FLY-P2-003 | `window.rs` | M | clippy; manual (botões trocados, DPI misto) | depende de 002 |
| 6 | FLY-P2-005 | `window.rs` | M | `external_open_budget_caps_openings_per_minute` | **decisão humana** (5/60 s) |
| 7 | FLY-P2-009 | `window.rs` (INIT/LOGIN_INIT) | P | `rg -i calend`; `check:drag-overlay` | — |
| 8 | FLY-P2-010 | `window.rs` (INIT/CSS) | M | manual no site de exemplo; `check:drag-overlay` | validar rolagem antes de remover `!important` |
| 9 | FLY-P2-014 | `window.rs` (INIT) | M | manual (Trusted Types/CSP); coordenar com H1 (Parte 3) | depende de 009 |

### Onda 3 — baixas e documentação
| Ordem | ID | Arquivo(s) | Esforço | Como validar | Notas |
|---:|---|---|---|---|---|
| 10 | FLY-P2-006 | `window.rs` | P | `internal_origin_rejects_other_hosts_and_ports` | — |
| 11 | FLY-P2-012 | `window.rs`, `i18n.rs` | P | `normalize_language_never_returns_untrusted_text` | paralelizável (i18n) |
| 12 | FLY-P2-016 | `window.rs` | P | `reset_on_panic_clears_flag_only_when_panicking` | — |
| 13 | FLY-P2-018 | `window.rs` | P | manual (minimizar) | — |
| 14 | FLY-P2-019 | `window.rs` | P | manual (redirecionamento no popup) | verificar assinatura de `on_page_load` |
| 15 | FLY-P2-011 | `window.rs` (INIT) | P | manual + `check:drag-overlay` | — |
| 16 | FLY-P2-013 | `window.rs` (INIT) | P | manual | **decisão humana** |
| 17 | FLY-P2-004 / -015 | `window.rs` (INIT) | M | **testes de hipótese primeiro** (Navigation API; `beforeunload`) | fechar sem mudança se refutado |
| 18 | FLY-P2-017 | `window.rs` | P | manual | **decisão humana** (tamanho × posição) |
| 19 | FLY-P2-020 | `tray.rs` | P | manual | **decisão humana** (UX do duplo-clique) — paralelizável |
| 20 | FLY-P2-021 | vários | M | conforme item | itens independentes |

**Exigem decisão humana:** 005 (valores do orçamento), 007 (lista de IdPs), 013 (isolar o site dos atalhos?), 017 (semântica de `remember_position`), 020 (comportamento da bandeja/falha de `setup`), 021-a (flash de inicialização).

---

## 8. Testes a adicionar

**Rust inline (`window.rs`, `navigation_policy_tests`, `internal_app_navigation_tests`)** — todos usam o `config(..)` e `Url::parse(..)` já presentes. Os valores esperados de parsing foram verificados com o parser WHATWG (Node `URL`).

| Nome sugerido | O que cobre | Achado |
|---|---|---|
| `main_window_does_not_divert_ambiguous_site_pages_to_login` | `/account/…`, `/accounts/…`, `/docs/auth/…`, `/api/callback` → `Allow` | 001 |
| `main_window_still_diverts_real_login_pages` | `/login`, `/signin`, `/oauth/authorize`, `/auth/start?client_id=…&redirect_uri=…` → `OpenLogin` | 001 |
| `www_configured_site_also_accepts_its_apex_but_not_siblings` | apex ok; `api.` e `evilexample.com` → `External` | 008 |
| `known_idps_open_in_login_popup` | GitHub/Auth0/B2C → `OpenLogin`; `github.com/user/repo` → `External`; `evilauth0.com` → `External` | 007 |
| `external_open_budget_caps_openings_per_minute` | orçamento 5/60 s | 005 |
| `internal_origin_rejects_other_hosts_and_ports` | `tauri://evil/`, `http://tauri.localhost:8080/`, `https://tauri.localhost/` → `false` | 006 |
| `reset_on_panic_clears_flag_only_when_panicking` | guard RAII | 016 |
| `host_matches_rejects_lookalike_suffix` | `evilexample.com` vs `example.com` → `false` (o existente cobre `investing`; generalizar) | lacuna |
| `userinfo_confusion_routes_to_real_host` | `https://example.com@evil.com/` → host `evil.com` → `External` | lacuna |
| `percent_encoded_host_is_decoded_before_matching` | `https://exa%6dple.com/` → host `example.com` → `Allow` | lacuna |
| `idn_hosts_compare_in_punycode` | site `https://exämple.com/` e `https://xn--exmple-cua.com/` → `Allow` (ambos `xn--exmple-cua.com`) | lacuna |
| `ipv6_literal_hosts_match_only_exactly` | site `http://[::1]:8080/` e `http://[::2]/` → `External` (host com colchetes) | lacuna |
| `double_slash_login_path_is_still_a_login_segment` | `//login` → segmento `login` | lacuna |
| `scheme_blocking_covers_javascript_data_blob_file_about` | `decide_main`/`decide_login` → `Block` | lacuna |

**Rust inline (`i18n.rs`)**
| Nome | O que cobre | Achado |
|---|---|---|
| `tray_labels_are_non_empty_for_every_field_and_language` | nenhum campo vazio | 021-k |
| `normalize_language_never_returns_untrusted_text` | allow-list | 012 |

**Scripts `check:*` (Parte 4 — sugestão):** estender `check-drag-overlay.mjs` para afirmar que `INIT` não contém a palavra "calend" e que não usa `innerHTML` com string (regressão de 009/014).

**Frontend:** não aplicável a esta parte.

---

## 9. Itens descartados e status das hipóteses H1–H14

### Hipóteses (status nesta execução)
| # | Situação | Detalhe |
|---|---|---|
| H1 | Fora da Parte 2 (não verificada) | Relação com FLY-P2-014 (item 4) |
| H2 | **Parcialmente refutada / refinada** | Tokens estão em escopo de função (não globais) → FLY-P2-004 (Baixa/Hipótese) |
| H3 | **Confirmada** | FLY-P2-001 |
| H4 | **Parcialmente confirmada** | Erros liberam a flag; *panic* não → FLY-P2-016 |
| H5 | **Confirmada** | FLY-P2-009 |
| H6 | Parcial (lado INIT) | CSP/Trusted Types afetam o overlay (FLY-P2-014); `window.__TAURI__` ausente só silencia o *heartbeat*; o *watchdog* é da Parte 1 (não verificado) |
| H7–H13 | Fora da Parte 2 | Não analisadas |
| H14 | Parcial | ID de bandeja `widget-website-tray` (FLY-P2-021-i); pasta `webview-data` compartilhada entre principal e login (cookies) — resto na Parte 1/4 |

### Refutados (evita retrabalho)
| Hipótese/ideia | Evidência da refutação |
|---|---|
| `cfg!(debug_assertions)` libera `127.0.0.1` em *release* | Em *release* a expressão é `false` (`window.rs:348`); só *debug* (residual em FLY-P2-006) |
| `host_matches` aceita *lookalikes* (`evilinvesting.com`) | Exige fronteira de ponto; testes existentes + port executada |
| Vazamento de HRGN em `apply_native_corners` | `DeleteObject` apenas quando `SetWindowRgn` falha; em sucesso o sistema assume a posse; `r==0` usa `SetWindowRgn(null)` (`1086-1092`) |
| Precedência em `GetAsyncKeyState(..) as u16 & 0x8000 == 0` | Em Rust `as` > `&` > `==`: avalia como `((x as u16) & 0x8000) == 0` |
| `pack_lparam` quebra coordenadas negativas | Teste `pack_lparam_preserves_negative_coordinates`; truncamento para `u16` é o formato de `POINTS` |
| *Deadlock*/*poison* em `LOGIN_WINDOW_LOCK` | Recuperação de *poison*; `open_login` só roda em *thread* criada por `open_login_limited`; `expect` em `Reuse` inalcançável (`existing>0`) |
| Corrida entre `schedule_debounced_config_save` e o *worker* | Protocolo `revision` + `running` correto (exceto *panic*, FLY-P2-016) |
| Estouro em `is_placeable`/`corner_region_dims` | `i64` e saturações; testes `extreme_coordinates_do_not_overflow` |
| Abertura externa com injeção de comando (`open::that`) | Só `http/https` chegam a `open_external_limited`; `open` 5.4.5 no Windows usa API de shell, não `cmd` (**não verificado no código-fonte da crate** — sem acesso à rede) |
| Vazamento de segredos/queries nos logs | `sanitize_url` remove credenciais/query/fragmento; o ramo `widget-action` retorna **antes** de `log_navigation`; logs usam apenas host |
| `widget-action://` aceita extras | `authorized_widget_action` exige token, sem query/fragmento, ação ∈ {drag, resize/8 direções} |
| Esquemas `javascript:`, `data:`, `blob:`, `file:`, `about:` passam | `decide_main`/`decide_login` → `Block`; `on_navigation` bloqueia antes (linha 572) |

---

## 10. Pontos não verificados / perguntas em aberto

**Dependem de execução em Windows/WebView2**
1. `cargo test`, `clippy -D warnings`, `fmt --check` com as propostas deste relatório (nada foi compilado).
2. FLY-P2-002: o `tao` 0.37.2 verifica o botão antes de `start_dragging`?
3. FLY-P2-004: `navigation.addEventListener('navigate')` expõe `widget-action://<token>/…`?
4. FLY-P2-015: `beforeunload` do site dispara ao usar a alça/bordas?
5. FLY-P2-014: CSP local bloqueia o `fetch` injetado em `http://tauri.localhost`? Trusted Types/`style-src` estritos quebram o overlay?
6. FLY-P2-020: `DoubleClick` da bandeja é alcançável com menu no clique esquerdo?
7. Chamadas Win32 (`SetWindowRgn`, `DwmSetWindowAttribute`) feitas a partir de *threads* de comando (`apply_config` via IPC): comportamento entre *threads* não verificado.
8. `initialization_script` em Tauri 2.12.5: roda só no quadro principal ou em *iframes*? (a guarda `window.top!==window` torna ambos seguros).
9. Download de `blob:`/anexos: não há `on_download` — comportamento padrão do WebView2 não verificado.
10. Compartilhamento de cookies entre `main` e `login-*` (mesmo `data_directory` `webview-data`) e compatibilidade de argumentos do ambiente WebView2: não verificados.

**Perguntas ao autor**
- Os atalhos Ctrl+S/P/etc. devem ficar invisíveis também para o site (FLY-P2-013)?
- `remember_position=false` deve também ignorar o tamanho (FLY-P2-017)?
- Falha ao criar a bandeja deve abortar o app (FLY-P2-020)?
- Quais IdPs adicionais são aceitáveis (FLY-P2-007) e qual orçamento de aberturas externas (FLY-P2-005)?

---

## 11. Instruções operacionais para a IA corretora

1. Aplicar **um achado por vez**, na ordem do plano (seção 7); após cada um, rodar os comandos pertinentes e só avançar se passarem:
   `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` · `cargo test --manifest-path src-tauri/Cargo.toml --locked` · `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings` · `npm run check:acl` · `npm run check:drag-overlay` (obrigatório em toda edição de `INIT`).
2. Mudar **somente** o que o achado pede; não refatorar por conta própria. **Reler o trecho atual antes de editar** (os números de linha deste relatório mudam).
3. Adicionar o teste sugerido junto com a correção. Comentários e logs em **pt-BR**; textos de interface nos **três idiomas** (`pt-BR`, `en`, `es`), mantendo `INIT`, `LOGIN_INIT` e `i18n.rs` coerentes.
4. **`scripts/check-drag-overlay.mjs` extrai `INIT` por regex** (`const INIT: &str = r#"(.*?)"#;`) e exige, entre outras, estas substrings literais — **não as altere**:
   `border:'4px solid transparent'`, `handle.textContent='⋮⋮'`, `handle.style.setProperty('background'`, `handle.addEventListener('pointerdown',dragWindow)`, `for(const side of ['top','bottom','left','right'])`, `edge.className='__widget_drag_edge __widget_drag_edge--'+side;`, `edge.addEventListener('pointerdown',dragWindow)`, `'pointer-events':'auto'`, `if(!event.isTrusted||event.button!==0)return;`, `widget-action://'+ACTION_TOKEN`, `edge.style.setProperty('pointer-events',locked?'none':'auto')`, `handle.style.setProperty('pointer-events',locked?'none':'auto')`, além das regras CSS `#__widget_drag_frame{…}` (com `border:4px solid transparent`, `box-shadow:none`, `pointer-events:none`), `:hover` e `.__widget_drag_locked`, e a mensagem Rust `Ação de janela ignorada porque mover/redimensionar está travado`. Se uma correção exigir alterar o script, faça-o **em conjunto** e justifique (é Parte 4).
5. Se uma correção exigir alterar `generate_handler!`, `build.rs` ou capabilities, atualizar os três **juntos** e conferir `npm run check:acl`.
6. Mudanças de plataforma: manter `#[cfg(windows)]` e os *stubs* `#[cfg(not(windows))]`; as propostas com Win32 usam `windows-sys` e *features* já habilitadas (`Win32_Foundation`, `Win32_UI_WindowsAndMessaging`, `Win32_UI_Input_KeyboardAndMouse`, `Win32_Graphics_Gdi`, `Win32_Graphics_Dwm`).
7. Nunca incluir segredos; nunca desativar verificações (`check:*`, `clippy -D warnings`) para "fazer passar".
8. Achados marcados **Hipótese** (004, 015, 020) exigem o **teste descrito antes** de qualquer mudança; se refutados, fechar sem alterar código.
9. Ao terminar cada achado, registrar: ID, arquivos alterados, resultado dos comandos e eventual desvio em relação ao proposto.

---

## 12. Prompt de execução sugerido

```text
Você é um(a) engenheiro(a) sênior de Rust/Tauri 2/WebView2 no Windows. Anexo o repositório Floatly e o arquivo
RELATORIO_ANALISE_FLOATLY_PARTE_2.md. Corrija os achados FLY-P2-nnn seguindo EXATAMENTE a seção 7 (plano ordenado)
e a seção 11 (instruções operacionais): um achado por vez, releia o trecho atual antes de editar, mude só o pedido,
adicione o teste sugerido, mantenha comentários/logs em pt-BR e textos de UI em pt-BR/en/es, e rode após cada achado:
cargo fmt --check, cargo test --locked, cargo clippy --all-targets --locked -- -D warnings, npm run check:acl e
npm run check:drag-overlay (sempre que editar o script INIT). Não altere as substrings que o check-drag-overlay.mjs exige.
Pare e me consulte nos itens marcados "decisão humana" (005, 007, 013, 017, 020, 021-a) e execute primeiro o teste das
hipóteses (004, 015, 020) antes de qualquer mudança. Ao final de cada achado informe: ID, arquivos alterados, resultado
dos comandos e desvios. Não refatore nada que não esteja no relatório.
```

---
*Fim do relatório da Parte 2. Interfaces pendentes com outras partes: H1/CSP e capability `main-remote-heartbeat` (Parte 3); `AppState`, `config::save`, `tray::setup(...)?` e watchdog (Parte 1); documentação da limitação de `window.opener` e script `check-drag-overlay.mjs` (Parte 4).*
