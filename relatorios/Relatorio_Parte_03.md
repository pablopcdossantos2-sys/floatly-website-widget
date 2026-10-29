# Relatório de Análise e Auditoria de Código — Widget Website (Tauri 2 / Windows)
## Parte 3 — Camada de Aplicação: Ciclo de Vida, Janelas, Navegação e Bandeja

> Escopo deste relatório: **somente a Parte 3** do prompt `Prompt-Analise-Widget-Website.md`. Achados das Partes 1, 2, 4 e 5 só aparecem como referência cruzada (`T-xxx`).
> Nenhum arquivo do repositório foi modificado. Scripts auxiliares ficaram em `/home/claude/tmp`.

---

## 0. Instruções para a IA/Dev que fará as correções

- Corrija UM achado por vez, na ordem da seção 3.
- Após cada alteração, rode: `cargo test --manifest-path src-tauri/Cargo.toml` e `npm run build`.
- Adicione/ajuste os testes indicados; não remova testes sem justificativa.
- Em caso de conflito, siga o campo "Depende de / Conflita com".
- Achados `HIPOTESE` exigem verificação (idealmente em Windows) antes da correção.
- **Todos os números de linha** abaixo vêm de `sed -n`/`grep -n` sobre o ZIP enviado. `window.rs` e `commands.rs` têm linhas de centenas de caracteres: localize pelo nome da função, não só pela linha.
- Para todo achado desta parte, o passo 1 da correção é **extrair a política de navegação para funções puras** (ver P3-003), pois isso torna o resto testável sem Windows.

---

## 1. Resumo Executivo

- **Ambiente:** sandbox Linux sem `cargo`/`rustc`. O app é Windows-only. **Nada em Rust foi compilado ou executado.** A lógica de navegação foi validada com um **port fiel em Python** (`/home/claude/tmp/port.py`) das funções `allowed`, `is_auth_page`, `is_configured_page` e da decisão de `on_navigation`. Por isso nenhum achado está `CONFIRMADO` por execução do Rust; os rotulados `PROVAVEL` têm evidência no código **e** no port.
- **Linha de base de testes (estática):** 13 `#[test]` no repositório (`config.rs` 6, `diagnostics.rs` 4, `window.rs` 3). Na Parte 3, apenas `is_internal_app_url` tem teste (3 casos). `allowed`, `is_auth_page`, `is_configured_page`, `tray::labels`, `start_auto_reload`, `open_login`, `open_settings` e os handlers de `main.rs` têm **zero** cobertura.
- **Achados da Parte 3:** 14 (0 CRÍTICA, 0 ALTA, 10 MÉDIA, 4 BAIXA) + 3 transversais.
- **Top 5 riscos:**
  1. **P3-001** — a allowlist fixa `investing.com`, `*.google.com` e `*.facebook.com` e trata todos como "fluxo de login"; qualquer link para eles abre uma janela de navegação completa de 960×760 que compartilha cookies com o perfil principal.
  2. **P3-002** — não há deduplicação nem limite de janelas de login; cada navegação ou `window.open` cria um WebView2 novo.
  3. **P3-003** — a regra "este host pertence ao site?" está implementada 4 vezes, com critérios divergentes.
  4. **P3-005** — a janela de login pode nunca fechar e, quando fecha, a janela principal **não é recarregada**, então a sessão recém-autenticada não aparece.
  5. **P3-008** — alternar "travar"/"sempre no topo" pela bandeja com Configurações aberta e clicar em Salvar desfaz a alteração em silêncio.
- **Nenhum achado CRÍTICO ou ALTO** foi sustentado na Parte 3. A superfície IPC remota (que o prompt marca como prioridade máxima) pertence à Parte 4 e não foi avaliada aqui.

---

## 2. Ambiente e Linha de Base

| Item | Resultado |
|---|---|
| `which cargo rustc` | não encontrado (nenhuma saída); `~/.cargo/bin` inexistente |
| `cargo test` / `clippy` / `audit` | **não executados** (sem toolchain; app Windows-only) |
| `npm install` / `tsc` / `vite build` / `npm audit` | **não executados** (fora do escopo da Parte 3; sem rede para o registro) |
| `grep -c "#\[test\]" src-tauri/src/*.rs` | `commands.rs:0 config.rs:6 diagnostics.rs:4 main.rs:0 tray.rs:0 window.rs:3` = **13** |
| `grep -o 'let _ *=' src-tauri/src/*.rs` | `commands 9, config 1, diagnostics 10, main 7, tray 2, window 32` = **61** descartes de `Result` |
| Validação de lógica | `python3 /home/claude/tmp/port.py` (port das linhas `window.rs:142-169` e `206-209`) |

**Limitações:** comportamento de WebView2 (`NavigationStarting` em subframes, `NewWindowRequested`, minimização, `ShowWindow`) **não pôde ser observado**. Onde um achado depende disso, está marcado `HIPOTESE`.

**Arquivos lidos integralmente no escopo:** `main.rs` 1–30, 55–241; `tray.rs` 1–112; `commands.rs` 1–19; `window.rs` 142–213, 214–275 (para contexto do arraste), 276–306, 344–370.

---

## 3. Plano de Correção Ordenado

| Ordem | ID | Severidade | Esforço | Arquivo(s) | Depende de | Resumo |
|---|---|---|---|---|---|---|
| 1 | P3-003 | MÉDIA | M | `window.rs:142-169,206-207` | nenhum | Extrair `host_matches` e uma política de navegação única e pura |
| 2 | P3-001 | MÉDIA | M | `window.rs:142,208,211` | P3-003 | Remover `investing/google/facebook` genéricos da allowlist |
| 3 | P3-004 | MÉDIA | P | `window.rs:149` | P3-003 | `is_auth_page`: casar segmentos de caminho, não substring |
| 4 | P3-005 | MÉDIA | M | `window.rs:161-169,276-306` | P3-003, P3-004 | Fechamento do login + recarregar a janela principal |
| 5 | P3-002 | MÉDIA | M | `window.rs:208,211,276-302,346` | P3-003 | Uma janela de login por vez e limite de pop-ups |
| 6 | P3-013 | MÉDIA | P | `window.rs:209,211,294,302` | P3-003 | Limitar `open::that` disparado por navegação de script |
| 7 | P3-008 | MÉDIA | M | `commands.rs:17-18`, `settings.ts:181-190` | P1 (merge de estado) | Avisar Configurações quando a bandeja altera a config |
| 8 | P3-007 | MÉDIA | P | `main.rs:62-77`, `commands.rs:14` | nenhum | `toggle`/`show` sem `unminimize` e sem foco |
| 9 | P3-014 | MÉDIA | G | `window.rs:211,297-303` | nenhum | Pop-ups de OAuth perdem `window.opener` |
| 10 | P3-006 | BAIXA | P | `window.rs:348` | nenhum | Contador de recarga não zera ao mudar o intervalo |
| 11 | P3-009 | BAIXA | M | `tray.rs`, `commands.rs:9-11`, `window.rs:279,345` | nenhum | i18n do lado Rust e rótulo ambíguo |
| 12 | P3-010 | BAIXA | M | vários | nenhum | `let _ =` que apagam diagnóstico relevante |
| 13 | P3-011 | BAIXA | P | `window.rs:345` | P3-010 | `open_settings` não atômico e com erro engolido |
| 14 | P3-012 | BAIXA | P | `main.rs:207-213` | nenhum | Janela principal destruída não é recriada |

---

## 4. Achados por Parte

### Parte 3 — Camada de Aplicação: Ciclo de Vida, Janelas, Navegação e Bandeja

#### P3-001 — Allowlist fixa trata `investing.com`, `*.google.com` e `*.facebook.com` como "login"
- **Severidade:** MÉDIA
- **Categoria:** `seguranca`
- **Confiança:** PROVAVEL
- **Local:** `window.rs:142` (`allowed`), `window.rs:208` (`create_main → on_navigation`), `window.rs:211` (`create_main → on_new_window`)
- **Evidência:**
  ```rust
  // window.rs:142 (trecho)
  let auth = h=="accounts.google.com" || ... || h=="google.com" || h.ends_with(".google.com")
             || h=="investing.com" || h.ends_with(".investing.com");
  // window.rs:207-208
  if host==target||host.ends_with(&format!(".{target}")){return true}
  if allowed(host,&cfg){ ... thread::spawn(move||{let _=open_login(&a,dest);}); return false }
  ```
  Saída do port (`site = https://br.investing.com/economic-calendar`):
  ```
  https://www.investing.com/news/x          -> abre janela de login
  https://uk.investing.com/                 -> abre janela de login
  https://www.facebook.com/sharer/...       -> abre janela de login
  https://www.google.com/aclk?x=1           -> abre janela de login
  https://adservice.google.com/x            -> abre janela de login
  # site configurado = https://example.com/
  https://www.investing.com/x               -> abre janela de login
  ```
- **Reprodução:** `python3 /home/claude/tmp/port.py` (seção "Decisão da janela PRINCIPAL"). Não reproduzível em Windows neste ambiente.
- **Causa raiz:** `allowed()` mistura duas ideias: "host pertence ao site configurado" e "host de provedor de autenticação". Os três domínios genéricos são aceitos **independentemente** da URL configurada e **de qualquer caminho**. A janela criada por `open_login` é um navegador completo (`decorations(true)`, `resizable(true)`, barra de endereço injetada) que usa `data_directory = webview-data`, o **mesmo perfil** da janela principal (`window.rs:171` e `window.rs:278`).
- **Impacto:** links de compartilhamento, anúncios e outros subdomínios `*.investing.com` abrem janelas de 960×760 com cookies da sessão do usuário, em vez de ir ao navegador padrão. A janela de login não está em nenhuma capability (`default.json` lista `main` e `settings`; `investing-drag.json` lista `main`), então **não foi identificado acesso a IPC** a partir dela; a verificação definitiva é da Parte 4.
- **Correção proposta:**
  1. Separar duas listas: `AUTH_PROVIDERS` (hosts exatos de OAuth: `accounts.google.com`, `appleid.apple.com`, `login.microsoftonline.com`, `login.live.com`, `facebook.com` e `www.facebook.com`/`m.facebook.com` só com caminho de login/`dialog`) e "host do site configurado".
  2. Remover `google.com`, `*.google.com`, `investing.com` e `*.investing.com` genéricos. Hosts fora das duas listas vão para o navegador padrão.
  3. Nenhum domínio literal de site deve ficar no código; tudo deriva de `config.site_url`.
- **Critério de aceite:** com `site_url = https://br.investing.com/economic-calendar`, `www.google.com/aclk`, `adservice.google.com` e `uk.investing.com` resultam em `External`; `accounts.google.com` resulta em `OpenLogin`; trocar `site_url` para `https://example.com/` não faz `www.investing.com` abrir login.
- **Teste a adicionar:** `nav_policy_routes_non_provider_google_and_facebook_hosts_to_external_browser`, `nav_policy_does_not_hardcode_investing_when_site_changes`.
- **Depende de / Conflita com:** depende de P3-003.
- **Esforço:** M

---

#### P3-002 — Janelas de login sem deduplicação nem limite
- **Severidade:** MÉDIA
- **Categoria:** `recursos-memoria`
- **Confiança:** PROVAVEL
- **Local:** `window.rs:5` (`LOGIN_ID`), `window.rs:208` e `window.rs:211` (`thread::spawn(open_login)`), `window.rs:297-302` (`open_login → on_new_window`), `window.rs:346` (`open_login_from_tray`)
- **Evidência:**
  ```rust
  // window.rs:277
  let id=LOGIN_ID.fetch_add(1,Ordering::Relaxed);let label=format!("login-{id}");
  // window.rs:301 — dentro da própria janela de login
  let a=pop.clone();thread::spawn(move||{let _=open_login(&a,u);});
  ```
  Nenhuma verificação de janela `login-*` existente em `open_login`, em `open_login_from_tray` (`window.rs:346`) nem nos dois `on_new_window`.
- **Reprodução:** clicar N vezes em "Entrar / mostrar janela de login" na bandeja; ou um script da página chamando `window.open(googleUrl)` em laço. Esperado atual: N janelas `login-1…login-N`. Não reproduzível aqui.
- **Causa raiz:** o label é sempre novo, então o Tauri nunca rejeita por label duplicado; não existe contador de janelas abertas nem limite de taxa. Cada chamada cria um WebView2 completo.
- **Impacto:** consumo de memória/processos proporcional ao número de cliques ou pop-ups; janelas que o usuário precisa fechar uma a uma. Pop-ups de anúncios são o vetor mais plausível (ver P3-001).
- **Correção proposta:**
  1. Em `open_login`, procurar `app.webview_windows()` com label iniciando em `login-`. Se existir, `navigate(url)` + `set_focus()` na existente em vez de criar outra.
  2. Proteger a criação com um `Mutex`/`AtomicBool` ("login em criação") para cobrir o intervalo entre checar e criar.
  3. Limite rígido (por exemplo, 3 janelas) e descarte de `on_new_window` com intervalo menor que ~1 s.
- **Critério de aceite:** chamar `open_login` 10 vezes seguidas deixa no máximo 1 janela `login-*`; a janela existente é focada e navegada para a nova URL.
- **Teste a adicionar:** extrair a decisão para `fn login_window_action(existing: usize, last_open_ms: u64, now_ms: u64) -> LoginAction` e testar `login_window_reuses_existing_window` e `login_window_rate_limits_popups`.
- **Depende de / Conflita com:** depende de P3-003 (para a decisão de política).
- **Esforço:** M

---

#### P3-003 — Regra "host pertence ao site" implementada 4 vezes, com critérios divergentes
- **Severidade:** MÉDIA
- **Categoria:** `correcao-logica`
- **Confiança:** PROVAVEL
- **Local:** `window.rs:142` (`allowed`), `window.rs:143-151` (`is_auth_page`), `window.rs:161-169` (`is_configured_page`), `window.rs:206-207` (`on_navigation`, comparação inline)
- **Evidência:** cada função recalcula `Url::parse(&c.site_url)` e compara `h==t || h.ends_with(&format!(".{t}"))`. As listas de provedores diferem:
  ```
  host                   allowed()  is_auth_page("/")
  login.live.com         False      True
  www.investing.com      True       True
  br.investing.com       True       False
  idmsa.apple.com        False      False
  ```
  Em `window.rs:168`, `target_path=="/"` é **código morto**: `target_path` vem de `trim_end_matches('/')`, então nunca é `"/"`.
  O port também mostrou que `br.investing.com.` (FQDN com ponto final) não casa em nenhuma das quatro e cai no navegador padrão.
- **Reprodução:** `python3 /home/claude/tmp/port.py`, seção "Divergência allowed() x is_auth_page()".
- **Causa raiz:** ausência de uma função única `host_matches(host, target)` e de uma estrutura de decisão compartilhada.
- **Impacto:** `login.live.com` é aceito dentro da janela de login, mas aberto no navegador externo quando chega pela janela principal; o fluxo Microsoft depende do ponto de entrada. Qualquer ajuste futuro tem de ser repetido em 4 lugares.
- **Correção proposta:**
  1. Criar `fn host_matches(host: &str, target: &str) -> bool` (minúsculas, remove ponto final, exige `.` antes do sufixo).
  2. Criar `enum NavAction { Allow, OpenLogin, External, Block }` e `fn decide_main(url:&Url, cfg:&AppConfig) -> NavAction` / `fn decide_login(...)`, **puras** (sem `AppHandle`).
  3. Fazer `on_navigation` e `on_new_window` apenas executarem o que a decisão retornar.
  4. Remover a comparação morta de `window.rs:168`.
- **Critério de aceite:** nenhuma ocorrência de `format!(".{` fora de `host_matches` (`grep -n 'format!(".{' src-tauri/src/window.rs`); testes de decisão passam sem criar janelas.
- **Teste a adicionar:** `host_matches_requires_dot_boundary` (`evilinvesting.com` e `notinvesting.com` não casam com `investing.com`), `host_matches_ignores_trailing_dot`, `host_matches_handles_ip_and_single_label_targets`.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** M

---

#### P3-004 — `is_auth_page` casa caminhos por substring
- **Severidade:** MÉDIA
- **Categoria:** `correcao-logica`
- **Confiança:** PROVAVEL
- **Local:** `window.rs:149` (`is_auth_page → login_path`), uso em `window.rs:289` e `window.rs:300`
- **Evidência:**
  ```rust
  // window.rs:149
  let login_path=["/login","/log-in","/signin","/sign-in","/account","/accounts","/auth","/oauth","/callback"]
      .iter().any(|part|p.contains(part));
  ```
  Saída do port (host `br.investing.com`):
  ```
  /news/authors/joao          -> True
  /analysis/authority-x       -> True
  /accounting-news            -> True
  /members/callbacks-report   -> True
  /economic-calendar          -> False
  ```
  Além disso, a janela **principal** não usa `is_auth_page` (linha 207 retorna `true` para o mesmo host): `https://br.investing.com/login` carrega dentro do widget (port: "fica na janela principal"). Se o desenho pretende isolar autenticação na janela secundária, esse caminho escapa.
- **Reprodução:** `python3 /home/claude/tmp/port.py`, seção de falsos positivos.
- **Causa raiz:** `contains` em vez de comparar segmentos do caminho.
- **Impacto:** dentro da janela de login, abrir um artigo em `/news/authors/…` marca `nav_seen_auth = true`; a próxima navegação para a página configurada fecha a janela sem o usuário ter autenticado. O inverso (login na janela principal) é inconsistência de desenho.
- **Correção proposta:** comparar por segmento: `path.split('/').filter(|s|!s.is_empty()).any(|s| LOGIN_SEGMENTS.contains(&s))`, com `LOGIN_SEGMENTS = ["login","log-in","signin","sign-in","account","accounts","auth","oauth","callback"]`. Decidir (e documentar) se `/login` do site configurado deve ser desviado para a janela secundária; se sim, aplicar `is_auth_page` também em `decide_main`.
- **Critério de aceite:** `/news/authors/joao`, `/analysis/authority-x`, `/accounting-news` → `false`; `/login`, `/oauth/authorize`, `/account/login` → `true`.
- **Teste a adicionar:** `is_auth_page_matches_path_segments_not_substrings`.
- **Depende de / Conflita com:** depende de P3-003.
- **Esforço:** P

---

#### P3-005 — Fechamento automático do login é frágil e a janela principal não é recarregada
- **Severidade:** MÉDIA
- **Categoria:** `correcao-logica`
- **Confiança:** PROVAVEL
- **Local:** `window.rs:161-169` (`is_configured_page`), `window.rs:289-292` (`open_login → on_navigation`)
- **Evidência:**
  ```rust
  // window.rs:290-292
  if nav_seen_auth.load(Ordering::SeqCst)&&is_configured_page(&u,&c){
    ... thread::spawn(move||{ ... w.close(); }); return false }
  ```
  Port, com `site = https://br.investing.com/economic-calendar`:
  ```
  https://br.investing.com/economic-calendar   -> True   (fecha)
  https://br.investing.com/                    -> False  (não fecha)
  https://br.investing.com/portfolio           -> False  (não fecha)
  https://www.investing.com/economic-calendar  -> False  (não fecha)
  ```
  Busca por recarga da janela principal após o fechamento: nenhuma chamada a `reload`/`eval` em `window.rs:276-306`.
- **Reprodução:** autenticar e deixar o site redirecionar para a raiz ou para outra página. Não reproduzível aqui.
- **Causa raiz:** o critério de "login concluído" é "voltou para um caminho sob `site_url`". Se o site redireciona para `/` (ou para outro caminho), a janela fica aberta. Quando fecha, só chama `close()`.
- **Impacto:** (a) a janela de login permanece aberta após o sucesso, mostrando o site inteiro; (b) quando fecha, a janela principal continua mostrando o estado deslogado até uma recarga manual ou automática (`auto_reload_minutes` pode ser 0). Se o usuário já estava autenticado, a janela aberta pela bandeja também nunca fecha sozinha (`nav_seen_auth` permanece `false`).
- **Correção proposta:**
  1. Ao fechar a janela de login por qualquer caminho (`widget-login-action://close`, fechamento automático, botão da barra), chamar `reload_site` na janela principal.
  2. Ampliar o critério para "mesmo host registrável do site configurado, fora de caminho de autenticação" (usando `is_auth_page` corrigido) em vez de exigir prefixo do caminho configurado.
  3. Registrar em log o motivo do fechamento.
- **Critério de aceite:** após um fluxo de login simulado que termina em `https://br.investing.com/`, a janela `login-*` fecha e a janela principal recebe `location.reload()`.
- **Teste a adicionar:** `is_configured_page_accepts_site_root_after_login_redirect`; `login_close_requests_main_reload` (testável via função pura `fn should_close_login(seen_auth: bool, url:&Url, cfg:&AppConfig) -> bool`).
- **Depende de / Conflita com:** depende de P3-003 e P3-004.
- **Esforço:** M

---

#### P3-006 — `start_auto_reload`: contador não reinicia ao mudar o intervalo e recarrega oculto
- **Severidade:** BAIXA
- **Categoria:** `correcao-logica`
- **Confiança:** PROVAVEL
- **Local:** `window.rs:348` (`start_auto_reload`), chamada em `main.rs:218`
- **Evidência:**
  ```rust
  // window.rs:348 (trecho)
  let mins=...auto_reload_minutes...unwrap_or(0) as u32;
  if mins==0{seconds=0;continue}
  seconds+=10; if seconds>=mins*60{seconds=0; ... w.eval("location.reload()") ...}
  ```
  Port: `elapsed=3000 s`, usuário muda de 60 para 5 min → próximo tick: **recarga imediata**.
- **Reprodução:** definir 60 min, aguardar >5 min, mudar para 5 min e salvar.
- **Causa raiz:** `seconds` é local à thread e só é zerado quando `mins == 0` ou após recarregar; mudar o valor não o reinicia. Também não há verificação de `is_visible()` nem de rede.
- **Impacto:** recarga inesperada logo após salvar Configurações; recarga com a janela oculta (tráfego sem benefício visível); perda de estado de tela (filtros). Falha de rede continua levando à página de erro do Chromium (limitação admitida no README).
- **Correção proposta:** guardar `last_minutes` na thread e zerar `seconds` quando mudar; pular o ciclo se a janela estiver oculta (e recarregar ao reexibir se o tempo já expirou); opcionalmente trocar o laço de 10 s por `Instant`.
- **Critério de aceite:** mudar `auto_reload_minutes` de 60 para 5 após 50 min **não** recarrega no tick seguinte; com a janela oculta não há `eval` de recarga.
- **Teste a adicionar:** extrair `fn next_reload_state(state: ReloadState, mins: u32, visible: bool) -> (ReloadState, bool)` e testar `reload_counter_resets_when_interval_changes`, `reload_is_skipped_while_hidden`.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

---

#### P3-007 — `toggle`/`show_existing_instance` não restauram janela minimizada nem dão foco
- **Severidade:** MÉDIA
- **Categoria:** `ui-ux`
- **Confiança:** HIPOTESE
- **Local:** `main.rs:62-70` (`toggle`), `main.rs:72-77` (`show_existing_instance`), `commands.rs:14` (`toggle_main_visibility`, cópia idêntica de `toggle`)
- **Evidência:**
  ```rust
  // main.rs:64-68
  if window.is_visible().unwrap_or(false) { let _ = window.hide(); } else { let _ = window.show(); }
  // main.rs:74-75
  let _ = window.show(); let _ = window.set_focus();
  ```
  Não há `unminimize()`, nem `set_focus()` em `toggle`. O atalho global (`main.rs:37-41`) e o duplo clique da bandeja (`tray.rs:107`) chamam essas funções.
- **Reprodução:** minimizar o widget (Win+↓ ou "Mostrar área de trabalho"), pressionar o atalho. Esperado atual (hipótese): `is_visible()` retorna `true` para janela minimizada, então o atalho a **oculta**; um segundo toque chama `show()` sem restaurar. Não reproduzível aqui.
- **Causa raiz:** `is_visible()` não distingue minimizada nem coberta por outras janelas; `show()` não restaura.
- **Impacto:** o atalho principal do produto "não faz nada" ou faz o oposto em cenários comuns; a segunda instância (single-instance) pode não trazer a janela minimizada à frente.
- **Correção proposta:** em uma única função `show_main(app)`: `unminimize()`, `show()`, `set_focus()`. Em `toggle`: tratar `is_minimized()` ou janela sem foco como "trazer à frente" e só ocultar se estiver visível **e** focada. Substituir `commands::toggle_main_visibility` por chamada à função única.
- **Critério de aceite:** com a janela minimizada, o atalho a restaura e a foca; com a janela visível e focada, oculta; `grep -n "fn toggle" src-tauri/src` retorna uma só definição.
- **Teste a adicionar:** manual em Windows; automatizável só a decisão pura `fn toggle_action(visible: bool, minimized: bool, focused: bool) -> ToggleAction` (`toggle_restores_minimized_window`).
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

---

#### P3-008 — Bandeja altera `locked`/`always_on_top` sem avisar Configurações; "Salvar" desfaz a alteração
- **Severidade:** MÉDIA
- **Categoria:** `integridade-de-dados`
- **Confiança:** PROVAVEL
- **Local:** `commands.rs:17` (`toggle_locked`), `commands.rs:18` (`toggle_topmost`), `settings.ts:181-182,185,190` (`read()`), `commands.rs:11` (`save_config`, `*stored=c.clone()`)
- **Evidência:**
  ```ts
  // settings.ts:181-190
  x: current?.x ?? null,  y: current?.y ?? null,
  always_on_top: checked("always_on_top"),  locked: checked("locked"),
  ```
  `grep -n "listen\|emit" src/settings.ts src/main.ts` → **nenhuma ocorrência**; só `invoke` (`settings.ts:171,201,244,253`; `main.ts:90`). O backend nunca emite evento ao alterar a config.
- **Reprodução:** abrir Configurações; pela bandeja, escolher "Travar…"; voltar a Configurações e clicar Salvar. `save_config` grava `locked` com o valor antigo da caixa de seleção e chama `window::apply_config`, destravando a janela. O mesmo vale para `x`/`y` movidos depois da abertura (persistidos antigos).
- **Causa raiz:** o front envia o estado inteiro (incluindo campos de propriedade do backend) e o backend aceita tudo (`commands.rs:9-11`); não há mecanismo de sincronização bandeja → UI.
- **Impacto:** perda silenciosa de uma preferência que o usuário acabou de alterar. Há também E/S sob lock em `toggle_*` (`config::save` com o `MutexGuard` vivo), relatada na Parte 1/2.
- **Correção proposta:**
  1. Emitir `app.emit("config-changed", &next)` ao fim de `toggle_locked`, `toggle_topmost` e `save_config`; em `settings.ts`, `listen("config-changed", …)` atualizando `current` e os campos afetados.
  2. No backend (correção principal, na Parte 1): em `save_config`, **ignorar** `x`, `y`, `locked`, `always_on_top` vindos do front quando não foram editados, mesclando sobre o estado em memória.
- **Critério de aceite:** com Configurações aberta, alternar "travar" pela bandeja atualiza a caixa de seleção em até 1 s; salvar sem tocar nela mantém `locked`.
- **Teste a adicionar:** `save_config_preserves_backend_owned_fields` (Rust, Parte 1) e teste manual descrito acima.
- **Depende de / Conflita com:** depende da correção de merge de estado na Parte 1 (pista 5). Ver T-001.
- **Esforço:** M

---

#### P3-009 — i18n do lado Rust incompleta, duplicada e com rótulo ambíguo
- **Severidade:** BAIXA
- **Categoria:** `i18n`
- **Confiança:** CONFIRMADO (por leitura direta do código; não depende de execução)
- **Local:** `tray.rs:8-41` (`labels`), `tray.rs:34`, `commands.rs:9-11`, `config.rs` (`normalize`), `window.rs:279` e `window.rs:345` (títulos)
- **Evidência:**
  - Rótulo pt-BR `"Travar / destravar (travado)"` / `"(destravado)"` (`tray.rs:34`): não diz o que o clique fará. Em en/es o texto é uma ação (`"Unlock moving / resizing"`).
  - Mensagens só em pt-BR: `"Falha ao ler a configuração."`, `"O atalho novo conflita com outro aplicativo; …"`, `"Não foi possível habilitar o início com o Windows."` (`commands.rs:9-11`), `"Informe a URL do site."` etc. (`config.rs normalize`).
  - O título de Configurações é definido em dois lugares (`commands.rs:11` com `match c.language` e `window.rs:345`).
  - `labels` retorna `[&'static str; 8]` acessado por posição (`text[0]`…`text[7]`, `tray.rs:51-58,71`).
- **Reprodução:** definir idioma `en`, digitar um atalho já em uso e salvar: o aviso aparece em português.
- **Causa raiz:** não existe módulo de tradução; cada ponto resolve idioma por conta própria.
- **Impacto:** UX inconsistente para en/es; risco de desalinhar índices ao inserir um item de menu.
- **Correção proposta:** criar `i18n.rs` com `struct TrayLabels { show_hide, reload, settings, lock, topmost, login, quit, tooltip }` e `fn settings_title(lang)`/`fn login_title(lang)`; trocar erros por um `enum UserMessage` traduzido em um único ponto. Rótulo: `"Travar mover/redimensionar"` ↔ `"Destravar mover/redimensionar"`.
- **Critério de aceite:** nenhum literal em português fora de `i18n.rs` e de `diagnostics::log` (`grep -n '"[^"]*ção[^"]*"'`); testes de `labels` passam para os 3 idiomas.
- **Teste a adicionar:** `tray_labels_have_distinct_lock_texts_for_each_state_and_language`, `unknown_language_falls_back_to_pt_br`.
- **Depende de / Conflita com:** nenhum. Coordenar com a Parte 4 (acoplamento `includes("atalho")` em `settings.ts`).
- **Esforço:** M

---

#### P3-010 — 61 `let _ =` descartam erros; alguns apagam diagnóstico relevante
- **Severidade:** BAIXA
- **Categoria:** `correcao-logica`
- **Confiança:** CONFIRMADO (contagem por `grep`)
- **Local:** por arquivo: `window.rs` 32, `diagnostics.rs` 10, `commands.rs` 9, `main.rs` 7, `tray.rs` 2, `config.rs` 1
- **Evidência:** os descartes com perda real de informação, em ordem de importância:
  - `window.rs:345` — `let _=WebviewWindowBuilder::new(&a,"settings",…).build();` (ver P3-011)
  - `window.rs:208,211,301,346` — `thread::spawn(move||{let _=open_login(&a,u);})` (falha ao abrir login não deixa rastro)
  - `window.rs:209,211,294,302` — `let _=open::that(u.as_str())`
  - `tray.rs:68,71` — `let _=tray.set_menu(...)`, `set_tooltip`
  - `commands.rs:11` — `let _=main.navigate(url)` após mudar `site_url`
  - `commands.rs:15` e `window.rs:348` — `let _=w.eval("location.reload()")`
  - `commands.rs:17-18`, `main.rs:141`, `window.rs:347` — `let _=config::save(...)` (Parte 1/2)
- **Reprodução:** não aplicável (inspeção estática).
- **Causa raiz:** estilo de tratamento de erro "melhor esforço" sem log.
- **Impacto:** falhas de abertura de janela, navegação e persistência ficam invisíveis em `diagnostics.log`, justamente o recurso criado para diagnosticar travamentos.
- **Correção proposta:** helper `fn log_err<T,E:Display>(dir:&Path, ctx:&str, r:Result<T,E>)` em `diagnostics.rs` e uso nos descartes acima; manter `let _ =` apenas em `hide/show/set_*` triviais.
- **Critério de aceite:** `grep -c "let _ *=" src-tauri/src/window.rs` cai para os casos triviais; falha forçada em `open_login` gera linha `ERRO` no log.
- **Teste a adicionar:** `log_err_writes_error_line_with_context`.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** M

---

#### P3-011 — `open_settings` não é atômico e engole o erro
- **Severidade:** BAIXA
- **Categoria:** `concorrencia`
- **Confiança:** PROVAVEL
- **Local:** `window.rs:345` (`open_settings`)
- **Evidência:**
  ```rust
  if let Some(w)=app.get_webview_window("settings"){ ...; return }
  let a=app.clone(); thread::spawn(move||{ ... let _=WebviewWindowBuilder::new(&a,"settings",…).build(); });
  ```
  A verificação e a criação ocorrem em momentos diferentes (a criação é numa thread).
- **Reprodução:** clicar duas vezes rápido em "Configurações" na bandeja.
- **Causa raiz:** janela de tempo entre `get_webview_window` e `build`.
- **Impacto:** a **pista inicial de "label duplicado" não se sustenta**: o Tauri exige labels únicos, então a segunda criação retorna erro e não cria janela duplicada (PROVAVEL; comportamento do Tauri não verificado por execução). O defeito real é o erro engolido: se a criação falha por outro motivo (por exemplo, permissão em `settings-data`), o usuário clica e nada acontece, sem log.
- **Correção proposta:** guarda `AtomicBool` "criando"; registrar o erro de `build()` com `log_err` (P3-010).
- **Critério de aceite:** dois cliques rápidos geram 1 janela e nenhuma linha de erro; uma falha forçada gera linha `ERRO` no log.
- **Teste a adicionar:** manual em Windows.
- **Depende de / Conflita com:** P3-010.
- **Esforço:** P

---

#### P3-012 — Janela principal destruída não é recriada
- **Severidade:** BAIXA
- **Categoria:** `correcao-logica`
- **Confiança:** HIPOTESE
- **Local:** `main.rs:207-213` (`WindowEvent::Destroyed`), `main.rs:62-77`
- **Evidência:** o handler apenas registra `WARN` (`main.rs:208-212`). `toggle`/`show_existing_instance` usam `if let Some(window)=app.get_webview_window("main")` e não fazem nada se for `None`.
- **Reprodução:** não reproduzível neste ambiente; cenário a testar em Windows: encerrar o processo `msedgewebview2.exe` da janela principal e observar se a janela é destruída ou apenas fica em branco.
- **Causa raiz:** não existe caminho de recriação nem tratamento de falha do processo do WebView2.
- **Impacto:** se a janela principal for destruída com o processo vivo, o app fica só na bandeja, o atalho e o duplo clique não fazem nada e nenhuma mensagem é exibida.
- **Correção proposta:** no `Destroyed` da janela principal, se o app não estiver encerrando, chamar `create_main` de novo (com limite de tentativas) ou `app.exit(1)` com mensagem; se `main` não existir em `toggle`, recriar.
- **Critério de aceite:** destruir a janela principal resulta em recriação automática ou saída explícita registrada em log.
- **Teste a adicionar:** manual em Windows.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

---

#### P3-013 — `open::that` disparado por navegação de script, sem limite
- **Severidade:** MÉDIA
- **Categoria:** `seguranca`
- **Confiança:** PROVAVEL
- **Local:** `window.rs:209` (`on_navigation`, janela principal), `window.rs:211` (`on_new_window`), `window.rs:294` e `window.rs:302` (janela de login)
- **Evidência:**
  ```rust
  // window.rs:209
  crate::diagnostics::log(&diag_dir,"INFO",format!("Encaminhando link externo ao navegador padrão (host={host})."));
  let _=open::that(u.as_str()); false
  ```
  Antes dessas linhas só há filtro de esquema (`window.rs:204`, `285`, `211`, `297`: apenas `http`/`https`). Sem verificação de gesto do usuário, de repetição ou de taxa.
- **Reprodução:** uma página que execute `setInterval(()=>location.href='https://exemplo.test/'+n++, 500)` abre uma aba do navegador padrão a cada 500 ms. Não reproduzível aqui.
- **Causa raiz:** `open::that` é síncrono, roda na thread de eventos e não é limitado.
- **Impacto:** script da página (ou de anúncio) pode inundar o navegador do usuário e bloquear a thread de UI. **Ponto verificado e correto:** o filtro de esquema evita `file:`/esquemas customizados, então não há execução de arquivo local por esse caminho.
- **Correção proposta:** `static LAST_EXTERNAL: AtomicU64` com intervalo mínimo (por exemplo, 2 s) e descarte de URL idêntica à anterior; executar `open::that` em `thread::spawn`.
- **Critério de aceite:** 20 navegações externas em 1 s resultam em no máximo 1 abertura; a thread de eventos não bloqueia.
- **Teste a adicionar:** `external_open_rate_limiter_drops_burst_and_duplicates` (função pura com relógio injetado).
- **Depende de / Conflita com:** depende de P3-003.
- **Esforço:** P

---

#### P3-014 — Pop-ups de OAuth perdem `window.opener`
- **Severidade:** MÉDIA
- **Categoria:** `compatibilidade-plataforma`
- **Confiança:** HIPOTESE
- **Local:** `window.rs:211` (`create_main → on_new_window`), `window.rs:297-303` (`open_login → on_new_window`)
- **Evidência:** `on_new_window` sempre retorna `NewWindowResponse::Deny` e cria uma janela **independente** com `WebviewUrl::External(url)` (`window.rs:282`). A conclusão do login é detectada apenas por redirecionamento (`is_configured_page`), nunca por `postMessage`.
- **Reprodução:** em Windows, usar "Entrar com Google/Facebook" no site e observar se o pop-up fecha e a sessão aparece. Não reproduzível aqui.
- **Causa raiz:** fluxos OAuth que dependem de `window.opener.postMessage` (por exemplo, SDKs JavaScript de login) exigem que o pop-up seja filho da janela de origem.
- **Impacto:** se o site usar esse fluxo, o login nunca completa mesmo com credenciais corretas. Se usar redirecionamento, funciona (e P3-005 se aplica).
- **Correção proposta:** testar nos dois fluxos. Se falhar, usar o mecanismo de nova janela do Tauri 2 que preserva a relação com a janela de origem, em vez de `Deny` + janela nova; ou forçar o fluxo por redirecionamento.
- **Critério de aceite:** login com Google e com Facebook conclui e a janela principal mostra a sessão ativa.
- **Teste a adicionar:** roteiro manual em Windows; registrar o resultado no README.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** G

---

## 5. Achados Transversais

#### T-001 — `config.json` escrito por vários caminhos sobre estado desatualizado
- **Partes:** 1, 2, 3
- **Local:** `commands.rs:11,17,18`, `main.rs:141,196-198`, `window.rs:347`
- **Resumo:** a bandeja (P3-008), o handler de `Moved`/`Resized` (`main.rs:196-198 → save_window_state`) e `save_config` gravam o mesmo arquivo sem ordem garantida. Corrigir P3-008 no front **não basta**; a regra de merge pertence à Parte 1 (pistas 4, 5 e 6).
- **Confiança:** PROVAVEL. **Depende de:** correções da Parte 1.

#### T-002 — `begin_native_drag` roda dentro de `on_navigation`
- **Partes:** 2, 3, 4
- **Local:** `window.rs:181-203` (`on_navigation`) chamando `begin_native_drag` (`window.rs:214-246`)
- **Resumo:** `SendMessageW(WM_NCLBUTTONDOWN)` abre um laço modal dentro do callback `NavigationStarting`. Verificado na Parte 3: **o `MutexGuard` de `config` é solto antes** (`window.rs:184`, temporário de `.lock().map(...)`), então `save_window_state` (`window.rs:347`), disparado por `Moved` durante o arraste, **não** causa deadlock nesse caminho. Verificado também: a ação só é aceita com `locked == false`. **Não verificado:** o efeito do laço modal sobre o watchdog e a reentrância de eventos (Parte 2).
- **Confiança:** HIPOTESE (Windows real).

#### T-003 — Esquemas `widget-action://` aceitos sem checar origem
- **Partes:** 3, 4
- **Local:** `window.rs:181-203`
- **Resumo:** o ramo `widget-action` executa para qualquer URL carregada na janela principal. Não há verificação de origem, de quadro ou de gesto, e a ACL do Tauri não governa esse canal (é uma navegação, não um comando IPC). O `investing-drag.json` concede só `core:window:allow-start-dragging` a `https://*.investing.com`, o que é um mecanismo diferente. A análise de impacto completa é da Parte 4.
- **Confiança:** PROVAVEL.

---

## 6. Cobertura de Testes Ausente

Todos viáveis em CI Linux se as funções forem **puras** (ver P3-003). Hoje só existem os 3 testes de `is_internal_app_url` (`window.rs:349-370`).

| Cenário sem teste | Nome sugerido |
|---|---|
| Sufixo com fronteira de ponto (`evilinvesting.com`) | `host_matches_requires_dot_boundary` |
| FQDN com ponto final | `host_matches_ignores_trailing_dot` |
| Alvo IP / rótulo único | `host_matches_handles_ip_and_single_label_targets` |
| Roteamento de hosts Google/Facebook não-provedores | `nav_policy_routes_non_provider_google_and_facebook_hosts_to_external_browser` |
| Site trocado em Configurações | `nav_policy_does_not_hardcode_investing_when_site_changes` |
| Substring em caminho | `is_auth_page_matches_path_segments_not_substrings` |
| Raiz do site após login | `is_configured_page_accepts_site_root_after_login_redirect` |
| Divergência entre listas de provedores | `provider_lists_are_consistent_between_main_and_login_windows` |
| Dedupe de janela de login | `login_window_reuses_existing_window` |
| Limite de taxa de pop-ups | `login_window_rate_limits_popups` |
| Limite de `open::that` | `external_open_rate_limiter_drops_burst_and_duplicates` |
| Contador de recarga | `reload_counter_resets_when_interval_changes` |
| Recarga com janela oculta | `reload_is_skipped_while_hidden` |
| Rótulos da bandeja por estado e idioma | `tray_labels_have_distinct_lock_texts_for_each_state_and_language` |
| Idioma desconhecido | `unknown_language_falls_back_to_pt_br` |
| Restaurar janela minimizada | `toggle_restores_minimized_window` |
| `is_internal_app_url` em release | `debug_loopback_is_not_internal_in_release` (`#[cfg(not(debug_assertions))]`) |

---

## 7. Falsos Positivos e Itens Verificados Sem Problema

### Resultado de cada "Pista inicial" da Parte 3

| # | Pista | Resultado |
|---|---|---|
| 1 | Allowlist triplicada e divergente | **Confirmada** → P3-001, P3-003. Sufixo com fronteira de ponto está correto: `evilinvesting.com` e `notinvesting.com` **não** casam (port). IP como alvo: `ends_with(".10.0.0.1")` não casa host válido, porque o parser de URL rejeita rótulo final numérico. Rótulo único (`intranet`) casa `x.intranet`, o que é o comportamento esperado. Se `on_navigation` dispara para subframes: **HIPOTESE**, não verificável aqui. Pelo uso de `NavigationStarting` pelo wry, a expectativa é de que só navegações do quadro principal cheguem ao callback; confirmar em Windows. |
| 2 | Janelas de login em excesso / falsos positivos de `is_auth_page` | **Confirmada** → P3-002, P3-004. "Laço infinito de janelas" **não demonstrado**: há recursão possível (`open_login → on_new_window → open_login`), mas depende de a página abrir pop-ups, e não existe laço automático por si só. |
| 3 | Corrida no fechamento automático | **Parcial.** A corrida é benigna: duas threads de fechamento seguidas resultam em `get_webview_window → None` na segunda. O problema real é a fragilidade do critério → P3-005. |
| 4 | `is_internal_app_url` em debug aceita qualquer `127.0.0.1` | **Confirmada no código** (`window.rs:158`, `cfg!(debug_assertions)`), mas **sem impacto em release**: a condição é falsa em builds de produção. Risco só em desenvolvimento. Ver melhoria opcional. |
| 5 | Estado bandeja × Configurações | **Confirmada** → P3-008 e P3-009. **Refutado:** `toggle_topmost` não precisa atualizar o menu, pois o rótulo "Alternar sempre no topo" é estático (`tray.rs:15,25,35`). **Refutado:** `labels(language,false)` no tooltip é inofensivo, pois `text[7]` não depende de `locked` (`tray.rs:70-71`). Reconstruir o menu a cada toggle é desperdício menor, não defeito. |
| 6 | Ciclo de vida | **Refutada em parte.** `CloseRequested` só esconde (`main.rs:193-194`), como pretendido, e Alt+F4 funciona assim. `quit_app` sem flush **não** perde dados: a config é gravada a cada alteração (`commands.rs:11,17,18`, `window.rs:347`); atalhos globais do Windows são liberados no fim do processo. Remoção do ícone da bandeja em `app.exit(0)`: **não verificado**. `Destroyed` sem recriação → P3-012 (HIPOTESE). |
| 7 | `start_auto_reload` | **Confirmada** → P3-006. |
| 8 | `open_settings` e `let _ =` | **Parcial.** Duplicação de janela **refutada** (label único); erro engolido **confirmado** → P3-011. Inventário dos 61 `let _ =` → P3-010. |
| 9 | Cobertura | **Confirmada** → seção 6. |

### Verificado sem problema
- `widget-diagnostic://` é tratado antes de qualquer outra decisão e retorna `false` (`window.rs:174`).
- Todos os caminhos que chamam `open::that` filtram o esquema para `http`/`https` antes (`window.rs:204`, `285`, `211`, `297`).
- `LOGIN_ID.fetch_add(1, Relaxed)` gera labels únicos; `Relaxed` é suficiente (`window.rs:277`).
- Nenhum `MutexGuard` de `config` fica retido durante a criação de janelas ou durante `begin_native_drag` (`window.rs:184,206,287,298`).
- `widget-login-action://close` fecha a janela do próprio `close_label` (`window.rs:284`).
- Janelas `login-*` não pertencem a nenhuma capability (`default.json` e `investing-drag.json` listam `main` e `settings`): sem ACL concedida a elas (confirmação final na Parte 4).
- O ícone da bandeja depende de `default_window_icon()` (`tray.rs:87-89`); como `tauri::generate_context!` exige os ícones configurados em `tauri.conf.json`, ausência de ícone não é caminho realista em runtime.

---

## 8. Melhorias Opcionais

- Em debug, restringir `is_internal_app_url` à porta do `devUrl` (`1420`) em vez de todo `127.0.0.1` (`window.rs:158`).
- Trocar `[&'static str; 8]` de `tray::labels` por struct nomeada (já em P3-009).
- Fazer `update_language` reconstruir o menu só quando o idioma ou `locked` mudar.
- Renomear `update_language` para `refresh_tray`, pois também atualiza o rótulo de bloqueio.
- Substituir a dupla definição de `toggle` por uma função em módulo comum (P3-007).
- Adicionar `Closed`/`CloseRequested` simétricos para `settings` e `login-*` apenas para log.

---

## 9. Checklist Final de Verificação Pós-Correção

- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` passa, com os testes da seção 6 incluídos.
- [ ] `npm run build` passa.
- [ ] `grep -n 'format!(".{' src-tauri/src/window.rs` retorna só `host_matches`.
- [ ] Nenhum domínio literal de site (`investing`, `google`, `facebook`) em `allowed`; apenas `AUTH_PROVIDERS` explícito.
- [ ] Teste manual (Windows): 10 cliques em "Entrar…" geram no máximo 1 janela `login-*`.
- [ ] Teste manual: login termina em `/` e a janela fecha, e a janela principal recarrega mostrando a sessão.
- [ ] Teste manual: atalho global restaura janela minimizada.
- [ ] Teste manual: alternar "travar" pela bandeja com Configurações aberta atualiza a UI; salvar não reverte.
- [ ] Teste manual: mudar `auto_reload_minutes` de 60 para 5 não recarrega imediatamente.
- [ ] Teste manual: idioma `en` mostra todos os avisos em inglês.
- [ ] `diagnostics.log` contém linha `ERRO` ao forçar falha em `open_login` e em `open_settings`.
- [ ] Nenhum arquivo fora do escopo dos achados foi alterado (`git diff --stat`).
