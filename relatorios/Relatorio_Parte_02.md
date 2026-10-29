# Relatório de Análise e Auditoria de Código — Widget Website (Tauri 2 / Windows)

> **Escopo desta entrega: somente a PARTE 2 — Integrações Windows, Arquivos e Diagnóstico (I/O).**
> As Partes 1, 3, 4 e 5 não foram analisadas aqui; aparecem apenas como referência cruzada onde um achado da Parte 2 depende delas.
> Data da auditoria: 2026-10-29. Repositório: `widget-website` 0.1.0 (`main`). Nenhum arquivo do repositório foi modificado (trabalho feito em cópia extraída em `/home/claude/repo`).

## 0. Instruções para a IA/Dev que fará as correções
- Corrija UM achado por vez, na ordem da seção 3.
- Após cada alteração, rode: `cargo test --manifest-path src-tauri/Cargo.toml` e `npm run build`.
- Adicione/ajuste os testes indicados; não remova testes sem justificativa.
- Em caso de conflito, siga o campo "Depende de / Conflita com".
- Achados `HIPOTESE` exigem verificação (idealmente em Windows) antes da correção. O passo de verificação está descrito em cada achado.
- **Nenhum achado desta parte foi `CONFIRMADO` por execução**: Rust/Cargo não estão disponíveis no ambiente de auditoria e o app é Windows-only. Os níveis são `PROVAVEL` (evidência clara no código) ou `HIPOTESE`.

## 1. Resumo Executivo

- **Ambiente:** sandbox Linux, Node v22.22.2 / npm 10.9.7, **sem** `cargo`/`rustc`/`clippy`/`cargo-audit`, **sem rede**. Análise estática + simulações em Python para trechos puros.
- **Linha de base:** nenhum teste Rust pôde ser executado. Contagem estática de `#[test]`: `config.rs` 6, `diagnostics.rs` 4, `window.rs` 3 = **13** (confere com o prompt). `npm install` expirou (90 s, exit 124, sem rede); `npm audit` falha por ausência de `package-lock.json`.
- **Achados da Parte 2: 18** — **3 ALTA, 8 MÉDIA, 7 BAIXA** (0 CRÍTICA). 8 são `PROVAVEL` com severidade ≥ MÉDIA; 7 são `HIPOTESE` (dependem de Windows real ou do código-fonte do Tauri).
- **Top 5 riscos:**
  1. **P2-003** — build de release sem `windows_subsystem = "windows"`: janela de console extra; fechá-la encerra o widget.
  2. **P2-002** — falha de leitura/parse de `config.json` → defaults → `main.rs:141` **sobrescreve** o arquivo (perda de preferências).
  3. **P2-001** — com "Lembrar posição e tamanho" desmarcado, a região arredondada (HRGN) não é recalculada após redimensionar → conteúdo recortado.
  4. **P2-007** — `site_url` completo (inclusive `user:senha@`) vai ao log, contradizendo o README.
  5. **P2-005** — arraste nativo usa `SendMessageW` (laço modal) dentro do callback `on_navigation`.
- **Pistas iniciais:** as 8 foram avaliadas. 4 confirmadas (1, 2, 3, 4), 4 parcialmente confirmadas/refutadas (5 a 8: LPARAM correto, sem vazamento de HRGN, snapshot sempre fechado, plugins sem erro silenciado) — ver seção 7.

## 2. Ambiente e Linha de Base

| Comando / verificação | Resultado exato |
|---|---|
| `node --version` / `npm --version` | `v22.22.2` / `10.9.7` |
| `cargo`, `rustc`, `cargo-clippy`, `cargo-audit` | **não instalados** → `cargo test`, `clippy`, `audit` **não executados** |
| `npm install` | expirou após 90 s (`exit=124`); sem rede, `node_modules` não criado |
| `npm audit` | `npm error audit ... loadVirtual requires existing shrinkwrap file` (não há `package-lock.json`) |
| `npx tsc --noEmit` / `npm run build` | não executados (sem `node_modules`) |
| `grep -c "#\[test\]" src-tauri/src/*.rs` | `commands.rs:0 config.rs:6 diagnostics.rs:4 main.rs:0 tray.rs:0 window.rs:3` |
| `grep -rn windows_subsystem .` | **0 ocorrências** (exit 1) |
| Presença de `package-lock.json`, `Cargo.lock`, `LICENSE` | **ausentes** (Parte 5) |
| CI (`build-windows.yml:37`) | executa `cargo test --manifest-path src-tauri/Cargo.toml` no `windows-latest` (resultado não disponível) |

**Simulações executadas (Python, equivalentes às expressões Rust):**
- Vetores de `formats_leap_day_and_modern_year_correctly` (`diagnostics.rs:327-336`): `951782400 → 2000-02-29T00:00:00.000Z` e `1793291247 → 2026-10-29T16:27:27.251Z` **conferem** com `datetime` UTC.
- Empacotamento de `LPARAM` de `begin_native_drag` (`window.rs:227-228`) com `(100,200)`, `(-1920,300)`, `(2500,-1080)`, `(-32768,32767)`, `(-1,-1)`: `GET_X_LPARAM/GET_Y_LPARAM` recuperam todos os pares corretamente (ver P2 seção 7, item 5).

**Limitações:** nada de FFI Win32, WebView2, rotação com arquivo bloqueado ou comportamento de DPI foi executado. Pontos que dependem do código do crate `tauri-runtime-wry`/`tao` (sem rede) estão rotulados `HIPOTESE`.

## 3. Plano de Correção Ordenado

| Ordem | ID | Severidade | Esforço | Arquivo(s) | Depende de | Resumo (1 linha) |
|---|---|---|---|---|---|---|
| 1 | P2-003 | ALTA | P | `main.rs:1` | nenhum | Adicionar `windows_subsystem = "windows"` (sem console em release) |
| 2 | P2-001 | ALTA | M | `window.rs:347,309-334`, `main.rs:196-198` | P1 (`save_window_state`) | Recalcular HRGN em todo `Resized`, independente de `remember_position` |
| 3 | P2-002 | ALTA | M | `config.rs:106-115`, `main.rs:137-141` | coordenar com P1 pista 1 | Não sobrescrever `config.json` ilegível/corrompido; fazer backup |
| 4 | P2-004 | MÉDIA | M | `config.rs:117-131` | P2-002 | Gravação durável (`sync_all`), sem fallback não atômico, limpar `.tmp` |
| 5 | P2-007 | MÉDIA | P | `main.rs:179-181`, `diagnostics.rs:101-113` | nenhum | Sanitizar URLs no log (sem userinfo/query/fragmento) |
| 6 | P2-005 | MÉDIA | M | `window.rs:190,215-240` | P3 (`on_navigation`) | `PostMessageW` + checar botão esquerdo pressionado |
| 7 | P2-006 | MÉDIA | M | `diagnostics.rs:77-91` | nenhum | Rotação que não perde backup nem cresce sem limite |
| 8 | P2-009 | MÉDIA | M | `main.rs:13-35,90-98` | nenhum | Pré-checagem do WebView2 não deve abortar o app |
| 9 | P2-011 | MÉDIA | M | `main.rs:131-240` | nenhum | Garantir mensagem FATAL para erro dentro do `setup` |
| 10 | P2-008 | MÉDIA | M | `diagnostics.rs:169-170,213-236` | nenhum | Watchdog sem chamadas que dependem da thread de UI |
| 11 | P2-010 | MÉDIA | P | `diagnostics.rs:93-98` | P2-003 | Não bloquear em `stdout`; desativar em release |
| 12 | P2-012 | BAIXA | P | `diagnostics.rs:147-201`, `main.rs:62-77` | nenhum | Reiniciar a carência do watchdog ao exibir a janela |
| 13 | P2-013 | BAIXA | P | `config.rs:99-104` | nenhum | Validar `APPDATA` e fallback seguro |
| 14 | P2-014 | BAIXA | P | `diagnostics.rs:72-99`, `window.rs:186,197` | P2-006 | Limitar tamanho/taxa de mensagens vindas da página |
| 15 | P2-015 | BAIXA | P | `window.rs:248-272` | P2-005 | Resize: passar cursor em `lParam` e checar botão |
| 16 | P2-016 | BAIXA | P | `window.rs:316,325` | P2-001 | Coerência DWM × `SetWindowRgn` (raio < 8 px, sombra) |
| 17 | P2-017 | BAIXA | P | `window.rs:171,278,345` | nenhum | Logar erros de `create_dir_all`; avaliar `settings-data` |
| 18 | P2-018 | BAIXA | P | `diagnostics.rs:115-133,184-200` | nenhum | Corrida `WATCHDOG_ALERTED` × heartbeat |

## 4. Achados por Parte

### Parte 1 — Núcleo: Configuração, Estado e Regras de Negócio
*Não analisada nesta entrega.*

### Parte 2 — Integrações Windows, Arquivos e Diagnóstico (I/O)

#### P2-001 — Região arredondada (HRGN) não é recalculada ao redimensionar quando `remember_position = false`
- **Severidade:** ALTA
- **Categoria:** correcao-logica
- **Confiança:** PROVAVEL
- **Local:** `window.rs:347 save_window_state` (trecho `if !c.remember_position{return}`); `window.rs:309-334 apply_native_corners`; `main.rs:196-198` (`Moved|Resized → save_window_state`)
- **Evidência:**
  ```text
  window.rs:347  ...let mut c=match state.config.lock(){Ok(v)=>v,Err(_)=>return};
                 if !c.remember_position{return}
                 match e{ Moved(p)=>..., Resized(z)=>..., _=>return }
                 let copy=c.clone();drop(c);let _=crate::config::save(...);
                 if let Some(window)=app.get_webview_window("main"){apply_native_corners(&window,copy.corner_radius);}
  ```
  `apply_native_corners` é chamado **apenas no fim** da função, depois do `return` antecipado. Os únicos outros chamadores são `apply_config` (`window.rs:344`) e `refresh_native_corners` (`main.rs:205`, só em `ScaleFactorChanged`). `remember_position` tem padrão `true` (`config.rs:41`) e é desmarcável na UI (`settings.html:46`).
- **Reprodução:** (Windows) desmarcar "Lembrar posição e tamanho" → Salvar → arrastar a borda do widget para **aumentar** a janela. Esperado: toda a área visível. Provável: a região `SetWindowRgn` mantém o tamanho antigo e a parte nova fica recortada (invisível e sem entrada de mouse). Não reproduzível neste ambiente: app Windows-only.
- **Causa raiz:** a atualização da região de recorte está acoplada à rotina de persistência da posição; o retorno antecipado por preferência de persistência pula a atualização visual.
- **Impacto:** quem desativa "Lembrar posição" não consegue ampliar o widget de forma utilizável; sem erro visível.
- **Correção proposta:**
  1. Em `main.rs:196-198`, separar os eventos: `Resized` → sempre `window::refresh_native_corners(&handle)`; depois `save_window_state` (que só persiste).
  2. Remover `apply_native_corners` do fim de `save_window_state` (e evitar recalcular em `Moved`: o tamanho não muda).
  3. Extrair função pura `corner_region_dims(inner_w: u32, inner_h: u32, radius: u8, scale: f64) -> (i32, i32, i32)` (retorna `w, h, r` com o clamp de `window.rs:319-321`) para permitir teste unitário.
- **Critério de aceite:** com `remember_position=false`, após `Resized` a região tem o novo `inner_size` (verificar com `GetWindowRgn`/`GetRgnBox` em teste manual no Windows) e `cargo test` passa com o teste abaixo.
- **Teste a adicionar:** `corner_region_dims_follow_inner_size_and_clamp_radius` — valida `(w,h,r)` para `(300,200,12,1.0)`, `(0,0,12,1.0)` (r=0) e `(40,40,64,1.0)` (r = 20).
- **Depende de / Conflita com:** coordenar com P1 (refatoração de `save_window_state`, pista 4); P2-016.
- **Esforço:** M

#### P2-002 — Falha de leitura/parse de `config.json` vira defaults e é gravada por cima no startup
- **Severidade:** ALTA
- **Categoria:** integridade-de-dados
- **Confiança:** PROVAVEL
- **Local:** `config.rs:106-115 load`; `main.rs:137-141 setup`
- **Evidência:**
  ```rust
  // config.rs:107-110
  let mut config = fs::read(dir.join("config.json"))
      .ok()                                                       // erro de I/O descartado
      .and_then(|bytes| serde_json::from_slice::<AppConfig>(&bytes).ok())  // erro de parse descartado
      .unwrap_or_default();
  // main.rs:137,141
  let mut cfg = config::load(&dir);
  ...
  let _ = config::save(&dir, &cfg);   // grava sempre, inclusive os defaults
  ```
  `load` não distingue `NotFound` de `PermissionDenied`/`sharing violation`/JSON inválido, e `setup` grava o resultado incondicionalmente.
- **Reprodução:** (a) truncar `config.json` ou deixar um campo fora do tipo (ex.: `"opacity": 300`) e iniciar o app; (b) no Windows, deixar o arquivo momentaneamente aberto sem `FILE_SHARE_READ` por antivírus/backup durante o início. Esperado: preferências preservadas ou backup. Provável: `config.json` substituído pelos padrões, sem cópia. Não reproduzível neste ambiente: sem `cargo`.
- **Causa raiz:** `Option`/`ok()` colapsa três situações (arquivo ausente, ilegível, inválido) em uma; `save` incondicional no startup consolida a perda.
- **Impacto:** perda irrecuperável de todas as preferências (URL, atalho, tamanho, idioma…) após leitura transitória falha, corrupção parcial (ver P2-004) ou edição manual com erro.
- **Correção proposta:**
  1. `load` retorna um enum (`Loaded(cfg)`, `Missing`, `Unreadable(err)`, `Invalid(err)`).
  2. Em `Invalid`, renomear o arquivo para `config.json.corrupt-<timestamp>` antes de usar defaults; em `Unreadable`, **não** gravar nada e registrar `ERRO` no log.
  3. Em `main.rs:141`, só chamar `save` se `Missing` ou se `autostart` realmente mudou.
  4. Tolerância campo a campo (valores inválidos individuais) permanece com a Parte 1, pista 1.
- **Critério de aceite:** com `config.json` inválido, após `load` existe `config.json.corrupt-*` com o conteúdo original; com arquivo ilegível, `config.json` permanece byte a byte igual.
- **Teste a adicionar:** `load_backs_up_invalid_config_before_using_defaults` e `startup_does_not_overwrite_unreadable_config` (em `config.rs`, usando diretório temporário).
- **Depende de / Conflita com:** P1 pista 1 (mesma função); P2-004.
- **Esforço:** M

#### P2-003 — Build de release sem `windows_subsystem = "windows"`
- **Severidade:** ALTA
- **Categoria:** compatibilidade-plataforma
- **Confiança:** PROVAVEL
- **Local:** `main.rs:1` (arquivo começa em `mod commands;`); `diagnostics.rs:93-94` (comentário presume "GUI-subsystem builds"); `README.md:64`
- **Evidência:**
  ```text
  $ grep -rn windows_subsystem . --include=*.rs --include=*.toml --include=*.yml --include=*.json --include=*.md
  (nenhuma ocorrência; exit 1)
  ```
  `src-tauri/build.rs` contém apenas `tauri_build::build()`. O comentário em `diagnostics.rs:93-94` e o README (`README.md:64`: "Em builds gráficos do Windows, o terminal pode não estar disponível") assumem um build GUI que o código não declara.
- **Reprodução:** (Windows) executar o `.exe` instalado pelo NSIS e observar uma janela de console. Não reproduzível neste ambiente.
- **Causa raiz:** ausência do atributo padrão do template Tauri (`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`); o subsistema padrão do alvo MSVC é *console*.
- **Impacto:** uma janela de console aparece a cada início (inclusive no autostart); **fechar o console encerra o processo do widget**; habilita o cenário de P2-010.
- **Correção proposta:** inserir como **primeira linha** de `main.rs`:
  `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
- **Critério de aceite:** no artefato de release, o campo *Subsystem* do cabeçalho PE é `Windows GUI` (ex.: `dumpbin /headers widget-website.exe | findstr /i subsystem`) e nenhuma janela de console abre ao iniciar.
- **Teste a adicionar:** passo no CI (`build-windows.yml`) que lê o campo Subsystem do PE do executável gerado e falha se for `Windows CUI`.
- **Depende de / Conflita com:** nenhum (habilita P2-010).
- **Esforço:** P

#### P2-004 — `config::save`: gravação sem `fsync`, temporário fixo não limpo e fallback não atômico
- **Severidade:** MÉDIA
- **Categoria:** integridade-de-dados
- **Confiança:** PROVAVEL
- **Local:** `config.rs:117-131 save`
- **Evidência:**
  ```rust
  let temp = dir.join("config.json.tmp");              // nome fixo
  fs::write(&temp, serde_json::to_vec_pretty(config)?) // sem sync_all
  fs::rename(&temp, dir.join("config.json"))
      .or_else(|_| { fs::copy(&temp, dir.join("config.json"))?; fs::remove_file(&temp) }) // copy trunca o destino
  ```
- **Reprodução:** reprodução por queda de energia/encerramento forçado logo após salvar; ou injeção de falha entre `copy` e o fim. Não reproduzível neste ambiente.
- **Causa raiz:** `fs::write` não força dados ao disco antes do `rename`; em perda de energia o NTFS pode concluir a renomeação com o arquivo de dados vazio/parcial. O `copy` de fallback trunca o destino antes de escrever. Em erro, o `.tmp` permanece.
- **Impacto:** `config.json` vazio/truncado → próximo início cai em P2-002 e perde tudo. O `save` é chamado a cada `Moved`/`Resized` (`window.rs:347`), então a janela de risco é frequente.
- **Correção proposta:** `File::create(tmp)` → `write_all` → `sync_all()` → `rename` (a `std::fs::rename` substitui destino existente no Windows; confirmar na documentação do toolchain usado). Remover o fallback `copy`+`remove`, ou substituí-lo por `ReplaceFileW`. Em qualquer erro, `remove_file(tmp)` (melhor esforço). Nome do temporário único por chamada (pid + contador) — coordenar com P1 pista 6 (concorrência).
- **Critério de aceite:** após falha induzida, `config.json` anterior permanece íntegro e não sobra `config.json.tmp`.
- **Teste a adicionar:** `save_roundtrip_leaves_no_temp_file` e `save_failure_keeps_previous_config` (destino é diretório → `save` falha e o arquivo anterior permanece).
- **Depende de / Conflita com:** P2-002; P1 pista 6.
- **Esforço:** M

#### P2-005 — Arraste nativo com `SendMessageW` (laço modal) dentro do callback `on_navigation`
- **Severidade:** MÉDIA
- **Categoria:** concorrencia
- **Confiança:** PROVAVEL (bloqueio do callback); HIPOTESE (arraste "grudado" se o botão já foi solto)
- **Local:** `window.rs:190` (`on_navigation → begin_native_drag`); `window.rs:215-240 begin_native_drag` (`SendMessageW` em `window.rs:232`)
- **Evidência:**
  ```rust
  unsafe {
      ReleaseCapture();
      SendMessageW(raw_hwnd.0, WM_NCLBUTTONDOWN, HTCAPTION as usize, packed_position); // só retorna quando o botão é solto
  }
  ```
  A ação é disparada por `location.href='widget-action://drag'` (`window.rs:45`) no JS injetado, ou seja, chega via `NavigationStarting` do WebView2, tratado na thread de UI. `begin_native_drag` não verifica se o botão esquerdo ainda está pressionado.
- **Reprodução:** (Windows) caso 1: arrastar por vários segundos e observar o log: o `NavigationStarting` só conclui quando o arraste termina. Caso 2: clicar muito rápido na alça superior quando o caminho `startDragging()` do JS falha e o fallback `action('drag')` é usado. Não reproduzível neste ambiente.
- **Causa raiz:** `SendMessageW` na mesma thread da janela invoca o `WndProc` de forma síncrona e o `DefWindowProc` entra no laço modal de movimento; usar isso dentro de um handler COM do WebView2 deixa o evento "em andamento" por toda a duração do arraste. A implementação do Tauri/tao para `start_dragging` usa `PostMessageW` (verificar no código do `tao` instalado).
- **Impacto:** reentrância (eventos `widget-diagnostic://` e `Moved` → `config::save` ocorrem dentro do laço); se o botão já foi solto, a janela fica "colada" ao cursor até o próximo clique.
- **Correção proposta:** (1) trocar `SendMessageW` por `PostMessageW` na mesma chamada; (2) antes de chamar, checar `GetAsyncKeyState(VK_LBUTTON) as u16 & 0x8000 != 0` (feature `Win32_UI_Input_KeyboardAndMouse` já habilitada em `Cargo.toml`); se solto, retornar `Err`/ignorar; (3) extrair `pack_lparam(x: i32, y: i32) -> isize` como função pura testável.
- **Critério de aceite:** durante o arraste, `on_navigation` retorna imediatamente (log mostra "retorno" antes do fim do arraste); clique rápido sem manter o botão não move/gruda a janela.
- **Teste a adicionar:** `pack_lparam_preserves_negative_coordinates` (`(-1920,300)` → `GET_X/Y` recuperam o par) — o resultado já foi validado por simulação (seção 2).
- **Depende de / Conflita com:** Parte 3 (`on_navigation`); T-001.
- **Esforço:** M

#### P2-006 — Rotação do log: `remove_file` antes do `rename`; rotação que falha deixa o log crescer sem limite
- **Severidade:** MÉDIA
- **Categoria:** recursos-memoria
- **Confiança:** PROVAVEL
- **Local:** `diagnostics.rs:80-84 log`
- **Evidência:**
  ```rust
  if fs::metadata(&path).map(|m| m.len() >= MAX_LOG_BYTES).unwrap_or(false) {
      let backup = dir.join("diagnostics.log.1");
      let _ = fs::remove_file(&backup);        // apaga o backup ANTES de garantir a rotação
      let _ = fs::rename(&path, &backup);      // erro ignorado
  }
  ```
- **Reprodução:** (Windows) abrir `diagnostics.log` com um leitor que não compartilha exclusão (`Get-Content -Wait`, ferramentas de tail, alguns antivírus) até passar de 1 MiB: o `rename` falha; o backup antigo já foi removido; o arquivo continua crescendo. Não reproduzível em Linux (rename de arquivo aberto é permitido).
- **Causa raiz:** ordem remove→rename não é atômica e a falha é silenciosa; não há teto alternativo. O backup é desnecessariamente removido (a `rename` da std substitui o destino no Windows — confirmar).
- **Impacto:** perda do log mais antigo e crescimento ilimitado quando a rotação falha; durante um congelamento (justamente quando mais se escreve) o diagnóstico fica cada vez mais pesado. Duas instâncias simultâneas (a 2ª loga em `main.rs:82-86` antes de o `single_instance` encerrá-la em `main.rs:104`) podem competir pela rotação.
- **Correção proposta:** remover o `remove_file`; tratar o resultado do `rename`; se falhar e o arquivo exceder um teto rígido (ex.: 4 × `MAX_LOG_BYTES`), truncar (`OpenOptions::truncate(true)`) ou parar de gravar e escrever um aviso único. Mover `create_dir_all` e `metadata` para checagem periódica (a cada N linhas) em vez de a cada linha.
- **Critério de aceite:** com o arquivo preso por outro processo, o log nunca excede o teto rígido e o backup anterior é preservado.
- **Teste a adicionar:** `rotation_replaces_existing_backup` e (`#[cfg(windows)]`) `rotation_failure_does_not_grow_unbounded` abrindo o log com `OpenOptionsExt::share_mode(1 | 2)` (sem `FILE_SHARE_DELETE`).
- **Depende de / Conflita com:** nenhum.
- **Esforço:** M

#### P2-007 — Log grava `site_url` completo (inclusive credenciais e query) e o path inteiro das navegações
- **Severidade:** MÉDIA
- **Categoria:** seguranca
- **Confiança:** PROVAVEL
- **Local:** `main.rs:179-181` (`"Janela principal criada. URL configurada: {}"`, `cfg.site_url`); `diagnostics.rs:110 log_navigation` (`url.path()`); `README.md:63`
- **Evidência:**
  ```rust
  // main.rs:179-181
  format!("Janela principal criada. URL configurada: {}", cfg.site_url)
  // README.md:63: "URLs são registradas sem parâmetros de consulta ou fragmentos para reduzir o risco de guardar tokens..."
  ```
  `site_url` é texto livre do usuário (validado apenas como URL em `normalize`, Parte 1) e pode conter `https://usuario:senha@host/?token=...#...`.
- **Reprodução:** salvar `site_url = https://u:p@example.com/x?token=abc#f` e iniciar; `diagnostics.log` contém a URL inteira. Não reproduzível aqui (sem `cargo`), mas a linha de código é inequívoca.
- **Causa raiz:** o filtro de query/fragmento existe só em `log_navigation`; esta mensagem usa `format!` direto. O `path` também pode carregar tokens (ex.: `/reset/<token>`).
- **Impacto:** vazamento de segredos em arquivo em texto puro (e em `stdout`) e contradição documentada.
- **Correção proposta:** `diagnostics::sanitize_url(&Url) -> String` (esquema + host + porta; sem userinfo, query, fragmento; path truncado a N segmentos/caracteres) e usá-la em todos os logs de URL.
- **Critério de aceite:** nenhum log contém `user:`, `?`, `#` nem tokens do path longo; README e comportamento coincidem.
- **Teste a adicionar:** `site_url_log_omits_credentials_query_and_fragment` (log de `https://u:p@example.com/a?token=x#f` não contém `u:p`, `token`, `#f`).
- **Depende de / Conflita com:** T-003.
- **Esforço:** P

#### P2-008 — Watchdog chama métodos de janela do Tauri a partir de thread secundária
- **Severidade:** MÉDIA
- **Categoria:** concorrencia
- **Confiança:** HIPOTESE
- **Local:** `diagnostics.rs:169-170` (`window.is_visible()`); `diagnostics.rs:217` (`window.hwnd()` em `native_window_status`)
- **Evidência:** ambos são chamados em `thread::Builder::spawn` ("widget-webview-watchdog", `diagnostics.rs:161-163`), não na thread de UI.
- **Reprodução:** bloquear a thread de UI (ex.: laço modal longo ou `sleep` injetado em um handler) e verificar se o watchdog continua escrevendo `WARN`. **Verificação necessária:** ler o macro `getter!`/`window_getter!` do `tauri-runtime-wry` instalado; se usar `rx.recv()` sem timeout, a hipótese se confirma.
- **Causa raiz:** em Tauri, getters de janela chamados fora da thread principal despacham uma mensagem para a thread de UI e aguardam a resposta.
- **Impacto:** se a thread de UI travar, o watchdog trava junto no `is_visible()` e **nenhum aviso é gravado** — exatamente o cenário que o diagnóstico deveria registrar.
- **Correção proposta:** no `setup`, guardar `hwnd.0 as isize` e a visibilidade em atômicos (atualizados por `WindowEvent`/`toggle`); no watchdog usar diretamente `IsWindowVisible(hwnd)`/`IsHungAppWindow(hwnd)` (APIs Win32 seguras para outras threads).
- **Critério de aceite:** com a thread de UI bloqueada por ≥ 30 s, o log registra `WARN` "Sem sinal..." e "A janela nativa também está sem resposta".
- **Teste a adicionar:** manual no Windows (sem teste unitário viável); opcional: extrair a decisão "visível/oculto" para função pura.
- **Depende de / Conflita com:** P2-012.
- **Esforço:** M

#### P2-009 — Pré-checagem do WebView2 pode abortar o app com falso negativo
- **Severidade:** MÉDIA
- **Categoria:** compatibilidade-plataforma
- **Confiança:** PROVAVEL
- **Local:** `main.rs:13-35 webview2_runtime_present`; `main.rs:90-98` (`return` após `FATAL`)
- **Evidência:** procura `msedgewebview2.exe` somente em `%ProgramFiles(x86)%`, `%ProgramFiles%` e `%LOCALAPPDATA%` `\Microsoft\EdgeWebView\Application[\<versão>]`. Não consulta o registro (`...\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}`) nem `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`, ambos suportados pelo WebView2.
- **Reprodução:** (Windows) definir `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` para um runtime de versão fixa fora dessas pastas: a checagem falha e o app sai antes de tentar iniciar. Não reproduzível neste ambiente.
- **Causa raiz:** a detecção é uma heurística de caminho e é bloqueante; qualquer falha de heurística impede o uso mesmo com runtime funcional.
- **Impacto:** usuários com runtime de versão fixa, layout atípico ou política corporativa ficam sem acesso; a mensagem diz "não foi encontrado" mesmo quando existe.
- **Correção proposta:** tornar a checagem **informativa** (logar `WARN` e continuar); se `create_main`/`setup` falharem, exibir a mensagem de WebView2 ausente (ver P2-011). Opcional: adicionar detecção por registro (feature `Win32_System_Registry`) e honrar `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`.
- **Critério de aceite:** com runtime em local não padrão e variável de ambiente definida, o app inicia; sem runtime algum, o usuário vê a mensagem com o link oficial.
- **Teste a adicionar:** extrair `runtime_candidate_dirs(env) -> Vec<PathBuf>` e testar `webview2_runtime_search_includes_env_override_and_standard_locations`.
- **Depende de / Conflita com:** P2-011.
- **Esforço:** M

#### P2-010 — Escrita síncrona em `stdout` fora do `LOG_LOCK`, na thread de UI
- **Severidade:** MÉDIA
- **Categoria:** concorrencia
- **Confiança:** HIPOTESE
- **Local:** `diagnostics.rs:95-98 log` (bloco `stdout`); chamado a partir de `on_navigation` (`window.rs:174-211`)
- **Evidência:**
  ```rust
  let stdout = std::io::stdout();
  let mut output = stdout.lock();
  let _ = output.write_all(line.as_bytes());   // bloqueante
  let _ = output.flush();
  ```
- **Reprodução:** (Windows, com P2-003 não corrigido) clicar na janela de console com o modo de seleção rápida ativo (QuickEdit) pausa a gravação no console; ou iniciar com `stdout` redirecionado para um pipe que ninguém lê. A thread que loga bloqueia — inclusive a de UI via `on_navigation`. **Verificação necessária:** testar no Windows 10 (conhost) com um build sem o atributo.
- **Causa raiz:** `write_all` em console/pipe é bloqueante e acontece depois de soltar `LOG_LOCK`, mas na mesma thread chamadora.
- **Impacto:** congelamento do widget causado pelo próprio diagnóstico; log de linhas intercaladas entre threads no console.
- **Correção proposta:** gravar em `stdout` apenas em `cfg!(debug_assertions)` (ou se houver console anexado) e nunca no caminho da thread de UI; alternativamente, usar um canal para uma thread de log dedicada.
- **Critério de aceite:** em release, nenhuma chamada a `stdout`; selecionar texto em um console anexado não congela o widget.
- **Teste a adicionar:** manual; unitário opcional `log_does_not_write_stdout_in_release_cfg` via função `should_mirror_to_stdout()`.
- **Depende de / Conflita com:** P2-003.
- **Esforço:** P

#### P2-011 — Erros no `setup` podem virar `panic` do Tauri em vez de `Err` de `run()`
- **Severidade:** MÉDIA
- **Categoria:** correcao-logica
- **Confiança:** HIPOTESE
- **Local:** `main.rs:131-222` (`setup` com `?` em `create_dir_all`, `create_main`, `tray::setup`); `main.rs:225-240` (único ponto que exibe `show_message` de erro)
- **Evidência:** `main.rs:133-134` e `main.rs:174` propagam erro com `?` dentro do closure de `setup`, e a caixa "erro ao iniciar" só existe em `main.rs:225-240`, após `.run(...)`.
- **Reprodução:** (Windows) tornar `%APPDATA%\WidgetWebsite` somente leitura ou iniciar sem o WebView2 e observar se a caixa de erro aparece. **Verificação necessária:** procurar no `tauri` instalado (`App::run`) a mensagem `Failed to setup app` — se o `setup` for executado no evento `Ready` e o erro causar `panic!`, o retorno `Err` nunca chega a `main.rs:225`.
- **Causa raiz:** erro do hook `setup` não necessariamente retorna de `Builder::run`.
- **Impacto:** o usuário vê o processo desaparecer (ou só o log, com P2-003 corrigido não há console) sem a mensagem prometida.
- **Correção proposta:** envolver o corpo do `setup` num closure interno; em `Err`, registrar `FATAL`, chamar `show_message` e então devolver o erro.
- **Critério de aceite:** pasta de dados somente leitura → caixa de diálogo com o caminho do log e linha `FATAL` no log.
- **Teste a adicionar:** manual no Windows; extrair `fn report_fatal(dir, error)` e testar que grava `FATAL` (`report_fatal_writes_fatal_line`).
- **Depende de / Conflita com:** P2-009, P2-013.
- **Esforço:** M

#### P2-012 — Falsos alertas do watchdog após reexibir a janela ou em página de erro
- **Severidade:** BAIXA
- **Categoria:** correcao-logica
- **Confiança:** HIPOTESE
- **Local:** `diagnostics.rs:164-201` (laço do watchdog); `main.rs:62-77 toggle/show_existing_instance`; heartbeat no JS injetado (`window.rs:107-109`)
- **Evidência:** o watchdog só renova `LAST_HEARTBEAT_MS` enquanto `is_visible() == false` (`diagnostics.rs:170-174`). Quando a janela é exibida, nada reinicia a carência; o heartbeat depende de `setInterval(heartbeat,5000)` no renderer, que o Chromium pode atrasar em página oculta por muito tempo (throttling intensivo) e que não roda em páginas de erro internas do Chromium (offline).
- **Reprodução:** (Windows) ocultar por > 10 min, reexibir e acompanhar o log por 60 s; desligar a rede e recarregar. **Verificação:** o WARN aparece sem congelamento real?
- **Causa raiz:** carência atrelada ao tick de 5 s, não ao evento de exibição; heartbeat acoplado a script de página.
- **Impacto:** `WARN` enganoso a cada 60 s (`REPEAT_WARNING_MS`) que dilui o diagnóstico.
- **Correção proposta:** `diagnostics::reset_grace()` (define `LAST_HEARTBEAT_MS = now`, zera `LAST_WARNING_MS`/`WATCHDOG_ALERTED`) chamado em `toggle`, `show_existing_instance` e na bandeja ao mostrar; omitir o aviso quando a página atual for de erro (Parte 3/4).
- **Critério de aceite:** nenhum `WARN` nos 20 s seguintes a uma reexibição normal.
- **Teste a adicionar:** `reset_grace_clears_alert_state` (sem janela: apenas atômicos).
- **Depende de / Conflita com:** P2-008.
- **Esforço:** P

#### P2-013 — `app_directory`: fallback para o diretório de trabalho e `APPDATA` vazio
- **Severidade:** BAIXA
- **Categoria:** validacao-entrada
- **Confiança:** PROVAVEL
- **Local:** `config.rs:99-104`; efeito em `main.rs:133-134` (`create_dir_all(...)?`)
- **Evidência:**
  ```rust
  std::env::var_os("APPDATA").map(PathBuf::from)
      .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
      .join("WidgetWebsite")
  ```
  `APPDATA=""` produz o caminho relativo `WidgetWebsite`; sem `APPDATA`, usa `current_dir()` (no autostart via `Run`, normalmente `System32`).
- **Reprodução:** `set APPDATA=` e iniciar; ou iniciar a partir de uma pasta qualquer. O resultado depende do diretório de trabalho (config diferente por local de execução).
- **Causa raiz:** fallback não determinístico, sem validação de caminho absoluto/gravável.
- **Impacto:** config "sumindo" conforme o diretório de execução; `create_dir_all` falha em pasta protegida → encerra (ver P2-011).
- **Correção proposta:** exigir caminho não vazio e absoluto; fallback para `USERPROFILE\AppData\Roaming` e por fim `std::env::temp_dir()`; registrar o fallback usado.
- **Critério de aceite:** `APPDATA` vazio/ausente nunca gera caminho relativo nem usa `System32`.
- **Teste a adicionar:** extrair `resolve_app_dir(appdata: Option<OsString>, userprofile: Option<OsString>, temp: PathBuf)`; testar `app_directory_rejects_empty_and_relative_appdata`.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

#### P2-014 — Mensagens de log com conteúdo controlado pela página, sem limite de tamanho ou taxa
- **Severidade:** BAIXA
- **Categoria:** recursos-memoria
- **Confiança:** PROVAVEL
- **Local:** `window.rs:186` e `window.rs:197` (`{action}` = host de `widget-action://…`); `diagnostics.rs:110` (`url.path()`); `diagnostics.rs:72-99 log`
- **Evidência:** `window.rs:197` loga `format!("Ação de janela desconhecida: {action}")` e `window.rs:186` o mesmo com a janela travada, com `action = u.host_str()`. Qualquer script da página pode fazer `location.href='widget-action://<texto qualquer>'` em laço (ver T-001). `log` não trunca nem limita taxa.
- **Reprodução:** script de página em laço apertado; observar `diagnostics.log` rotacionar com lixo. Não reproduzível aqui.
- **Causa raiz:** entrada externa é interpolada em log sem teto.
- **Impacto:** linhas gigantes e rotação rápida expulsam as linhas relevantes; custo de I/O na thread de UI.
- **Correção proposta:** em `log`, truncar a mensagem (ex.: 1.024 caracteres), substituir caracteres de controle e aplicar limite por categoria (ex.: 20 linhas/s).
- **Critério de aceite:** 10.000 chamadas com `action` de 100 KB geram < 1 MiB de log.
- **Teste a adicionar:** `log_truncates_long_messages_and_strips_control_chars`.
- **Depende de / Conflita com:** P2-006; T-001.
- **Esforço:** P

#### P2-015 — Redimensionamento nativo: `lParam = 0` e sem checagem do botão
- **Severidade:** BAIXA
- **Categoria:** compatibilidade-plataforma
- **Confiança:** HIPOTESE
- **Local:** `window.rs:269` (`SendMessageW(hwnd, WM_NCLBUTTONDOWN, hit_test as usize, 0)`)
- **Evidência:** ao contrário de `begin_native_drag` (`window.rs:227-228`), o resize não passa a posição do cursor em `lParam`; não verifica `VK_LBUTTON`. Também usa `SendMessageW` (ver P2-005).
- **Reprodução:** (Windows) redimensionar pelas bordas de 6 px em monitores com DPI misto e comparar o ponto inicial do arraste. **Verificação:** o `DefWindowProc` usa o `lParam` como ponto inicial do dimensionamento.
- **Causa raiz:** implementação incompleta do contrato de `WM_NCLBUTTONDOWN` (coordenadas de tela em `lParam`).
- **Impacto:** salto de tamanho no início do arraste ou comportamento inconsistente; mesmo risco de "grudar" do P2-005.
- **Correção proposta:** reutilizar `pack_lparam` com `GetCursorPos`; `PostMessageW`; checar o botão.
- **Critério de aceite:** o redimensionamento inicia no ponto exato do cursor em 100%, 125% e 150% de escala.
- **Teste a adicionar:** manual; reutiliza `pack_lparam_preserves_negative_coordinates`.
- **Depende de / Conflita com:** P2-005.
- **Esforço:** P

#### P2-016 — Região de recorte e preferência de cantos do DWM sobrepostas
- **Severidade:** BAIXA
- **Categoria:** ui-ux
- **Confiança:** HIPOTESE
- **Local:** `window.rs:315-316` (`DWMWA_WINDOW_CORNER_PREFERENCE`: 2 = arredondar); `window.rs:325` (`CreateRoundRectRgn(0,0,w+1,h+1,r*2,r*2)`); `window.rs:344` (`set_shadow(c.corner_radius>0)`); `README.md:72`
- **Evidência:** para `radius > 0` o código pede arredondamento ao DWM (≈ 8 px no Windows 11) **e** aplica uma região com raio `r`; a forma final é a interseção. Para `r < 8`, o DWM domina. A região recorta a sombra DWM habilitada por `set_shadow(true)`. O `README.md:72` já admite variação no Windows 10.
- **Reprodução:** (Windows 11) raio 2, 4, 12 px; verificar o raio visível. (Windows 11) conferir se a sombra aparece. 100/125/150 % de escala.
- **Causa raiz:** dois mecanismos de arredondamento ativos ao mesmo tempo e região baseada em `inner_size`.
- **Impacto:** raios pequenos configurados não têm efeito no Windows 11; sombra recortada; discrepâncias em DPI fracionário.
- **Correção proposta:** usar `DWMWCP_DONOTROUND` (1) sempre que houver região própria; ou usar só DWM e descartar a região; documentar a escolha.
- **Critério de aceite:** raio visível = raio configurado (±1 px) em 100/125/150 %.
- **Teste a adicionar:** manual (captura de tela); unitário já coberto por `corner_region_dims_*` (P2-001).
- **Depende de / Conflita com:** P2-001.
- **Esforço:** P

#### P2-017 — Erros de criação de diretórios engolidos; `settings-data` cria um segundo ambiente WebView2
- **Severidade:** BAIXA
- **Categoria:** recursos-memoria
- **Confiança:** HIPOTESE
- **Local:** `window.rs:171` (`create_dir_all(&data).ok()`), `window.rs:278` (idem para login), `window.rs:345` (`settings-data`); `main.rs:141` (`let _ = config::save`); `diagnostics.rs:289-297` (contagem de `msedgewebview2.exe`)
- **Evidência:** `.ok()`/`let _ =` descartam o erro; a janela de Configurações é uma página local mas usa `data_directory(app_dir/settings-data)`, diferente de `webview-data`. Dois diretórios de usuário distintos no mesmo processo implicam dois ambientes WebView2 (processos adicionais) e `webview2_descendant_process_status` conta todos os descendentes, mascarando o "nenhum processo" da janela principal.
- **Reprodução:** (Windows) abrir Configurações e contar processos `msedgewebview2.exe` no Gerenciador de Tarefas; comparar com o log do watchdog.
- **Causa raiz:** decisões de isolamento sem log; contador sem distinguir a janela de origem.
- **Impacto:** consumo extra de memória enquanto Configurações está aberta; diagnóstico menos preciso; falhas de permissão sem rastro até um erro obscuro do WebView2.
- **Correção proposta:** registrar `WARN` quando `create_dir_all` falhar; avaliar remover `settings-data` (usar o mesmo `webview-data` ou perfil efêmero); no watchdog, contar apenas processos cujo `--user-data-dir` coincide com `webview-data` (ou documentar a limitação).
- **Critério de aceite:** com Configurações aberta, o número de árvores de processo WebView2 não aumenta (ou a decisão fica documentada).
- **Teste a adicionar:** manual.
- **Depende de / Conflita com:** Parte 3 (`open_settings`).
- **Esforço:** P

#### P2-018 — Corrida entre `record_heartbeat` e o watchdog (`WATCHDOG_ALERTED`/`LAST_WARNING_MS`)
- **Severidade:** BAIXA
- **Categoria:** concorrencia
- **Confiança:** PROVAVEL
- **Local:** `diagnostics.rs:115-133 record_heartbeat`; `diagnostics.rs:178-200` (watchdog)
- **Evidência:** o watchdog lê `LAST_HEARTBEAT_MS` (l.178), executa `native_window_status` e `log` (I/O, podem levar centenas de ms) e só depois grava `LAST_WARNING_MS` e `WATCHDOG_ALERTED = true` (l.199-200). Se um heartbeat chegar nesse intervalo, `swap(ALERTED, false)` não vê o alerta e, em seguida, o watchdog o marca `true` com o heartbeat já recente.
- **Reprodução:** teste de estresse com heartbeat concorrente (não reproduzível aqui).
- **Causa raiz:** estado composto (3 atômicos) atualizado sem sincronização conjunta.
- **Impacto:** linha `RECUPERADO` espúria no próximo heartbeat; sem perda de dados.
- **Correção proposta:** agrupar o estado num `Mutex<WatchdogState>` ou gravar `ALERTED` **antes** de logar e revalidar `LAST_HEARTBEAT_MS` com `compare_exchange`.
- **Critério de aceite:** sob teste concorrente (10⁵ iterações de `record_heartbeat` × decisão do watchdog), `RECUPERADO` só é registrado se um `WARN` foi emitido e o heartbeat é posterior a ele.
- **Teste a adicionar:** `watchdog_state_transitions_are_consistent` (extrair a decisão para função pura `evaluate(now, last, alerted, last_warning) -> Action`).
- **Depende de / Conflita com:** P2-008, P2-012.
- **Esforço:** P

### Parte 3 — Camada de Aplicação: Ciclo de Vida, Janelas, Navegação e Bandeja
*Não analisada nesta entrega.*

### Parte 4 — Interface e Superfície de Entrada Externa
*Não analisada nesta entrega.*

### Parte 5 — Infraestrutura: Build, CI/CD, Dependências e Distribuição
*Não analisada nesta entrega.*

## 5. Achados Transversais (T-001…)
*(Somente os que tocam a Parte 2; a revisão cruzada completa depende das demais partes.)*

- **T-001 — `widget-action://` e `widget-diagnostic://` aceitos de qualquer origem carregada na janela principal** (Partes 2, 3, 4). `on_navigation` (`window.rs:174-200`) não verifica a origem do documento: qualquer página/iframe pode disparar `drag`/`resize` (P2-005, P2-015), forjar heartbeats (silenciando o watchdog) e inundar o log (P2-014). Severidade provável: MÉDIA; confiança `HIPOTESE` até verificar se `on_navigation` é chamado para subframes. Responsável: Parte 4.
- **T-002 — `config.json` escrito por vários caminhos** (Partes 1, 2, 3). `setup` (`main.rs:141`), `save_window_state` (`window.rs:347`, a cada `Moved/Resized`, na thread de UI) e comandos/bandeja usam o mesmo `config::save` com `.tmp` fixo; P2-002 e P2-004 amplificam o risco de perda. Resolver junto com P1 pistas 4 e 6.
- **T-003 — Logs × README** (Partes 2 e 5). `README.md:63` (sem query/fragmento) × `main.rs:179-181` (P2-007); `README.md:64` (build GUI) × ausência do atributo (P2-003).
- **T-004 — Custo por evento de `Moved/Resized`** (Partes 1 e 2): `save_window_state` faz `config::save` + `apply_native_corners` + `SetWindowRgn(..., redraw=1)` a cada evento (`window.rs:347`), aumentando o custo durante o arraste (P2-001, P2-004, P2-005).

## 6. Cobertura de Testes Ausente
Testes existentes revisados (`diagnostics.rs:318-368`): `formats_utc_timestamp_from_unix_epoch`, `formats_leap_day_and_modern_year_correctly` (vetores conferidos por simulação), `writes_a_persistent_diagnostic_line`, `navigation_log_omits_query_and_fragment`. **Não executados** (sem `cargo`). Observação: os dois últimos usam diretórios temporários distintos por `process::id()` e não conflitam entre si.

Cenários críticos sem teste (nomes sugeridos):
- `corner_region_dims_follow_inner_size_and_clamp_radius` (P2-001)
- `load_backs_up_invalid_config_before_using_defaults`, `startup_does_not_overwrite_unreadable_config` (P2-002)
- `save_roundtrip_leaves_no_temp_file`, `save_failure_keeps_previous_config` (P2-004)
- pack_lparam: `pack_lparam_preserves_negative_coordinates` (P2-005, P2-015)
- `rotation_replaces_existing_backup`, `rotation_failure_does_not_grow_unbounded` (`#[cfg(windows)]`) (P2-006)
- `site_url_log_omits_credentials_query_and_fragment` (P2-007)
- `webview2_runtime_search_includes_env_override_and_standard_locations` (P2-009)
- `report_fatal_writes_fatal_line` (P2-011)
- `reset_grace_clears_alert_state`, `watchdog_state_transitions_are_consistent` (P2-012, P2-018)
- `app_directory_rejects_empty_and_relative_appdata` (P2-013)
- `log_truncates_long_messages_and_strips_control_chars`, log concorrente com várias threads (P2-014)
- Sem testes para `main.rs`, `webview2_descendant_process_status` e FFI Win32 (somente manual no Windows).

## 7. Falsos Positivos e Itens Verificados Sem Problema
Pistas iniciais da Parte 2 e resultado:

1. **Log × README (pista 1): CONFIRMADA** (P2-007). `log_navigation` realmente omite query e fragmento (teste em `diagnostics.rs:352-368`); o `path` inteiro permanece (parcial).
2. **`diagnostics::log` (pista 2): CONFIRMADA em parte** — `stdout` fora do `LOG_LOCK` (P2-010); rotação com `remove_file` + `rename` sem tratamento (P2-006). O exemplo "Bloco de Notas" é **improvável**: o Bloco de Notas lê e libera o arquivo; os casos reais são leitores de tail/antivírus. Abrir/fechar o arquivo a cada linha é aceitável dada a baixa frequência (o heartbeat não loga, `diagnostics.rs:115-133`).
3. **`webview2_runtime_present` (pista 3): CONFIRMADA** (P2-009).
4. **`app_directory` (pista 4): CONFIRMADA** (P2-013). Efeito em `create_dir_all(...)?` descrito em P2-011/P2-013.
5. **`begin_native_drag` (pista 5): PARCIALMENTE CONFIRMADA** — bloqueio por `SendMessageW` (P2-005). **Refutado:** o empacotamento `cursor.x as u16 as u32 | (cursor.y as u16 as u32) << 16` está correto com coordenadas negativas (simulação: `(-1920,300)`, `(2500,-1080)`, `(-32768,32767)`, `(-1,-1)` recuperados exatamente por `GET_X/Y_LPARAM`); a conversão para `isize` é segura. **Tipo de `raw_hwnd.0`:** compila quando `windows` (Tauri) e `windows-sys 0.59` usam ponteiro (`*mut c_void`); acoplamento frágil, sem erro hoje. `HIPOTESE`: validar com `cargo build` no Windows (CI já compila em `windows-latest`); mitigação opcional `window.hwnd()?.0 as windows_sys::Win32::Foundation::HWND`.
6. **`apply_native_corners` (pista 6): PARCIALMENTE REFUTADA** — **não há vazamento de HRGN**: `SetWindowRgn` assume a posse em caso de sucesso; em falha, `DeleteObject(region)` é chamado (`window.rs:326-329`); com `r == 0`, `SetWindowRgn(NULL)` (`window.rs:323`); `region.is_null()` é checado. O recálculo a cada `Moved` existe, mas `Moved` não altera o tamanho (custo, não vazamento: T-004). Problemas reais encontrados: P2-001 e P2-016.
7. **Watchdog (pista 7): PARCIALMENTE CONFIRMADA** — corrida menor (P2-018), carência pós-`show` (P2-012), chamadas fora da thread de UI (P2-008). **Refutado:** em `webview2_descendant_process_status` o snapshot é fechado em todos os caminhos (retorno antecipado só ocorre quando `CreateToolhelp32Snapshot` falhou, `diagnostics.rs:247-250`; o `CloseHandle` está em `:267-269` antes dos demais `return`); `Process32FirstW` falho resulta em lista vazia e mensagem explícita (`:271-273`); `dwSize` é inicializado (`:253`); o teste `snapshot as isize == -1` está correto para `HANDLE` ponteiro. `REPEAT_WARNING_MS` funciona como esperado (`:184-187`).
8. **Plugins (pista 8): REFUTADA em parte** — `MacosLauncher::LaunchAgent` é ignorado no Windows; `init(..., None)` não passa argumentos ao autostart (o app inicia normalmente); erros de inicialização de plugins propagam por `setup`/`run()` (mas ver P2-011 sobre como isso é exibido). `single_instance` está registrado primeiro (`main.rs:104`), como recomendado.

Itens verificados sem problema:
- `format_utc_timestamp` (algoritmo de Hinnant) correto, incluindo 29/02/2000 e 2026.
- `now_ms`/`utc_timestamp`: `unwrap_or_default` seguro para relógio anterior a 1970.
- `LOG_LOCK` com `.lock().ok()`: envenenamento praticamente impossível (nada dentro da seção crítica faz panic); o efeito seria parar de logar silenciosamente — apenas melhoria (seção 8).
- `install_panic_hook` encadeia o hook anterior e recalcula o diretório; não há re-entrância problemática sobre `LOG_LOCK`.
- `native_window_status`: `IsHungAppWindow` é a API pensada para chamada de outra thread.
- `show_message`: strings UTF-16 terminadas em NUL corretas.

## 8. Melhorias Opcionais (máx. 10 linhas)
- `config::load/save` recebem `&PathBuf` (usar `&Path`; alerta do `clippy::ptr_arg`).
- `LOG_LOCK.lock()` com `unwrap_or_else(|e| e.into_inner())` para nunca silenciar o log após envenenamento.
- `MessageBoxW` com `MB_SETFOREGROUND | MB_TOPMOST` para não abrir atrás de outras janelas no autostart.
- Registrar a versão do app e do runtime WebView2 em `Widget Website iniciando` (`main.rs:82-86`).
- `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` e `USERPROFILE` documentados no README.
- Adicionar `Cargo.lock`/`package-lock.json` e rodar `cargo audit` (Parte 5).

## 9. Checklist Final de Verificação Pós-Correção
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` passa (≥ 13 testes originais + os novos listados na seção 6).
- [ ] `npm run build` passa.
- [ ] PE do release: *Subsystem* = `Windows GUI`; nenhuma janela de console ao iniciar (P2-003).
- [ ] Com `remember_position=false`, redimensionar amplia a área visível (P2-001).
- [ ] `config.json` inválido gera `config.json.corrupt-*` e não é sobrescrito; `.tmp` não permanece (P2-002, P2-004).
- [ ] `diagnostics.log` não contém credenciais, query, fragmento nem path longo; README coerente (P2-007).
- [ ] Rotação do log com arquivo preso respeita o teto e preserva o backup (P2-006).
- [ ] Arraste/resize não bloqueiam `on_navigation` nem "grudam" a janela (P2-005, P2-015).
- [ ] Com a thread de UI bloqueada ≥ 30 s o watchdog ainda escreve `WARN` (P2-008).
- [ ] WebView2 de versão fixa/`WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` não é bloqueado; erro de `setup` mostra caixa e `FATAL` (P2-009, P2-011).
- [ ] `APPDATA` vazio/ausente não gera caminho relativo/`System32` (P2-013).
- [ ] Nenhum arquivo foi alterado além dos citados nas correções.
