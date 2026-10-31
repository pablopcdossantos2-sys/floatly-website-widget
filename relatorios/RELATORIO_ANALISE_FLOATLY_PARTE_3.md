# Relatório de auditoria — Floatly (`floatly-website-widget-main`) — **PARTE 3 de 4**

> **Parte executada neste relatório: Parte 3 — Interface local (frontend), IPC e superfície de segurança.**
> As Partes 1, 2 e 4 **não** foram auditadas aqui (foram lidas apenas onde necessário para entender as interfaces com a Parte 3).
> Arquivo de saída: `RELATORIO_ANALISE_FLOATLY_PARTE_3.md` · Idioma: pt-BR · Público-alvo: **outra IA que irá corrigir** (cada achado é autossuficiente).

---

## 0. Delimitação das 4 partes e parte analisada

| Nº | Parte | Arquivos (delimitação, conforme o prompt) | Situação neste relatório |
|---|---|---|---|
| **1** | Núcleo Rust: ciclo de vida, configuração, comandos IPC e diagnóstico | `src-tauri/src/main.rs`, `config.rs`, `commands.rs`, `diagnostics.rs`, `autostart.rs`, `src-tauri/Cargo.toml`, `build.rs` | **Não auditada.** Lidos só trechos de `config.rs` (`normalize`, `AppConfig`, `AppState`), `commands.rs` (inteiro, por ser a outra ponta do IPC), `main.rs` (apenas `generate_handler!`) e `diagnostics.rs` (`record_heartbeat`). |
| **2** | Janela, navegação, login e script injetado | `src-tauri/src/window.rs` (1.768 linhas), `tray.rs`, `i18n.rs` | **Não auditada.** Lidos só trechos de `window.rs` (script `INIT` linhas 17-175, `decide_main`/`decide_login`/`is_auth_provider`/`is_internal_app_url`, `create_main`/`on_navigation`, `open_settings`). |
| **3** | **Interface local (frontend), IPC e superfície de segurança** | `index.html`, `settings.html`, `src/main.ts`, `src/settings.ts`, `src/styles.css`, `vite.config.ts`, `tsconfig.json`, `package.json` (scripts/deps de runtime), `src-tauri/tauri.conf.json` (seção `app`/`security`), `src-tauri/capabilities/*.json` (3 arquivos) | **ANALISADA POR INTEIRO (esta é a parte deste relatório).** |
| **4** | Build, CI/CD, empacotamento, dependências, documentação e distribuição | `.github/workflows/*.yml`, `dependabot.yml`, `scripts/*` (todos), `packaging/msix/*`, `src-tauri/windows/hooks.nsh`, `tauri.conf.json` (seção `bundle`), `Cargo.toml`/`Cargo.lock`, `package.json`/`package-lock.json`, `rust-toolchain.toml`, `.gitignore`, `README*.md`, `PRIVACY_POLICY.md`, `SUPPORT.md`, `GUIA_PUBLICACAO_*`, `MICROSOFT_STORE_*` | **Não auditada.** Lidos só `scripts/check-acl.mjs`, `check-theme-contrast.mjs`, `check-tauri-versions.mjs`, `check-drag-overlay.mjs`, `Cargo.toml`, `rust-toolchain.toml`, `.gitignore` e buscas por `http://` em docs (para dimensionar o impacto de FLY-P3-001). |

**IDs dos achados:** `FLY-P3-nnn` (estáveis; compatíveis com a consolidação futura com as Partes 1, 2 e 4).

---

## 1. Resumo executivo

**Visão geral.** A superfície de IPC é pequena e bem desenhada: 5 comandos, capabilities separadas por janela, nenhum `innerHTML`/`insertAdjacentHTML` no frontend (apenas `textContent`), nenhum script ou `style=` inline nos HTML, tokens por processo para `heartbeat` e para ações de janela, e `check:acl` cobrindo `generate_handler!` ↔ `build.rs` ↔ capabilities. Não encontrei vulnerabilidade de execução remota nem escalonamento de privilégio. Os problemas reais estão em **(a)** uma incompatibilidade entre a CSP e o suporte declarado a sites `http://`, **(b)** confiança parcial do token de *heartbeat* em um contexto que a página remota controla, **(c)** divergências entre o contrato do frontend e o do backend (atalho, validação de formulário, mensagens) e **(d)** ausência de verificações automáticas do contrato UI↔Rust.

**Nota de risco da Parte 3: MÉDIO** (1 achado Alta; nenhum Crítica).

| Severidade | Parte 1 | Parte 2 | **Parte 3** | Parte 4 | Total |
|---|---|---|---|---|---|
| Crítica | — | — | 0 | — | 0 |
| Alta | — | — | **1** | — | 1 |
| Média | — | — | **6** | — | 6 |
| Baixa | — | — | **7** | — | 7 |
| Info | — | — | **1** | — | 1 |
| **Total** | n/a | n/a | **15** | n/a | 15 |

(— = parte não analisada neste relatório.)

**Os 5 achados mais importantes**
1. **FLY-P3-001 (Alta, Confirmado)** — A CSP (`connect-src … https:`) bloqueia o `fetch` de sondagem de `main.ts` para sites `http://`; o backend, o README, a política de privacidade e o *listing* da Store prometem suporte a HTTP. Sites HTTP (ex.: painéis em rede local) nunca abrem em *release* — e funcionam em `tauri dev`, o que esconde o defeito.
2. **FLY-P3-003 (Média, Hipótese)** — Os padrões `http://*`/`https://*` da capability remota podem não casar com URLs que têm **porta explícita**; nesse caso o *heartbeat* é negado e o *watchdog* (Parte 1) vê "sem sinal" em sites como `https://host:8443`. Requer teste em Windows.
3. **FLY-P3-004 / FLY-P3-002 (Média, Provável)** — O token de *heartbeat* é entregue por um script que o chama via `window.__TAURI__.core.invoke`, objeto mutável acessível ao site; a capability remota vale para **qualquer** `http(s)://`. H2 foi **parcialmente confirmada**: o risco real é forjar atividade/ações cosméticas, não executar comandos.
4. **FLY-P3-010 (Média, Hipótese; transversal 1/2/3)** — Só o *bootstrap* tem tratamento de falha de rede. `save_config` navega direto ao site e *reloads* usam `location.reload()`; em falha, a janela sem bordas pode ficar numa página de erro nativa sem alça de arrastar nem "Tentar novamente".
5. **FLY-P3-014 (Média, Confirmado)** — Não há nenhuma verificação automática do contrato UI↔Rust (IDs, i18n nos 3 idiomas, códigos de erro, limites, CSP×esquemas). Entrego um script `check-ui-contract.mjs` **já validado**: detecta hoje exatamente 3 falhas e passa após as correções.

---

## 2. Escopo, premissas e limitações

**Lido por inteiro (Parte 3):** `index.html` (1 linha), `settings.html` (74), `src/main.ts` (151), `src/settings.ts` (374), `src/styles.css` (53), `vite.config.ts`, `tsconfig.json`, `package.json`, `src-tauri/tauri.conf.json`, as 3 capabilities, `src-tauri/build.rs`, `.gitignore`.
**Lido por necessidade de interface (outras partes):** ver coluna "Situação" da seção 0.

**Executado (Node 22.22.2, Linux, sem rede):**

| Comando | Resultado |
|---|---|
| `npm run check:tauri-versions` | ✅ passou — `{"api":"2.12.4","cli":"2.12.4","rustTauri":"2.12.5","rustBuild":"2.7.4"}` (mesmo major.minor 2.12) |
| `npm run check:acl` | ✅ passou |
| `npm run check:drag-overlay` | ✅ passou |
| `npm run check:contrast` | ✅ passou (mas cobre só **texto** ≥ 4,5:1; ver FLY-P3-012) |
| `npm run check:msix-config` | ✅ passou |
| Cálculos próprios (Node puro) | razões de contraste (FLY-P3-012), paridade de chaves i18n pt-BR/en/es, IDs HTML×TS, limites HTML×TS×Rust, uso de `innerHTML`/`style=` |
| `check-ui-contract.mjs` (proposto) | ✅ validado: falha hoje com 3 mensagens esperadas; passa em cópia com as correções simuladas |

**NÃO executado (e por quê):**
- `npm ci`, `npm run typecheck`, `npm run build`, `npm audit`: **sem acesso à rede** (`ENOTFOUND registry.npmjs.org`); `node_modules` não pôde ser instalado. O `typescript ~7.0.2` e o `vite ^8.3.5` **não foram executados**; a coerência do `package-lock.json` foi verificada por leitura (FLY-P3-015).
- `cargo fmt/test/clippy/audit`: `cargo` não instalado no ambiente (Linux; o alvo é `x86_64-pc-windows-msvc`).
- Qualquer teste em WebView2/Windows (CSP real, capabilities remotas, navegação com falha, teclado em layouts não-US). **Todas as conclusões sobre comportamento de runtime são por análise estática** e estão marcadas com a confiança correspondente.
- Comportamento interno do Tauri (`tauri-utils`, `urlpattern`, ordem de *initialization scripts*, `Manager::add_capability`): **não verificado** (sem rede/código-fonte do crate). Onde dependo disso, digo "não verificado".

**Premissas:** o frontend roda em `http://tauri.localhost` (Windows, `useHttpsScheme` padrão `false`); a CSP do `tauri.conf.json` é aplicada às páginas servidas pelo protocolo do app (não ao site remoto nem ao `devUrl`).

---

## 3. Mapa das 4 partes e interfaces com a Parte 3

```
Parte 3 (esta)                         Interface                         Outra parte
─────────────────────────────────────────────────────────────────────────────────────────────────────
src/settings.ts  ──invoke save_config/get_config/close_settings──►  commands.rs (P1)
src/settings.ts  ◄──event "config-changed"────────────────────────  commands.rs:321,403,422 / window.rs:1358 (P1/P2)
src/main.ts      ──invoke get_config / open_settings──────────────►  commands.rs (P1) / window::open_settings (P2)
src/main.ts      ──location.replace(target)──► on_navigation/decide_main (P2)  [FLY-P3-009]
tauri.conf.json  csp ◄── config.rs::normalize aceita http (P1)       [FLY-P3-001]
capabilities/main-remote-heartbeat.json ◄──► INIT heartbeat (window.rs:172, P2) ◄──► diagnostics.rs watchdog (P1)
tauri.conf.json withGlobalTauri ◄── INIT usa window.__TAURI__ (P2)    [FLY-P3-004]
scripts/check-acl.mjs (P4) fixa remote.urls == ["http://*","https://*"]  [FLY-P3-002/003]
settings.ts atalho ◄──► config.rs::normalize (P1) / Shortcut::from_str   [FLY-P3-005/006/007]
```

**Interfaces pendentes para as outras partes** (não analisadas aqui; ver seção 10): Parte 1 — `emit` global de `config-changed`, `heartbeat` sem *rate limit*, watchdog dependente de heartbeat; Parte 2 — `INIT` (token capturável, textos "calendário"), `LOGIN_SEGMENTS`, `SETTINGS_WINDOW_CREATING`; Parte 4 — `check-acl.mjs`, `dependabot`, docs/Store.

---

## 4. Índice de achados

| ID | Título | Parte | Severidade | Confiança | Esforço | Depende de |
|---|---|---|---|---|---|---|
| FLY-P3-001 | CSP bloqueia a sondagem de conectividade para sites `http://` | 3 (4) | **Alta** | Confirmado | P | — |
| FLY-P3-002 | Capability remota vale para qualquer `http(s)://`, não só o site configurado | 3 (1, 2, 4) | Média | Provável | M | FLY-P3-003 |
| FLY-P3-003 | Padrões `http://*`/`https://*` podem não casar com URLs com porta explícita | 3 (1, 4) | Média | Hipótese | P | — |
| FLY-P3-004 | Token de heartbeat é usado via `window.__TAURI__` mutável (capturável pelo site); `withGlobalTauri` | 3, 2 | Média | Provável | M | — |
| FLY-P3-005 | Atalho: Shift-only aceito no TS e rejeitado no Rust; mensagem enganosa | 3 (1) | Baixa | Confirmado | P | — |
| FLY-P3-006 | Captura de atalho por `event.code` ignora o layout; AltGr/teclas auxiliares geram aviso espúrio | 3 | Baixa | Provável | M | — |
| FLY-P3-007 | Sem proteção contra atalhos globais que sequestram teclas do sistema (Ctrl+C, Alt+F4…) | 3 (1) | Baixa | Provável | M | FLY-P3-006 |
| FLY-P3-008 | Validação nativa do `<form>` torna inalcançáveis as mensagens i18n; URL vazia não é validada pelo app | 3 | Média | Provável | P | — |
| FLY-P3-009 | Bootstrap (`main.ts`): `navigating` pode ficar preso; `location.replace` sem revalidar esquema; tentativas sobrepostas | 3 (2) | Baixa | Provável | P | FLY-P3-001 |
| FLY-P3-010 | Navegações pós-bootstrap contornam o tratamento de erro de rede (página de erro sem alça/retry) | 3, 1, 2 | Média | Hipótese | G | FLY-P3-001 |
| FLY-P3-011 | Endurecimento da CSP (`unsafe-inline`, `asset:`, `form-action https:`) | 3 | Baixa | Hipótese | P | FLY-P3-001 |
| FLY-P3-012 | Contraste de bordas de campo e do indicador de foco < 3:1 (tema claro) e `check:contrast` não cobre | 3 (4) | Baixa | Confirmado | P | — |
| FLY-P3-013 | Pequenos defeitos agrupados (i18n morto, estilo, tipagem, UX) | 3 | Baixa | Confirmado | M | FLY-P3-008, FLY-P3-005 |
| FLY-P3-014 | Ausência de verificações automáticas do contrato UI↔Rust/CSP (script proposto) | 3 (4) | Média | Confirmado | M | FLY-P3-001, FLY-P3-013 |
| FLY-P3-015 | Versões do frontend (`typescript ~7.0.2`, `vite ^8.3.5`): lockfile coerente; build não executado | 3 (4) | Info | Confirmado (lock) / não executado | P | — |

---

## 5. Achados detalhados — Parte 3

### FLY-P3-001 — CSP bloqueia a sondagem de conectividade para sites `http://`

| Campo | Valor |
|---|---|
| Parte | 3 (afeta o comportamento prometido pelas Partes 1 e 4) |
| Severidade | Alta |
| Categoria | Corretude |
| Confiança | Confirmado (análise estática da semântica de CSP; **não executado** em WebView2) |
| Esforço | P (<30 min) |
| Risco de regressão | Baixo |
| Local | `src-tauri/tauri.conf.json:16`; `src/main.ts:111-130` (especialmente 115-119); `src-tauri/src/config.rs:79-96` |
| Depende de | — |

**Evidência**

```json
// src-tauri/tauri.conf.json:16
"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: asset: http://asset.localhost; connect-src ipc: http://ipc.localhost https:; object-src 'none'; base-uri 'self'; form-action 'self' https:"
```
```ts
// src/main.ts:115-119
await fetch(new URL("/favicon.ico", target).href, {
  mode: "no-cors",
  cache: "no-store",
  signal: controller.signal
});
```
```rust
// src-tauri/src/config.rs:90 — o backend aceita http explicitamente
if !["http", "https"].contains(&url.scheme()) || url.host_str().is_none() || ...
```

**Descrição e causa raiz.** `index.html` é servido em `http://tauri.localhost` com a CSP acima. A diretiva `connect-src` só admite `ipc:`, `http://ipc.localhost` e o *scheme-source* `https:`. Um *scheme-source* `https:` casa `https:` (e `wss:`), **não** `http:`. Logo, `fetch("http://servidor/favicon.ico")` é bloqueado pela CSP **antes de sair da máquina** (a opção `mode: "no-cors"` não afeta CSP), o `fetch` rejeita com `TypeError`, o `catch` chama `failed()` e o usuário vê "Não foi possível acessar o site configurado" indefinidamente (com nova tentativa a cada 30 s que falha igualmente). `location.replace(target)` (linha 125) só é alcançado após o `fetch` resolver.

**Por que passa despercebido:** em `tauri dev` a página vem do servidor Vite (`devUrl`), que não envia essa CSP; o defeito só aparece em *build* de produção/MSIX.

**Impacto / cenário de reprodução.** Todo usuário que configure `http://…` (caso típico de widget: painel doméstico/IoT/servidor local, p.ex. `http://192.168.0.10:8080`, `http://localhost:3000`) não consegue abrir o site em *release*. A documentação (`README.md:81`, `PRIVACY_POLICY.md:32/106`, `MICROSOFT_STORE_LISTING_DRAFT.md:5/16`) e a mensagem `siteUrlHttpOnly` ("Use uma URL HTTP ou HTTPS válida…") prometem suporte a HTTP — há risco de reprovação/avaliação negativa na Store por funcionalidade anunciada que não funciona.
Reprodução: instalar o build de produção → Configurações → URL `http://127.0.0.1:8080/` (com `python -m http.server 8080` em outro terminal) → Salvar. Esperado: abrir; obtido: ver `failed`. (Após `save_config`, o Rust navega direto — ver FLY-P3-010 — então a falha se manifesta ao **reiniciar** o app, quando o *bootstrap* faz a sondagem.)

**Solução proposta**
1. Em `src-tauri/tauri.conf.json:16`, incluir `http:` em `connect-src` (o *scheme-source* `http:` casa `http` e `https`; mantém-se `https:` por clareza).
2. Não alterar as demais diretivas neste achado (FLY-P3-011 trata do endurecimento).
3. Adicionar a verificação de CSP×esquemas (FLY-P3-014) para impedir regressão.

```diff
- connect-src ipc: http://ipc.localhost https:;
+ connect-src ipc: http://ipc.localhost http: https:;
```
*Alternativa descartada:* mover a sondagem para o Rust (evitaria ampliar `connect-src`, mas exige cliente HTTP/dependência nova e muda a arquitetura — viola a regra 6 do prompt sem necessidade). *Alternativa descartada 2:* remover a sondagem e navegar direto — perde o fluxo "Tentar novamente" (ver FLY-P3-010).

**Arquivos a alterar:** `src-tauri/tauri.conf.json`.

**Critérios de aceite**
- Em build de produção, `http://127.0.0.1:8080/` e `http://localhost:<porta>/` abrem na janela principal após reiniciar o app.
- Sites `https://` continuam funcionando; site inalcançável continua mostrando "Não foi possível acessar…" e botão "Tentar novamente".
- `node scripts/check-ui-contract.mjs` (FLY-P3-014) passa a validar `http:` em `connect-src`.

**Como verificar:** passos de reprodução acima em build `tauri build` (ou `tauri build --debug` para ver erros de CSP no *devtools*). Em `--debug`, antes da correção o console mostra `Refused to connect to 'http://…' because it violates the following Content Security Policy directive: "connect-src …"`.

**Cuidados:** não adicionar `*`/`'unsafe-eval'`; não mexer em `script-src`. Preservar `ipc:` e `http://ipc.localhost` (IPC do Tauri no Windows). Se um dia `useHttpsScheme` for ativado, revisar `http://ipc.localhost` → `https://ipc.localhost`.

---

### FLY-P3-002 — Capability remota vale para qualquer `http(s)://`, não só o site configurado

| Campo | Valor |
|---|---|
| Parte | 3 (1, 2, 4) |
| Severidade | Média |
| Categoria | Segurança |
| Confiança | Provável |
| Esforço | M (até 2 h) para a opção B; P para a opção A |
| Risco de regressão | Médio (opção B) / Baixo (opção A) |
| Local | `src-tauri/capabilities/main-remote-heartbeat.json:8-13`; `src-tauri/tauri.conf.json:18-22`; `scripts/check-acl.mjs:29` |
| Depende de | FLY-P3-003 (padrão de URL/porta) |

**Evidência**
```json
// main-remote-heartbeat.json:6-16
"windows": ["main"],
"remote": { "urls": ["http://*", "https://*"] },
"permissions": ["allow-heartbeat"]
```
```js
// scripts/check-acl.mjs:29 — fixa exatamente esse valor
JSON.stringify(remoteCapability.remote?.urls) !== JSON.stringify(["http://*", "https://*"])
```

**Descrição e causa raiz.** Por ser estático, o JSON não pode citar o host que o usuário escolhe; foi usado um curinga total. Qualquer documento carregado na janela `main` (o site configurado, subdomínios permitidos por `host_matches`, *iframes* de terceiros embutidos nele e páginas de erro redirecionadas) pode chamar `heartbeat`. A única barreira é o token (`state.heartbeat_token`, UUID por processo), que o site pode capturar (FLY-P3-004). Janelas `login-*` **não** estão em `windows` (correto).

**Avaliação de impacto real (H2 — parte "capability").** O único comando liberado é `heartbeat`, que apenas atualiza um carimbo de tempo (`diagnostics.rs:280-302`). Não há leitura de configuração, escrita, abertura de janelas nem execução. Um site hostil que consiga o token pode **forjar atividade** (silenciar o diagnóstico do *watchdog*, Parte 1) e, via FLY-P3-004, disparar ações de arrastar/redimensionar da própria janela (apenas se `locked == false`). Impacto: **baixo-médio**; sem escalonamento. Por isso a severidade é Média e não Alta.

**Solução proposta (decisão humana — apresentar as opções):**
- **Opção A (recomendada agora):** manter o curinga, **documentar o modelo de ameaça** e garantir (Parte 1) que o *watchdog* não toma ações destrutivas baseadas apenas em ausência/presença de heartbeat. Registrar em comentário no `main-remote-heartbeat.json` (campo `description`, em pt-BR, ver FLY-P3-013) que o escopo é amplo de propósito e protegido por token. Esforço P.
- **Opção B:** capability **dinâmica** limitada ao host configurado (e subdomínios), registrada em tempo de execução ao criar a janela e ao salvar o site. Remover `main-remote-heartbeat` da lista estática de `tauri.conf.json` e do `check-acl.mjs`, substituindo por checagem de que a capability dinâmica é criada com `permissions == ["allow-heartbeat"]`. **Não verificado:** a existência e a assinatura exata de `Manager::add_capability` na versão 2.12 do `tauri` (consultar a documentação oficial antes de implementar); capabilities adicionadas em runtime podem não ser revogáveis ao trocar de site (exigir reinício ou nova janela). Por esse custo/risco, só recomendo B se o dono quiser reduzir a superfície mesmo com o token.

**Arquivos a alterar:** A: `src-tauri/capabilities/main-remote-heartbeat.json` (descrição). B: `tauri.conf.json`, `capabilities/main-remote-heartbeat.json`, `window.rs` (Parte 2), `commands.rs` (Parte 1), `scripts/check-acl.mjs` (Parte 4) — **atualizar todos juntos**.

**Critérios de aceite:** A: descrição atualizada; `npm run check:acl` passa. B: heartbeat continua funcionando no site configurado e **é negado** em host não configurado (teste manual: abrir página de outro host na janela `main` e verificar no log a ausência de "Primeiro sinal de atividade").

**Como verificar:** `npm run check:acl`; em Windows, `tauri build --debug`, observar `diagnostics` (log) para o evento de heartbeat.

**Cuidados:** não conceder nenhuma outra permissão ao `remote`; manter `windows: ["main"]`; nunca adicionar `core:default`. Se alterar `build.rs`/`generate_handler!`, atualizar os três e rodar `check:acl`.

---

### FLY-P3-003 — Padrões `http://*`/`https://*` podem não casar com URLs com porta explícita

| Campo | Valor |
|---|---|
| Parte | 3 (1, 4) |
| Severidade | Média |
| Categoria | Corretude |
| Confiança | Hipótese (depende de detalhes do `urlpattern`/`tauri-utils` **não verificados**) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `src-tauri/capabilities/main-remote-heartbeat.json:8-13`; `scripts/check-acl.mjs:29` |
| Depende de | — |

**Evidência:** mesmo trecho de FLY-P3-002.

**Descrição e causa raiz.** Os padrões são interpretados como *URLPattern*. Em *URLPattern*, componentes não escritos no padrão (aqui, a porta) não são necessariamente curingas; para `http://*` a porta pode ser tratada como vazia (porta padrão). Se for assim, `https://host:8443/` e `http://192.168.0.10:8080/` **não** casam, o `invoke('heartbeat')` é negado pela ACL (rejeição silenciada por `.catch(()=>{})` em `window.rs:172`) e o *watchdog* (Parte 1) registra ausência de sinal para sites com porta explícita. Não consegui confirmar no código-fonte do `tauri-utils` (sem rede): **não verificado**.

**Impacto / cenário.** Usuários com sites em portas não padrão (os mesmos de FLY-P3-001) teriam diagnósticos/alertas indevidos. Hipótese a testar em Windows.

**Solução proposta.** 1) **Verificar primeiro** (ver "Como verificar"). 2) Se confirmado, incluir curinga de porta e atualizar o *script* de checagem:
```diff
 "remote": {
   "urls": [
-    "http://*",
-    "https://*"
+    "http://*:*",
+    "https://*:*"
   ]
 }
```
```diff
- JSON.stringify(remoteCapability.remote?.urls) !== JSON.stringify(["http://*", "https://*"])
+ JSON.stringify(remoteCapability.remote?.urls) !== JSON.stringify(["http://*:*", "https://*:*"])
```
(Se `*:*` também casar URLs sem porta — comportamento esperado de um curinga que aceita vazio — a cobertura é total; **validar** com site em porta 80/443 e em porta explícita.)

**Arquivos a alterar:** `src-tauri/capabilities/main-remote-heartbeat.json`, `scripts/check-acl.mjs` (Parte 4; alterar junto).

**Critérios de aceite:** com `http://127.0.0.1:8080/` **e** `https://example.com/` (sem porta) o log mostra "Primeiro sinal de atividade do JavaScript/WebView2 recebido."; `npm run check:acl` passa.

**Como verificar (Windows):** servir uma página qualquer em `http://127.0.0.1:8080/` (`python -m http.server 8080`), configurar no Floatly, abrir `%APPDATA%\WidgetWebsite` e procurar o log de INFO "Primeiro sinal de atividade…" em até ~10 s; repetir com um site sem porta como controle.

**Cuidados:** não ampliar `permissions`; manter coerência com FLY-P3-002.

---

### FLY-P3-004 — Token de heartbeat é usado via `window.__TAURI__` mutável (capturável pelo site); papel de `withGlobalTauri`

| Campo | Valor |
|---|---|
| Parte | 3, 2 (arquivo `window.rs`) |
| Severidade | Média |
| Categoria | Segurança |
| Confiança | Provável (não reproduzido; a mecânica é lógica de JS padrão) |
| Esforço | M |
| Risco de regressão | Médio (pode quebrar o heartbeat se o objeto global ainda não existir no *boot*) |
| Local | `src-tauri/tauri.conf.json:13`; `src-tauri/src/window.rs:23-24, 58, 172-174`; `src-tauri/src/commands.rs:121-129` |
| Depende de | — |

**Evidência**
```js
// window.rs:23-24 e 172-174 (dentro da string INIT)
const ACTION_TOKEN='__WIDGET_ACTION_TOKEN__';
const HEARTBEAT_TOKEN='__WIDGET_HEARTBEAT_TOKEN__';
...
function heartbeat(){try{const core=window.__TAURI__&&window.__TAURI__.core;if(core&&typeof core.invoke==='function')core.invoke('heartbeat',{token:HEARTBEAT_TOKEN}).catch(()=>{})}catch(_){}}
setTimeout(heartbeat,1000);
setInterval(heartbeat,5000)
```
```json
// tauri.conf.json:13
"withGlobalTauri": true,
```

**Descrição e causa raiz (H2 — parte "tokens").**
- Os tokens são constantes **dentro de um IIFE** (`(function boot(){…})()`): scripts do site **não** os leem por nome global. Isso foi corretamente projetado.
- Porém `heartbeat()` resolve `window.__TAURI__.core.invoke` **a cada chamada**, em um objeto global do mesmo *world* do site. Um script do site que substitua `window.__TAURI__` (ou `.core.invoke`) antes do primeiro disparo (1 s após o carregamento) recebe `{token}` como argumento. *Provável* — depende de `window.__TAURI__` ser regravável (não verificado).
- `action()` (linha 58) usa `location.href='widget-action://TOKEN/…'`. O site pode, em Chromium, observar navegações iniciadas por script (p.ex. via Navigation API `navigation.addEventListener('navigate', …)`) e ler `event.destination.url`. **Hipótese**, não testada.
- **Parte refutada de H2:** `withGlobalTauri: true` **não amplia** o que o site pode *executar*: a ACL (capabilities) é quem decide, e o remoto só tem `allow-heartbeat`; `window.__TAURI_INTERNALS__` já existe em qualquer página do Tauri independentemente dessa opção. O efeito de `withGlobalTauri` aqui é só conveniência — e **dependência**: o `INIT` é o único consumidor de `window.__TAURI__` (o frontend local usa `import { invoke } from "@tauri-apps/api/core"`, que não precisa do global). Desligar a opção quebraria o heartbeat sem outra mudança.

**Impacto / cenário.** Site hostil/comprometido (ou *script* de terceiros embutido) captura o token de heartbeat → forja atividade (ver FLY-P3-002) e, se capturar também o de ação, dispara drag/resize da própria janela quando destravada. Sem leitura de dados do usuário nem comandos extras.

**Solução proposta (melhor esforço, proporcional).**
1. **Capturar a referência no *boot*** do `INIT` (executa em `document-created`, antes dos scripts do site) e usá-la depois; manter *fallback* ao comportamento atual se o global ainda não existir naquele instante:
```diff
-function heartbeat(){try{const core=window.__TAURI__&&window.__TAURI__.core;if(core&&typeof core.invoke==='function')core.invoke('heartbeat',{token:HEARTBEAT_TOKEN}).catch(()=>{})}catch(_){}}
+const BOOT_CORE=(window.__TAURI__&&window.__TAURI__.core&&typeof window.__TAURI__.core.invoke==='function')?window.__TAURI__.core:null;
+const BOOT_INVOKE=BOOT_CORE?BOOT_CORE.invoke.bind(BOOT_CORE):null;
+function heartbeat(){try{const invoke=BOOT_INVOKE||(window.__TAURI__&&window.__TAURI__.core&&window.__TAURI__.core.invoke&&window.__TAURI__.core.invoke.bind(window.__TAURI__.core));if(invoke)invoke('heartbeat',{token:HEARTBEAT_TOKEN}).catch(()=>{})}catch(_){}}
```
   Limite honesto: `invoke` do Tauri pode chamar `window.__TAURI_INTERNALS__.invoke` internamente (não verificado); portanto isto reduz, não elimina, a captura.
2. **Aceitar o risco residual** e documentá-lo (FLY-P3-002 opção A). Confirmar na Parte 1 que o *watchdog* não encerra/reinicia nada só por falta de heartbeat.
3. **Não** remover `withGlobalTauri` sem antes trocar o `INIT` para outra via (instável/interna) — alternativa descartada.

**Arquivos a alterar:** `src-tauri/src/window.rs` (string `INIT`; **arquivo da Parte 2** — coordenar com quem corrige a Parte 2 e conferir `npm run check:drag-overlay`, que testa trechos do `INIT`).

**Critérios de aceite:** em Windows, o log registra "Primeiro sinal de atividade…" em ≤ 10 s após abrir o site; `npm run check:drag-overlay` e `cargo test` passam; teste manual: página de teste que sobrescreve `window.__TAURI__` em `<head>` **não** recebe o token via `core.invoke`.

**Como verificar:** criar `teste-token.html` com `<script>window.__TAURI__={core:{invoke:(c,a)=>{document.title=JSON.stringify(a)}}}</script>` servido em `http://127.0.0.1:8080/`; antes do patch o título passa a conter `token`; depois, não.

**Cuidados:** `INIT` roda também no `index.html` local; o *fallback* garante que o heartbeat continue lá. Não logar o token. Não alterar `generate_handler!`.

---

### FLY-P3-005 — Atalho: Shift-only aceito no TypeScript e rejeitado no Rust; mensagem enganosa

| Campo | Valor |
|---|---|
| Parte | 3 (1) |
| Severidade | Baixa |
| Categoria | Corretude / i18n |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `src/settings.ts:304-327` (esp. 307-316); `src/settings.ts:48, 78, 108` (texto `shortcutModifierRequired`); `src-tauri/src/config.rs:132-137` |
| Depende de | — |

**Evidência**
```ts
// settings.ts:307-316 — qualquer modificador (inclusive só SHIFT) é aceito
if (event.ctrlKey) parts.push("CTRL");
if (event.altKey) parts.push("ALT");
if (event.shiftKey) parts.push("SHIFT");
if (event.metaKey) parts.push("SUPER");
if (parts.length === 0) { ... say(t("shortcutModifierRequired"), "warning"); return; }
```
```rust
// config.rs:132-137 — o backend exige CTRL, ALT ou SUPER
if !modifiers.iter().any(|m| matches!(*m, "CTRL" | "ALT" | "SUPER")) {
    return Err("config.shortcut_modifier_required".into());
}
```

**Descrição e causa raiz.** O frontend exibe `SHIFT+A` no campo como válido; ao salvar, `normalize()` devolve `config.shortcut_modifier_required` e a UI mostra "Inclua pelo menos um modificador (Ctrl, Alt, Shift ou Win)." — texto que **lista Shift como suficiente**, contradizendo a regra real. O usuário não entende por que `Shift+A` "tem modificador" e foi recusado.

**Solução proposta.**
1. Em `settings.ts`, exigir CTRL/ALT/SUPER **no momento da captura** (mesma regra do Rust).
2. Corrigir o texto `shortcutModifierRequired` nos **3 idiomas** para não citar Shift como suficiente.
```diff
-  if (parts.length === 0) {
+  if (!parts.some((p) => p === "CTRL" || p === "ALT" || p === "SUPER")) {
     event.preventDefault();
     say(t("shortcutModifierRequired"), "warning");
     return;
   }
```
```diff
 // pt-BR
-shortcutModifierRequired: "Inclua pelo menos um modificador (Ctrl, Alt, Shift ou Win).",
+shortcutModifierRequired: "Inclua pelo menos Ctrl, Alt ou Win (Shift sozinho não basta).",
 // en
-shortcutModifierRequired: "Include at least one modifier (Ctrl, Alt, Shift or Win).",
+shortcutModifierRequired: "Include at least Ctrl, Alt or Win (Shift alone is not enough).",
 // es
-shortcutModifierRequired: "Incluye al menos un modificador (Ctrl, Alt, Shift o Win).",
+shortcutModifierRequired: "Incluye al menos Ctrl, Alt o Win (Shift solo no basta).",
```
**Arquivos a alterar:** `src/settings.ts` (as 3 linhas de tradução e o `keydown`). *Não* alterar o Rust (a regra do backend é a de referência).

**Critérios de aceite:** pressionar `Shift+A` no campo mostra o aviso e **não** altera o valor; `Ctrl+Shift+A` é aceito e salvo; os 3 idiomas atualizados; `check-ui-contract.mjs` passa.
**Como verificar:** manual na janela de Configurações; teste Rust existente de `normalize` (Parte 1) deve cobrir `SHIFT+A` → `Err("config.shortcut_modifier_required")` (se não cobrir, a Parte 1 deve adicioná-lo).
**Cuidados:** o mesmo texto é usado quando o backend devolve `config.shortcut_modifier_required` (mapa `backendErrorKeys`); manter a chave.

---

### FLY-P3-006 — Captura de atalho por `event.code` ignora o layout; AltGr/teclas auxiliares geram aviso espúrio

| Campo | Valor |
|---|---|
| Parte | 3 |
| Severidade | Baixa |
| Categoria | UX/A11y |
| Confiança | Provável (não testado em layouts não-US) |
| Esforço | M |
| Risco de regressão | Médio (afeta a captura do atalho) |
| Local | `src/settings.ts:291-303, 304-327` |
| Depende de | — |

**Evidência**
```ts
// settings.ts:306 e 317
if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) return;
...
const primary = shortcutKeyFromCode(event.code);   // /^Key[A-Z]$/ → code.slice(3)
```

**Descrição e causa raiz.**
1. `event.code` é a **posição física** da tecla (padrão US). Em AZERTY/QWERTZ, a tecla rotulada "A" tem `code === "KeyQ"` (AZERTY). O campo exibirá `CTRL+ALT+Q` e o Rust registrará a tecla virtual `Q` (o registro global do Windows usa o **código virtual**, que segue a letra do layout) — o usuário precisará apertar a tecla rotulada "Q", não a que apertou. (A suposição "VK = letra do layout" é do Windows; o mapeamento exato do crate `global-hotkey` **não foi verificado**.)
2. A lista de teclas ignoradas omite `AltGraph`, `CapsLock`, `NumLock`, `ScrollLock`, `ContextMenu`, `Dead`, `Process`, `Unidentified`. Em layouts com AltGr (pt-PT, es, de…), pressionar AltGr como primeira tecla do combo dispara "Essa tecla não é compatível com o atalho global" e `preventDefault`, interrompendo a captura enquanto o usuário ainda monta a combinação.

**Impacto.** Usuários de teclados não-US podem configurar um atalho diferente do que pretendiam e receber avisos confusos.

**Solução proposta.**
```diff
-const primary = shortcutKeyFromCode(event.code);
+const primary = shortcutKeyFromEvent(event);
```
```ts
// novo, junto de shortcutKeyFromCode
function shortcutKeyFromEvent(event: KeyboardEvent): string | null {
  // Letras: usa o caractere produzido (respeita AZERTY/QWERTZ etc.); demais teclas: código físico.
  if (/^[a-z]$/i.test(event.key)) return event.key.toUpperCase();
  return shortcutKeyFromCode(event.code);
}
```
```diff
-if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) return;
+if (["Control", "Alt", "AltGraph", "Shift", "Meta", "OS", "CapsLock", "NumLock", "ScrollLock", "ContextMenu", "Dead", "Process", "Unidentified"].includes(event.key) || event.isComposing) return;
```
**Arquivos a alterar:** `src/settings.ts`.
**Critérios de aceite:** em AZERTY (ou emulando-o em Configurações do Windows), pressionar Ctrl+Alt+<tecla rotulada A> exibe `CTRL+ALT+A`, salva e **ativa** com essa mesma tecla; pressionar AltGr isolado não exibe aviso; em US o comportamento é idêntico ao atual.
**Como verificar:** manual (Windows: adicionar teclado francês/alemão em Idioma e região); verificar no log de diagnóstico o registro do atalho.
**Cuidados:** manter `Digit0-9`, `F1-F24`, `NUMPAD*` e nomeadas por `event.code` (um dígito com Shift muda `event.key`). Manter nomes aceitos por `Shortcut::from_str` (ver FLY-P3-014 sobre teste de contrato).

---

### FLY-P3-007 — Sem proteção contra atalhos globais que sequestram teclas do sistema

| Campo | Valor |
|---|---|
| Parte | 3 (1) |
| Severidade | Baixa |
| Categoria | UX / Robustez |
| Confiança | Provável (não executado em Windows) |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | `src/settings.ts:304-327`; `src-tauri/src/config.rs:107-147`; `src-tauri/src/commands.rs:187-217` |
| Depende de | FLY-P3-006 |

**Descrição e causa raiz.** A UI aceita qualquer combinação com Ctrl/Alt/Win + tecla suportada, inclusive `CTRL+C`, `CTRL+V`, `CTRL+X`, `CTRL+Z`, `ALT+F4`, `ALT+SPACE`, `ALT+TAB`. Um atalho **global** registrado com essas combinações consome o evento em todo o sistema (copiar/colar/fechar janela deixam de funcionar fora do Floatly). Como `RegisterHotKey` só falha se *outro hotkey* já o tiver registrado, o registro tende a ter sucesso e o aviso `shortcutWarning` não aparece.

**Solução proposta.** Lista de bloqueio **no frontend** (mensagem imediata, 3 idiomas) — a Parte 1 pode espelhar no `normalize()` se desejar.
```ts
const RESERVED_SHORTCUTS = new Set([
  "CTRL+C","CTRL+V","CTRL+X","CTRL+Z","CTRL+Y","CTRL+A","CTRL+S",
  "ALT+F4","ALT+TAB","ALT+SPACE","ALT+ESC","CTRL+ESC","CTRL+SHIFT+ESC"
]);
// em keydown, após montar `parts` e `primary`:
const combo = [...parts, primary].join("+");
if (RESERVED_SHORTCUTS.has(combo)) {
  event.preventDefault();
  say(t("shortcutReserved"), "warning");
  return;
}
```
Novas traduções (`shortcutReserved`): pt-BR "Essa combinação é usada pelo Windows ou por outros programas; escolha outra."; en "This combination is used by Windows or other programs; choose another one."; es "Esta combinación la usan Windows u otros programas; elige otra."
**Arquivos a alterar:** `src/settings.ts` (constante, `keydown`, 3 traduções). **Critérios de aceite:** as combinações da lista exibem o aviso e não preenchem o campo; as demais continuam aceitas; `check-ui-contract.mjs` (paridade de chaves) passa. **Como verificar:** manual. **Cuidados:** a ordem canônica das partes é CTRL, ALT, SHIFT, SUPER (igual à de `normalize`); `CTRL+ALT+C` (padrão) não pode entrar na lista.

---

### FLY-P3-008 — Validação nativa do `<form>` torna inalcançáveis as mensagens i18n; URL vazia não é validada pelo app

| Campo | Valor |
|---|---|
| Parte | 3 |
| Severidade | Média |
| Categoria | UX/A11y · i18n |
| Confiança | Provável (comportamento padrão do HTML; o idioma exato do balão nativo depende do WebView2/SO) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `settings.html:24, 38, 40-41, 56-57`; `src/settings.ts:202-229, 252-282`; `src-tauri/src/config.rs:75-78` |
| Depende de | — |

**Evidência**
```html
<!-- settings.html:24 e 38-41 -->
<form id="settings-form">
<input id="site_url" type="text" placeholder="https://example.com" required>
<input id="width" type="number" min="150" max="2000" step="1" required>
```
```ts
// settings.ts:202-208 e 252-256 — mensagens próprias de campo vazio/fora da faixa
if (!raw) throw new Error("FIELD_REQUIRED");
if (!Number.isInteger(parsed) || parsed < min || parsed > max) throw new Error("FIELD_RANGE");
...
form.addEventListener("submit", async (event) => { event.preventDefault(); ...
```

**Descrição e causa raiz.** Sem `novalidate`, o navegador executa a *constraint validation* **antes** de disparar `submit`: campos `required` vazios ou fora de `min/max/step` bloqueiam o envio e mostram um balão nativo (no idioma do WebView2/SO, **não** no idioma escolhido no Floatly). O handler `submit` e as mensagens `fieldRequired`/`fieldRange` (3 idiomas) ficam, na prática, **inalcançáveis**. Além disso, o `required` de `site_url` é a **única** barreira contra URL vazia: o `normalize()` aceita vazio (`config.rs:75-78` → `site_url.clear()`), então a chave `siteUrlRequired` mapeada para `config.site_url_required` nunca é produzida pelo Rust (FLY-P3-013).

**Impacto.** Interface parcialmente em outro idioma e sem `aria-invalid`/foco; mensagens traduzidas "mortas".

**Solução proposta.**
1. `settings.html:24` → `<form id="settings-form" novalidate>` (mantém `min/max/required` como dicas de teclado/semântica).
2. Em `settings.ts`, tratar o campo vazio da URL no cliente e associar o erro ao campo (foco + `aria-invalid`):
```diff
+class FieldError extends Error {
+  constructor(readonly code: "FIELD_REQUIRED" | "FIELD_RANGE" | "SITE_REQUIRED", readonly field: string) { super(code); }
+}
 function readInt(id: string, min: number, max: number): number {
   const raw = f(id).value.trim();
-  if (!raw) throw new Error("FIELD_REQUIRED");
+  if (!raw) throw new FieldError("FIELD_REQUIRED", id);
   const parsed = Number(raw);
-  if (!Number.isInteger(parsed) || parsed < min || parsed > max) throw new Error("FIELD_RANGE");
+  if (!Number.isInteger(parsed) || parsed < min || parsed > max) throw new FieldError("FIELD_RANGE", id);
   return parsed;
 }
 function read(): C {
   const value = (id: string) => f(id).value;
+  if (!value("site_url").trim()) throw new FieldError("SITE_REQUIRED", "site_url");
```
```diff
 form.addEventListener("submit", async (event) => {
   event.preventDefault();
+  for (const id of ids) f(id).removeAttribute("aria-invalid");
   say(t("saving"));
   try { ... } catch (error) {
+    if (error instanceof FieldError) {
+      const el = f(error.field);
+      el.setAttribute("aria-invalid", "true");
+      el.focus();
+      say(t(error.code === "SITE_REQUIRED" ? "siteUrlRequired" : error.code === "FIELD_REQUIRED" ? "fieldRequired" : "fieldRange"), "error");
+      return;
+    }
     const message = error instanceof Error ? error.message : String(error);
-    if (message === "FIELD_REQUIRED") ...  // remover os dois ramos antigos FIELD_*
```
3. CSS opcional: `.settings-section input[aria-invalid="true"]{border-color:var(--settings-message-error)}`.
**Arquivos a alterar:** `settings.html`, `src/settings.ts`, `src/styles.css` (opcional). **Critérios de aceite:** com campo vazio ou fora da faixa o app (não o navegador) mostra a mensagem no idioma escolhido, foca o campo e marca `aria-invalid`; URL vazia **não** chega ao backend; `npm run typecheck` passa. **Como verificar:** manual nos 3 idiomas; (opcional) teste de DOM com jsdom se o time adotar testes de frontend (FLY-P3-014). **Cuidados:** `config.site_url_required` deixa de ser necessário no mapa `backendErrorKeys` (remover junto, FLY-P3-013); **não** passar a rejeitar URL vazia no Rust (o estado "sem site" é legítimo no primeiro uso).

---

### FLY-P3-009 — Bootstrap (`main.ts`): `navigating` pode ficar preso; `location.replace` sem revalidar esquema; tentativas sobrepostas

| Campo | Valor |
|---|---|
| Parte | 3 (2) |
| Severidade | Baixa |
| Categoria | Robustez |
| Confiança | Provável (o cenário de bloqueio depende de `decide_main`, Parte 2) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `src/main.ts:59-60, 83-91, 101-130, 125`; `src-tauri/src/window.rs:247-269, 305-322, 528-594` |
| Depende de | FLY-P3-001 |

**Evidência**
```ts
// main.ts:123-125
navigating = true;
status.textContent = text("opening");
location.replace(target);
```
```rust
// window.rs:305-312 — provedor de autenticação é decidido ANTES do host configurado
let host = url.host_str().unwrap_or("");
if is_auth_provider(host, url.path()) { return NavAction::OpenLogin; }
```

**Descrição e causa raiz.**
1. **`navigating` preso.** `navigating=true` é definido antes de `location.replace`. Se `on_navigation` **recusar** a navegação (retorna `false`), a página continua viva, o texto fica "Abrindo o site…", o botão segue oculto (`retry.hidden=true` desde a linha 109) e `attempt()` retorna na linha 102 para sempre. Cenário concreto: o usuário configura como site um host que `is_auth_provider` reconhece (`accounts.google.com`, `login.microsoftonline.com`, `login.live.com`, `appleid.apple.com`, ou `facebook.com/login…`): `decide_main` devolve `OpenLogin` (abre popup) e **bloqueia** a navegação principal. (Cenário raro; a política em si é da Parte 2.)
2. **Sem revalidar o esquema.** `target` vem de `get_config` (normalizado: só http/https), mas `location.replace(target)` não confirma isso no cliente. Hoje a CSP (`script-src 'self'`) impediria `javascript:`; é defesa em profundidade barata.
3. **Tentativas sobrepostas e *timer* nunca limpo.** O clique em "Tentar novamente" pode sobrepor uma tentativa silenciosa do `setInterval` (`navigating` só vira `true` após o `fetch`). Efeito colateral pequeno (texto oscila), mas o intervalo de 30 s continua rodando enquanto a janela estiver oculta.

**Solução proposta.**
```diff
+let probing = false;
+function httpTarget(value: string): URL | null {
+  try { const u = new URL(value); return u.protocol === "http:" || u.protocol === "https:" ? u : null; } catch { return null; }
+}
 async function attempt(quiet = false) {
-  if (navigating) return;
+  if (navigating || probing) return;
   if (!target) { showSetup(); return; }
+  const url = httpTarget(target);
+  if (!url) { showSetup(text("configFailed")); return; }
   if (!quiet) { status.textContent = text("checking"); retry.hidden = true; }
+  probing = true;
   try {
     ...
-      await fetch(new URL("/favicon.ico", target).href, {
+      await fetch(new URL("/favicon.ico", url).href, {
     ...
     navigating = true;
     status.textContent = text("opening");
-    location.replace(target);
+    // Se a navegação for recusada/cancelada (a página continua viva), volta ao estado de erro.
+    window.setTimeout(() => { navigating = false; failed(); }, 20000);
+    location.replace(url.href);
   } catch {
     navigating = false;
     failed();
+  } finally {
+    probing = false;
   }
 }
```
**Arquivos a alterar:** `src/main.ts`. **Critérios de aceite:** site inalcançável → "Não foi possível acessar…" + retry (inalterado); site válido → abre (a página é descarregada, o *timer* de 20 s nunca dispara); com `site_url = https://login.live.com/` a tela volta a mostrar a falha/retry em ≤ 20 s em vez de ficar presa; `npm run typecheck` passa. **Como verificar:** manual com os 3 casos; `typecheck`. **Cuidados:** não aumentar o tempo de sondagem (8 s); textos continuam nos 3 idiomas; não alterar `decide_main` neste achado (Parte 2).

---

### FLY-P3-010 — Navegações pós-bootstrap contornam o tratamento de erro de rede

| Campo | Valor |
|---|---|
| Parte | 3, 1, 2 (transversal) |
| Severidade | Média |
| Categoria | Robustez / UX |
| Confiança | Hipótese (depende do comportamento do WebView2 em falha de navegação — **não testado**) |
| Esforço | G |
| Risco de regressão | Médio |
| Local | `src-tauri/src/commands.rs:326-338` (navegação direta ao salvar); `commands.rs:362-377`, `window.rs:161, 954, 1387-1405` (`location.reload()` via `eval`); `src/main.ts:101-130` (único lugar com *retry*) |
| Depende de | FLY-P3-001 |

**Evidência**
```rust
// commands.rs:326-331 — ao trocar o site, navega direto, sem passar pela sondagem do main.ts
if old.site_url != c.site_url {
    if let (Some(main), Ok(url)) = (app.get_webview_window("main"), url::Url::parse(&c.site_url)) {
        if let Err(error) = main.navigate(url) { ...
```
```rust
// window.rs:1401 (auto-reload) e commands.rs:368 (bandeja): window.eval("location.reload()")
```

**Descrição e causa raiz.** O fluxo "sondar → abrir → *tentar de novo*" existe só no `index.html`. Depois que o site está carregado, o tratamento de falha depende do `INIT` (overlay "sem conexão", `window.rs:168`), que roda **dentro de uma página já carregada**. Se a navegação em si falhar (DNS/rede fora no momento do `main.navigate` após salvar, ou do `location.reload()` do auto-reload/F5/bandeja), o WebView2 pode exibir sua **página de erro nativa** (documento de erro, em que scripts de inicialização não rodam — *não verificado*), na janela **sem decorações**: sem alça de arrastar, sem botão "Tentar novamente" do Floatly. O usuário só se recupera pela bandeja ("recarregar") — que repete o mesmo caminho.

**Impacto.** Janela flutuante "presa" numa página de erro após um *reload* automático com rede instável (Wi-Fi caindo é o cenário típico de um widget sempre aberto) ou ao salvar um site momentaneamente inalcançável.

**Verificar primeiro (Hipótese):** em Windows, abrir um site, desligar a rede, (a) aguardar o auto-reload, (b) apertar F5, (c) trocar o site nas Configurações para um host inexistente. Observar se aparece a página de erro do Edge e se há como arrastar/tentar de novo.

**Solução proposta (se confirmado) — recomendada:**
1. Em `save_config`, quando o site mudar, navegar a janela principal para a **página de bootstrap** (que sonda e oferece retry), em vez de ao site. Helper sugerido (**não verificado**: confirmar `app.config().build.dev_url` e a URL de produção do app na versão 2.12):
```rust
fn bootstrap_url(app: &AppHandle) -> Option<url::Url> {
    if cfg!(debug_assertions) {
        if let Some(dev) = app.config().build.dev_url.clone() { return Some(dev); }
    }
    url::Url::parse("http://tauri.localhost/").ok()
}
// em save_config: main.navigate(bootstrap_url(&app)?) em vez de navigate(url)
```
   (`is_internal_app_url` já autoriza `http://tauri.localhost` e, em debug, `127.0.0.1`.)
2. Para o *reload* (auto-reload/F5/bandeja): em vez de `location.reload()` incondicional, o `eval` pode executar uma sondagem de `/favicon.ico` com o mesmo critério de `main.ts` e só então recarregar; se falhar, **não recarrega** (o overlay do `INIT` já informa a falta de conexão). Isso é alteração de Parte 2 (`window.rs`) e Parte 1 (`commands.rs`).
*Alternativa descartada:* manter como está e documentar — aceitável se o teste mostrar que o WebView2 mantém o `INIT` ativo em páginas de erro.

**Arquivos a alterar:** `src-tauri/src/commands.rs`, `src-tauri/src/window.rs` (INIT/auto-reload), possivelmente `src/main.ts` (nenhuma mudança se o bootstrap já cobre). **Critérios de aceite:** com a rede desligada, nenhum dos três caminhos deixa a janela sem mecanismo de recuperação (overlay ou tela de retry); com a rede ligada, trocar o site continua abrindo o novo site. **Como verificar:** passos acima + `cargo test` (adicionar teste unitário para `bootstrap_url` não é viável sem `AppHandle`; testar manualmente). **Cuidados:** não quebrar `check:acl`; em *dev* a URL é o Vite.

---

### FLY-P3-011 — Endurecimento da CSP (`unsafe-inline`, `asset:`, `form-action https:`, `connect-src` amplo)

| Campo | Valor |
|---|---|
| Parte | 3 |
| Severidade | Baixa |
| Categoria | Segurança |
| Confiança | Hipótese (a remoção de `'unsafe-inline'` precisa ser validada em build real) |
| Esforço | P |
| Risco de regressão | Médio para `style-src`; baixo para o resto |
| Local | `src-tauri/tauri.conf.json:16` |
| Depende de | FLY-P3-001 (editar a mesma string depois) |

**Evidência e análise.** As páginas locais **não** usam `style=` inline nem `<script>` inline (verificado por busca em `index.html`/`settings.html`) e **não** usam `innerHTML`; os estilos aplicados por JS usam `element.style`/`dataset` (CSSOM, não bloqueado por `style-src`). Imagens: nenhuma `<img>` nas páginas; o protocolo `asset:` não está habilitado. O `<form>` é tratado por JS com `preventDefault`.
- `style-src … 'unsafe-inline'`: provavelmente desnecessário (**Hipótese**; o Tauri pode injetar estilos/nonces próprios — validar).
- `img-src … asset: http://asset.localhost`: sem uso.
- `form-action 'self' https:`: não há envio de formulário para fora.
- `connect-src … http: https:` (após FLY-P3-001): amplo, porém necessário para a sondagem de um site escolhido pelo usuário.

**Solução proposta.**
```diff
-"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: asset: http://asset.localhost; connect-src ipc: http://ipc.localhost http: https:; object-src 'none'; base-uri 'self'; form-action 'self' https:"
+"csp": "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src ipc: http://ipc.localhost http: https:; object-src 'none'; base-uri 'self'; form-action 'none'"
```
Aplicar em **duas etapas**: (1) `img-src` e `form-action` (baixo risco); (2) `style-src` sem `'unsafe-inline'`, validando em `tauri build --debug` (abrir Configurações e a tela inicial; o console do *devtools* não pode mostrar `Refused to apply inline style`). Se houver violação, restaurar `'unsafe-inline'` e registrar o motivo.
**Arquivos a alterar:** `src-tauri/tauri.conf.json`. **Critérios de aceite:** Configurações e tela inicial renderizam idênticas (claro/escuro); sem violações de CSP no console do build `--debug`. **Como verificar:** `tauri build --debug` e inspecionar. **Cuidados:** manter `script-src 'self'` sem `unsafe-*`; `check-ui-contract.mjs` rejeita `'unsafe-eval'`/`'unsafe-inline'` em `script-src`.

---

### FLY-P3-012 — Contraste de bordas de campo e do indicador de foco < 3:1 (tema claro); `check:contrast` não cobre

| Campo | Valor |
|---|---|
| Parte | 3 (4: `scripts/check-theme-contrast.mjs`) |
| Severidade | Baixa |
| Categoria | UX/A11y |
| Confiança | Confirmado (cálculo WCAG de luminância relativa) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `src/styles.css:14, 18-19, 35-36`; `scripts/check-theme-contrast.mjs:26-33` |
| Depende de | — |

**Evidência (razões calculadas)**

| Par | Razão | WCAG 2.2 AA (não-texto, 3:1) |
|---|---|---|
| Borda de campo (tema claro) `#e1e8f0` sobre `#fff` | **1,24:1** | ❌ |
| Borda de campo (tema escuro) `#354355` sobre `#121b27` | **1,72:1** | ❌ |
| Foco `#9ac5ff` sobre `#fff` (tema claro / página inicial) | **1,78:1** | ❌ |
| Foco `#9ac5ff` sobre `#1b2532` (escuro) | 8,70:1 | ✅ |
| Texto (todos os pares já testados) | ≥ 4,5:1 | ✅ (script passa) |

```css
/* styles.css:14 e 35 */
button:focus-visible,input:focus-visible,select:focus-visible{outline:3px solid #9ac5ff;outline-offset:2px}
.settings-section input:not([type=checkbox]):not([type=range]){...border:1px solid var(--settings-border);...}
```
**Causa raiz.** O mesmo token `--settings-border` (decorativo, de cartão) serve de limite de campo de formulário; o foco usa uma cor fixa clara. `check-theme-contrast.mjs` só valida texto a 4,5:1.

**Solução proposta.** Tokens novos (valores já calculados: ≥ 3:1 contra os fundos relevantes) e uso nos campos/foco:
```diff
-.settings-page{--settings-page-bg:#f4f7fb; ... --settings-message-error:#b4232d;
+.settings-page{--settings-page-bg:#f4f7fb; ... --settings-message-error:#b4232d;--settings-input-border:#76839a;--settings-focus:#1769d2;
-.settings-page[data-theme="dark"]{color-scheme:dark; ... --settings-message-error:#ff9299}
+.settings-page[data-theme="dark"]{color-scheme:dark; ... --settings-message-error:#ff9299;--settings-input-border:#7d90a8;--settings-focus:#8abaff}
-button:focus-visible,input:focus-visible,select:focus-visible{outline:3px solid #9ac5ff;outline-offset:2px}
+button:focus-visible,input:focus-visible,select:focus-visible{outline:3px solid var(--settings-focus,#1769d2);outline-offset:2px}
```
(aplicar `border:1px solid var(--settings-input-border)` nas regras de `input` e `select` das linhas 35-36). **Importante:** acrescentar as variáveis **dentro dos dois blocos existentes** (linhas 18 e 19), e não em um bloco novo — `check-theme-contrast.mjs` lê apenas o **primeiro** bloco de cada seletor. Razões obtidas: claro 3,83:1 (borda/branco), 5,27:1 (foco/card); escuro 5,30:1 (borda/campo), 7,77:1 (foco/card).
Estender `scripts/check-theme-contrast.mjs` (Parte 4) com um segundo conjunto a **3:1**:
```js
const nonText = [["settings-input-border","settings-input-bg"],["settings-focus","settings-card"],["settings-focus","settings-page-bg"]];
// loop idêntico ao dos textos, com limite 3 e a mesma mensagem de erro
```
**Arquivos a alterar:** `src/styles.css`, `scripts/check-theme-contrast.mjs`. **Critérios de aceite:** `npm run check:contrast` passa com os novos pares; aparência: campos com contorno nítido nos dois temas, foco visível no tema claro. **Como verificar:** `npm run check:contrast` e inspeção visual. **Cuidados:** o regex do script só lê cores `#rgb/#rrggbb` e blocos de seletor exatos (`.settings-page`, `.settings-page[data-theme="dark"]`); **não** reformatar esses seletores. A regra de foco global também vale para a página inicial (`.bootstrap`), por isso o *fallback* `#1769d2` (5,27:1 sobre branco).

---

### FLY-P3-013 — Pequenos defeitos agrupados (frontend/UI)

| Campo | Valor |
|---|---|
| Parte | 3 |
| Severidade | Baixa |
| Categoria | Manutenibilidade · i18n · UX · Documentação |
| Confiança | Confirmado (cada item abaixo verificado por leitura/script) |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | ver cada item |
| Depende de | FLY-P3-008, FLY-P3-005 (mesmos trechos de `settings.ts`) |

1. **Chaves i18n mortas** — `autostartWarning` e `shortcutDragHint` (3 idiomas, `settings.ts:52-53/82-83/112-113`) nunca são usadas (script de varredura). Remover (ou usar `shortcutDragHint` como `placeholder`/`aria-describedby` do campo de atalho).
2. **Códigos de erro mapeados e nunca emitidos** — `config.site_url_required` e `config.read_failed` (`settings.ts:151,156`) não existem no Rust (0 ocorrências em `src-tauri/src`). Remover do `backendErrorKeys` (manter as chaves de tradução `siteUrlRequired`/`configReadFailed` se usadas pelo TS: ver FLY-P3-008 e `settings.ts:276`).
3. **Aviso desconhecido vira texto de atalho** — `settings.ts:267`: `t(warningKeys[code] ?? "shortcutWarning")` mostra "conflita com outro aplicativo" para qualquer código novo. Usar um texto genérico (`autostartWarning` renomeado para `genericWarning`) como *fallback*.
4. **Texto estático desatualizado** — `settings.html:49` (`lockedHelp` em pt-BR) fala em "borda azul ao redor do widget", enquanto a tradução em `settings.ts:36` (e `check:drag-overlay`) descreve "aba superior; bordas transparentes". O texto estático aparece se `load()` falhar. Sincronizar com a tradução.
5. **Idioma estático e *flash*** — `index.html` e `settings.html` nascem em pt-BR e só trocam após `get_config` resolver (`main.ts:137-151`); usuários en/es veem texto em pt-BR por instantes. Mitigação simples: esconder o painel (`visibility:hidden`) até `applyLanguage`.
6. **Tema escuro parcial** — `color-scheme: dark` só é aplicado em `.settings-page[data-theme="dark"]` (`styles.css:19`) enquanto `:root` mantém `color-scheme: light` e `background:#f3f6fa` (`styles.css:1`): barra de rolagem do *viewport* e área fora do `body` podem continuar claras (**Hipótese visual**, não testada). Sem `prefers-color-scheme` para o tema inicial (`settings.ts:347-351` sempre começa claro).
7. **Tipagem/robustez TS** — `type Cfg = {…; [key: string]: unknown}` (`main.ts:4`) esconde erros de campo; `f<T>(id)` faz `as T` sem checar `null` (`settings.ts:144-146`) e há `!` em `querySelector`/`getElementById`; `tsconfig` sem `noUncheckedIndexedAccess`/`noImplicitOverride`. O script de FLY-P3-014 cobre a existência dos IDs.
8. **Constantes duplicadas sem verificação** — `350/550` (`settings.ts:128-129` × `config.rs:18-19`) e `"CTRL+ALT+C"` (`settings.ts:343` × `config.rs:66`); coberto por FLY-P3-014.
9. **`close_settings` sem tratamento** — `settings.ts:336-338`: `void invoke("close_settings")` ignora rejeição (promessa não tratada). Usar `.catch(() => {})`.
10. **Campo de URL** — `settings.html:38`: faltam `inputmode="url"`, `autocomplete="off"`, `spellcheck="false"`. O `placeholder="https://example.com"` não é traduzido (aceitável).
11. **A11y menor** — `<output id="zoom-value">` dentro do `<label>` do `range` torna o nome acessível verboso; mensagens de erro usam `role="status"`/`aria-live="polite"` (considerar `role="alert"` para `.error`); após falha, `retry` não recebe foco em `main.ts`; `lang="pt-BR"` estático.
12. **Capabilities** — `$schema` aponta para `../gen/schemas/desktop-schema.json` (pasta ignorada pelo Git; só afeta o editor em *clone* limpo); `main-remote-heartbeat.json:4` está em espanhol, enquanto o repositório usa pt-BR (regra 7 do prompt). Traduzir a `description`.
13. **`emit` global** (Parte 1, registrado aqui por ser o canal que `settings.ts` escuta) — `app.emit("config-changed", …)` (`commands.rs:321,403,422`) envia a todos os *webviews*. O remoto não tem permissão de `listen`, então não recebe nada; ainda assim, `emit_to("settings", …)` seria mais estrito. Decisão da Parte 1.

**Solução proposta.** Aplicar item a item, sem refatorar além do descrito. **Arquivos:** `src/settings.ts`, `src/main.ts`, `settings.html`, `index.html`, `src/styles.css`, `src-tauri/capabilities/main-remote-heartbeat.json`. **Critérios de aceite:** `npm run typecheck` e `node scripts/check-ui-contract.mjs` passam; nenhum texto novo fora dos 3 idiomas. **Como verificar:** typecheck + execução do script + inspeção manual. **Cuidados:** não renomear IDs (`check:drag-overlay`/`check:acl` leem trechos de `settings.html`/`settings.ts`; o `check-acl.mjs:33` proíbe a palavra `opacity` em `settings.ts`/`settings.html`).

---

### FLY-P3-014 — Ausência de verificações automáticas do contrato UI↔Rust/CSP

| Campo | Valor |
|---|---|
| Parte | 3 (4) |
| Severidade | Média |
| Categoria | Testes |
| Confiança | Confirmado (não há testes de frontend nem script de contrato; `package.json:6-19` só tem `typecheck`/`check:*` de outros temas) |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | `package.json:6-19`; `scripts/` (novo arquivo); `.github/workflows/build-windows.yml` (Parte 4: acrescentar o passo) |
| Depende de | FLY-P3-001, FLY-P3-013 (o script falha enquanto esses defeitos existirem — comportamento desejado) |

**Descrição.** Os defeitos FLY-P3-001, -005, -008 e -013 são todos "contratos" entre HTML, TS, Rust e `tauri.conf.json` que nenhuma verificação cobre. Não há ESLint/Prettier nem testes de frontend; os `check-*.mjs` existentes validam outros pontos (H11 — Parte 4 trata da fragilidade por regex).

**Solução proposta.** Adicionar `scripts/check-ui-contract.mjs` (abaixo, **testado**: no repositório atual falha com exatamente 3 mensagens — `config.site_url_required`, `config.read_failed` e CSP sem `http:` — e passa em cópia com essas 3 correções) e o npm script `"check:ui": "node scripts/check-ui-contract.mjs"`; executá-lo no CI (Parte 4) junto dos demais `check:*`.

```js
// scripts/check-ui-contract.mjs — contrato entre HTML, TypeScript, Rust e tauri.conf.json (sem dependências).
import fs from "node:fs";
const read = (p) => fs.readFileSync(new URL("../" + p, import.meta.url), "utf8");
const settingsHtml = read("settings.html");
const indexHtml = read("index.html");
const settingsTs = read("src/settings.ts");
const mainTs = read("src/main.ts");
const configRs = read("src-tauri/src/config.rs");
const commandsRs = read("src-tauri/src/commands.rs");
const tauriConf = JSON.parse(read("src-tauri/tauri.conf.json"));
const errors = [];
const fail = (m) => errors.push(m);

// 1) Todo id usado no TypeScript existe no HTML.
const htmlIds = new Set([...settingsHtml.matchAll(/\bid="([^"]+)"/g)].map((m) => m[1]));
const usedIds = new Set([...settingsTs.matchAll(/getElementById\("([^"]+)"\)/g)].map((m) => m[1]));
for (const m of settingsTs.matchAll(/\bf(?:<[^>]+>)?\("([^"]+)"\)/g)) usedIds.add(m[1]);
for (const m of (settingsTs.match(/const ids = \[([\s\S]*?)\];/)?.[1] ?? "").matchAll(/"([a-z_]+)"/g)) usedIds.add(m[1]);
for (const id of usedIds) if (!htmlIds.has(id)) fail(`settings.ts usa #${id}, que não existe em settings.html`);
for (const id of ["status", "retry"]) if (!new RegExp(`id="${id}"`).test(indexHtml)) fail(`main.ts usa #${id}, que não existe em index.html`);

// 2) i18n: pt-BR, en e es com o mesmo conjunto de chaves; todo data-i18n existe.
const block = settingsTs.slice(settingsTs.indexOf("const translations"), settingsTs.indexOf("const form ="));
const slice = (re) => [...(block.match(re)?.[1] ?? "").matchAll(/(?:^|[\s,{])([A-Za-z]+):\s*"(?:[^"\\]|\\.)*"/g)].map((m) => m[1]);
const langs = {
  "pt-BR": slice(/"pt-BR": \{([\s\S]*?)\n  \},\n  en:/),
  en: slice(/\n  en: \{([\s\S]*?)\n  \},\n  es:/),
  es: slice(/\n  es: \{([\s\S]*?)\n  \}\n\};/)
};
const base = new Set(langs["pt-BR"]);
if (base.size < 30) fail("Não foi possível extrair as chaves de tradução de settings.ts (formato mudou?)");
for (const l of ["en", "es"]) {
  const s = new Set(langs[l]);
  for (const k of base) if (!s.has(k)) fail(`Tradução ausente em ${l}: ${k}`);
  for (const k of s) if (!base.has(k)) fail(`Chave extra em ${l} (ausente em pt-BR): ${k}`);
}
for (const m of settingsHtml.matchAll(/data-i18n(?:-aria-label)?="([^"]+)"/g)) if (!base.has(m[1])) fail(`data-i18n sem tradução: ${m[1]}`);

// 3) Códigos de erro tratados no TS existem no Rust (e vice-versa).
const tsCodes = [...(settingsTs.match(/const backendErrorKeys[\s\S]*?\};/)?.[0] ?? "").matchAll(/"(config\.[a-z_]+)"/g)].map((m) => m[1]);
const rust = configRs + commandsRs;
for (const c of tsCodes) if (!rust.includes(`"${c}"`)) fail(`Código de erro mapeado no TS e nunca emitido pelo Rust: ${c}`);
for (const m of rust.matchAll(/"(config\.[a-z_]+)"(?:\.to_owned\(\)|\.into\(\))/g)) if (!tsCodes.includes(m[1])) fail(`Código de erro do Rust sem tradução no TS: ${m[1]}`);

// 4) Limites e padrões: HTML == TS == Rust.
const rustLimits = {
  width: configRs.match(/self\.width\.clamp\((\d+), (\d+)\)/)?.slice(1).map(Number),
  height: configRs.match(/self\.height\.clamp\((\d+), (\d+)\)/)?.slice(1).map(Number),
  zoom: configRs.match(/self\.zoom\.clamp\((\d+), (\d+)\)/)?.slice(1).map(Number),
  auto_reload_minutes: [0, Number(configRs.match(/auto_reload_minutes\.min\((\d+)\)/)?.[1])],
  corner_radius: [0, Number(configRs.match(/corner_radius\.min\((\d+)\)/)?.[1])]
};
for (const [id, [min, max]] of Object.entries(rustLimits)) {
  const tag = settingsHtml.match(new RegExp(`<input[^>]*id="${id}"[^>]*>`))?.[0] ?? "";
  const hMin = Number(tag.match(/min="(\d+)"/)?.[1]), hMax = Number(tag.match(/max="(\d+)"/)?.[1]);
  if (hMin !== min || hMax !== max) fail(`Limites divergentes em ${id}: HTML ${hMin}-${hMax} × Rust ${min}-${max}`);
  const ts = settingsTs.match(new RegExp(`readInt\\("${id}", (\\d+), (\\d+)\\)`));
  if (ts && (Number(ts[1]) !== min || Number(ts[2]) !== max)) fail(`Limites divergentes em ${id}: TS ${ts[1]}-${ts[2]} × Rust ${min}-${max}`);
}
const rs = (n) => Number(configRs.match(new RegExp(`pub const DEFAULT_${n}: u32 = (\\d+);`))?.[1]);
if (Number(settingsTs.match(/DEFAULT_WIDGET_WIDTH = (\d+)/)?.[1]) !== rs("WIDTH")) fail("DEFAULT_WIDGET_WIDTH diverge de DEFAULT_WIDTH (Rust)");
if (Number(settingsTs.match(/DEFAULT_WIDGET_HEIGHT = (\d+)/)?.[1]) !== rs("HEIGHT")) fail("DEFAULT_WIDGET_HEIGHT diverge de DEFAULT_HEIGHT (Rust)");
const defShortcut = configRs.match(/global_shortcut: "([^"]+)"\.into\(\)/)?.[1];
if (!settingsTs.includes(`value = "${defShortcut}"`)) fail(`Atalho padrão do Rust (${defShortcut}) não encontrado em settings.ts`);

// 5) CSP: se o Rust aceita http, connect-src precisa permitir http: (a sondagem de main.ts usa fetch).
const csp = tauriConf.app.security.csp;
const connect = /(?:^|;)\s*connect-src ([^;]*)/.exec(csp)?.[1] ?? "";
if (configRs.includes('["http", "https"]') && mainTs.includes("fetch(") && !/(^|\s)http:(\s|$)/.test(connect)) fail("CSP connect-src não permite http:, mas normalize() aceita sites http:// e main.ts os sonda com fetch");
if (/'unsafe-eval'|script-src[^;]*'unsafe-inline'/.test(csp)) fail("CSP não pode conter 'unsafe-eval' nem 'unsafe-inline' em script-src");

if (errors.length) { console.error("Falhas de contrato UI:\n - " + errors.join("\n - ")); process.exit(1); }
console.log("Contrato UI OK: ids, i18n (pt-BR/en/es), códigos de erro, limites, padrões e CSP coerentes.");
```
**Teste Rust complementar (Parte 1, recomendado):** em `config.rs`, um teste que percorra as teclas que `shortcutKeyFromCode` pode emitir (`A`–`Z`, `0`–`9`, `NUMPAD0`–`NUMPAD9`, `F1`–`F24`, `ESC`, `ENTER`, `SPACE`, `BACKSPACE`, `DELETE`, `INSERT`, `HOME`, `END`, `PAGEUP`, `PAGEDOWN`, `UP`, `DOWN`, `LEFT`, `RIGHT`) e confirme que `format!("CTRL+{key}")` passa em `normalize()` (nome sugerido: `shortcut_keys_emitted_by_settings_ui_are_accepted`). **Não verificado:** se `Shortcut::from_str` aceita todos esses nomes (ex.: `NUMPAD5`, `ESC`, `PAGEUP`); o teste responde à dúvida.
**Arquivos a alterar:** `scripts/check-ui-contract.mjs` (novo), `package.json` (script `check:ui`), `.github/workflows/build-windows.yml` (Parte 4). **Critérios de aceite:** `npm run check:ui` falha hoje (3 mensagens) e passa após FLY-P3-001/-013. **Como verificar:** `node scripts/check-ui-contract.mjs`. **Cuidados:** o script usa regex sobre o código-fonte (mesma fragilidade apontada em H11); ele **falha de forma explícita** (`base.size < 30`, regex sem match) em vez de passar em silêncio quando o formato muda.

---

### FLY-P3-015 — Versões do frontend: lockfile coerente; build não executado

| Campo | Valor |
|---|---|
| Parte | 3 (4) |
| Severidade | Info |
| Categoria | Dependência |
| Confiança | Confirmado (lockfile) / **não executado** (typecheck, build, audit) |
| Esforço | P |
| Risco de regressão | — |
| Local | `package.json:20-27`; `package-lock.json:312-324, 984-1022` |
| Depende de | — |

**Constatações.** `package-lock.json` resolve `typescript 7.0.2` (com pacote nativo `@typescript/typescript-linux-x64`), `vite 8.3.5` (com `rolldown 1.2.15`), `@tauri-apps/api 2.12.4` e `@tauri-apps/cli 2.12.4`; o crate `tauri` está em 2.12.5 (`check:tauri-versions` ✅). As faixas em `package.json` (`@tauri-apps/api ^2.8.0` × `@tauri-apps/cli ^2.9.5`) divergem em *major.minor* mínimo, mas o *lock* alinha ambos — H9 (Dependabot) é da Parte 4. **Não executado:** `typecheck` com TypeScript 7 e `vite build` (sem rede). **Não verificado:** se `build.rollupOptions` (`vite.config.ts:5`) permanece sem aviso de depreciação no Vite 8/Rolldown — a opção equivalente pode ter outro nome; se o `build` emitir aviso, migrar conforme a documentação oficial da versão instalada. `vite.config.ts` está coerente com `devUrl` (`127.0.0.1:1420`, `strictPort`). **Ação:** rodar `npm ci && npm run typecheck && npm run build` em ambiente com rede e registrar a saída. **Critério de aceite:** os três comandos passam sem avisos novos.

---

## 6. Achados transversais (envolvem mais de uma parte)

| ID | Partes | O que cruza | Ordem de correção sugerida |
|---|---|---|---|
| FLY-P3-001 | 3 + 4 (+1) | CSP (P3) × `normalize()` aceitando http (P1) × docs/Store prometendo HTTP (P4) | 1) corrigir CSP; 2) adicionar `check:ui` (P3-014); 3) P4 inclui o passo no CI. Nenhuma mudança de documentação é necessária (a doc já promete HTTP). |
| FLY-P3-002 / -003 | 3 + 4 (+1, +2) | capability remota (P3) × `check-acl.mjs` (P4) × watchdog/heartbeat (P1) × `INIT` (P2) | -003 (verificar e, se preciso, corrigir o padrão e o `check-acl`) → -002 (decisão A/B). Alterar JSON e `check-acl.mjs` **no mesmo commit**. |
| FLY-P3-004 | 3 + 2 | `withGlobalTauri` (P3) × `INIT` em `window.rs` (P2) | Coordenar com a correção da Parte 2 (mesmo arquivo; usar a numeração de linhas já atualizada). |
| FLY-P3-010 | 3 + 1 + 2 | `main.ts` (P3) × `commands.rs::save_config`/`reload_site` (P1) × auto-reload/`INIT` (P2) | Primeiro **testar a hipótese** em Windows; só então alterar P1/P2. |
| FLY-P3-005 / -006 / -007 | 3 + 1 | UI de atalho (P3) × `normalize()`/`Shortcut::from_str` (P1) | A regra do Rust é a referência; corrigir só o TS (e o teste Rust sugerido em P3-014 é da Parte 1). |

---

## 7. Plano de correção ordenado

**Convenção:** um achado por vez; dentro da onda, agrupado por arquivo; para cada arquivo, aplicar do **final para o início** ou **reler** o trecho antes de cada edição (números de linha mudam). *Paralelo* = pode ser feito por pessoas/IAs diferentes sem conflito de arquivo.

### Onda 1 — Alta, pré-requisitos e verificações que decidem outras correções
| Ordem | ID | Arquivo(s) | Esforço | Como validar | Observação |
|---|---|---|---|---|---|
| 1 | FLY-P3-001 | `src-tauri/tauri.conf.json` | P | `tauri build --debug`; site `http://127.0.0.1:8080/` abre ao reiniciar | Editar **apenas** `connect-src` agora |
| 2 | FLY-P3-003 *(verificar)* | `capabilities/main-remote-heartbeat.json`, `scripts/check-acl.mjs` | P | log "Primeiro sinal de atividade…" com site em porta explícita | **Teste em Windows primeiro**; só altera se falhar |
| 3 | FLY-P3-008 | `settings.html`, `src/settings.ts`, `src/styles.css` | P | `npm run typecheck`; manual nos 3 idiomas | — |
| 4 | FLY-P3-004 | `src-tauri/src/window.rs` (INIT) | M | `check:drag-overlay`, `cargo test`, log de heartbeat, página de teste | **Coordenar com Parte 2**; paralelo ao 3 (arquivos distintos) |

### Onda 2 — Média e Baixas funcionais
| Ordem | ID | Arquivo(s) | Esforço | Como validar | Observação |
|---|---|---|---|---|---|
| 5 | FLY-P3-006 → FLY-P3-005 → FLY-P3-007 | `src/settings.ts` (mesmas funções do `keydown`) | M+P+M | manual (US e AZERTY); typecheck | Nesta ordem, no mesmo arquivo; incluir 3 idiomas |
| 6 | FLY-P3-009 | `src/main.ts` | P | typecheck; 3 cenários manuais | Depende do 1 |
| 7 | FLY-P3-014 | `scripts/check-ui-contract.mjs`, `package.json` | M | `npm run check:ui` | **Só depois** de FLY-P3-013 itens 1-2 e do 1 (senão falha) |
| 8 | FLY-P3-010 *(verificar)* | `commands.rs`, `window.rs`, possivelmente `main.ts` | G | rede desligada: auto-reload/F5/trocar site | **Exige decisão humana**: ver abaixo |
| 9 | FLY-P3-002 | capability JSON (A) ou vários (B) | P / M | `check:acl` | **Exige decisão humana** (A recomendada) |

### Onda 3 — Baixas/documentação
| Ordem | ID | Arquivo(s) | Esforço | Como validar |
|---|---|---|---|---|
| 10 | FLY-P3-013 | `settings.ts`, `main.ts`, `settings.html`, `index.html`, `styles.css`, `main-remote-heartbeat.json` | M | typecheck + `check:ui` |
| 11 | FLY-P3-012 | `src/styles.css`, `scripts/check-theme-contrast.mjs` | P | `npm run check:contrast` |
| 12 | FLY-P3-011 | `src-tauri/tauri.conf.json` | P | `tauri build --debug`, console sem violações (2 etapas) |
| 13 | FLY-P3-015 | — | P | `npm ci && npm run typecheck && npm run build` (com rede) |

**Decisões humanas necessárias**
- **FLY-P3-002:** A (manter curinga + documentar + garantir que o watchdog não age só por heartbeat) **recomendada** vs B (capability dinâmica por host; maior custo e API não verificada).
- **FLY-P3-010:** confirmar o comportamento no WebView2; se confirmado, escolher entre "navegar ao bootstrap" (recomendado) e "condicionar o *reload* a uma sondagem".
- **FLY-P3-004:** aceitar o risco residual (a captura da referência é melhor-esforço) ou investir em outra via de heartbeat (não recomendado: depende de API interna).

---

## 8. Testes a adicionar

| Onde | Nome sugerido | O que cobre | Achado |
|---|---|---|---|
| Script Node | `scripts/check-ui-contract.mjs` (`npm run check:ui`) — **código completo em FLY-P3-014** | IDs HTML×TS, i18n pt-BR/en/es, códigos de erro TS×Rust, limites e padrões HTML×TS×Rust, CSP×http | 001, 005, 008, 013 |
| Rust inline (Parte 1) | `shortcut_keys_emitted_by_settings_ui_are_accepted` | todas as teclas que `shortcutKeyFromCode` emite passam em `normalize()` | 006, 014 |
| Rust inline (Parte 1) | `normalize_rejects_shift_only_shortcut` | `SHIFT+A` → `Err("config.shortcut_modifier_required")` | 005 |
| `check-theme-contrast.mjs` | pares não-texto a 3:1 (`settings-input-border`/`settings-input-bg`, `settings-focus`/`settings-card`/`settings-page-bg`) | contraste de borda e foco | 012 |
| `check-acl.mjs` | atualizar a asserção de `remote.urls` junto com o JSON | padrão de URL/porta | 002, 003 |
| Manual (Windows) | roteiro "HTTP em porta explícita" (`python -m http.server 8080`) | CSP, capability e heartbeat | 001, 003 |
| Manual (Windows) | roteiro "página de teste sobrescreve `window.__TAURI__`" | captura de token | 004 |
| Manual (Windows) | roteiro "rede desligada: auto-reload/F5/trocar site" | recuperação de falha | 010 |
| Frontend (opcional) | `settings.test.ts` com jsdom/vitest, se o time adotar | `FieldError`, `readPatch`, tradução de avisos | 008, 013 |

---

## 9. Itens descartados (hipóteses refutadas ou parcialmente refutadas)

| Item | Resultado | Evidência da refutação |
|---|---|---|
| **XSS no frontend local** | Descartado | Nenhum `innerHTML`/`insertAdjacentHTML`/`outerHTML` em `src/` (busca); mensagens usam `textContent` (`settings.ts:166`, `main.ts:70-80`); nenhum `<script>` inline nem `style=` inline nos HTML; `script-src 'self'`. |
| **`withGlobalTauri: true` amplia o que o site remoto pode executar (parte de H2)** | Refutado | A execução é limitada pela ACL; `remote` só tem `allow-heartbeat`; `__TAURI_INTERNALS__` existe independentemente. O efeito real é a dependência do `INIT` (FLY-P3-004). |
| **Site remoto invoca comandos além de `heartbeat`** | Refutado (por configuração) | `main-remote-heartbeat.json:14-16` só concede `allow-heartbeat`; `main-local` não tem `remote`; `check:acl` exige essa forma. Execução real da ACL **não verificada**. |
| **Janelas de login herdam IPC** | Refutado | Todas as capabilities usam `windows: ["main"]` ou `["settings"]`; `login-*` não consta (`main-remote-heartbeat.json:6-8`). |
| **`settings` pode ser navegada a conteúdo remoto com IPC** | Refutado | `settings-local.json` não tem `remote`; sem `remote.urls`, páginas remotas não herdam as permissões. |
| **`heartbeat` vaza dados / `get_config` vaza token** | Refutado | `AppConfig` (`config.rs:31-49`) não contém tokens; `heartbeat` só grava um carimbo (`diagnostics.rs:280-302`). |
| **`timer` de 30 s "nunca limpo" é vazamento** | Refutado como vazamento | Ao ter sucesso, `location.replace` descarrega a página e destrói o intervalo; só há custo enquanto a falha persiste. O que sobra está em FLY-P3-009 (sobreposição/presos). |
| **`/favicon.ico` com 403/404/redirect causa falso "sem conexão"** | Refutado em parte | Com `mode: "no-cors"`, qualquer resposta HTTP (inclusive 403/404) **resolve** o `fetch` (resposta opaca); só falham erro de rede, timeout (8 s) ou CSP. Resta o caso de redirecionamento para esquema bloqueado pela CSP (coberto por FLY-P3-001). |
| **`location.replace(target)` sem validar abre `javascript:`** | Refutado como vulnerabilidade | `target` vem normalizado (só http/https, `config.rs:90-96`) e `script-src 'self'` bloquearia `javascript:`; mantido como endurecimento em FLY-P3-009. |
| **`listen` sem `unlisten` vaza** | Refutado | A janela de configurações é destruída ao fechar (`close_settings` → `w.close()`); registrado como Info em FLY-P3-013. |
| **Chaves `data-i18n` sem tradução** | Refutado | Varredura: 0 chaves sem tradução; paridade pt-BR/en/es completa (56 chaves cada; o "extra" detectado em `en` era falso positivo da regex). |
| **Limites divergentes HTML×TS×Rust** | Refutado | Largura 150–2000, altura 100–1600, zoom 50–200, recarga 0–1440, raio 0–64 idênticos nos três; padrões 350×550 idênticos. |
| **`localStorage` falha no WebView** | Refutado | Uso protegido por `try/catch` (`settings.ts:140, 347-351`). |

---

## 10. Pontos não verificados / perguntas em aberto

**Dependem de execução em Windows/WebView2 (não executado):**
1. Efeito real da CSP sobre `fetch` a `http://` em *release* (FLY-P3-001 — alta confiança por semântica de CSP).
2. Se `http://*`/`https://*` casam URLs com porta explícita (FLY-P3-003).
3. Se `window.__TAURI__` é regravável e se `invoke` usa `__TAURI_INTERNALS__` em tempo de chamada; se o Navigation API expõe `widget-action://` (FLY-P3-004).
4. Comportamento do WebView2 em falha de navegação (página de erro; `INIT` ativo ou não) (FLY-P3-010).
5. Captura de teclas em AZERTY/QWERTZ/AltGr e o mapeamento do `global-hotkey` (FLY-P3-006/007).
6. Remoção de `'unsafe-inline'` de `style-src` (FLY-P3-011).
7. Aparência do tema escuro (barra de rolagem, `color-scheme`) (FLY-P3-013 item 6).

**Dependem de rede/ferramentas ausentes aqui:** `npm ci`, `typecheck` (TypeScript 7.0.2), `vite build` (Vite 8.3.5), `npm audit`, `cargo test/clippy/fmt/audit` (FLY-P3-015).

**Perguntas ao autor:**
- O uso de HTTP é um requisito de produto (README/Store dizem que sim)? Se não, alternativa é bloquear `http` em `normalize()` (mudaria Parte 1/4 e docs) — **não recomendado**.
- Aceita-se o risco residual de heartbeat forjável (FLY-P3-002 opção A)?
- O estado "sem site configurado" deve continuar permitido no backend (sim, o fluxo de primeiro uso depende dele)?

**Status das hipóteses H1–H14 do prompt nesta Parte 3**

| H | Status | Onde |
|---|---|---|
| H1 | **Confirmada** (estática) | FLY-P3-001 |
| H2 | **Parcialmente confirmada / parcialmente refutada** | FLY-P3-002, -003, -004; refutação em §9 |
| H3 | *Observação de passagem, não analisada (Parte 2):* `LOGIN_SEGMENTS` (`window.rs:~216-218`) inclui `account`, `accounts`, `auth`, `callback`; `decide_main` (`window.rs:313-319`) só isenta a URL **exata** configurada. Consistente com a hipótese; **pendente na Parte 2**. | — |
| H4 | *Observação de passagem, não analisada (Parte 2):* `SETTINGS_WINDOW_CREATING` é liberado após o resultado de `build()` mesmo em `Err` (`window.rs:~1252`); só ficaria preso por **panic** na *thread*. **Pendente na Parte 2.** | — |
| H5 | *Observação de passagem (Parte 2):* `UI_COPY` em `window.rs:26-28` contém "Sem conexão com o calendário"/"Calendar connection unavailable"/"Sin conexión con el calendario" — **confirmada visualmente**, correção pertence à Parte 2. | — |
| H6 | **Não analisada** (Partes 1/2); registrado o elo: heartbeat depende de `window.__TAURI__` e da capability (FLY-P3-003/-004). | — |
| H7, H8 | Não analisadas (Parte 1) | — |
| H9, H10, H11, H12, H14 | Não analisadas (Parte 4). H11 tangencialmente confirmada para os `check-*` lidos (regex sobre código-fonte); mitigação no script proposto (falha explícita) | FLY-P3-014 |
| H13 | **Parcial:** o *lock* resolve `typescript 7.0.2`, `vite 8.3.5`, `@tauri-apps/api/cli 2.12.4`; `windows 0.62.2`/`windows-future 0.3.2` **não verificados** (Parte 4) | FLY-P3-015 |

---

## 11. Instruções operacionais para a IA corretora

1. Aplicar **um achado por vez**, na ordem do plano (seção 7); após cada um, rodar os comandos da seção 7 do prompt original que forem pertinentes (`npm run typecheck`, `npm run check:*`, `cargo fmt/test/clippy` quando tocar Rust) e **só avançar se passarem**.
2. Mudar **somente** o que o achado pede; não refatorar por conta própria.
3. **Reler o trecho atual** antes de editar (os números de linha deste relatório podem ter mudado, principalmente em `settings.ts` e `window.rs`).
4. Adicionar o teste/script sugerido junto com a correção.
5. Manter comentários e logs em **pt-BR**; ao alterar texto de interface, atualizar **os 3 idiomas** (`pt-BR`, `en`, `es`) em `settings.ts`, `main.ts`, `i18n.rs` e no script injetado, de forma consistente.
6. Se uma correção exigir alterar `generate_handler!`, `build.rs` ou capabilities, atualizar os **três juntos** e conferir `npm run check:acl` (e `scripts/check-acl.mjs` se o padrão de URL mudar).
7. **Nunca** incluir segredos nem desativar verificações (`check:*`, `clippy -D warnings`, `cargo audit`) para "fazer passar". O `check-ui-contract.mjs` deve falhar enquanto FLY-P3-001/-013 não forem corrigidos.
8. Para os achados marcados **Hipótese** (003, 010, 011, 013-6), **executar primeiro a verificação** descrita; só alterar o código se a hipótese se confirmar, e registrar o resultado.
9. Não editar `window.rs`/`commands.rs` por conta própria nos achados 004/010 sem coordenar com o responsável pelas Partes 1 e 2.
10. Ao terminar cada achado, registrar: **ID, arquivos alterados, resultado dos comandos e eventual desvio** em relação ao proposto.
11. O repositório só compila para `x86_64-pc-windows-msvc`; em outro SO, não rodar `tauri build` — analisar estaticamente e declarar o que não foi executado.

---

## 12. Prompt de execução sugerido (pronto para colar)

```text
Você é um(a) engenheiro(a) sênior de Rust/Tauri 2/TypeScript. Anexo o repositório Floatly (floatly-website-widget-main) e o relatório RELATORIO_ANALISE_FLOATLY_PARTE_3.md.
Corrija os achados FLY-P3-001 a FLY-P3-015 SEGUINDO a seção 7 (plano ordenado) e a seção 11 (instruções operacionais) do relatório. Regras:
- Um achado por vez; reler o trecho atual antes de editar; adicionar o teste/script sugerido junto com a correção.
- Para achados "Hipótese" (003, 010, 011, 013 item 6), execute primeiro a verificação indicada e só altere código se for confirmado; os de "decisão humana" (002, 004, 010), pare e apresente as opções com a recomendação.
- Mantenha comentários/logs em pt-BR e textos de interface nos 3 idiomas (pt-BR, en, es).
- Se tocar em generate_handler!, build.rs ou capabilities, atualize os três juntos e rode npm run check:acl.
- Após cada achado rode: npm run typecheck, npm run check:acl, check:drag-overlay, check:contrast, check:msix-config, check:tauri-versions, e (quando tocar Rust) cargo fmt --check, cargo test --locked, cargo clippy --all-targets --locked -- -D warnings. Não desative nenhuma verificação.
- Não refatore além do pedido. Não inclua segredos.
Ao final de cada achado, informe: ID, arquivos alterados, resultado dos comandos e desvios. Ao terminar, liste o que não pôde ser executado e por quê.
```

---

## 13. Checklist final (prompt, seção 10) — Parte 3

- [x] Li integralmente todos os arquivos da **Parte 3** (e os trechos necessários das Partes 1, 2 e 4, listados na seção 0). *Não* li `window.rs` inteiro nem `build-msix.yml` — pertencem às Partes 2 e 4, **não auditadas** aqui.
- [x] Cada achado tem local, evidência, causa raiz, solução com passos/diff, critérios de aceite e verificação.
- [x] Cada achado traz confiança e esforço; hipóteses não foram tratadas como fato.
- [x] Tentei refutar cada achado; os refutados estão na seção 9.
- [x] H1–H14: confirmadas/refutadas/marcadas como não verificadas ou pendentes (seção 10).
- [x] Plano ordenado por dependência e agrupado por arquivo (seção 7).
- [x] Informei o que executei e o que não pude executar (seção 2).
- [x] Relatório em `.md`, pt-BR, **sem alterar nenhum arquivo do projeto** (o script `check-ui-contract.mjs` foi testado em cópia temporária e removido; o `node_modules` parcial gerado pelo `npm ci` falho foi apagado).
- [x] Incluí "Instruções operacionais para a IA corretora" e o "Prompt de execução sugerido".

**Para continuar:** peça "Execute a Parte 1", "Execute a Parte 2" ou "Execute a Parte 4" para gerar os relatórios correspondentes; os IDs `FLY-P3-nnn` permanecem estáveis para a consolidação.
