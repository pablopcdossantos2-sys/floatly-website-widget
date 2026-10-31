# RELATÓRIO DE ANÁLISE — FLOATLY — **PARTE 1 de 4**

> **Parte trabalhada neste relatório: PARTE 1 — Núcleo Rust (ciclo de vida, configuração, comandos IPC, autostart e diagnóstico).**
> As Partes 2, 3 e 4 **não foram analisadas** aqui; elas aparecem apenas como pontos de interface (seção 3).
> Repositório: `floatly-website-widget-main` (crate/pacote `widget-website` v1.0.0) · Prompt usado: `00_Prompt-Analise-Floatly.md` · Convenção de IDs: `FLY-P1-nnn` (estáveis para consolidação posterior).

---

## 1. Resumo executivo

A Parte 1 está **bem construída**: escrita de configuração atômica (arquivo temporário + `rename` com tentativas), recuperação de `Mutex` envenenado, normalização rigorosa de URL (rejeita esquemas não-HTTP e credenciais embutidas), logs com rotação e limite de taxa, e testes inline relevantes. Não foram encontradas falhas **Críticas** nem **Altas**. Os problemas são de robustez e de experiência: o mais relevante é que um conflito **temporário** de atalho no boot **apaga permanentemente** o atalho escolhido pelo usuário.

**Nota de risco da Parte 1:** **Médio-Baixo**.

| Severidade | Qtde | IDs |
|---|---|---|
| Crítica | 0 | — |
| Alta | 0 | — |
| Média | 4 | FLY-P1-001, 002, 003, 004 |
| Baixa | 10 | FLY-P1-005 a 014 |
| Info | 0 | — |

**Os 5 achados mais importantes**
1. **FLY-P1-001** — Atalho preferido do usuário é sobrescrito e **persistido** com o atalho de fallback após um conflito momentâneo no boot.
2. **FLY-P1-002** — `normalize()` aceita atalhos globais reservados (ex.: `CTRL+C`), que sequestrariam copiar/colar no Windows inteiro.
3. **FLY-P1-003** — Comandos IPC síncronos (`get_config`/`save_config`) rodam na thread principal fazendo E/S de disco e chamadas WinRT bloqueantes (`join()`).
4. **FLY-P1-004** — Encerramento de sessão (logoff/desligamento) pode ser tratado como "falha fatal" (mensagem de erro + `exit(1)`).
5. **FLY-P1-005** — Apagar a URL nas Configurações não tira a janela principal do site anterior.

---

## 2. Escopo, premissas e limitações

**Lido integralmente (Parte 1):** `src-tauri/src/main.rs` (305 linhas), `config.rs` (703), `commands.rs` (439), `diagnostics.rs` (571), `autostart.rs` (85), `src-tauri/Cargo.toml`, `build.rs`. Também consultei, **só para entender as interfaces**: trechos de `window.rs` (script `INIT`, `create_main`, uso de `shutdown_requested`), `tray.rs`, `settings.ts` (captura de atalho), `capabilities/*.json`, `tauri.conf.json`, `hooks.nsh`, `Cargo.lock` (versões).

**Executado:**
- `node scripts/check-acl.mjs` → passou.
- `node scripts/check-tauri-versions.mjs` → passou (API/CLI 2.12.4 · crate `tauri` 2.12.5 · `tauri-build` 2.7.4).

**NÃO executado (e por quê):**
- `cargo fmt/test/clippy`, `cargo audit`: **o ambiente de análise não tem Rust instalado** e o alvo é Windows (`x86_64-pc-windows-msvc`). Os testes Rust foram **lidos**, não executados.
- `npm ci`, `typecheck`, `build`, `npm audit`: sem rede / fora do escopo da Parte 1.
- Nenhum comportamento de runtime (WebView2, WinRT, hotkeys, MSIX) foi observado; todos os achados vêm de leitura estática. Por isso cada achado traz **Confiança** explícita.

**Premissas:** Tauri 2.x — comandos **síncronos** (`fn` sem `async`) são executados na **thread principal** (comportamento documentado do Tauri 2). Rust 1.90, edition 2021.

---

## 3. Mapa das 4 partes e a Parte 1

| Nº | Parte | Arquivos | Linhas aprox. | Situação neste relatório |
|---|---|---|---|---|
| **1** | **Núcleo Rust: ciclo de vida, configuração, comandos IPC, autostart e diagnóstico** | `src-tauri/src/main.rs`, `config.rs`, `commands.rs`, `diagnostics.rs`, `autostart.rs`, `src-tauri/Cargo.toml`, `src-tauri/build.rs` | ≈ 2.150 | ✅ **ANALISADA** |
| 2 | Janela, navegação, login e script injetado | `src-tauri/src/window.rs`, `tray.rs`, `i18n.rs` | ≈ 1.960 | ⏳ não analisada |
| 3 | Interface local (frontend), IPC e superfície de segurança | `index.html`, `settings.html`, `src/main.ts`, `src/settings.ts`, `src/styles.css`, `vite.config.ts`, `tsconfig.json`, `package.json` (scripts/runtime), `tauri.conf.json` (seção `app`/`security`), `capabilities/*.json` | ≈ 700 | ⏳ não analisada |
| 4 | Build, CI/CD, empacotamento, dependências, documentação e distribuição | `.github/workflows/*`, `.github/dependabot.yml`, `scripts/*`, `packaging/msix/*`, `src-tauri/windows/hooks.nsh`, `tauri.conf.json` (seção `bundle`), `Cargo.lock`/`package-lock.json`, `rust-toolchain.toml`, `.gitignore`, `README*`, `PRIVACY_POLICY.md`, `SUPPORT.md`, `GUIA_PUBLICACAO_*`, `MICROSOFT_STORE_*` | ≈ 1.000 + docs | ⏳ não analisada |

**Interfaces da Parte 1 com as demais partes (pendentes de análise nelas):**

| Interface | Parte 1 | Outra parte |
|---|---|---|
| Chaves de erro `config.*` e avisos (`shortcut_conflict`, `autostart_*_failed`) | `config.rs`, `commands.rs` | **Parte 3** (`settings.ts` traduz/mostra) |
| `AppState`, `shutdown_requested`, `registered_shortcut` | `config.rs`, `main.rs` | **Parte 2** (`window.rs`, `tray.rs`) |
| `heartbeat` / `HEARTBEAT_TOKEN` | `commands.rs`, `diagnostics.rs` | **Parte 2** (script injetado) e **Parte 3** (capability remota) |
| `app_directory()`, nome `WidgetWebsite`, entrada `Run` do autostart | `config.rs`, `autostart.rs` | **Parte 4** (`hooks.nsh`, MSIX, política de privacidade) |
| Evento `config-changed` | `commands.rs` | **Parte 3** (`settings.ts`) |

---

## 4. Índice de achados (Parte 1)

| ID | Título | Sev. | Confiança | Esforço | Depende de |
|---|---|---|---|---|---|
| FLY-P1-001 | Atalho preferido sobrescrito e persistido pelo fallback após conflito temporário | Média | Confirmado | P | — |
| FLY-P1-002 | `normalize()` aceita atalhos globais reservados do sistema (ex.: `CTRL+C`) | Média | Confirmado | M | — (toca Parte 3) |
| FLY-P1-003 | Comandos IPC síncronos fazem E/S e WinRT bloqueante na thread principal | Média | Provável | M | FLY-P1-006 |
| FLY-P1-004 | Logoff/desligamento pode ser tratado como falha fatal da janela | Média | Hipótese | P | — |
| FLY-P1-005 | Limpar `site_url` não navega a janela principal | Baixa | Confirmado | P | — |
| FLY-P1-006 | `registered_shortcut` não recupera `Mutex` envenenado | Baixa | Confirmado | P | — |
| FLY-P1-007 | Logs de navegação gravam o caminho completo da URL (pode conter segredos) | Baixa | Confirmado | P | — |
| FLY-P1-008 | Campos inválidos são substituídos e persistidos sem backup do original | Baixa | Confirmado | M | — |
| FLY-P1-009 | `app_directory()` recorre a `current_dir()`; criação do diretório sem fallback | Baixa | Confirmado | P | — |
| FLY-P1-010 | Watchdog emite avisos falsos com a janela minimizada | Baixa | Provável | P | — |
| FLY-P1-011 | `config-changed` é transmitido a todos os webviews (inclui o site remoto) | Baixa | Provável | P | — (verificar Parte 3) |
| FLY-P1-012 | Nome da entrada `Run` do autostart pode divergir do que o desinstalador remove | Baixa | Hipótese | P | — (afeta Parte 4) |
| FLY-P1-013 | MSIX: `set_enabled(false)` devolve `false` sem reler o estado real | Baixa | Confirmado | P | — |
| FLY-P1-014 | Pequenas melhorias agrupadas (temporários órfãos, testes frágeis, redundâncias) | Baixa | Confirmado | M | — |

---

## 5. Achados detalhados — PARTE 1

### FLY-P1-001 — Atalho preferido sobrescrito e persistido pelo fallback após conflito temporário

| Campo | Valor |
|---|---|
| Parte | 1 (interface com 3: tela de Configurações) |
| Severidade | Média |
| Categoria | Corretude / Robustez |
| Confiança | Confirmado (leitura do código) |
| Esforço | P (<30 min) |
| Risco de regressão | Baixo |
| Local | `src-tauri/src/main.rs:183-218` (principalmente 191 e 205-217) |
| Depende de | — |

**Evidência**
```rust
// main.rs ~191
if active_shortcut.is_none() && shortcut_text != "CTRL+ALT+C" { /* tenta registrar CTRL+ALT+C */ }
...
// main.rs ~205-217
if let Some(active) = active_shortcut {
    if cfg.global_shortcut != active {
        cfg.global_shortcut = active.clone();
        *app.state::<AppState>().lock_config() = cfg.clone();
        if safe_to_persist {
            if let Err(error) = config::save(&dir, &cfg) { /* log */ }
        }
    }
}
```

**Descrição e causa raiz.** Se o atalho salvo não puder ser registrado no boot (por exemplo, outro programa está segurando a combinação **naquele momento**), o app ativa `CTRL+ALT+C` e **grava esse valor no `config.json`**. A preferência do usuário é descartada de forma permanente e silenciosa (só há uma linha de log `WARN`).

**Impacto / reprodução.** (1) Configurar `CTRL+ALT+K` nas Configurações. (2) Fechar o Floatly. (3) Abrir outro app que registre `CTRL+ALT+K` e **iniciar o Floatly**. (4) Fechar o outro app e reiniciar o Floatly: o atalho agora é `CTRL+ALT+C`; o `K` nunca mais volta sem o usuário reconfigurar.

**Solução proposta** (mantém o atalho **em uso** só em memória, como já faz `registered_shortcut`, e preserva o preferido em disco)
1. Em `main.rs`, remover o bloco que altera `cfg.global_shortcut` e chama `config::save`; substituir por um log informativo.
2. Remover o `mut` de `cfg` se deixar de ser mutado (senão `clippy -D warnings` falha com `unused_mut`).
3. Nada muda em `save_config`: como `c.global_shortcut != active_shortcut`, a próxima gravação das Configurações tentará registrar o preferido de novo (reativação natural).

```diff
-            if let Some(active) = active_shortcut {
-                if cfg.global_shortcut != active {
-                    cfg.global_shortcut = active.clone();
-                    *app.state::<AppState>().lock_config() = cfg.clone();
-                    if safe_to_persist {
-                        if let Err(error) = config::save(&dir, &cfg) {
-                            diagnostics::log(&dir, "ERRO", format!("Não foi possível salvar o atalho alternativo ativo: {error}"));
-                        }
-                    }
-                }
-            }
+            if let Some(active) = &active_shortcut {
+                if cfg.global_shortcut != *active {
+                    diagnostics::log(
+                        &dir,
+                        "WARN",
+                        format!(
+                            "O atalho preferido {} está indisponível; {} será usado nesta sessão sem alterar o config.json.",
+                            cfg.global_shortcut, active
+                        ),
+                    );
+                }
+            }
```
*(O trecho acima é um esboço: ajuste nomes/indentação ao arquivo real e confirme que `active_shortcut` ainda é usado depois para preencher `registered_shortcut`, o que já ocorre antes.)*

**Arquivos a alterar:** `src-tauri/src/main.rs`.

**Critérios de aceite**
- Com o atalho salvo indisponível no boot, o `config.json` **mantém** o atalho preferido após o encerramento.
- O app continua utilizável via `CTRL+ALT+C` na sessão.
- `cargo clippy ... -D warnings` e `cargo test` passam.

**Como verificar.** Manual no Windows (passos da reprodução acima) + checar `%APPDATA%\WidgetWebsite\config.json` e `diagnostics.log`.

**Cuidados.** Na Parte 3, a tela de Configurações mostrará o atalho **preferido** (não o ativo). Se o atalho preferido continuar indisponível, `save_config` devolverá o aviso `shortcut_conflict` a cada gravação — comportamento aceitável e informativo. Opcional (fora do mínimo): expor o atalho **ativo** ao frontend (nova estrutura de resposta) — exige mudança coordenada com a Parte 3.

---

### FLY-P1-002 — `normalize()` aceita atalhos globais reservados do sistema

| Campo | Valor |
|---|---|
| Parte | 1 (e 3: `settings.ts` e traduções) |
| Severidade | Média |
| Categoria | Segurança de uso / Corretude |
| Confiança | Confirmado |
| Esforço | M (até 2 h) |
| Risco de regressão | Médio (configurações já salvas com combinações reservadas serão redefinidas) |
| Local | `src-tauri/src/config.rs:105-148`; captura no frontend `src/settings.ts:~304-327` |
| Depende de | — |

**Evidência**
```rust
// config.rs ~121-129: exige ao menos UM entre CTRL/ALT/SUPER
if !modifiers.iter().any(|m| matches!(*m, "CTRL" | "ALT" | "SUPER")) {
    return Err("config.shortcut_modifier_required".into());
}
```

**Descrição e causa raiz.** A regra aceita `CTRL+C`, `CTRL+V`, `ALT+F4`, `ALT+TAB` etc. Registrado como atalho **global**, `CTRL+C` passa a ser consumido pelo Floatly e **deixa de copiar em todos os programas** (idem colar/recortar/desfazer). O frontend também não filtra (apenas exige modificador e tecla conhecida).

**Impacto / reprodução.** Nas Configurações, focar o campo de atalho e pressionar `Ctrl+C` → salvar. A combinação é aceita e registrada; copiar passa a alternar a janela do Floatly. Usuário comum não associa o problema ao app.

**Solução proposta**
1. Em `config.rs`, adicionar uma lista de combinações reservadas (comparação após a canonicalização) e devolver uma nova chave de erro `config.shortcut_reserved`.
2. A chave começa com `config.shortcut`, então o laço de `load_with_warnings` (linhas ~309-313) já a trata e volta ao padrão.
3. **Parte 3 (coordenar):** adicionar a tradução de `config.shortcut_reserved` nos 3 idiomas em `settings.ts` (e onde mais as chaves `config.*` forem mapeadas).

```diff
+const RESERVED_SHORTCUTS: &[&str] = &[
+    "CTRL+C", "CTRL+V", "CTRL+X", "CTRL+Z", "CTRL+Y", "CTRL+A", "CTRL+S",
+    "CTRL+F", "CTRL+P", "CTRL+W", "CTRL+T", "CTRL+N",
+    "CTRL+ESC", "CTRL+ESCAPE", "CTRL+SHIFT+ESC", "CTRL+SHIFT+ESCAPE",
+    "ALT+F4", "ALT+TAB", "ALT+ESC", "ALT+ESCAPE", "ALT+SPACE",
+    "SUPER+L", "SUPER+D", "SUPER+E", "SUPER+R", "SUPER+TAB",
+];
 ...
         let shortcut = format!("{}+{}", modifiers.join("+"), key);
+        if RESERVED_SHORTCUTS.contains(&shortcut.as_str()) {
+            return Err("config.shortcut_reserved".into());
+        }
         Shortcut::from_str(&shortcut).map_err(|_| "config.shortcut_key_unsupported".to_owned())?;
```

**Arquivos a alterar:** `src-tauri/src/config.rs` (+ tradução em `src/settings.ts`, Parte 3).

**Critérios de aceite**
- `normalize()` retorna `Err("config.shortcut_reserved")` para `CTRL+C`, `ALT+F4`, `CTRL+SHIFT+ESC` e `SUPER+L`; continua aceitando `CTRL+ALT+C`.
- A mensagem aparece traduzida (pt-BR/en/es) ao tentar salvar.

**Como verificar.** Novo teste em `config.rs`:
```rust
#[test]
fn normalization_rejects_reserved_system_shortcuts() {
    for s in ["CTRL+C", "ctrl+v", "ALT+F4", "CTRL+SHIFT+ESC", "SUPER+L"] {
        let mut c = AppConfig { global_shortcut: s.into(), ..AppConfig::default() };
        assert_eq!(c.normalize().unwrap_err(), "config.shortcut_reserved", "{s}");
    }
    let mut ok = AppConfig { global_shortcut: "CTRL+ALT+C".into(), ..AppConfig::default() };
    assert!(ok.normalize().is_ok());
}
```
**Cuidados.** Confirmar os nomes de tecla aceitos por `global-hotkey` 0.8 (`ESC` vs `ESCAPE`): a lista cobre ambos. Quem já tem uma combinação reservada salva terá o atalho redefinido para `CTRL+ALT+C` no próximo boot (com aviso no log) — aceitável.

---

### FLY-P1-003 — Comandos IPC síncronos bloqueiam a thread principal (disco + WinRT)

| Campo | Valor |
|---|---|
| Parte | 1 (interface com 2: `apply_config`, e 3: chamadas do frontend) |
| Severidade | Média |
| Categoria | Desempenho / Robustez |
| Confiança | Provável (depende do comportamento do Tauri 2 e do WinRT em MSIX) |
| Esforço | M |
| Risco de regressão | Médio (passa a haver concorrência real entre comandos) |
| Local | `commands.rs:122-345` (`heartbeat`, `get_config`, `save_config`); `autostart.rs:24` e `:62` (`operation.join()`) |
| Depende de | FLY-P1-006 |

**Evidência**
```rust
#[tauri::command]
pub fn get_config(app: AppHandle, state: State<'_, AppState>) -> AppConfig {
    let mut c = state.snapshot();
    if let Ok(enabled) = crate::autostart::is_enabled(&app) { c.autostart = enabled; }
    c
}
// autostart.rs:24
let task = operation.join().map_err(/* ... */)?;   // bloqueia até o WinRT concluir
```

**Descrição e causa raiz.** No Tauri 2, comandos sem `async` executam na **thread principal**. `get_config` (chamado em **todo** boot pela página inicial e ao abrir Configurações) e `save_config` fazem: bloqueio do `Mutex` da configuração, `fs::sync_all` + `rename` com até 3 tentativas e `sleep` de 50 ms (`config.rs:360-405`), registro/remoção de atalho, e — no pacote **MSIX** — `StartupTask::GetAsync(...).join()` e `RequestEnableAsync(...).join()`, que podem demorar (inclusive exibir diálogo de consentimento do sistema). Enquanto isso, a UI de todas as janelas fica sem processar mensagens. O próprio `main.rs:160-163` reconhece o risco ("evitar bloquear a thread de setup no WinRT"), mas o mesmo risco permanece nos comandos.

**Impacto.** Congelamento perceptível da janela ao abrir/salvar Configurações (antivírus lento, perfil em rede, WinRT demorado); no pior caso a janela aparece "Não respondendo" e o watchdog/Windows interpretam como travamento.

**Solução proposta**
1. Marcar `get_config`, `save_config` e `heartbeat` como `#[tauri::command(async)]` (executam em thread do pool).
2. Evitar parâmetros emprestados (`State<'_, …>`): receber `AppHandle` e obter `app.state::<AppState>()` dentro — evita o erro "async commands with references must return Result".
3. Serializar `save_config` com um `Mutex` estático, pois agora comandos podem rodar em paralelo.
4. (Opcional) Armazenar em cache por alguns segundos o resultado de `autostart::is_enabled` em `get_config`.

```diff
+static SAVE_CONFIG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
-#[tauri::command]
-pub fn get_config(app: AppHandle, state: State<'_, AppState>) -> AppConfig {
-    let mut c = state.snapshot();
+#[tauri::command(async)]
+pub fn get_config(app: AppHandle) -> AppConfig {
+    let mut c = app.state::<AppState>().snapshot();
     ...
-#[tauri::command]
-pub fn save_config(app: AppHandle, state: State<'_, AppState>, config: ConfigPatch, force_resize: Option<bool>) -> Result<SaveResult, String> {
+#[tauri::command(async)]
+pub fn save_config(app: AppHandle, config: ConfigPatch, force_resize: Option<bool>) -> Result<SaveResult, String> {
+    let _serialize = SAVE_CONFIG_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
+    let state = app.state::<AppState>();
     let patch = config;
```
*(Esboço: adapte as demais referências a `state` — passam a ser `&state`/`state.` normalmente — e o `heartbeat` da mesma forma.)*

**Arquivos a alterar:** `src-tauri/src/commands.rs`.

**Critérios de aceite**
- Compila sem warnings (`cargo clippy -D warnings`); `npm run check:acl` continua passando (nomes dos comandos inalterados).
- Em Windows: abrir/salvar Configurações com disco lento (simular com antivírus/`Process Monitor`) **não** congela a janela principal.
- Duas gravações simultâneas não corrompem `config.json` nem o estado do atalho.

**Como verificar.** `cargo test`; teste manual em MSIX e NSIS; log de `diagnostics.log` sem avisos de watchdog durante salvamento.

**Cuidados.** `apply_config`, `settings.set_title` e `main.navigate` passam a ser chamados fora da thread principal — o Tauri os encaminha à thread principal, mas **não** segure o `lock_config()` durante essas chamadas (hoje já não segura). Não alterar os nomes dos comandos (`build.rs`/capabilities dependem deles).

---

### FLY-P1-004 — Logoff/desligamento pode ser tratado como falha fatal da janela

| Campo | Valor |
|---|---|
| Parte | 1 (interface com 2) |
| Severidade | Média |
| Categoria | Robustez / UX |
| Confiança | **Hipótese** (precisa ser reproduzido no Windows) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `src-tauri/src/main.rs:264-277` |
| Depende de | — |

**Evidência**
```rust
WindowEvent::Destroyed => {
    window::flush_pending_config(&handle);
    if handle.state::<AppState>().shutdown_requested.load(Ordering::SeqCst) { /* INFO */ }
    else {
        diagnostics::log(&event_diag_dir, "FATAL", "A janela principal foi destruída inesperadamente; ...");
        #[cfg(windows)] show_message("Floatly — falha da janela", /* ... */);
        handle.exit(1);
    }
}
```

**Descrição e causa raiz.** `shutdown_requested` só é ligado em `quit_app` (menu da bandeja, `commands.rs:378-383`). Qualquer outro caminho legítimo que destrua a janela principal — encerramento de sessão do Windows, reinício para atualização, "Finalizar tarefa" gracioso — cai no ramo "inesperado", com `MessageBox` de erro e código de saída 1.

**Impacto.** Possível caixa de erro "A janela principal foi encerrada inesperadamente" no logoff/desligamento e entradas `FATAL` falsas no log, o que mina a confiança no diagnóstico.

**Solução proposta.** Tratar "sessão encerrando" como encerramento legítimo, usando `GetSystemMetrics(SM_SHUTTINGDOWN)` (a feature `Win32_UI_WindowsAndMessaging` do `windows-sys` já está habilitada em `Cargo.toml`).
```diff
+#[cfg(windows)]
+fn session_is_ending() -> bool {
+    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SHUTTINGDOWN};
+    unsafe { GetSystemMetrics(SM_SHUTTINGDOWN) != 0 }
+}
+#[cfg(not(windows))]
+fn session_is_ending() -> bool { false }
 ...
-                    if handle.state::<AppState>().shutdown_requested.load(std::sync::atomic::Ordering::SeqCst) {
+                    if handle.state::<AppState>().shutdown_requested.load(std::sync::atomic::Ordering::SeqCst)
+                        || session_is_ending()
+                    {
```
**Alternativa (maior mudança, opcional):** trocar `.run(generate_context!())` por `.build(...)?.run(|app, event| …)` e marcar `shutdown_requested` em `RunEvent::ExitRequested`/`Exit`. Exige refatorar o tratamento do erro final do `main()`.

**Arquivos a alterar:** `src-tauri/src/main.rs`.

**Critérios de aceite.** Em um Windows de teste: encerrar a sessão com o Floatly aberto **não** exibe o `MessageBox` e o log registra `INFO` (não `FATAL`); fechar a janela pela bandeja continua igual; destruição realmente inesperada continua gerando `FATAL`.

**Como verificar.** Manual (logoff com o app aberto) + leitura do `diagnostics.log`.

**Cuidados.** Se a reprodução **não** mostrar o problema (o tao/Tauri pode finalizar o processo antes de entregar o evento), registrar o achado como descartado.

---

### FLY-P1-005 — Limpar `site_url` não navega a janela principal

| Campo | Valor |
|---|---|
| Parte | 1 (interface com 2 e 3) |
| Severidade | Baixa |
| Categoria | Corretude / UX |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `src-tauri/src/commands.rs:~326-337` |
| Depende de | — |

**Evidência**
```rust
if old.site_url != c.site_url {
    if let (Some(main), Ok(url)) = (app.get_webview_window("main"), url::Url::parse(&c.site_url)) {
        if let Err(error) = main.navigate(url) { /* log */ }
    }
}
```
**Descrição.** Se o usuário apaga a URL (`site_url` vazio é permitido), `Url::parse("")` falha e nenhuma navegação ocorre: a janela continua exibindo o site antigo até reiniciar, embora a configuração diga "nenhum site".

**Solução proposta.** Quando `c.site_url.is_empty()`, navegar a janela principal para a página local inicial (a mesma URL que `window::create_main` usa quando não há site; localizar em `window.rs:488-612`, extrair para uma função `local_start_url(app) -> Url` e reutilizar aqui).

**Arquivos a alterar:** `commands.rs` (+ pequena extração em `window.rs`, Parte 2).

**Critérios de aceite.** Salvar com URL vazia leva a janela principal à tela "Escolha um site"; salvar com nova URL continua navegando.

**Como verificar.** Manual (Configurações → apagar URL → salvar).

**Cuidados.** Em desenvolvimento a URL local é `devUrl` (`http://127.0.0.1:1420`); em release, `tauri://`/`http://tauri.localhost` — usar a mesma lógica já presente em `create_main`.

---

### FLY-P1-006 — `registered_shortcut` não recupera `Mutex` envenenado

| Campo | Valor |
|---|---|
| Parte | 1 (confirma H7) |
| Severidade | Baixa |
| Categoria | Robustez / Concorrência |
| Confiança | Confirmado (probabilidade de ocorrer é muito baixa) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `commands.rs:153-155`, `:213`, `:301`; `main.rs:203`; definição `config.rs:158` |
| Depende de | — |

**Evidência**
```rust
let active_shortcut = state.registered_shortcut.lock()
    .map_err(|_| "config.write_failed".to_owned())?   // commands.rs:153-155
    .clone();
```
**Descrição.** `config` usa `lock_config()` com recuperação de *poison*; `registered_shortcut` não. Um *panic* com esse lock preso faria **todo** `save_config` falhar com `config.write_failed` até reiniciar (e as atualizações com `if let Ok(...)` seriam ignoradas silenciosamente).

**Solução proposta.** Adicionar um helper em `AppState` e usá-lo nos 4 pontos.
```diff
 impl AppState {
+    pub fn lock_registered_shortcut(&self) -> MutexGuard<'_, String> {
+        self.registered_shortcut
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+    }
```
```diff
-    let active_shortcut = state.registered_shortcut.lock().map_err(|_| "config.write_failed".to_owned())?.clone();
+    let active_shortcut = state.lock_registered_shortcut().clone();
-    if let Ok(mut registered) = state.registered_shortcut.lock() { *registered = ...; }
+    *state.lock_registered_shortcut() = ...;
```
**Critérios de aceite.** Teste novo análogo a `snapshot_recovers_config_from_a_poisoned_mutex` (`config.rs:~655`) que envenena `registered_shortcut` e verifica que `lock_registered_shortcut()` ainda devolve o valor.

**Cuidados.** O construtor de teste em `config.rs:~666` já inicializa o campo; não alterar a struct.

---

### FLY-P1-007 — Logs de navegação gravam o caminho completo da URL

| Campo | Valor |
|---|---|
| Parte | 1 (afeta 2: `log_navigation`; e 4: política de privacidade) |
| Severidade | Baixa |
| Categoria | Privacidade |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `diagnostics.rs:209-228` (`sanitize_url`), `:244-250` (`log_navigation`), `:252-257` (`safe_url_summary`) |
| Depende de | — |

**Descrição.** `sanitize_url` remove credenciais, *query* e *fragment*, mas mantém até 96 caracteres do **caminho**. Muitos sites colocam segredos no caminho (`/reset/<token>`, `/invite/<id>`, `/share/<hash>`), que ficam em `%APPDATA%\WidgetWebsite\diagnostics.log` (até ~4 MB com backups).

**Solução proposta.** Manter só os 2 primeiros segmentos e mascarar segmentos que "parecem identificadores" (≥ 8 caracteres contendo dígito, ou ≥ 20 caracteres).
```diff
-    let path: String = url.path().chars().take(96).collect();
+    let path = redact_path(url.path());
+fn redact_path(path: &str) -> String {
+    let segs: Vec<String> = path.split('/').filter(|s| !s.is_empty()).take(2)
+        .map(|s| if s.len() >= 20 || (s.len() >= 8 && s.chars().any(|c| c.is_ascii_digit())) { "…".into() } else { s.to_owned() })
+        .collect();
+    if segs.is_empty() { "/".into() } else { format!("/{}", segs.join("/")) }
+}
```
**Critérios de aceite.** Os testes existentes (`configured_url_summary_omits_query_and_fragment`, `navigation_log_omits_query_and_fragment`) continuam passando (`path=/login`); novo teste: `https://x.com/reset/ab12cd34ef56?t=1` → `path=/reset/…`.

**Cuidados.** Coordenar a redação da `PRIVACY_POLICY.md` com o novo comportamento (Parte 4).

---

### FLY-P1-008 — Campos inválidos são substituídos e persistidos sem backup do original

| Campo | Valor |
|---|---|
| Parte | 1 |
| Severidade | Baixa |
| Categoria | Robustez / Perda de dados |
| Confiança | Confirmado |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | `config.rs:217-358` (`load_with_warnings`), `main.rs:158-170` |
| Depende de | — |

**Descrição.** Só o caso "JSON inválido" gera backup (`config.json.bad-…`, `config.rs:~235-250`). Quando o JSON é válido mas há campos inválidos (ex.: `site_url` digitado errado, `global_shortcut` corrompido), o valor é trocado pelo padrão e `main.rs` regrava o arquivo (`!load_warnings.is_empty()`), perdendo o original sem rastro além do log.

**Solução proposta.** Antes de regravar por causa de *warnings*, copiar `config.json` para `config.json.repaired-<timestamp>` e manter no máximo 3 cópias.
```diff
+pub fn backup_before_repair(dir: &Path) {
+    let src = dir.join("config.json");
+    if !src.exists() { return; }
+    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
+    let _ = fs::copy(&src, dir.join(format!("config.json.repaired-{ts}")));
+    // podar: manter as 3 mais recentes (mesma ideia de prune_log_backups)
+}
```
Chamar em `main.rs` imediatamente antes de `config::save` quando `config_file_existed && !load_warnings.is_empty()`.

**Critérios de aceite.** Teste: gravar `config.json` com `site_url` inválido, carregar+reparar → existe um `config.json.repaired-*` com o conteúdo original e o `config.json` novo é válido.

---

### FLY-P1-009 — `app_directory()` recorre a `current_dir()`; criação do diretório sem fallback

| Campo | Valor |
|---|---|
| Parte | 1 (interface com 4: `hooks.nsh`, MSIX; confirma H8 e parte de H14) |
| Severidade | Baixa |
| Categoria | Robustez |
| Confiança | Confirmado (cenário raro) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `config.rs:203-208`; `main.rs:153-154` |

**Descrição.** Sem `APPDATA`, o app grava config/logs/`webview-data` no **diretório atual** (imprevisível e muitas vezes sem permissão, ex.: `C:\Windows\System32` ao iniciar por tarefa agendada). Se `create_dir_all` falhar (`main.rs:153`), o `?` aborta o `setup` e o app não abre, sem alternativa.

**Solução proposta.**
```diff
-pub fn app_directory() -> PathBuf {
-    std::env::var_os("APPDATA")
-        .map(PathBuf::from)
-        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
-        .join("WidgetWebsite")
-}
+const APP_DIR_NAME: &str = "WidgetWebsite"; // NÃO renomear sem migração (hooks.nsh e dados dos usuários dependem)
+pub fn app_directory() -> PathBuf {
+    std::env::var_os("APPDATA")
+        .or_else(|| std::env::var_os("LOCALAPPDATA"))
+        .map(PathBuf::from)
+        .unwrap_or_else(std::env::temp_dir)
+        .join(APP_DIR_NAME)
+}
```
**Critérios de aceite.** Com `APPDATA` removido do ambiente, o app usa `LOCALAPPDATA` ou o diretório temporário, nunca o CWD. Os testes existentes continuam passando.

**Cuidados.** **Não** renomear `WidgetWebsite` para `Floatly` neste momento (exigiria migração e ajuste do `hooks.nsh`, Parte 4); apenas centralizar o nome na constante.

---

### FLY-P1-010 — Watchdog emite avisos falsos com a janela minimizada

| Campo | Valor |
|---|---|
| Parte | 1 (interface com 2: script injetado de heartbeat) |
| Severidade | Baixa |
| Categoria | Diagnóstico / Ruído |
| Confiança | Provável |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `diagnostics.rs:316-350` (laço) e `:337` (`NATIVE_WINDOW_VISIBLE`) |

**Descrição.** O laço só ignora o heartbeat quando a janela foi **ocultada** pelo app (`set_window_visible(false)`, chamado em `main.rs:246` e `window.rs:1175`). Se o usuário **minimiza** a janela, o WebView2 pode reduzir a frequência dos temporizadores da página (`setInterval` de 5 s no script injetado, `window.rs:174`), ultrapassando os 20 s de `HEARTBEAT_TIMEOUT_MS` e gerando `WARN` a cada 60 s ("possível página travada") sem haver problema.

**Solução proposta.** No laço, pular (e reiniciar a carência) quando a janela nativa estiver minimizada.
```diff
+#[cfg(windows)]
+fn native_window_minimized() -> bool {
+    use windows_sys::Win32::UI::WindowsAndMessaging::IsIconic;
+    let raw = NATIVE_WINDOW_HANDLE.load(Ordering::SeqCst);
+    raw != 0 && unsafe { IsIconic(raw as windows_sys::Win32::Foundation::HWND) != 0 }
+}
+#[cfg(not(windows))]
+fn native_window_minimized() -> bool { false }
 ...
-            if !NATIVE_WINDOW_VISIBLE.load(Ordering::SeqCst) {
+            if !NATIVE_WINDOW_VISIBLE.load(Ordering::SeqCst) || native_window_minimized() {
                 reset_grace();
                 continue;
             }
```
**Critérios de aceite.** Janela minimizada por >5 min não gera `WARN` do watchdog; janela visível travada de fato continua gerando.

**Cuidados.** Confirmar em teste manual que a premissa (heartbeat espaçado quando minimizada) realmente ocorre; se não ocorrer, registrar como descartado.

---

### FLY-P1-011 — `config-changed` é transmitido a todos os webviews

| Campo | Valor |
|---|---|
| Parte | 1 (verificar em 3: quem escuta o evento) |
| Severidade | Baixa |
| Categoria | Segurança (endurecimento) |
| Confiança | Provável |
| Esforço | P |
| Risco de regressão | Baixo/Médio (se algum listener existir fora de `settings`) |
| Local | `commands.rs` (chamadas `app.emit("config-changed", …)` em `save_config`, `toggle_locked`, `toggle_topmost`) |

**Descrição.** `app.emit` envia a configuração completa (incluindo `site_url` e atalho) a **todos** os webviews, inclusive o que carrega o site remoto. A ACL atual (`main-remote-heartbeat` só permite `heartbeat`) impede que o site escute o evento — hoje é mitigado —, mas basta uma permissão de eventos ser ampliada no futuro para vazar a configuração.

**Solução proposta.** Trocar por `app.emit_to("settings", "config-changed", &c)`.

**Critérios de aceite.** A janela de Configurações continua refletindo alterações vindas da bandeja (travar/sempre no topo); nenhum outro webview recebe o evento.

**Cuidados.** **Antes** de alterar, confirmar na Parte 3 (`settings.ts`/`main.ts`) e na Parte 2 (script injetado) quem usa `listen("config-changed")`; se a janela principal local também escutar, usar duas chamadas `emit_to`.

---

### FLY-P1-012 — Nome da entrada `Run` do autostart pode divergir do que o desinstalador remove

| Campo | Valor |
|---|---|
| Parte | 1 (afeta **Parte 4**: `hooks.nsh`) — confirma parte de H14 |
| Severidade | Baixa |
| Categoria | Consistência / Instalação |
| Confiança | **Hipótese** |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `main.rs:~113-116` (`tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None)`); `src-tauri/windows/hooks.nsh:8-9` |

**Descrição.** O `hooks.nsh` assume que o plugin grava `HKCU\…\Run\widget-website` (nome do crate) e remove também `Widget Website`. Se `tauri-plugin-autostart` 2.7.0 usar o **nome do produto** (`Floatly`, de `productName`), a entrada `Floatly` **permaneceria** após desinstalar. Não consegui confirmar no código do plugin (sem acesso ao registro de crates).

**Como verificar.** Instalar o NSIS em uma VM, ativar "Iniciar com o Windows", ler `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, desinstalar e conferir se a entrada sumiu.

**Solução proposta (se confirmado).** Acrescentar no `hooks.nsh` `DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Floatly"` (Parte 4) **ou** fixar o nome com `AutoLaunchBuilder`/argumento do plugin. Não alterar o nome do crate.

---

### FLY-P1-013 — MSIX: `set_enabled(false)` devolve `false` sem reler o estado real

| Campo | Valor |
|---|---|
| Parte | 1 |
| Severidade | Baixa |
| Categoria | Corretude |
| Confiança | Confirmado (cenário depende de política do SO) |
| Esforço | P |
| Local | `src-tauri/src/autostart.rs:66-69` |

**Evidência**
```rust
task.Disable().map_err(/* ... */)?;
return Ok(false);
```
**Descrição.** Em `EnabledByPolicy` (política de grupo), `Disable()` não tem efeito, mas a função informa `false`; as Configurações mostrariam "desativado" enquanto o app continua iniciando com o Windows. O ramo `enabled == true` já relê o estado devolvido pelo SO.

**Solução proposta.**
```diff
-        task.Disable().map_err(|error| format!("Não foi possível desativar a inicialização MSIX: {error}"))?;
-        return Ok(false);
+        task.Disable().map_err(|error| format!("Não foi possível desativar a inicialização MSIX: {error}"))?;
+        let state = task.State().map_err(|error| format!("Não foi possível confirmar a inicialização MSIX: {error}"))?;
+        return Ok(startup_state_enabled(state));
```
**Critérios de aceite.** Em `EnabledByPolicy`, `set_enabled(false)` devolve `true`, fazendo `save_config` emitir `autostart_disable_failed`. Em estado normal, devolve `false`.

---

### FLY-P1-014 — Melhorias agrupadas de baixa prioridade

| Campo | Valor |
|---|---|
| Parte | 1 |
| Severidade | Baixa |
| Categoria | Manutenibilidade / Testes |
| Confiança | Confirmado |
| Esforço | M |

Itens independentes (podem ser feitos em qualquer ordem; um commit por item):

1. **Temporários órfãos:** `config.json.<pid>.<n>.tmp` (`config.rs:~366-372`) ficam no disco se o processo morrer entre a criação e o `rename`. Remover, no boot, `config.json.*.tmp` com mais de 1 dia.
2. **Testes de log com estado global:** `allow_log_line` usa um limitador global (20 linhas/s) compartilhado entre testes em paralelo (`diagnostics.rs:98-115`); aumentar a robustez isolando o limite em testes (ex.: parâmetro/`#[cfg(test)]` que o desativa) para evitar flakiness futura.
3. **Fonte de verdade do autostart:** `AppConfig.autostart` é gravado em disco mas **sempre** sobrescrito pelo estado real do SO (`get_config`, `save_config`). Documentar com comentário em `config.rs` que o campo é apenas um espelho (evita que alguém passe a confiar nele).
4. **`#[serde(default = "default_language")]`** em `language` é redundante com `#[serde(default)]` no struct (`config.rs:~44-46`): manter por clareza, mas comentar o motivo.
5. **Parâmetro não usado** `_app` em `start_watchdog` (`diagnostics.rs:316`): remover do contrato ou usá-lo; ajustar a chamada em `main.rs`.
6. **Comparação do token** de `heartbeat` com `!=` (`commands.rs:~125`): não há ganho prático em tempo constante (token por processo, canal local), mas registrar a decisão em comentário para futuras auditorias.

**Critérios de aceite.** `cargo fmt --check`, `cargo clippy -D warnings` e `cargo test` seguem passando após cada item.

---

## 6. Achados transversais (envolvem mais de uma parte)

| Tema | Partes | Achados relacionados | Ordem recomendada |
|---|---|---|---|
| Atalho global (preferido × ativo × reservado) | 1 e 3 | FLY-P1-001, FLY-P1-002 | Corrigir 002 (Rust) e traduções (Parte 3) **no mesmo lote** |
| Estado de encerramento e eventos de janela | 1 e 2 | FLY-P1-004, FLY-P1-010, FLY-P1-011 | Parte 1 primeiro; depois revisar `window.rs` |
| Nomes/autostart/desinstalação | 1 e 4 | FLY-P1-009, FLY-P1-012 | Decidir nome em Parte 4; Parte 1 só centraliza a constante |
| Privacidade de logs × política de privacidade | 1 e 4 | FLY-P1-007 | Corrigir código e depois revisar `PRIVACY_POLICY.md` |

---

## 7. Plano de correção ordenado (Parte 1)

**Onda 1 — Médias e pré-requisitos**
| Ordem | ID | Arquivos | Esforço | Como validar |
|---|---|---|---|---|
| 1 | FLY-P1-006 | `config.rs`, `commands.rs`, `main.rs` | P | `cargo test` (novo teste de *poison*) |
| 2 | FLY-P1-001 | `main.rs` | P | manual: conflito de atalho no boot; `clippy` |
| 3 | FLY-P1-002 | `config.rs` (+ traduções na Parte 3) | M | novo teste de atalhos reservados; manual |
| 4 | FLY-P1-004 | `main.rs` | P | manual: logoff com o app aberto |
| 5 | FLY-P1-003 | `commands.rs` | M | `clippy`, `check:acl`; manual com disco lento/MSIX |

**Onda 2 — Baixas funcionais:** FLY-P1-005, 013, 010, 009, 008, 007, 011.
**Onda 3 — Baixas/documentação:** FLY-P1-012 (depende da verificação em VM), FLY-P1-014.

**Agrupamento por arquivo (evita conflitos de edição):**
- `main.rs`: 006 (linha ~203) → 001 (205-217) → 004 (264-277) → 008 (164-170) → 014.5.
- `commands.rs`: 006 → 003 (assinaturas) → 005 → 011.
- `config.rs`: 006 → 002 → 009 → 008 → 014.
- `diagnostics.rs`: 010 → 007 → 014.
- `autostart.rs`: 013.

**Em paralelo:** itens de arquivos diferentes (ex.: 013 em `autostart.rs` e 007 em `diagnostics.rs`).
**Exigem decisão humana:** FLY-P1-012 (nome do autostart; depende do resultado em VM) e a variante opcional de FLY-P1-001 (expor o atalho ativo ao frontend).

---

## 8. Testes a adicionar

| Teste sugerido | Arquivo | Cobre |
|---|---|---|
| `normalization_rejects_reserved_system_shortcuts` | `config.rs` | FLY-P1-002 |
| `registered_shortcut_recovers_from_poisoned_mutex` | `config.rs` | FLY-P1-006 |
| `repair_creates_backup_of_original_config` | `config.rs` | FLY-P1-008 |
| `app_directory_never_uses_current_dir` (usa `temp_env`/manipulação de `APPDATA` em teste serial) | `config.rs` | FLY-P1-009 |
| `redact_path_masks_identifier_like_segments` | `diagnostics.rs` | FLY-P1-007 |
| `startup_state_enabled_covers_policy_states` | `autostart.rs` (`#[cfg(windows)]`) | FLY-P1-013 |
| Roteiro manual no Windows: conflito de atalho, logoff, MSIX, disco lento | — | FLY-P1-001, 003, 004 |

---

## 9. Itens descartados ou refinados (verificados)

| Item | Resultado |
|---|---|
| **H6** (watchdog/heartbeat causaria "ações indevidas") | **Refutado em parte:** o watchdog só **registra** avisos; não reinicia nem fecha nada. O heartbeat também é enviado nas páginas locais (o script injetado aceita origens `tauri://`/`tauri.localhost`), então a tela inicial não gera aviso. Restou apenas o ruído com a janela minimizada (FLY-P1-010). |
| **H13** (versões do crate `windows`) | **Parcialmente refutado para a Parte 1:** `windows` 0.62.2, `windows-core` 0.62.2 e `windows-future` 0.3.2 aparecem **uma única vez** no `Cargo.lock` (sem duplicidade). Há três `windows-sys` (0.45/0.59/0.61.2), todos transitivos e normais. Falta a verificação de existência/compatibilidade das versões na Parte 4. |
| Deadlock entre `config` e `SAVE_LOCK` | **Descartado:** a ordem é sempre *config → SAVE_LOCK*; `config::save` nunca pega o lock de configuração. |
| Escrita de `config.json` | **Correta:** arquivo temporário único, `sync_all`, `rename` com 3 tentativas; `rename` substitui o destino no Windows. |
| Algoritmo de data/hora dos logs | **Correto:** corresponde ao `civil_from_days` de Hinnant; coberto por testes (epoch, 29/02/2000, 2026). |
| *Panic* dentro da seção crítica do log (reentrada pelo `panic hook`) | **Descartado:** a seção só faz E/S com resultados ignorados; não há caminho realista de *panic* enquanto `LOG_LOCK` está preso. |
| Injeção/abuso via URL em `normalize()` | **Sem falha:** rejeita esquemas não-HTTP(S), host ausente e credenciais embutidas; `javascript:` etc. caem em erro. (Observação: `http://` é aceito — o impacto na página inicial é tema da Parte 3, hipótese H1.) |
| Comparação não constante do token do `heartbeat` | **Sem impacto prático** (registrado em FLY-P1-014.6). |

---

## 10. Pontos não verificados / perguntas em aberto

1. Nenhum teste Rust foi **executado** (sem toolchain/Windows). Rodar `cargo test`, `clippy` e `fmt` antes e depois de cada correção.
2. FLY-P1-003: confirmar em Windows (NSIS e MSIX) o tempo real de `get_config`/`save_config` e se `#[tauri::command(async)]` compila com as assinaturas propostas; se o compilador reclamar de referências em comandos assíncronos, manter `AppHandle` por valor (como sugerido) ou retornar `Result`.
3. FLY-P1-004 e FLY-P1-010: dependem de reprodução (logoff; janela minimizada por tempo prolongado).
4. FLY-P1-012: depende de inspeção do nome gravado pelo plugin de autostart (VM com instalador NSIS).
5. Decisão do autor: o atalho **ativo** (≠ preferido) deve ser mostrado na UI? (FLY-P1-001, variante opcional.)
6. Interfaces pendentes: Partes 2, 3 e 4 ainda não foram analisadas — hipóteses H1–H5, H9–H12 permanecem **abertas**.

---

## 11. Instruções operacionais para a IA corretora

- Aplique **um achado por vez**, na ordem da seção 7. Depois de cada um, rode: `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`, `cargo test --manifest-path src-tauri/Cargo.toml --locked`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings` e `npm run check:acl`. Só avance se tudo passar. (Em clone limpo, rode antes `npm run icons` e `npm run build`.)
- Altere **somente** o que o achado pede; não refatore por conta própria.
- **Releia o trecho atual** antes de editar: os números de linha deste relatório mudam após cada edição, principalmente em `main.rs`, `commands.rs` e `config.rs`.
- Adicione o teste sugerido junto com a correção.
- Mantenha comentários e logs em **pt-BR**; ao criar chaves de erro/aviso novas (ex.: `config.shortcut_reserved`), registre as traduções em **pt-BR, en e es** no frontend (Parte 3) e confira o mapeamento das chaves `config.*`.
- Se mexer em `generate_handler!`, `build.rs` ou capabilities, atualize os três **juntos** e rode `npm run check:acl`. (FLY-P1-003 **não** deve alterar nomes de comandos.)
- Nunca desative verificações (`check:*`, `clippy -D warnings`, `cargo audit`) para "fazer passar".
- **Não** renomeie `WidgetWebsite`, o crate `widget-website` nem o `identifier` sem plano de migração (Parte 4).
- Ao terminar cada achado, registre: ID, arquivos alterados, resultado dos comandos e qualquer desvio em relação ao proposto.

---

## 12. Prompt de execução sugerido (para colar na IA corretora)

```text
Você receberá o repositório Floatly (floatly-website-widget-main) e o relatório
RELATORIO_ANALISE_FLOATLY_PARTE_1.md. Corrija os problemas da PARTE 1 (núcleo Rust:
main.rs, config.rs, commands.rs, diagnostics.rs, autostart.rs) seguindo a seção 7
(plano de correção) e a seção 11 (instruções operacionais) do relatório.

Regras: um achado por vez (FLY-P1-nnn), na ordem do plano; releia o código atual antes
de editar; adicione os testes sugeridos; mantenha comentários/logs em pt-BR; atualize
traduções pt-BR/en/es quando criar novas chaves; após cada achado rode cargo fmt --check,
cargo test --locked, cargo clippy --all-targets --locked -- -D warnings e npm run check:acl.
Não desative nenhuma verificação. Não renomeie WidgetWebsite/widget-website.
Itens marcados "Hipótese" (FLY-P1-004, FLY-P1-012) só devem ser alterados depois de
reproduzidos; se não reproduzirem, registre como descartados. Ao final, entregue uma
tabela: ID · arquivos alterados · resultado dos comandos · desvios.
```

---

## 13. Checklist de conformidade com o prompt (Parte 1)

- [x] Parte trabalhada declarada (Parte 1 de 4) e delimitação das 4 partes com numeração.
- [x] Todos os arquivos da Parte 1 lidos integralmente.
- [x] Cada achado com local, evidência, causa raiz, solução, critérios de aceite, verificação e cuidados.
- [x] Confiança e esforço por achado; hipóteses sinalizadas como tais.
- [x] Tentativa de refutação registrada (seção 9).
- [x] Hipóteses H6, H7, H8, H13 (parcial) e H14 (parcial) tratadas; demais ficam para as Partes 2–4.
- [x] Plano ordenado por dependência e agrupado por arquivo.
- [x] Execução e limitações informadas (seção 2).
- [x] Nenhum arquivo do projeto foi alterado.
