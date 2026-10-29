# Relatório de Análise e Auditoria de Código — Widget Website (Tauri 2 / Windows)

> **ESCOPO DESTE ARQUIVO: SOMENTE A PARTE 4 de 5** — *Interface e Superfície de Entrada Externa (front-end, scripts injetados, IPC/ACL)*.
> Partes 1, 2, 3 e 5: **NÃO analisadas nesta sessão** (seções marcadas como pendentes). Achados que tocam outras partes aparecem como `T-xxx` (transversais) com a referência de parte.
> Repositório: `widget-website` v0.1.0 (`widget-website-main.zip`). Auditoria somente-leitura; nenhum arquivo do repositório foi alterado (trabalho feito numa cópia).

---

## 0. Instruções para a IA/Dev que fará as correções
- Corrija UM achado por vez, na ordem da seção 3.
- Após cada alteração, rode: `cargo test --manifest-path src-tauri/Cargo.toml` e `npm run build`.
- Adicione/ajuste os testes indicados; não remova testes sem justificativa.
- Em caso de conflito, siga o campo "Depende de / Conflita com".
- Achados `HIPOTESE` exigem verificação (idealmente em Windows) antes da correção. **P4-001 e P4-004 são `HIPOTESE`: execute o procedimento de verificação descrito neles antes de decidir o escopo da correção.**
- Todas as linhas citadas referem-se ao ZIP analisado. `window.rs`, `commands.rs`, `index.html`, `package.json` e `tsconfig.json` são densos/linha única: use também o trecho citado para localizar.

## 1. Resumo Executivo
- **Parte analisada:** 4 de 5. **Arquivos lidos por inteiro:** `index.html`, `settings.html`, `src/main.ts`, `src/settings.ts`, `src/styles.css`, `window.rs` linhas 1–141 (`INIT`, `LOGIN_INIT`) e o trecho de `on_navigation`/`on_new_window`/`open_login`/`apply_config` (142–347) para rastrear os esquemas customizados, `capabilities/default.json`, `capabilities/investing-drag.json`, `tauri.conf.json`, `build.rs`, `commands.rs`, `Cargo.toml` (para o contexto do IPC).
- **Linha de base de execução:** ver seção 2. Não foi possível instalar dependências (npm/crates.io bloqueados) nem executar Rust/Windows. Verificado por execução: sintaxe dos dois scripts injetados (`node --check`), lógica pura de `settings.ts` (réplica em Node) e cálculo de contraste WCAG.
- **Total: 16 achados** — **ALTA 4** (P4-001 `HIPOTESE`, P4-002, P4-006, P4-009), **MÉDIA 9**, **BAIXA 3**. Nenhum `CRITICA` pôde ser provado; **P4-001 pode subir a `CRITICA`** se o teste da seção do achado confirmar.
- **Top 5 riscos:**
  1. **P4-001** — comandos próprios do app (`save_config`, `quit_app`…) podem estar acessíveis a páginas remotas (investing.com/anúncios de terceiros e janelas de login em `*.google.com`/`*.facebook.com`) por falta de `AppManifest` — `HIPOTESE`, verificar primeiro.
  2. **P4-002** — o estado "travado", idioma, opacidade e raio dos cantos injetados na página remota **voltam ao valor da inicialização a cada recarga** (script de init é fixo; `apply_config` só faz `eval` no documento atual).
  3. **P4-006** — campo de atalho global: armadilha de teclado, aceita atalho sem modificador e, em teclado ABNT2 (AltGr = Ctrl+Alt), o próprio atalho padrão `Ctrl+Alt+C` é gravado como `CTRL+ALT+₢`.
  4. **P4-009** — "Salvar" em Configurações sobrescreve `x`/`y`/`locked`/`always_on_top` com valores velhos.
  5. **P4-003/P4-005** — canal `widget-action://` sem autenticação e trava (`locked`) não aplicada no caminho IPC `startDragging`.

## 2. Ambiente e Linha de Base
| Comando | Resultado |
|---|---|
| `unzip` do repositório | OK — 25 arquivos (33 citados no prompt incluem READMEs/ícones; sem `package-lock.json`, `Cargo.lock`, `LICENSE`). |
| `npm install` | **Falhou**: `ENOTFOUND registry.npmjs.org` (rede bloqueada na sandbox). Dependências não instaladas. |
| `npx tsc --noEmit` (tsc global 7.0.2, sem `node_modules`) | 4 erros: `TS2307` ×3 (módulos `@tauri-apps/api/core` e `vite` ausentes) e `TS2345` em `settings.ts:172` (`display(current)` com `C \| null`). **O `TS2345` é artefato da ausência dos tipos** (`invoke<C>` resolve como `any` e não estreita a atribuição); com `@tauri-apps/api` instalado esperado: sem erro. Marcado **não verificado**. |
| `npm run build` | Falhou pelos mesmos motivos (dependências). |
| `npm audit` | `ENOLOCK` (não há `package-lock.json`) + sem rede. **Não executado.** |
| `cargo test` / `clippy` / `audit` | **Não executados**: crates.io inacessível (`static.crates.io` → 403; `index.crates.io` não resolve) e o app é Windows-only. Contagem de testes Rust não verificada por execução. |
| `node --check` nos blocos `INIT` e `LOGIN_INIT` extraídos de `window.rs` (com placeholders substituídos) | **OK — sintaxe válida** (105 e 30 linhas). |
| Réplica em Node da lógica de `settings.ts` (keydown, `read()`, mapeamento de aviso) | Executada; saída citada em P4-006/007/008. |
| Cálculo de contraste WCAG 2.x dos pares de cor de `styles.css` | Executado; tabela em P4-015. |
| Documentação Tauri consultada | `v2.tauri.app/security/capabilities/`, `/security/runtime-authority/`, `/reference/config/`. **Código-fonte do Tauri/`global-hotkey` não acessível** (GitHub/raw bloqueados) → comportamento exato do ACL para comandos de app em origem remota e sintaxe de `Shortcut::from_str` ficam `HIPOTESE`/`PROVAVEL`. |

Limitações: sem Windows/WebView2, nenhum achado de comportamento de runtime do WebView2 é `CONFIRMADO` por execução; os marcados `CONFIRMADO` referem-se à lógica reproduzida em Node ou a cálculo.

## 3. Plano de Correção Ordenado (Parte 4)
| Ordem | ID | Severidade | Esforço | Arquivo(s) | Depende de | Resumo |
|---|---|---|---|---|---|---|
| 1 | P4-001 | ALTA (`HIPOTESE`; pode ser CRITICA) | M | `build.rs`, `capabilities/*.json`, `main.rs`, `commands.rs`, `tauri.conf.json` | — | Verificar e fechar o ACL de comandos de app para origens remotas |
| 2 | P4-002 | ALTA | M | `window.rs` (INIT, `create_main`, `apply_config`) | — | Estado injetado (travado/idioma/opacidade/raio) volta ao inicial a cada carga de página |
| 3 | P4-005 | MÉDIA | P | `window.rs` (INIT `dragWindow`, `on_navigation`) | P4-002 | `locked` não é aplicado no caminho IPC `startDragging` |
| 4 | P4-006 | ALTA | M | `settings.ts`, (`config.rs` normalize — Parte 1) | — | Captura de atalho: armadilha de teclado, sem modificador, `event.key`/AltGr |
| 5 | P4-009 | ALTA | M | `settings.ts`, `commands.rs` | T-001 | "Salvar" sobrescreve x/y/locked/always_on_top com estado velho |
| 6 | P4-007 | MÉDIA | M | `settings.ts`, `commands.rs`, `config.rs` | T-004 | Avisos/erros mapeados por texto pt-BR; aviso de autostart some |
| 7 | P4-008 | MÉDIA | P | `settings.html`, `settings.ts` | — | Campos vazios viram 0 silenciosamente |
| 8 | P4-003 | MÉDIA | M | `window.rs` (INIT, `on_navigation`) | P4-001 | `widget-action://` sem autenticação; resize ignora `locked` |
| 9 | P4-004 | MÉDIA (`HIPOTESE`) | M | `window.rs` INIT heartbeat | — | Heartbeat via navegação pode cancelar navegações em andamento |
| 10 | P4-013 | MÉDIA | P | `window.rs` INIT (resize) | P4-003 | Faixa de resize de 6 px ignora zoom e disputa com a barra de rolagem |
| 11 | P4-011 | MÉDIA (`HIPOTESE`) | M | `main.ts`, `window.rs` INIT | — | Sonda `no-cors` não prova carregamento; página de erro sem alça |
| 12 | P4-012 | MÉDIA | P | `window.rs` INIT | — | Bloqueio global de menu de contexto e atalhos |
| 13 | P4-015 | MÉDIA | P | `styles.css` | — | Contraste abaixo de WCAG AA |
| 14 | P4-010 | BAIXA | P | `settings.ts` | P4-009 | "Restaurar tamanho padrão" descarta edições não salvas |
| 15 | P4-014 | BAIXA | P | `window.rs` LOGIN_INIT/INIT | — | i18n fixo em PT, `close` sem nonce, barra cobre o site |
| 16 | P4-016 | BAIXA | P | `capabilities/default.json`, `tauri.conf.json` | P4-001 | Privilégio mínimo e CSP nulo |

---

## 4. Achados por Parte

### Parte 1 — Núcleo: Configuração, Estado e Regras de Negócio
**NÃO ANALISADA nesta sessão** (pendente). Ver `T-001`, `T-004`, `T-005` para pontos que cruzam esta parte.

### Parte 2 — Integrações Windows, Arquivos e Diagnóstico (I/O)
**NÃO ANALISADA nesta sessão** (pendente). Ver `T-003`.

### Parte 3 — Camada de Aplicação: Ciclo de Vida, Janelas, Navegação e Bandeja
**NÃO ANALISADA nesta sessão** (pendente). Ver `T-001`, `T-002`, `T-003`.

### Parte 4 — Interface e Superfície de Entrada Externa (front-end, scripts injetados, IPC/ACL) — **ANALISADA**

#### P4-001 — Comandos próprios do app possivelmente invocáveis por páginas remotas (ACL sem `AppManifest`)
- **Severidade:** ALTA (sobe a CRITICA se confirmado)
- **Categoria:** seguranca
- **Confiança:** HIPOTESE (não foi possível ler o código-fonte do Tauri nem executar no Windows)
- **Local:** `src-tauri/build.rs:1` (`tauri_build::build()` sem `AppManifest`); `src-tauri/src/main.rs:122–130` (`generate_handler!` com 7 comandos); `src-tauri/tauri.conf.json:7` (`withGlobalTauri:true`, `csp:null`); `src-tauri/capabilities/investing-drag.json:6–9` (`remote.urls` = `https://br.investing.com`, `https://*.investing.com`); `src-tauri/capabilities/default.json:1–15`.
- **Evidência:**
  - `build.rs`: `fn main() { tauri_build::build() }` — sem `Attributes::new().app_manifest(...)`.
  - Comandos registrados: `get_config`, `save_config`, `restore_default_size`, `close_settings`, `toggle_main_visibility`, `reload_site`, `quit_app`.
  - Documentação oficial (`v2.tauri.app/security/capabilities/`), segundo a leitura obtida nesta sessão: *"By default the API is only accessible to bundled code shipped with the Tauri App."*; e que os comandos registrados no app ficam, por padrão, permitidos para todas as janelas/webviews, e que a restrição se faz com `AppManifest::commands` em `build.rs`. A mesma página alerta que em Linux/Android o Tauri não distingue `<iframe>` da janela. A página `/security/runtime-authority/` diz apenas que o runtime "checks if the origin is part of capabilities" e nega a chamada caso contrário — **não esclarece** se comandos de app sem manifesto passam pela checagem de origem remota.
  - O script `INIT` já pressupõe `window.__TAURI__` na página remota (`window.rs:61`), ou seja, o IPC é injetado em `investing.com`.
  - Superfície adicional: janelas `login-N` (`window.rs:282`) carregam hosts de terceiros (`allowed()` aceita **todo** `*.google.com` e `*.facebook.com`, p.ex. `sites.google.com`, que hospeda conteúdo/JS controlado por terceiros) e **não pertencem a nenhuma capability**; se o padrão "comando de app liberado para todas as webviews" valer, elas também seriam alcançadas.
- **Reprodução (a executar em Windows):** compilar em *debug* (DevTools habilitado por `devtools(cfg!(debug_assertions))`, `window.rs:172`), abrir a janela principal em `https://br.investing.com/economic-calendar`, abrir o DevTools e executar no console:
  ```js
  await window.__TAURI_INTERNALS__.invoke('get_config')
  await window.__TAURI__.core.invoke('save_config', { config: { /* config válida com site_url diferente */ } })
  ```
  Repetir numa janela de login (`login-N`) aberta em `https://sites.google.com/`. **Resultado esperado correto:** erro de ACL ("not allowed"). Se retornar a configuração/alterar `config.json`, o achado é `CONFIRMADO` e vira CRITICA.
- **Causa raiz:** os comandos de app não têm permissões explícitas (`allow-*`) nem manifesto; a fronteira local×remota depende do comportamento padrão do Tauri para comandos sem manifesto. Além disso, `investing-drag` abre o IPC a `*.investing.com` inteiro (inclui subdomínios de terceiros/ads e qualquer iframe do mesmo padrão).
- **Impacto (se confirmado):** qualquer script na página (anúncio/analytics comprometido, XSS no investing.com) ou página em janela de login poderia: ler a config (`get_config`), **trocar `site_url` por um host malicioso e navegar a janela principal para ele** de forma persistente (`save_config` → `config::save` + `main.navigate`), alterar o atalho global, fechar o app (`quit_app`), ocultar a janela. Não há RCE direto identificado, mas há controle total do app e phishing persistente dentro de uma janela de confiança.
- **Correção proposta (aplicar mesmo que o teste não confirme — defesa em profundidade):**
  1. `build.rs`:
     ```rust
     fn main() {
         tauri_build::try_build(
             tauri_build::Attributes::new().app_manifest(
                 tauri_build::AppManifest::new().commands(&[
                     "get_config", "save_config", "restore_default_size", "close_settings",
                 ]),
             ),
         ).expect("falha no tauri-build");
     }
     ```
  2. Remover `toggle_main_visibility`, `reload_site` e `quit_app` de `generate_handler!` e o atributo `#[tauri::command]` (só a bandeja os chama diretamente: `tray.rs:92,93,98,107`; nenhum front-end usa).
  3. Criar capabilities **só para origem local**: `main` → `allow-get-config`; `settings` → `allow-get-config`, `allow-save-config`, `allow-restore-default-size`, `allow-close-settings`. Não conceder nada a `login-*`.
  4. `investing-drag`: manter apenas `core:window:allow-start-dragging`; estreitar para `https://br.investing.com` (ou o host configurado) se não for necessário `*.investing.com`; ver T-002 sobre o apex `investing.com`.
  5. Considerar remover `withGlobalTauri` (a fronteira real é o ACL; `__TAURI_INTERNALS__` continua injetado). Se mantido, o `INIT` continua dependendo dele (`window.rs:61`).
- **Critério de aceite:** (a) as chamadas do passo "Reprodução" na página investing.com **e** em `login-N` retornam erro de ACL; (b) `settings` e a tela inicial continuam funcionando (`get_config`, `save_config`, reset, fechar); (c) o arraste via `startDragging` continua funcionando em investing.com; (d) `cargo build` gera `gen/schemas` com as permissões `allow-*` do app.
- **Teste a adicionar:** roteiro manual documentado no README/`docs/` (ACL smoke test) + teste de CI que falha se `build.rs` não declarar `AppManifest` (ex.: `acl_manifest_lists_all_registered_commands` comparando a lista do manifesto com `generate_handler!`).
- **Depende de / Conflita com:** nenhum; **P4-016** e **P4-003** dependem do resultado.
- **Esforço:** M

#### P4-002 — Estado injetado na página remota volta ao valor da inicialização a cada carga (travado, idioma, opacidade, raio dos cantos)
- **Severidade:** ALTA
- **Categoria:** correcao-logica
- **Confiança:** PROVAVEL (leitura estática; confirmar no Windows)
- **Local:** `window.rs:172` (`create_main` → `.initialization_script(INIT.replace("__WIDGET_LANGUAGE__",&c.language).replace("__WIDGET_LOCKED__",&c.locked.to_string()))`); `window.rs:9–10` (`UI_LANGUAGE`, `DRAG_LOCKED` do `INIT`); `window.rs:344` (`apply_config` → `w.eval(&script)`); ausência de `on_page_load` (`grep -rn "on_page_load\|PageLoad" src-tauri/src` → vazio).
- **Evidência:** o script de inicialização é uma `String` fixa, montada **uma vez** na criação da janela com os valores de `c.locked`/`c.language` daquele instante. O Tauri reexecuta esse mesmo texto em toda carga de documento. `apply_config` só faz `eval` no documento **atual**, e `opacity`/`corner_radius` são aplicados **exclusivamente** por esse `eval` (`document.documentElement.style.opacity=…; …style.borderRadius=…`, `window.rs:344`; `grep -rn opacity src-tauri/src` não mostra nenhuma outra aplicação). Nenhum trecho do `INIT` reaplica opacidade/raio.
- **Reprodução:** (1) iniciar com `locked=false`; (2) travar pelo menu da bandeja → `__widgetSetLocked(true)` atualiza só o documento atual; (3) deixar a recarga automática ocorrer (ou F5, que o `INIT` mapeia para `location.reload()`, `window.rs:97`); (4) após o recarregamento a borda volta ao estado "destravado" enquanto `config.locked=true`. Variantes: opacidade < 100% e `corner_radius` não se aplicam ao abrir o app (a página inicial é `index.html`, e `main.ts` faz `location.replace(target)` para o site — novo documento sem o estilo) até o usuário clicar em "Salvar"; alterar idioma e recarregar reverte os textos injetados ao idioma do início. Para forçar sem esperar: `reload_site` ou `location.reload()` no DevTools (debug).
- **Causa raiz:** estado dinâmico embutido em script de inicialização estático + aplicação "push" por `eval` sem reaplicação "pull" em cada carga.
- **Impacto:** a opção **Opacidade** e **Raio dos cantos** parecem não funcionar após o primeiro carregamento/recarga; borda/alça mostram estado de trava errado (usuário vê "destravado" mas arrasto é ignorado, ou vice-versa — ver P4-005); idioma da UI injetada diverge das Configurações. Falha silenciosa, afeta todo usuário que usa essas opções.
- **Correção proposta:** (a) registrar `.on_page_load(|w, payload| if payload.event()==PageLoadEvent::Finished { re-aplicar estado atual de AppState })`, lendo `AppState` no momento da carga (não o valor capturado na criação); ou (b) no `INIT`, obter o estado ao iniciar via canal confiável (ver P4-001/P4-003) em vez de `__WIDGET_LOCKED__`; ou (c) reconstruir o script de init a cada `apply_config` não é possível sem recriar a janela — preferir (a). Mover `opacity`/`border-radius` para o `INIT` com valor vindo do (a). Manter `__widgetSetLocked/Language` apenas para atualização imediata.
- **Critério de aceite:** com `opacity=70`, `corner_radius=24`, `locked=true`, idioma `en` definidos via bandeja/Configurações, após `location.reload()` e após reiniciar o app a página mostra opacidade 0,7, raio 24 px, borda travada e textos em inglês, sem clicar em Salvar.
- **Teste a adicionar:** teste de unidade Rust `init_state_script_reflects_current_config` (função pura que gera o script de estado a partir de `AppConfig`, validando opacidade/raio/lock/idioma e escape via `serde_json`) + roteiro manual no Windows.
- **Depende de / Conflita com:** relacionado a T-001 e P4-005; nenhum bloqueio.
- **Esforço:** M

#### P4-003 — Canal `widget-action://` sem autenticação/gesto; resize ignora a trava no lado do script
- **Severidade:** MÉDIA
- **Categoria:** seguranca
- **Confiança:** PROVAVEL (subframes: HIPOTESE)
- **Local:** `window.rs:45` (`action()`), `window.rs:85–95` (listener global `pointerdown` de resize), `window.rs:181–203` (`on_navigation` → `widget-action`), `window.rs:174` (`widget-diagnostic`).
- **Evidência:** `function action(p){location.href='widget-action://'+p}` é acessível a qualquer script da página; `on_navigation` aceita `drag`/`resize/<dir>` sem verificar gesto, origem do frame ou `isTrusted`; só a trava de configuração (`locked`) é checada (`window.rs:184–186`). O listener de resize (`window.rs:85–95`) **não consulta `DRAG_LOCKED`**: com o widget travado, todo clique a ≤ 6 px da borda dispara navegação + `preventDefault`, e o Rust apenas registra `WARN` (`window.rs:186`) — um log por clique.
- **Reprodução:** em investing.com (debug), console: `location.href='widget-action://drag'` com o mouse parado e sem botão → inicia o laço modal nativo (`begin_native_drag` → `SendMessageW(WM_NCLBUTTONDOWN, HTCAPTION)`, `window.rs:215–240`) sem clique; `location.href='widget-action://resize/north-west'` idem para redimensionar. Com `locked=true`: clicar repetidamente na borda e contar linhas `Ação de janela ignorada…` em `diagnostics.log`.
- **Causa raiz:** o esquema customizado é um canal de comando por navegação sem segredo compartilhado nem checagem de ativação do usuário; o script de borda não replica a regra de trava.
- **Impacto:** qualquer script de terceiros pode mover/redimensionar o widget (incômodo/clickjacking de posição) e travar a thread de eventos dentro do laço modal enquanto o botão não for solto (ver T-003); com widget travado há inflação de log (rotação de log é Parte 2) e `preventDefault` indevido em cliques de borda. O spoof de `widget-diagnostic://heartbeat` tem impacto baixo (ver seção 7).
- **Correção proposta:** (1) no `INIT`, exigir `e.isTrusted` e abortar em `DRAG_LOCKED` também no listener de resize (`if(DRAG_LOCKED)return;`); (2) acrescentar um token aleatório por carga (injetado por `on_page_load`/init) em `widget-action://<token>/drag` e validar em `on_navigation`; (3) opcionalmente processar só `drag/resize` se o botão primário estiver realmente pressionado (`GetAsyncKeyState(VK_LBUTTON)`) antes de entrar no laço modal.
- **Critério de aceite:** com mouse parado, `location.href='widget-action://drag'` (sem token) é ignorado; com widget travado nenhum clique de borda gera navegação nem linha de log; arraste/resize legítimos continuam funcionando.
- **Teste a adicionar:** Rust `widget_action_requires_valid_token` (função pura de parsing/validação da URL de ação) e `widget_action_ignored_when_locked`.
- **Depende de / Conflita com:** P4-002 (fonte do token); T-003.
- **Esforço:** M

#### P4-004 — Heartbeat por navegação a cada 5 s pode cancelar navegações em andamento e disparar `beforeunload`
- **Severidade:** MÉDIA
- **Categoria:** correcao-logica
- **Confiança:** HIPOTESE
- **Local:** `window.rs:107–109` (`heartbeat()`, `setTimeout(...,1000)`, `setInterval(heartbeat,5000)`); `window.rs:174` (`on_navigation` → `record_heartbeat` e `return false`).
- **Evidência:** `location.href='widget-diagnostic://heartbeat?ts='+Date.now()` é uma navegação iniciada pelo renderer; o Rust a cancela depois (`return false`). Em Chromium, uma nova navegação do mesmo frame substitui a navegação pendente anterior e, para navegações iniciadas pela página, o `beforeunload` pode ser disparado antes do pedido chegar ao processo do navegador. Não foi possível verificar isso no WebView2 nesta sessão.
- **Reprodução:** no Windows, com rede lenta (throttling), clicar em um link/filtro de tela inteira e observar se, em ~1 em cada N cliques (janela de 5 s), a navegação é perdida; e abrir uma página de teste com `window.onbeforeunload=()=>'x'` e observar diálogo "Sair do site?" a cada 5 s.
- **Causa raiz:** uso de navegação como canal de telemetria.
- **Impacto:** cliques/submissões que ficam "sem resposta" de forma intermitente; possíveis diálogos de `beforeunload` em páginas que os usam (p.ex. formulários); carga contínua no ciclo de navegação do site.
- **Correção proposta:** trocar o canal por `fetch`/`sendBeacon` para um esquema registrado com `register_uri_scheme_protocol` (ou outro canal que não navega), ou checar liveness pelo lado Rust (evento de `ProcessFailed`/ping `eval` com retorno), eliminando o script de heartbeat. Se mantiver a navegação, aumentar o intervalo (ex.: 15 s, abaixo do `HEARTBEAT_TIMEOUT_MS` de 20 s em `diagnostics.rs:14`) apenas como mitigação parcial.
- **Critério de aceite:** com throttling de rede, 200 cliques de navegação consecutivos não perdem nenhuma navegação; nenhum diálogo de `beforeunload` aparece; watchdog continua detectando congelamento real (Parte 2).
- **Teste a adicionar:** roteiro manual; Rust `heartbeat_channel_does_not_cancel_navigation` não é automatizável sem WebView — documentar como teste manual.
- **Depende de / Conflita com:** T-003 (watchdog, Parte 2).
- **Esforço:** M

#### P4-005 — A trava (`locked`) não é aplicada no caminho IPC `startDragging`
- **Severidade:** MÉDIA
- **Categoria:** correcao-logica / seguranca
- **Confiança:** PROVAVEL
- **Local:** `window.rs:54–69` (`dragWindow`: `current.startDragging()` antes de `action('drag')`); `window.rs:17,53` (`__widgetSetLanguage`/`__widgetSetLocked` expostos como globais graváveis); `capabilities/investing-drag.json:13` (`core:window:allow-start-dragging`).
- **Evidência:** a regra `locked` existe (i) em `DRAG_LOCKED` no JS (`window.rs:56`) e (ii) apenas no ramo `widget-action` do Rust (`window.rs:184–186`). O caminho preferido, `window.__TAURI__.window.getCurrentWindow().startDragging()` (`window.rs:60–66`), chama o IPC nativo do Tauri, que **não** consulta `AppState.locked`.
- **Reprodução:** com `locked=true` e página recarregada (P4-002 devolve `DRAG_LOCKED` inicial `false`) ou, em debug, executar no console `__TAURI__.window.getCurrentWindow().startDragging()` após `mousedown`: a janela arrasta mesmo travada. Também `__widgetSetLocked(false)` destrava a UI injetada.
- **Causa raiz:** a trava é uma regra de UI replicada no cliente, não aplicada no ponto único de execução.
- **Impacto:** usuário/script move widget "travado"; combinado a P4-002, a trava se perde após recarga.
- **Correção proposta:** remover o caminho IPC (usar somente `widget-action` com validação no Rust — P4-003) e eliminar a permissão `core:window:allow-start-dragging` da capability remota. Solução coerente com P4-001: um único canal (Rust) que lê `AppState.locked` no momento da execução.
- **Critério de aceite:** com `locked=true`, nenhum método (clique na borda/alça, `startDragging` no console, `widget-action`) move a janela; com `locked=false` o arraste funciona.
- **Teste a adicionar:** manual (Windows); Rust `drag_rejected_when_locked` para o handler unificado.
- **Depende de / Conflita com:** P4-001, P4-002, P4-003.
- **Esforço:** P

#### P4-006 — Campo de atalho global: armadilha de teclado, atalho sem modificador e mapeamento por `event.key` (AltGr/ABNT2)
- **Severidade:** ALTA
- **Categoria:** validacao-entrada / ui-ux
- **Confiança:** CONFIRMADO para a lógica do código (réplica executada em Node); PROVAVEL para o efeito AltGr em hardware ABNT2; HIPOTESE para a lista exata de teclas aceitas por `Shortcut::from_str` (código-fonte indisponível).
- **Local:** `src/settings.ts:227–241` (`keydown` de `#global_shortcut`); `settings.html:63` (`<input id="global_shortcut" readonly>`); `src-tauri/src/config.rs` `normalize` (só exige não vazio — Parte 1); `src-tauri/src/commands.rs:9` (`Shortcut::from_str` + fallback com aviso).
- **Evidência (saída da réplica em Node, `settings.ts:227–241`):**
  ```
  Tab sozinho                     => "TAB"
  Enter sozinho                   => "ENTER"
  Backspace sozinho               => "BACKSPACE"
  Esc sozinho                     => "ESC"
  A sem modificador               => "A"
  Ctrl+Alt+C (ABNT2: AltGr+C = ₢) => "CTRL+ALT+₢"
  Ctrl+Shift+1 (key='!')          => "CTRL+SHIFT+!"
  Ctrl+Alt++                      => "CTRL+ALT++"
  Ctrl+Alt+Numpad1 (NumLock on)   => "CTRL+ALT+1"   (colide com a tecla 1 da fileira superior)
  Ctrl+Alt+Dead (acento)          => "CTRL+ALT+DEAD"
  Ctrl+Ctx menu                   => "CTRL+CONTEXTMENU"
  ```
  O handler faz `event.preventDefault()` incondicionalmente (`settings.ts:228`) e usa `event.key`, não `event.code`.
- **Reprodução:** abrir Configurações, focar o campo, pressionar `Tab` → o foco não sai (armadilha de teclado; WCAG 2.1.2); pressionar `A` e salvar → atalho global **sem modificador** `A` (se registrar, captura a tecla em todo o sistema). Em teclado ABNT2 (padrão no Brasil, e o idioma padrão do app é pt-BR), pressionar `Ctrl+Alt+C` produz `event.key==='₢'` → `CTRL+ALT+₢`, que o backend recusa (cai no `else` de `commands.rs:9` com o aviso enganoso de "conflito com outro aplicativo", ver P4-007).
- **Causa raiz:** captura por `event.key` (depende do layout; AltGr = Ctrl+Alt no Windows) + nenhuma validação de modificador, nenhuma saída do campo por teclado, nenhuma validação no cliente/servidor da sintaxe.
- **Impacto:** usuário brasileiro não consegue reconfigurar o atalho para `Ctrl+Alt+<letra>` (o próprio padrão); possível atalho global sem modificador que "rouba" teclas de todo o sistema; teclado inacessível no campo.
- **Correção proposta:** (1) usar `event.code` (`KeyC`→`C`, `Digit1`→`1`, `Numpad1`→`NUM1`/`NUMPAD1` conforme `global-hotkey`, `F1–F24`, setas, etc.) com tabela explícita e rejeitar códigos fora dela; (2) exigir ≥ 1 modificador entre `CTRL/ALT/SHIFT/SUPER` e uma tecla principal (rejeitar `ESC/TAB/ENTER/BACKSPACE/SPACE` sozinhos); (3) não chamar `preventDefault` para `Tab`/`Shift+Tab` (e permitir `Esc`/`Backspace` para limpar/cancelar com aviso textual), mantendo `readonly`; (4) validar no Rust em `normalize` com `Shortcut::from_str` e exigir modificador, devolvendo erro específico; (5) manter botão "Restaurar atalho padrão".
- **Critério de aceite:** em layout ABNT2, `Ctrl+Alt+C` grava `CTRL+ALT+C`; `Tab` move o foco; `A`/`Tab`/`Enter` sozinhos são rejeitados com mensagem; teclas numéricas do numpad geram o token aceito pelo backend; `Shortcut::from_str` aceita 100% dos valores que o campo consegue produzir (teste de propriedade).
- **Teste a adicionar:** extrair `shortcutFromEvent(ev)` para `src/shortcut.ts` e testar (vitest/node:test): `shortcut_from_event_uses_code_not_key`, `shortcut_requires_modifier`, `shortcut_ignores_tab_for_focus`; Rust `normalize_rejects_shortcut_without_modifier`, `normalize_rejects_unparseable_shortcut` (Parte 1).
- **Depende de / Conflita com:** T-004 (validação no backend, Parte 1).
- **Esforço:** M

#### P4-007 — Avisos/erros do backend mapeados por substring em pt-BR; aviso de autostart some; erros crus em pt-BR
- **Severidade:** MÉDIA
- **Categoria:** i18n / correcao-logica
- **Confiança:** CONFIRMADO (réplica da lógica executada)
- **Local:** `src/settings.ts:204–208` (`includes("atalho")`/`includes("início")`), `settings.ts:213` e `249` (`String(error)`); `src-tauri/src/commands.rs:9` (mensagens `warns.push("O atalho novo conflita…")`, `"Não foi possível habilitar o início com o Windows."`, e `Err("Informe a URL do site.")` etc. em `config.rs:57–90`).
- **Evidência:** quando as duas condições ocorrem, `warns.join(" ")` contém ambas as frases; o mapeamento testa `includes("atalho")` primeiro → exibe só `shortcutWarning` (execução: `warning combinado mapeado para: shortcutWarning (autostart oculto)`). Mensagens de `normalize` ("URL inválida.", "Use uma URL HTTP/HTTPS válida, sem credenciais.", "Informe um atalho global.") só existem em pt-BR e aparecem com prefixo localizado (`invalidConfig`) seguido de texto em português mesmo com UI em EN/ES. Além disso, atalho com **sintaxe inválida** também cai no mesmo aviso "conflita com outro aplicativo" (`commands.rs:9`), que é enganoso.
- **Reprodução:** em modo EN, salvar URL vazia → "Could not save settings: Informe a URL do site."; forçar falha simultânea de atalho e autostart (mock) → só o aviso de atalho aparece.
- **Causa raiz:** contrato textual entre Rust e TS em vez de códigos estruturados.
- **Impacto:** mensagens incorretas/ocultas; i18n quebrado em EN/ES; usuário não sabe que o autostart falhou ou que o atalho é inválido (e não "em conflito").
- **Correção proposta:** `SaveResult { config, warnings: Vec<WarningCode> }` com `enum WarningCode { ShortcutConflict, ShortcutInvalid, AutostartEnableFailed, AutostartDisableFailed }`; erros de `normalize`/comandos como `{code, field}`; o front traduz por `t(code)` (mantendo `translations`). Exibir todos os avisos, não só o primeiro.
- **Critério de aceite:** com UI em EN/ES nenhuma mensagem em português aparece; falha simultânea de atalho e autostart mostra ambos os avisos; atalho inválido exibe aviso próprio (`ShortcutInvalid`).
- **Teste a adicionar:** Rust `save_result_reports_all_warning_codes`; TS `maps_every_warning_code_to_every_language` (garante que `translations` tem as 3 línguas para cada código).
- **Depende de / Conflita com:** T-004, T-005.
- **Esforço:** M

#### P4-008 — Campos numéricos opcionais vazios são enviados como `0` e salvos silenciosamente
- **Severidade:** MÉDIA
- **Categoria:** validacao-entrada
- **Confiança:** CONFIRMADO (lógica) — `NaN` **não** é alcançável pela UI (ver seção 7)
- **Local:** `src/settings.ts:179–192` (`Number(value(...))`); `settings.html:57,58` (`corner_radius` e `auto_reload_minutes` sem `required`).
- **Evidência:** `Number("") === 0`. `width`/`height` têm `required`+`min`/`max`; `corner_radius` (`min=0 max=64`) e `auto_reload_minutes` (`min=0 max=1440`) **não têm `required`**, então o navegador aceita o campo vazio e `read()` envia `0`, gravado sem aviso (raio 0 = cantos retos; recarga 0 = desligada). Campos `type=number` com texto inválido retornam `""` (não `NaN`).
- **Reprodução:** limpar "Raio dos cantos" e "Recarregar a cada" e salvar → configuração gravada com `0`, mensagem "Configurações salvas.".
- **Causa raiz:** conversão sem validação e atributos HTML incompletos.
- **Impacto:** mudança silenciosa de preferência.
- **Correção proposta:** adicionar `required` aos dois inputs (e `step="1"`); em `read()` usar helper `readInt(id,{min,max})` que lança erro localizado se o campo estiver vazio/fora do intervalo, em vez de converter para 0; manter `0` válido apenas quando digitado.
- **Critério de aceite:** campo vazio impede o envio com mensagem localizada; `0` digitado continua válido.
- **Teste a adicionar:** TS `read_int_rejects_empty_and_out_of_range`.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

#### P4-009 — "Salvar" sobrescreve `x`/`y`/`locked`/`always_on_top` com valores antigos
- **Severidade:** ALTA
- **Categoria:** integridade-de-dados
- **Confiança:** PROVAVEL (confirmado no lado do front-end; efeito no disco depende do Rust — Parte 1/3)
- **Local:** `src/settings.ts:170–173` (`current` só é lido em `load()`), `settings.ts:181–182` (`x: current?.x ?? null`), `settings.ts:184–190` (`locked`, `always_on_top` lidos do formulário), `settings.ts:201–203`; `commands.rs:9` (`save_config` grava tudo o que chega).
- **Evidência:** o formulário reflete o estado de quando a janela de Configurações foi aberta. Se depois o usuário mover o widget (o Rust atualiza `x/y`) ou alternar "Travar"/"Sempre no topo" pela bandeja (`toggle_locked`/`toggle_topmost`, que não avisam a janela), o próximo "Salvar" reenvia valores velhos e `save_config` os persiste e reaplica (`apply_config`).
- **Reprodução:** abrir Configurações; pela bandeja, travar o widget e movê-lo antes de travar; em Configurações alterar só o idioma e salvar → o widget destrava/volta à posição/tamanho antigos.
- **Causa raiz:** o front envia a configuração inteira (modelo "last write wins") com snapshot obsoleto; não há evento de sincronização.
- **Impacto:** perda de preferências/posição.
- **Correção proposta:** (1) front não envia `x`,`y` (nem campos que o formulário não edita) — o Rust faz merge sobre o estado atual (`save_config(patch)`); (2) emitir evento `config-changed` do Rust nos toggles da bandeja/`save_window_state` e o front reidrata os campos não "sujos" (`core:event:default` já está em `core:default`); (3) incluir `revision` na config e rejeitar salvamento com revisão antiga.
- **Critério de aceite:** com Configurações aberta, alternar "Travar"/"Sempre no topo" pela bandeja atualiza os checkboxes em < 1 s; "Salvar" após mover a janela não altera a posição; teste automatizado de merge passa.
- **Teste a adicionar:** Rust `save_config_patch_preserves_untouched_fields`; TS `read_omits_fields_not_edited`.
- **Depende de / Conflita com:** T-001 (Partes 1 e 3), P4-010.
- **Esforço:** M

#### P4-010 — "Restaurar tamanho padrão" descarta silenciosamente as edições não salvas do formulário
- **Severidade:** BAIXA
- **Categoria:** ui-ux
- **Confiança:** PROVAVEL
- **Local:** `src/settings.ts:242–251`; `commands.rs:12` (`restore_default_size` parte do estado armazenado).
- **Evidência:** o comando ignora o formulário (`state.config.lock()…clone()`), salva e `display(result.config)` repõe **todos** os campos com o estado salvo.
- **Reprodução:** editar URL e idioma sem salvar; clicar "Restaurar tamanho padrão" → edições perdidas; mensagem só diz "Tamanho padrão restaurado.".
- **Causa raiz:** ação "local" de um campo persiste e reidrata o formulário inteiro.
- **Impacto:** perda de trabalho do usuário (baixo).
- **Correção proposta:** ao restaurar, só preencher `width`/`height` no formulário (500×700) sem persistir; o usuário confirma em "Salvar". Se mantiver persistência imediata, avisar e preservar campos "sujos".
- **Critério de aceite:** após reset, os outros campos editados permanecem como o usuário deixou.
- **Teste a adicionar:** TS `reset_size_does_not_overwrite_dirty_fields`.
- **Depende de / Conflita com:** P4-009.
- **Esforço:** P

#### P4-011 — `main.ts` testa `/favicon.ico` com `no-cors` e depois navega; página de erro fica sem alça de arraste
- **Severidade:** MÉDIA
- **Categoria:** correcao-logica / ui-ux
- **Confiança:** HIPOTESE (comportamento do WebView2 em página de erro não verificado)
- **Local:** `src/main.ts:59–84` (`attempt`), `main.ts:69–73` (fetch `no-cors`), `main.ts:79` (`location.replace(target)`), `main.ts:56` (`setInterval` 30 s), `main.ts:39` (URL padrão duplicada); `window.rs:7` (`INIT` só roda em `^https?:`).
- **Evidência:** com `mode:"no-cors"` o `fetch` só rejeita em erro de rede; qualquer resposta HTTP (404, 500, portal cativo) resolve, então "site acessível" não significa "calendário carrega". Depois `location.replace(target)` troca o documento; se o site falhar na navegação, a página de erro do Chromium substitui a interface — o README admite a limitação — e o `INIT` (borda/alça/overlay offline) provavelmente não é injetado em `chrome-error://`.
- **Reprodução:** no Windows, configurar um host que responda `503` ou bloqueie a navegação principal; observar se a janela (sem moldura) ainda pode ser arrastada/redimensionada e se aparece o overlay de erro do widget.
- **Causa raiz:** pré-checagem não conclusiva e navegação sem tratamento de falha de carregamento no lado Rust.
- **Impacto:** widget "preso" numa página de erro sem como arrastar/recarregar (só bandeja); mensagens de status enganosas.
- **Correção proposta:** tratar falha de carga no Rust (`on_page_load`/evento de erro de navegação) e voltar a `index.html` com mensagem; manter o `INIT` também em páginas de erro se possível; no front usar `fetch` com `mode:"cors"` apenas quando o site permitir, ou aceitar que a sonda só mede conectividade e rotular como tal. Centralizar a URL padrão: pedir ao Rust (`get_config` já a devolve) e remover a constante de `main.ts:39`.
- **Critério de aceite:** ao falhar a navegação (host `503`/DNS), o widget exibe a tela local com "Tentar novamente" e continua arrastável.
- **Teste a adicionar:** manual; TS `attempt_reports_failure_when_navigation_fails` (mock).
- **Depende de / Conflita com:** P4-002 (hook de carga).
- **Esforço:** M

#### P4-012 — O `INIT` bloqueia menu de contexto e vários atalhos em todo o site
- **Severidade:** MÉDIA
- **Categoria:** ui-ux (acessibilidade)
- **Confiança:** PROVAVEL
- **Local:** `window.rs:96` (`contextmenu` → `preventDefault`), `window.rs:97` (`Ctrl/Meta + L,T,N,W,P,J,U,S` → `preventDefault`+`stopImmediatePropagation`).
- **Evidência:** ambos em fase de captura no documento do site; `contextmenu` também é emitido pelo teclado (tecla Menu/`Shift+F10`).
- **Reprodução:** na janela principal, clicar com o botão direito sobre texto selecionado ou usar `Shift+F10`: nada acontece (sem Copiar/Colar/Verificar ortografia); `Ctrl+P`, `Ctrl+S` também não funcionam.
- **Causa raiz:** bloqueio amplo para esconder recursos de navegador em vez de desativá-los na configuração do WebView.
- **Impacto:** usuários de teclado/leitor de tela e quem usa só mouse perdem Colar/Copiar; atalhos legítimos do site (`Ctrl+S` em editores etc.) interceptados. (`Ctrl+V` **não** é bloqueado — ver seção 7; as janelas de login usam apenas `LOGIN_INIT` e não sofrem esse bloqueio.)
- **Correção proposta:** desativar os aceleradores do navegador pelas configurações do WebView2 (`AreBrowserAcceleratorKeysEnabled=false`, `AreDefaultContextMenusEnabled=false` ou manter o menu padrão) via API do wry/Tauri quando disponível; se não, bloquear apenas `Ctrl+P/U/J/S` e liberar `contextmenu` em campos editáveis/seleção de texto (`e.target.closest('input,textarea,[contenteditable]')` ou `getSelection().toString()`).
- **Critério de aceite:** botão direito em campo de texto/seleção abre o menu com Colar/Copiar; `Shift+F10` funciona nesses casos; atalhos indesejados do navegador continuam desativados.
- **Teste a adicionar:** manual (Windows).
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

#### P4-013 — Faixa de resize de 6 px ignora o zoom e disputa com a barra de rolagem
- **Severidade:** MÉDIA
- **Categoria:** ui-ux
- **Confiança:** PROVAVEL (zoom); HIPOTESE (barra de rolagem)
- **Local:** `window.rs:85–95` (`x<6||x>w-6||y<6||y>h-6` com `innerWidth/innerHeight`), `window.rs:102` (revela a barra de rolagem a ≤ 24 px da borda direita), `window.rs:344` (`set_zoom`).
- **Evidência:** `clientX`/`innerWidth` estão em px CSS; o zoom do WebView2 (50–200 %) muda a relação px CSS↔px físico: em 50 % a faixa tem 3 px físicos (difícil de acertar), em 200 % tem 12 px (invade conteúdo). A barra de rolagem fina (9 px, `window.rs:38`) começa em `w-9`; os 6 px mais à direita pertencem à faixa de resize, deixando ~3 px para agarrar a barra quando destravado.
- **Reprodução:** definir zoom 50 % e tentar redimensionar pela borda; definir 100 % e tentar arrastar a barra de rolagem vertical pela direita.
- **Causa raiz:** limiar em px CSS sem normalização pelo zoom nem exclusão da área da barra.
- **Impacto:** redimensionar/rolar de forma imprecisa.
- **Correção proposta:** `const band=6/(window.visualViewport?.scale||1)/(devicePixelRatio_normalizado)` ou, preferível, usar elementos de borda do próprio `__widget_drag_frame` com `position:fixed` e `width:calc(6px / var(--zoom))`; recuar a faixa direita para fora da barra de rolagem.
- **Critério de aceite:** em 50 %, 100 % e 200 % a faixa tem largura física constante (~6 px) e a barra de rolagem continua utilizável.
- **Teste a adicionar:** manual; TS `resize_band_is_constant_in_device_pixels` (função pura de largura da faixa).
- **Depende de / Conflita com:** P4-003.
- **Esforço:** P

#### P4-014 — `LOGIN_INIT`/`INIT`: i18n parcial, `close` sem nonce e barra que cobre o site
- **Severidade:** BAIXA
- **Categoria:** i18n / ui-ux
- **Confiança:** CONFIRMADO (leitura) / PROVAVEL
- **Local:** `window.rs:132` (`setCustomValidity('Informe um endereço HTTP ou HTTPS válido.')` fixo em PT), `window.rs:73` (`aria-label='Borda para mover o widget'` fixo em PT e nunca atualizado por `setUiLanguage`), `window.rs:135` + `window.rs:284` (`widget-login-action://close` aceito de qualquer script da página de login), `window.rs:122,137` (`position:fixed` + `padding-top:46px !important` em `<html>`), `window.rs:130` (endereço inicial `location.href` e nunca atualizado em `pushState`).
- **Evidência:** `TEXT` traduz rótulos/botões mas não a mensagem de validação; qualquer script em `accounts.google.com`/`facebook.com` pode fechar a janela com `location.href='widget-login-action://close'`; sites com cabeçalho `position:fixed; top:0` ficam sob a barra (z-index máximo) e o `padding-top !important` desloca o layout; URLs de OAuth (com `code`/`state`) ficam visíveis no campo.
- **Reprodução:** em EN, digitar `javascript:` na barra → mensagem em português; em página de login com cabeçalho fixo, observar sobreposição.
- **Causa raiz:** texto hard-coded e canal de fechamento sem segredo.
- **Impacto:** UX/i18n; fechamento intencional por script de terceiros (incômodo).
- **Correção proposta:** mover `validity` para `TEXT`; localizar `aria-label` do frame em `setUiLanguage`; usar token em `widget-login-action://<token>/close`; aplicar a barra num `<div>` de wrapper com `top:46px` em vez de `padding-top !important`; atualizar o campo em `popstate`/`navigation` events e ocultar parâmetros sensíveis (`code`, `token`).
- **Critério de aceite:** nenhuma string de UI em PT quando o idioma é EN/ES; fechamento exige token; barra não cobre cabeçalhos fixos.
- **Teste a adicionar:** manual; TS `login_text_covers_all_languages`.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

#### P4-015 — Contraste de cores abaixo de WCAG AA (texto pequeno, tema claro e escuro)
- **Severidade:** MÉDIA
- **Categoria:** ui-ux (acessibilidade)
- **Confiança:** CONFIRMADO (cálculo executado)
- **Local:** `src/styles.css:50–52` (`.message`, `.message.error`, `.message.warning` sem variação para o tema escuro), `styles.css:23` (`.eyebrow`), `styles.css:45` (`.range-label output`), `styles.css:8` (`.status-panel small`), `styles.css:18` (`--settings-small`).
- **Evidência (razão de contraste, mínimo AA = 4,5 para texto < 18 px):**
  | Elemento | Cores | Razão |
  |---|---|---|
  | `.message.warning` (claro) | `#ad7a20` / `#f4f7fb` | **3,50** |
  | `.message.error` (claro) | `#d64f55` / `#f4f7fb` | **3,83** |
  | `.message` sucesso (claro) | `#268255` / `#f4f7fb` | 4,44 |
  | `.message` sucesso (escuro) | `#268255` / `#111821` | **3,75** |
  | `.message.error` (escuro) | `#d64f55` / `#111821` | **4,34** |
  | `.eyebrow` (escuro) | `#1769d2` / `#111821` | **3,39** |
  | `.range-label output` (escuro) | `#1769d2` / `#1b2532` | **2,94** |
  | `--settings-small` (claro) | `#758294` / `#fff` | **3,91** |
  | `.status-panel small` | `#8793a2` / `#fff` | **3,12** |
- **Reprodução:** executar `node` com a fórmula WCAG 2.x (script na sessão) ou usar um verificador de contraste.
- **Causa raiz:** cores de mensagem e destaque fixas e não tokenizadas por tema.
- **Impacto:** legibilidade reduzida para baixa visão, principalmente nos avisos/erros.
- **Correção proposta:** tokens `--message-ok/--message-warn/--message-error/--accent` por tema com razão ≥ 4,5; escurecer `#ad7a20`, `#d64f55`, `#758294`, `#8793a2` no claro e clarear `#1769d2`/`#268255`/`#d64f55` no escuro.
- **Critério de aceite:** todos os pares acima ≥ 4,5:1 nos dois temas (script de verificação em CI ou teste).
- **Teste a adicionar:** TS `theme_tokens_meet_wcag_aa` (calcula razões a partir dos tokens).
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

#### P4-016 — Privilégio mínimo e CSP: permissões não utilizadas e `csp: null`
- **Severidade:** BAIXA
- **Categoria:** seguranca
- **Confiança:** PROVAVEL
- **Local:** `capabilities/default.json:10–14` (`core:default`, `global-shortcut:allow-register/unregister/is-registered`, `core:webview:allow-set-webview-zoom`), `tauri.conf.json:7` (`"csp":null`), `package.json` (sem `@tauri-apps/plugin-*`).
- **Evidência:** `grep` em `src/*.ts` não mostra uso de plugins; o registro de atalho e o zoom são feitos pelo Rust (`commands.rs`, `window.rs:344`). As permissões de `global-shortcut` e `set-webview-zoom` concedidas às janelas locais `main` e `settings` não são usadas pelo front-end. O CSP nulo se aplica apenas aos documentos locais (`index.html`, `settings.html`), que não carregam conteúdo remoto e **não** têm sinks de HTML (`grep` por `innerHTML/eval/document.write` em `src/` e nas páginas: nenhum).
- **Impacto:** superfície desnecessária se alguma página local vier a ser comprometida/estendida (baixo hoje).
- **Correção proposta:** remover as 4 permissões não usadas (manter só o necessário em `core:default` ou trocar por permissões específicas); definir CSP restritivo para as páginas locais, p.ex. `default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost https:` (o `connect-src https:` é necessário para a sonda `no-cors` de `main.ts:69`, ver P4-011).
- **Critério de aceite:** app abre `index.html` e `settings.html` sem violações de CSP no console; `get_config`/`save_config` continuam funcionando; removidas permissões não usadas.
- **Teste a adicionar:** manual/CI: verificar `capabilities/*.json` contra lista de permissões esperadas.
- **Depende de / Conflita com:** P4-001 (reestruturação das capabilities).
- **Esforço:** P

### Parte 5 — Infraestrutura: Build, CI/CD, Dependências e Distribuição
**NÃO ANALISADA nesta sessão** (pendente).

---

## 5. Achados Transversais (T-xxx) — identificados durante a Parte 4; completar nas Partes correspondentes
- **T-001 — Estado de configuração escrito/lido por vários caminhos sem sincronização** (Partes 1, 3, 4): `settings.ts:read()` reenvia `x,y,locked,always_on_top` obsoletos (P4-009); `toggle_locked/toggle_topmost` (`commands.rs:17,18`) não notificam o front nem o `INIT` (P4-002). Correção conjunta: merge por patch + evento `config-changed` + `on_page_load`.
- **T-002 — Allowlist de navegação × capabilities** (Partes 3 e 4): `allowed()` (`window.rs:142`) aceita o apex `investing.com`, enquanto `investing-drag.json` usa `https://*.investing.com` (não casa com o apex, a verificar) e a URL configurável pode ser outro host sem nenhum IPC; o `INIT` cai em `widget-action://drag` nesses casos. Alinhar uma única fonte de verdade para os hosts.
- **T-003 — `begin_native_drag` dentro do callback `on_navigation`** (Partes 2, 3 e 4): o laço modal de `SendMessageW(WM_NCLBUTTONDOWN)` roda dentro da navegação (`window.rs:181–195`), podendo bloquear o callback enquanto o botão não for solto e interagir com o watchdog/heartbeat (`diagnostics.rs`). Não verificado por execução.
- **T-004 — Validação de atalho dividida entre front e back** (Partes 1 e 4): front produz tokens por `event.key` (P4-006), back só valida "não vazio" e trata qualquer falha de `Shortcut::from_str` como "conflito" (`commands.rs:9`). Unificar a sintaxe e os códigos de erro.
- **T-005 — Mensagens do Rust só em pt-BR** (Partes 1, 3 e 4): `normalize` e `commands.rs` devolvem texto; migrar para códigos estruturados (P4-007).

## 6. Cobertura de Testes Ausente (Parte 4)
Não há testes de front-end nem lint no repositório (`package.json` sem script de teste). Lacunas e nomes sugeridos:
- `settings.ts`: `shortcut_from_event_uses_code_not_key`, `shortcut_requires_modifier`, `shortcut_ignores_tab_for_focus`, `read_int_rejects_empty_and_out_of_range`, `read_omits_fields_not_edited`, `maps_every_warning_code_to_every_language`.
- `main.ts`: `attempt_reports_failure_when_navigation_fails`.
- Scripts injetados: extrair `INIT`/`LOGIN_INIT` para arquivos `.js` testáveis (ou testar com jsdom): `init_syntax_is_valid` (já verificado com `node --check`), `init_applies_current_state_on_load`, `resize_ignored_when_locked`, `login_text_covers_all_languages`.
- Rust: `acl_manifest_lists_all_registered_commands`, `widget_action_requires_valid_token`, `widget_action_ignored_when_locked`, `init_state_script_reflects_current_config`.
- Acessibilidade: `theme_tokens_meet_wcag_aa`.

## 7. Falsos Positivos e Itens Verificados Sem Problema
**Pistas iniciais da Parte 4 (resultado):**
1. **Pista 1 (IPC/ACL):** não refutada nem provada → `P4-001` (`HIPOTESE`). O trecho "`default.json` se aplica só à origem local" **confirmado por leitura**: não há `remote` em `default.json`, logo suas permissões são para origem local (`main`/`settings` locais). Observação: `core:default` **não** inclui `allow-start-dragging`; por isso o `INIT` na página local `index.html` cai no fallback `widget-action://drag` (funciona).
2. **Pista 2 (comandos por navegação):** **parcialmente confirmada** → `P4-003`. O *spoof* de `widget-diagnostic://heartbeat` foi **rebaixado**: só é possível com JavaScript rodando, o que já indica renderer vivo; não mascara congelamento do renderer. Subframes: o `INIT` sai cedo com `window.top!==window` (`window.rs:7`) e o `on_navigation` do wry atua em navegações de nível superior; chamadas vindas de iframes de outra origem **não verificadas** (`HIPOTESE` baixa).
3. **Pista 3 (campo de atalho):** confirmada → `P4-006` (inclusive o caso adicional AltGr/ABNT2).
4. **Pista 4 (`read()`/mensagens):** confirmada em parte → `P4-007` e `P4-008`. **`NaN` não é alcançável pela UI**: `<input type=number>` devolve `""` para texto inválido, convertido em `0` (não `NaN`); o erro de desserialização cru só ocorre com valores fora do tipo (Parte 1).
5. **Pista 5 (`main.ts`):** sonda `no-cors` → `P4-011`. **Refutado:** o `setInterval` não é um vazamento relevante — o documento é descarregado por `location.replace` e o timer só vive enquanto a tela de falha é exibida. URL padrão duplicada: confirmada (`main.ts:39` × `DEFAULT_SITE`), **BAIXA**, tratada em P4-011.
6. **Pista 6 (`INIT`):** bloqueios de contexto/atalhos confirmados → `P4-012`; **refutado** que impeçam "colar senha no login": `Ctrl+V` não está na lista e `INIT` só existe na janela principal (as janelas de login usam `LOGIN_INIT`). `MutationObserver` + `install()`: custo baixo (duas buscas por id; o callback é agrupado), sem laço (`install()` retorna cedo quando os elementos existem) — apenas melhoria opcional. `setInterval(check,30000)` com `fetch` ao mesmo origin: custo baixo, sem finding. Heartbeat → `P4-004` (`HIPOTESE`).
7. **Pista 7 (`LOGIN_INIT`):** confirmada em parte → `P4-014`. A barra aceita só `http(s)` (`new URL` + checagem de protocolo; `javascript:`/`data:` são rejeitados — verificado por leitura); o bloqueio de hosts não permitidos ocorre depois em `on_navigation` (`window.rs:294`, vai ao navegador padrão).
8. **Pista 8 (sem testes de front-end):** confirmada → seção 6.

**Examinado sem problema:**
- **Injeção/XSS:** `grep` por `innerHTML|insertAdjacentHTML|eval(|document.write` em `src/*.ts`, `index.html`, `settings.html` → nenhum. O `INIT` usa `innerHTML` apenas com string estática (`window.rs:103`), preenchendo o texto por `textContent`.
- **Injeção via configuração no script de init:** `INIT.replace("__WIDGET_LANGUAGE__",…)` usa `c.language`, que passa por `normalize` (whitelist `pt-BR|en|es`) também em `config::load` (`config.rs:111`); `apply_config` serializa o idioma com `serde_json` (`window.rs:344`) e opacidade/raio são numéricos.
- **Sintaxe dos scripts injetados:** `node --check` OK em `INIT` e `LOGIN_INIT`.
- **Autofill do Chromium** que dispara `keydown` sem `key`: o handler `window.rs:97` curto-circuita (`e.ctrlKey||e.metaKey` indefinido) e não lança exceção.
- **`TS2345` em `settings.ts:172`:** artefato da ausência de tipos (seção 2), não achado.
- **`display()`/`applyLanguage()`** em `settings.ts:141–169`: cobrem os 14 ids, usam `textContent`/`setAttribute` e fazem fallback para pt-BR.
- **Persistência do tema** (`localStorage`) está dentro de `try/catch` (`settings.ts:126–128,258–262`).

## 8. Melhorias Opcionais (máx. 10 linhas)
- Janela de Configurações é criada com `inner_size(760,850)` (`window.rs:345`): em telas 1366×768 ou 125 % de DPI pode exceder a área útil e esconder "Salvar" (`HIPOTESE`) — limitar à altura do monitor.
- Agrupar os campos em `<fieldset>/<legend>` e associar `aria-describedby` aos textos de ajuda (`<small>`).
- `index.html`/`settings.html`: texto inicial em PT visível antes de `applyLanguage` (flicker para EN/ES); definir `lang` e textos via CSS/atributo ou ocultar até a hidratação.
- Seleção de idioma pré-visualiza imediatamente mas só persiste em "Salvar" — indicar com texto.
- Desabilitar o botão "Salvar" enquanto salva (evita duplo envio).
- `html,body,*{scrollbar-color:transparent…}` + `*` com `!important` durante o scroll força recálculo de estilo em DOM grande; preferir regras apenas em `html`/contêineres de rolagem conhecidos.
- Extrair `INIT`/`LOGIN_INIT` de `window.rs` para `src-tauri/src/inject/init.js` com `include_str!` (diffável, lintável, testável).
- Documentar no README o modelo de ameaça da página remota e o procedimento de teste de ACL de P4-001.

## 9. Checklist Final de Verificação Pós-Correção (Parte 4)
- [ ] P4-001: chamadas de `get_config`/`save_config`/`quit_app` de investing.com **e** de `login-N` retornam erro de ACL; Configurações e tela inicial funcionam.
- [ ] `build.rs` com `AppManifest`; `generate_handler!` só com os 4 comandos usados pelo front-end; capabilities separadas local/remota.
- [ ] Opacidade, raio, trava e idioma persistem após recarregar a página e reiniciar o app (P4-002).
- [ ] Trava respeitada por todos os caminhos de movimento/redimensionamento (P4-003, P4-005).
- [ ] Campo de atalho: `Ctrl+Alt+C` em ABNT2 grava `CTRL+ALT+C`; `Tab` sai do campo; sem atalho sem modificador (P4-006).
- [ ] Avisos e erros localizados nas 3 línguas, com todos os avisos exibidos (P4-007).
- [ ] Campos vazios rejeitados (P4-008); "Salvar" não sobrescreve x/y/locked/always_on_top (P4-009); reset não perde edições (P4-010).
- [ ] Falha de carregamento do site mostra tela local com "Tentar novamente" (P4-011).
- [ ] Menu de contexto disponível em campos/seleção; atalhos do navegador desativados (P4-012).
- [ ] Faixa de resize com largura física constante em 50/100/200 % (P4-013).
- [ ] Sem textos em PT com UI em EN/ES nos scripts injetados; `close` com token (P4-014).
- [ ] Todos os pares de cor ≥ 4,5:1 (P4-015).
- [ ] Permissões não usadas removidas; CSP definido para as páginas locais (P4-016).
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` e `npm run build` passam; testes novos adicionados.
- [ ] **Pendente:** Partes 1, 2, 3 e 5 ainda precisam ser analisadas; reconciliar `T-001…T-005` ao final.
