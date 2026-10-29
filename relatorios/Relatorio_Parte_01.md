# Relatório de Análise e Auditoria de Código — Widget Website (Tauri 2 / Windows)

> **Escopo desta entrega: PARTE 1 — Núcleo: Configuração, Estado e Regras de Negócio.**
> As Partes 2–5 **não foram analisadas** nesta entrega; suas seções existem apenas para manter o formato do prompt e estão marcadas como pendentes. Os achados `T-xxx` da seção 5 são **preliminares** (só a perspectiva da Parte 1).

## 0. Instruções para a IA/Dev que fará as correções

- Corrija UM achado por vez, na ordem da seção 3 (a ordem já respeita as dependências).
- Após cada alteração, rode: `cargo test --manifest-path src-tauri/Cargo.toml` e `npm run build`.
- Adicione/ajuste os testes indicados; não remova testes sem justificativa.
- Em caso de conflito, siga o campo "Depende de / Conflita com".
- Achados `HIPOTESE` exigem verificação (idealmente em Windows real) antes da correção. **Nenhum achado deste relatório é `CONFIRMADO`**: não havia `rustc`/`cargo` nem Windows no ambiente de análise (ver seção 2).
- Números de linha são do ZIP `widget-website-main.zip` como recebido. `commands.rs` tem 18 linhas (o prompt dizia 19) e quase tudo está em linhas únicas; **as referências citam a linha e a função**. Reformatar com `rustfmt` altera os números; faça isso por último ou em commit separado.
- `window.rs` e `commands.rs` são densos (várias instruções por linha). Se for editar, rode `cargo fmt` num commit isolado antes de corrigir, para facilitar revisão.

## 1. Resumo Executivo

- **Ambiente:** análise estática do ZIP (25 arquivos, ~1.800 linhas de fonte) + simulações em Python da lógica Rust + consulta à documentação (Tauri, `global-hotkey`). Sem `rustc`/`cargo`, sem Windows e sem rede para `npm install`.
- **Linha de base:** 13 testes `#[test]` por leitura estática (`config.rs` 6, `diagnostics.rs` 4, `window.rs` 3). **Nenhum foi executado.** Os 6 de `config.rs` (os da Parte 1) cobrem defaults, clamp, idioma, config antiga e config corrompida; nada cobre comandos, atalho, geometria ou persistência.
- **Parte 1: 15 achados — 0 CRÍTICA, 2 ALTA, 7 MÉDIA, 6 BAIXA.** Confiança: 12 `PROVAVEL`, 3 `HIPOTESE`, 0 `CONFIRMADO`.
- **Top 5 riscos:**
  1. **P1-001 (ALTA):** um único campo inválido em `config.json` faz `load` descartar **todas** as preferências, e o `setup` sobrescreve o arquivo logo em seguida (perda permanente, sem backup).
  2. **P1-005 (ALTA):** "Salvar" nas Configurações grava `x`, `y`, `width`, `height`, `locked` e `always_on_top` **obsoletos** do formulário por cima do estado atual (posição perdida; alternância da bandeja desfeita).
  3. **P1-004 (MÉDIA):** `normalize` aceita atalho global **sem modificador** (ex.: `TAB`, `A`), que sequestra a tecla no sistema inteiro.
  4. **P1-003 (MÉDIA):** fallback de atalho no `setup` deixa estado fantasma (atalho ativo ≠ atalho configurado; `registered_shortcut` nunca é lido; aviso de "conflito" enganoso).
  5. **P1-006 (MÉDIA):** `apply_config` reaplica sempre `set_size` com o tamanho da memória; com `remember_position=false` a janela "volta" ao tamanho antigo ao alternar travar/topo.
- **Pistas do prompt:** das 9 pistas da Parte 1, 5 foram confirmadas, 3 foram parcialmente refutadas/rebaixadas e 1 (concorrência de gravação) foi rebaixada a risco latente (seção 7).

## 2. Ambiente e Linha de Base

**Preparação.** ZIP extraído em `/home/claude/work/` (cópia de trabalho; manifesto `sha256` registrado antes e conferido ao final — nenhum arquivo do repositório foi alterado). Testes de comando foram feitos em cópia separada (`/home/claude/scratch/`).

| Comando / ação | Resultado exato |
|---|---|
| `unzip -l widget-website-main.zip` | 33 entradas = 25 arquivos + 8 diretórios (o prompt dizia "33 arquivos"). Sem `Cargo.lock`, `package-lock.json` ou `LICENSE` (confirmado). |
| `wc -l` nos fontes (`*.rs`, `*.ts`, `*.css`, 2 `.html`) | 1.819 linhas. |
| `grep -c "#\[test\]" src-tauri/src/*.rs` | `config.rs`=6, `diagnostics.rs`=4, `window.rs`=3, demais=0 → **13** (confere com o prompt, por leitura estática). |
| `cargo --version` / `rustc --version` | `cargo: not found` / `rustc: not found` → **`cargo test`, `cargo clippy`, `cargo audit` não puderam ser executados.** |
| `node --version` / `npm --version` | `v22.22.2` / `10.9.7`. |
| `npm install` (cópia em `/home/claude/scratch`) | **Timeout após 90 s (exit 124), sem saída** — sem acesso à rede (bash sem egress). Sem `node_modules`, sem lockfile gerado. |
| `npx --no-install tsc --noEmit` | Usou o `tsc` **global 6.0.3** (o projeto pede `~5.8.3`). Saída: `TS2307` ×3 (`@tauri-apps/api/core` em `src/main.ts(1,24)` e `src/settings.ts(1,24)`; `vite` em `vite.config.ts(1,30)`) e `TS2345` em `src/settings.ts(172,11)` (`'C \| null'` não atribuível a `'C'`). **Os 4 erros são artefato de dependências ausentes**: sem os tipos de `invoke<C>`, o TypeScript não estreita `current` após a atribuição (ver seção 7). Não são achados. |
| `npm run build`, `npm audit` | Não executados (dependem de `npm install`). |
| Rust: simulações | `sim_p1.py` (Python) reproduz a lógica de `restore_position` e do registro de atalhos. É **evidência auxiliar** (`PROVAVEL`), não substitui teste Rust. |
| Documentação consultada (só leitura, via busca) | Tauri: comandos síncronos rodam na thread principal. `global-hotkey` (docs.rs): `HotKey` = "combinação *opcional* de modificadores + uma tecla". `tauri-plugin-autostart`: API `enable/disable/is_enabled`; o comportamento de `disable()` sem entrada registrada **não foi encontrado** na documentação (→ `HIPOTESE` em P1-011). |

**Limitações:** nenhum achado pôde ser "reproduzido"; todos são análise estática (+ simulação). Todo comportamento de Win32/WebView2/DPI/minimização é `HIPOTESE` até teste em Windows.

## 3. Plano de Correção Ordenado

Critério: Severidade → dependências → esforço. P1-013 é pré-requisito de P1-007 (gravação em segundo plano).

| Ordem | ID | Severidade | Esforço | Arquivo(s) | Depende de | Resumo (1 linha) |
|---|---|---|---|---|---|---|
| 1 | P1-001 | ALTA | M | `config.rs:106-115`, `main.rs:137-141` | nenhum | `load` perde todas as preferências por 1 campo inválido e o `setup` sobrescreve o arquivo |
| 2 | P1-005 | ALTA | M | `commands.rs:9-11`, `settings.ts:181-191` | nenhum | `save_config` grava x/y/size/locked/topmost obsoletos do formulário |
| 3 | P1-004 | MÉDIA | P | `config.rs:83-85` | nenhum | `normalize` aceita atalho sem modificador/não parseável |
| 4 | P1-003 | MÉDIA | M | `main.rs:148-172`, `commands.rs:9` | P1-004 | Fallback de atalho gera estado fantasma; `registered_shortcut` nunca lido |
| 5 | P1-002 | MÉDIA | M | `commands.rs:9-11` | P1-003 | `save_config` aplica efeitos no SO antes de persistir, sem rollback |
| 6 | P1-006 | MÉDIA | M | `window.rs:344,347` | nenhum | `apply_config` reaplica sempre `set_size` → "snap-back" com `remember_position=false` |
| 7 | P1-009 | MÉDIA | M | `window.rs:307` | nenhum | `restore_position`: 1 px de sobreposição = "visível"; escala errada; overflow `i32` |
| 8 | P1-008 | MÉDIA | P | `window.rs:347` | nenhum | `Moved`/`Resized` com valores sentinela (minimizar) são persistidos (`HIPOTESE`) |
| 9 | P1-013 | BAIXA | M | `config.rs:117-131`, `window.rs:347` | nenhum | Gravação sem serialização, temp de nome fixo e fallback `copy` não atômico (latente) |
| 10 | P1-007 | MÉDIA | M | `window.rs:347`, `main.rs:196-198` | P1-013, P1-006, P1-008 | `config.json` regravado a cada `Moved`/`Resized`, na thread da UI |
| 11 | P1-011 | BAIXA | P | `commands.rs:8,10`, `main.rs:138-141` | nenhum | Autostart chamado em todo "Salvar" mesmo sem mudança (`HIPOTESE`) |
| 12 | P1-012 | BAIXA | P | `commands.rs:12` | P1-006 | `restore_default_size`: 500×700 duplicado, passa por todo `save_config` |
| 13 | P1-010 | BAIXA | P | `commands.rs:8`, `window.rs:184-348` | nenhum | Mutex envenenado devolve `AppConfig::default()` silenciosamente |
| 14 | P1-015 | BAIXA | P | `commands.rs:9,11,17,18`, `main.rs:141,170`, `window.rs:347` | P1-013 | Falhas de persistência/registro descartadas com `let _ =` sem log |
| 15 | P1-014 | BAIXA | P | `config.rs:62-64` | nenhum | Heurística `contains("://")` rejeita URL válida com `://` na query |

## 4. Achados por Parte

### Parte 1 — Núcleo: Configuração, Estado e Regras de Negócio

**Escopo lido integralmente:** `config.rs` 1–98 e 133–222; `commands.rs` 1–18; `main.rs` 131–222 (cobre 131–170); `window.rs:307` (`restore_position`), `:344` (`apply_config`, necessário para P1-006), `:347` (`save_window_state`); `settings.ts` 105–264 (só para o contrato front↔Rust); `settings.html:63`; `tray.rs:88-100` (origem das chamadas `toggle_*`).

---

#### P1-001 — `load` descarta TODAS as preferências por um campo inválido e o `setup` sobrescreve o arquivo
- **Severidade:** ALTA
- **Categoria:** integridade-de-dados
- **Confiança:** PROVAVEL
- **Local:** `config.rs:106-115` (`load`); `main.rs:137-141` (`setup`)
- **Evidência:**
  ```rust
  // config.rs:106-111
  pub fn load(dir: &PathBuf) -> AppConfig {
      let mut config = fs::read(dir.join("config.json"))
          .ok()
          .and_then(|bytes| serde_json::from_slice::<AppConfig>(&bytes).ok())
          .unwrap_or_default();
      if config.normalize().is_err() { config = AppConfig::default(); }
  ```
  ```rust
  // main.rs:137,141 (setup)
  let mut cfg = config::load(&dir);
  ...
  let _ = config::save(&dir, &cfg);
  ```
  `AppConfig` (config.rs:11-31) só tem `#[serde(default)]` em `language`. Qualquer outro campo ausente, ou fora do tipo/limite do inteiro (`opacity: u8` com 300; `width: u32` com -5; `zoom: u16`; `x: i32` com 1.5), faz `from_slice` falhar **inteiro**. `normalize()` com `site_url` vazio/inválido também reseta tudo (linha 111-113). Em seguida o `setup` grava os padrões em `config.json`, destruindo o original.
- **Reprodução:** (não reproduzível neste ambiente: sem `rustc`) criar `%APPDATA%\WidgetWebsite\config.json` com os valores padrão mas `"opacity": 300` e `"width": 800`; iniciar o app. Esperado: `width=800` preservado. Previsto: `width=500`, todos os campos padrão, `config.json` reescrito. Teste mínimo na seção "Teste a adicionar".
- **Causa raiz:** desserialização "tudo ou nada" + `unwrap_or_default` + gravação incondicional no startup, sem backup nem log do motivo (`load` nem tem acesso ao diagnóstico).
- **Impacto:** quem editou `config.json` à mão, quem atualiza para uma versão que acrescenta um campo sem `#[serde(default)]`, ou qualquer corrupção parcial perde posição, tamanho, URL, atalho e idioma, sem aviso e sem como recuperar.
- **Correção proposta:**
  1. Ler o JSON como `serde_json::Value`; se não for objeto → renomear o arquivo para `config.json.bad-<epoch>` (backup) e usar padrões.
  2. Aplicar **campo a campo**: partir de `AppConfig::default()` serializado em `Value`; para cada chave do arquivo, substituir só se `serde_json::from_value::<AppConfig>(valor_mesclado)` continuar válido; senão manter o padrão daquele campo e registrar `WARN` com o nome do campo.
  3. Colocar `#[serde(default)]` no nível do struct (com `impl Default` existente) para campos novos nunca quebrarem configs antigas.
  4. Se `normalize()` falhar, resetar **só** o campo culpado (`site_url`/`global_shortcut`), não tudo.
  5. `load` deve devolver também a lista de avisos; o `setup` os envia ao `diagnostics::log`. No `setup`, só chamar `config::save` quando algo mudou (autostart sincronizado ou avisos), e nunca antes do backup.
- **Critério de aceite:** com `opacity: 300`, `width: 800` e `site_url` válida, `load` retorna `width == 800`, `opacity == 100` e um aviso citando `opacity`; um `config.json` com JSON inválido gera `config.json.bad-*` com o conteúdo original; `cargo test` verde.
- **Teste a adicionar:** em `config.rs`: `load_keeps_valid_fields_when_one_field_is_out_of_range`, `load_backs_up_unparsable_config_before_overwriting`, `load_resets_only_invalid_site_url`, `load_tolerates_missing_fields_via_struct_default`.
- **Depende de / Conflita com:** nenhum. (Agrava-se com P1-013: fallback `copy` não atômico pode truncar o arquivo.)
- **Esforço:** M

---

#### P1-002 — `save_config` aplica efeitos no SO antes de persistir e não faz rollback
- **Severidade:** MÉDIA
- **Categoria:** integridade-de-dados
- **Confiança:** PROVAVEL
- **Local:** `commands.rs:9` (`save_config`: registro/desregistro de atalho), `:10` (autostart), `:11` (`config::save(...)?` e atualização da memória)
- **Evidência:** a ordem em `save_config` é: `normalize` → ler `old` → `register(novo)` e `unregister(antigo)` (linha 9) → `enable()/disable()` do autostart (linha 10) → `config::save(&state.app_dir,&c)?` (linha 11) → só então `*stored = c.clone()` (linha 11). Se `config::save` retornar `Err` (disco cheio, arquivo bloqueado por antivírus), a função sai pelo `?` com o atalho **já trocado** e o autostart **já alterado**, enquanto disco e memória ficam com os valores antigos. Do mesmo modo, se `state.config.lock()` falhar depois de gravar (linha 11), o disco já foi alterado e a memória não.
- **Reprodução:** não reproduzível neste ambiente (sem Rust/Windows). Em Windows: tornar `config.json` somente leitura **e** o diretório sem permissão de criação; salvar um novo atalho nas Configurações. Previsto: erro exibido, mas o novo atalho funciona e o antigo não; no próximo início volta o antigo.
- **Causa raiz:** efeitos externos (RegisterHotKey, registro do Windows) aplicados antes do ponto de confirmação (persistência) e sem compensação.
- **Impacto:** estado divergente SO × memória × disco após falha de I/O; o usuário vê "erro ao salvar" mas o app já mudou de comportamento.
- **Correção proposta:** reordenar: (1) validar; (2) calcular a nova config; (3) **persistir** (`config::save`); (4) atualizar a memória; (5) só então aplicar efeitos (atalho/autostart); se um efeito falhar, reverter **o campo** em memória+disco e devolver `warning`. Alternativa: manter a ordem atual mas, em caso de `Err` na persistência, desfazer (`unregister(novo)` + `register(antigo)`, restaurar autostart). Extrair a lógica para uma função testável `fn commit_settings(...)`.
- **Critério de aceite:** teste que injeta falha de gravação (ex.: `dir` inexistente/somente leitura) e verifica que o atalho ativo e `registered_shortcut` permanecem os antigos e que a memória não mudou.
- **Teste a adicionar:** `save_config_rolls_back_shortcut_when_persist_fails`, `save_config_keeps_memory_unchanged_when_persist_fails` (exigem extrair a lógica pura/injetável de `commands.rs`).
- **Depende de / Conflita com:** depende de P1-003 (modelo de estado do atalho). Interage com P1-013.
- **Esforço:** M

---

#### P1-003 — Fallback de atalho no `setup` deixa estado fantasma; `registered_shortcut` é código morto
- **Severidade:** MÉDIA
- **Categoria:** correcao-logica
- **Confiança:** PROVAVEL
- **Local:** `main.rs:148-172` (`setup`), `main.rs:154-162` (escrita de `registered_shortcut`), `commands.rs:9` (`save_config`), `config.rs:96` (campo)
- **Evidência:**
  ```rust
  // main.rs:163-171
  if !registered {
      diagnostics::log(&dir, "WARN", "O atalho configurado não pôde ser registrado; tentando CTRL+ALT+C.");
      if let Ok(shortcut) = Shortcut::from_str("CTRL+ALT+C") {
          let _ = app.global_shortcut().register(shortcut);
      }
  }
  ```
  `grep -n registered_shortcut`: `config.rs:96` (declaração), `main.rs:145` (init), `main.rs:154` (escrita), `commands.rs:9` (escrita). **Nenhuma leitura em todo o projeto.**
  Simulação do modelo (`RegisterHotKey`: registrar duas vezes o mesmo atalho falha; desregistrar o que não existe falha) — `sim_p1.py`:
  ```
  B1 após setup: registro real = {'CTRL+ALT+C'} | cfg.global_shortcut = CTRL+SHIFT+X | registered_shortcut = ''
  B2 após salvar Y: registro real = {'CTRL+ALT+Y', 'CTRL+ALT+C'} -> CTRL+ALT+C continua ativo (fantasma)
  B3 salvar CTRL+ALT+C com fallback ativo -> register ok? False => aviso 'conflita' e cfg continua CTRL+SHIFT+X
  ```
- **Reprodução:** não reproduzível aqui (sem Windows/Rust). Em Windows: ocupar `CTRL+SHIFT+X` em outro app, salvar esse atalho no widget e reiniciar. Verificar com `diagnostics.log` o WARN; em seguida abrir Configurações (mostra `CTRL+SHIFT+X`) e pressionar `CTRL+ALT+C` (o widget responde).
- **Causa raiz:** (a) três fontes de verdade (`cfg.global_shortcut`, `registered_shortcut`, estado real do SO) sem leitura da que seria correta; (b) `save_config` compara **strings** (`c.global_shortcut != old.global_shortcut`) em vez de atalhos parseados e desregistra o `old` configurado, não o realmente ativo; (c) `register` e `unregister` têm o resultado reduzido a `is_ok()`/`let _`, perdendo a causa.
- **Impacto:** (1) a UI mostra um atalho que não está ativo e um atalho que ninguém configurou está ativo; (2) ao trocar o atalho, o fallback `CTRL+ALT+C` **nunca é desregistrado** até reiniciar; o `unregister(old)` é feito num atalho que nunca foi registrado; (3) tentar salvar `CTRL+ALT+C` (o que de fato funcionava) falha com "o atalho novo conflita com outro aplicativo" — mensagem enganosa (o conflito é com o próprio app) e o valor inválido anterior é mantido; (4) o mesmo atalho com grafia diferente (ex.: `ALT+CTRL+C` × `CTRL+ALT+C`) passa pela comparação de strings, `register` falha por duplicidade e o usuário recebe a mesma mensagem enganosa (`PROVAVEL`; a duplicidade falhar depende do `RegisterHotKey` retornar `ERROR_HOTKEY_ALREADY_REGISTERED` para o mesmo id).
- **Correção proposta:**
  1. Fazer de `registered_shortcut` a **fonte de verdade do atalho ativo** (guardar `Option<Shortcut>` parseado). No `setup`, após o fallback, gravar nele o atalho realmente ativo (ou `None`).
  2. Extrair `fn change_shortcut(app, state, novo: &str) -> Result<(), ShortcutError>`: parsear `novo`; se `parsed(novo) == ativo` → no-op com sucesso; senão `register(novo)`; se ok → `unregister(ativo)` (ignorar erro quando `ativo` é `None`) → atualizar `registered_shortcut`. Distinguir `ShortcutError::Parse(msg)` de `ShortcutError::Register(msg)` e usar mensagens diferentes ("Atalho inválido: …" × "Atalho em uso por outro aplicativo").
  3. No `setup`, registrar no `diagnostics` a causa real (`Err` de `parse`/`register`) em vez de descartá-la, e usar a mesma função.
  4. Decidir e documentar a política: ao cair no fallback, manter `cfg.global_shortcut` com a preferência do usuário **e** expor o atalho ativo (ex.: novo campo `active_shortcut` em `get_config`/`SaveResult`; coordenar com a Parte 4 para a UI). Se preferir simplicidade, gravar o fallback em `cfg.global_shortcut`.
- **Critério de aceite:** (a) após falha no `setup`, `registered_shortcut` == `CTRL+ALT+C`; (b) trocar para outro atalho válido desregistra o fallback; (c) salvar o atalho que já está ativo com outra grafia retorna sucesso sem aviso; (d) a mensagem para atalho não parseável é distinta da de conflito.
- **Teste a adicionar:** `change_shortcut_is_noop_for_same_hotkey_different_spelling`, `change_shortcut_unregisters_active_fallback`, `setup_fallback_sets_registered_state`, `change_shortcut_distinguishes_parse_error_from_conflict` (usar um `trait ShortcutBackend` falso).
- **Depende de / Conflita com:** depende de P1-004 (parse/validação). P1-002 depende deste.
- **Esforço:** M

---

#### P1-004 — `normalize` não valida o atalho global (aceita sem modificador e não parseável)
- **Severidade:** MÉDIA
- **Categoria:** validacao-entrada
- **Confiança:** PROVAVEL
- **Local:** `config.rs:83-85` (`normalize`)
- **Evidência:**
  ```rust
  if self.global_shortcut.trim().is_empty() {
      return Err("Informe um atalho global.".into());
  }
  ```
  É a única verificação (e o valor **não é aparado**: `" CTRL+ALT+C "` é guardado com espaços). Pela documentação de `global-hotkey` (docs.rs), um `HotKey` é "uma combinação *opcional* de modificadores mais uma tecla": `TAB`, `A` ou `ENTER` são atalhos válidos e `Shortcut::from_str` os aceita. O campo de Configurações (`settings.html:63`, `readonly`, com `keydown` em `settings.ts:227-241`) gera exatamente essas strings ao pressionar uma tecla sem modificador (ver Parte 4, pista 3).
- **Reprodução:** não reproduzível aqui. Em Windows: focar o campo "Combinação de teclas", pressionar `Tab`, Salvar; depois digitar `Tab` em qualquer aplicativo (previsto: o widget alterna e o app destino não recebe a tecla).
- **Causa raiz:** validação de domínio feita só em `save_config` (`Shortcut::from_str`, linha 9) e no `setup`, **depois** do `normalize`; falta regra de negócio "exige modificador" e `load` aceita qualquer string.
- **Impacto:** um atalho de uma tecla só, registrado globalmente, intercepta essa tecla em todo o sistema enquanto o widget estiver aberto. A reversão exige usar as Configurações (por mouse) ou editar `config.json`. A string não parseável só é detectada em `save_config`/`setup`, nunca em `load`.
- **Correção proposta:** em `normalize`: `trim()`; `Shortcut::from_str`; exigir ao menos um entre `CTRL`, `ALT`, `SUPER` (rejeitar apenas `SHIFT+tecla` e tecla sozinha; opcionalmente permitir F13–F24 sozinhas); devolver a string canônica em maiúsculas. Verificar no crate o nome do campo público de modificadores (`Shortcut.mods`) na versão usada. Usar a mesma função em `save_config` e `setup`, eliminando o parse duplicado.
- **Critério de aceite:** `normalize` retorna `Err` para `""`, `"TAB"`, `"A"`, `"SHIFT+A"`, `"CTRL+"`, `"FOO+BAR"` e `Ok` para `"CTRL+ALT+C"` e `" ctrl+alt+c "` (devolvendo `"CTRL+ALT+C"`).
- **Teste a adicionar:** `normalize_rejects_unparseable_shortcut`, `normalize_rejects_shortcut_without_ctrl_alt_or_super`, `normalize_trims_and_canonicalizes_shortcut`.
- **Depende de / Conflita com:** nenhum. P1-003 depende deste. Coordenar com a Parte 4 (UI não deve aceitar `Tab/Enter/Esc/Backspace`).
- **Esforço:** P

---

#### P1-005 — "Salvar" grava estado de runtime obsoleto (`x`, `y`, `width`, `height`, `locked`, `always_on_top`)
- **Severidade:** ALTA
- **Categoria:** integridade-de-dados
- **Confiança:** PROVAVEL
- **Local:** `settings.ts:171` (`load` guarda `current`), `settings.ts:181-182,185,190` (`read`), `settings.ts:201-202` (submit); `commands.rs:11` (`*stored=c.clone()` e `config::save`)
- **Evidência:**
  ```ts
  // settings.ts:181-182,185,190
  x: current?.x ?? null,
  y: current?.y ?? null,
  always_on_top: checked("always_on_top"),
  locked: checked("locked"),
  ```
  `current` é uma foto tirada em `load()` (ao abrir) ou no último `save`. Enquanto a janela de Configurações fica aberta, `save_window_state` (`window.rs:347`) atualiza `x/y/width/height` na memória e a bandeja altera `locked`/`always_on_top` (`commands.rs:17-18`); o formulário **não é notificado** (não há evento backend→front). `save_config` (commands.rs:9-11) usa o payload inteiro, só aplicando `normalize`, e faz `*stored = c.clone()` e `config::save`.
- **Reprodução:** não reproduzível aqui. Em Windows: (1) abrir Configurações; (2) arrastar o widget para outro canto; (3) clicar em Salvar sem tocar em nada → `config.json` volta com `x/y` antigos (conferir no arquivo e reiniciando). Variante: com Configurações aberta, usar "Travar / destravar" na bandeja e depois Salvar → o travamento é desfeito silenciosamente; ou redimensionar o widget por arraste e Salvar → ele "volta" ao tamanho do formulário.
- **Causa raiz:** `save_config` trata o formulário como snapshot autoritativo de **todos** os campos, inclusive os que o usuário não editou e que o backend altera por outros caminhos (modelo "last write wins" sem versão).
- **Impacto:** perda silenciosa de posição, desfazer de alternância feita na bandeja, tamanho revertido. Afeta qualquer usuário que use Configurações e bandeja/arraste na mesma sessão.
- **Correção proposta:** (1) o backend **ignora `x`/`y` vindos do front** (`c.x = old.x; c.y = old.y;`; não há campo de posição na UI — `ids` em `settings.ts:110-114` não os inclui). (2) Trocar `save_config` por **semântica de patch**: o front envia só os campos alterados (comparando com `current`), e o backend aplica o patch sobre o estado **mais recente** (`#[derive(Deserialize)] struct ConfigPatch { site_url: Option<String>, ... }` + `fn merge(latest:&AppConfig, patch:ConfigPatch)->AppConfig`). (3) Complementar: emitir `app.emit_to("settings","config-changed",&cfg)` após `toggle_locked/topmost` e após mudanças de tamanho, e o `settings.ts` atualizar os campos **não editados** (coordenar com Parte 4). Alternativa mínima: campo `revision: u64` incrementado a cada mudança; `save_config` rejeita payload com `revision` antigo ("a configuração mudou; reabra").
- **Critério de aceite:** teste de `merge`: dado `latest` com `x=10,y=20,locked=true` e patch contendo só `opacity=80`, o resultado preserva `x,y,locked`. Manual: o cenário de reprodução não altera `x/y/locked`.
- **Teste a adicionar:** `merge_patch_preserves_runtime_position`, `merge_patch_preserves_locked_and_always_on_top_when_not_in_patch`, `save_config_ignores_client_supplied_xy`.
- **Depende de / Conflita com:** nenhum. Coordenar com Parte 4 (settings.ts) e T-001.
- **Esforço:** M

---

#### P1-006 — `apply_config` reaplica sempre o tamanho da memória; com `remember_position=false` a janela "volta" ao tamanho antigo
- **Severidade:** MÉDIA
- **Categoria:** correcao-logica
- **Confiança:** PROVAVEL
- **Local:** `window.rs:344` (`apply_config → set_size`), `window.rs:347` (`save_window_state` → `if !c.remember_position{return}`); chamadores de `apply_config`: `commands.rs:11,17,18` e `window.rs:212`
- **Evidência:**
  ```rust
  // window.rs:344 (trecho)
  let _=w.set_size(LogicalSize::new(c.width as f64,c.height as f64));
  // window.rs:347 (trecho)
  let mut c=match state.config.lock(){Ok(v)=>v,Err(_)=>return};
  if !c.remember_position{return}
  ```
  Com `remember_position=false`, a memória **nunca** acompanha redimensionamentos feitos por arraste; mas `apply_config` é chamado também por `toggle_locked`, `toggle_topmost` e qualquer `save_config` (até mudar só a opacidade) e executa `set_size` incondicional.
- **Reprodução:** não reproduzível aqui. Em Windows: desmarcar "Lembrar posição e tamanho"; redimensionar o widget; bandeja → "Sempre no topo". Previsto: o widget volta a 500×700 (ou ao último valor salvo no formulário).
- **Causa raiz:** `apply_config` mistura "aplicar configuração alterada" com "reaplicar tudo"; a geometria é tratada como configuração em vez de estado de runtime quando `remember_position=false`.
- **Impacto:** salto visível do tamanho em ações sem relação (travar, topo, mudar opacidade). Também há uma regravação redundante: com `remember_position=true`, o `set_size` de `apply_config` dispara `Resized` → `save_window_state` → nova gravação.
- **Correção proposta:** `apply_config(app, novo, anterior: Option<&AppConfig>, forcar_tamanho: bool)` e chamar `set_size` apenas se `forcar_tamanho` ou `anterior.map_or(true, |p| p.width != novo.width || p.height != novo.height)`. `toggle_*` passam `anterior=Some(...)` (tamanho igual → não redimensiona). `create_main` passa `None`. `restore_default_size` (P1-012) passa `forcar_tamanho=true`. Extrair a decisão para `fn should_resize(prev, new, force) -> bool` testável.
- **Critério de aceite:** `should_resize` retorna `false` quando só `locked`/`always_on_top`/`opacity` mudam e `true` quando `width`/`height` mudam ou `force=true`.
- **Teste a adicionar:** `apply_config_does_not_resize_when_size_unchanged`, `apply_config_resizes_when_size_changed`, `restore_default_size_forces_resize`.
- **Depende de / Conflita com:** nenhum. P1-007 e P1-012 dependem deste.
- **Esforço:** M

---

#### P1-007 — `config.json` é regravado a cada `Moved`/`Resized`, na thread da UI
- **Severidade:** MÉDIA
- **Categoria:** recursos-memoria
- **Confiança:** PROVAVEL (o número de gravações é certo; o impacto perceptível em tempo real é `HIPOTESE` até medir em Windows)
- **Local:** `window.rs:347` (`save_window_state`), `main.rs:196-198` (`on_window_event`)
- **Evidência:** `main.rs:196-198` encaminha todo `Moved`/`Resized` para `save_window_state`, que clona a config, e chama `crate::config::save` (criar diretório + escrever temporário + `rename`) e `apply_native_corners` (Parte 2) **por evento**. O handler roda na thread do loop de eventos (callbacks de `on_window_event`), a mesma da UI.
- **Reprodução:** não reproduzível aqui. Em Windows: arrastar o widget por 2 s com o Process Monitor filtrando `config.json*` — esperado dezenas/centenas de `WriteFile`+`SetRenameInformationFile`. Antivírus em tempo real amplifica o custo.
- **Causa raiz:** persistência síncrona por evento de alta frequência, sem debounce.
- **Impacto:** I/O contínuo no desktop do usuário, risco de travadas durante o arraste (E/S de antivírus no `rename`), desgaste desnecessário, e muitas oportunidades de corromper o arquivo (ver P1-013).
- **Correção proposta:** em `save_window_state`, apenas atualizar a memória (barato) e marcar `dirty`/timestamp; uma única thread de gravação (ou `tauri::async_runtime`) persiste após ~500 ms sem novos eventos e **no encerramento** (`RunEvent::Exit`/`ExitRequested` — para isso trocar `.run(...)` por `.build(...)?.run(|app, ev| ...)`, e chamar flush também em `quit_app`). A gravação sempre lê o **estado mais recente** sob o lock de gravação (P1-013).
- **Critério de aceite:** simular 1.000 chamadas de `record_geometry` em 1 s resulta em ≤ 2 gravações em disco; ao receber `Exit`, o estado pendente é gravado; `cargo test` verde.
- **Teste a adicionar:** `window_state_writes_are_debounced`, `window_state_is_flushed_on_exit` (abstrair relógio e destino de gravação).
- **Depende de / Conflita com:** depende de P1-013, P1-006 e P1-008. Conflita com qualquer mudança que torne `quit_app` imediato sem flush (Parte 3).
- **Esforço:** M

---

#### P1-008 — Valores sentinela/degenerados de `Moved`/`Resized` (minimizar/ocultar) podem ser persistidos
- **Severidade:** MÉDIA
- **Categoria:** compatibilidade-plataforma
- **Confiança:** HIPOTESE
- **Local:** `window.rs:347` (`save_window_state`)
- **Evidência:** a função grava `Moved(p)` → `c.x=Some(p.x); c.y=Some(p.y)` e `Resized(z)` → `width=((z.width as f64/scale).round() as u32).clamp(150,2000)`, `height=…clamp(100,1600)` **sem** filtrar `z.width==0 || z.height==0`, `window.is_minimized()`/`is_visible()` ou posições sentinela (no Windows, janelas minimizadas usam `-32000,-32000`). Um `Resized(0,0)` viraria 150×100 (o piso do `clamp`) e seria persistido. **Não verifiquei** se o `tao` emite `Moved(-32000,-32000)`/`Resized(0,0)` nesse cenário; por isso `HIPOTESE`.
- **Reprodução:** em Windows: com `remember_position=true`, minimizar a janela (Win+↓ ou "Mostrar área de trabalho") e restaurar; abrir `config.json`. Se aparecer `x/y ≈ -32000` ou `width/height = 150/100`, confirmar o achado.
- **Causa raiz:** ausência de filtro de eventos degenerados antes de converter em preferência persistente.
- **Impacto:** se confirmado: janela minúscula (150×100) na próxima chamada de `apply_config`/reinício; `x/y` sentinela é autocorrigido por `restore_position` (centraliza), mas a posição do usuário é perdida.
- **Correção proposta:** ignorar o evento quando `width==0||height==0`, quando `window.is_minimized()==Ok(true)`, ou quando `x <= -30000 || y <= -30000`; só persistir geometria de janela visível. Extrair `fn sanitize_geometry(event, scale) -> Option<GeometryUpdate>` pura.
- **Critério de aceite:** testes puros: `Resized(0,0)` → `None`; `Moved(-32000,-32000)` → `None`; `Resized(1000×1400 @ 2.0)` → `500×700` lógicos.
- **Teste a adicionar:** `window_state_ignores_zero_size_resize`, `window_state_ignores_sentinel_position`, `window_state_converts_physical_to_logical`.
- **Depende de / Conflita com:** nenhum. P1-007 depende deste.
- **Esforço:** P

---

#### P1-009 — `restore_position`: 1 px de sobreposição conta como "visível", escala do monitor errado e overflow `i32`
- **Severidade:** MÉDIA
- **Categoria:** correcao-logica
- **Confiança:** PROVAVEL
- **Local:** `window.rs:307` (`restore_position`); chamada em `window.rs:212` (`create_main`)
- **Evidência:**
  ```rust
  let scale=w.scale_factor().unwrap_or(1.0);
  let s=PhysicalSize::new((c.width as f64*scale)as u32,(c.height as f64*scale)as u32);
  let visible=monitors.iter().any(|m|{let p=m.position();let z=m.size();
      (x+s.width as i32).min(p.x+z.width as i32)>x.max(p.x)&&(y+s.height as i32).min(p.y+z.height as i32)>y.max(p.y)});
  ```
  Simulação em Python da mesma aritmética (`sim_p1.py`):
  ```
  A1 janela 500x700 com 1px dentro do monitor (x=1919,y=0): True
  A2 idem, 2px em y (x=100,y=1079): True
  A3 totalmente fora (x=-32000,y=-32000): False
  A4 escala 1.0 (atual) vs 1.5 (destino), x=-700: False / True
  A5 x=i32::MAX: x+500 (release, wrapping) = -2147483149 (em debug/test: panic por overflow)
  ```
  Monitores com coordenadas negativas **são tratados corretamente** (a interseção usa `i32` com sinal; sem achado).
- **Reprodução:** não reproduzível aqui (sem Rust/Windows). Em Windows com 2 monitores: mover o widget para que 1–2 px fiquem no monitor da direita, salvar, desconectar o monitor e reiniciar.
- **Causa raiz:** (a) a condição é "interseção não vazia", sem área mínima; (b) usa `scale_factor()` da janela **recém-criada** (monitor primário), não o do monitor onde `(x,y)` foi salvo, para estimar o tamanho físico; (c) `x + s.width as i32` e `p.x + z.width as i32` sem aritmética saturada.
- **Impacto:** (a) janela sem moldura e fora da barra de tarefas (padrão `show_in_taskbar=false`) quase totalmente fora da tela e praticamente impossível de agarrar; recuperação exige tray → Configurações → desmarcar "Lembrar posição" e reiniciar; (b) decisão de visibilidade errada em DPI misto; (c) com `x` extremo (arquivo editado) panic em debug/teste e valor corrompido em release.
- **Correção proposta:** extrair `fn is_placeable(x,y,w_phys,h_phys, monitors:&[Rect], min_visible:(i64,i64)) -> bool` usando `i64`, exigindo área visível mínima (ex.: ≥ 80×40 px físicos **dentro de um mesmo monitor**) e calcular `w_phys/h_phys` com o `scale_factor` do monitor de destino (`monitor.scale_factor()`, escolhido pelo ponto `(x,y)` ou pela maior interseção). Quando parcialmente visível, preferir **encaixar** (`clamp`) dentro do monitor mais próximo em vez de centralizar.
- **Critério de aceite:** testes puros: 1 px de sobreposição → `false`; janela 100% dentro → `true`; monitor com `x` negativo → `true`; `x=i32::MAX` não entra em pânico; escala 1.5 do monitor de destino muda a decisão do caso A4.
- **Teste a adicionar:** `rect_with_one_pixel_overlap_is_not_visible`, `rect_visible_on_negative_coordinate_monitor`, `rect_visibility_uses_target_monitor_scale`, `rect_visibility_does_not_overflow_at_i32_max`.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** M

---

#### P1-010 — Mutex envenenado devolve `AppConfig::default()` silenciosamente
- **Severidade:** BAIXA
- **Categoria:** correcao-logica
- **Confiança:** PROVAVEL (o padrão é certo; o gatilho — pânico com o lock segurado — não foi encontrado no código atual)
- **Local:** `commands.rs:8` (`get_config`); `window.rs:184` (`on_navigation`, `unwrap_or(true)` para `locked`), `:206`, `:211`, `:287`, `:298` (`unwrap_or_default()` sobre a config); `:340` (`unwrap_or(12)`), `:345-346`, `:348` (`unwrap_or(0)`). `commands.rs:9,12,17,18` tratam o erro de formas diferentes (erro/retorno silencioso).
- **Evidência:** `lock().map(|v|v.clone()).unwrap_or_default()` converte `PoisonError` em `AppConfig::default()`. Revisei todas as seções críticas que seguram o lock (`save_window_state`, `toggle_*`, `save_config`): são infalíveis (sem `unwrap`/indexação/divisão inteira), logo o envenenamento hoje é praticamente inalcançável.
- **Reprodução:** não reproduzível aqui; exigiria injetar um `panic!` com o lock segurado (teste: ver abaixo).
- **Causa raiz:** tratamento de erro de lock decidido por ponto de uso e sem política central.
- **Impacto:** se ocorresse, `get_config` mostraria os padrões; `on_navigation` usaria `locked=true` e a URL padrão; `toggle_*` retornariam sem efeito e sem log. `save_config` **não** grava nesse estado (retorna "Falha ao ler a configuração."), o que é seguro.
- **Correção proposta:** `impl AppState { fn snapshot(&self) -> AppConfig { self.config.lock().unwrap_or_else(PoisonError::into_inner).clone() } fn update<R>(&self, f: impl FnOnce(&mut AppConfig)->R) -> R }`. A config não tem invariantes quebradas por um pânico alheio, então recuperar com `into_inner` é preferível a trocar por padrões. Substituir os ~14 pontos de uso. Registrar um `WARN` na primeira recuperação.
- **Critério de aceite:** teste que envenena o mutex de propósito e verifica que `snapshot()` devolve o valor anterior ao pânico (não o padrão).
- **Teste a adicionar:** `snapshot_recovers_from_poisoned_mutex_without_resetting_to_defaults`.
- **Depende de / Conflita com:** nenhum. Facilita P1-005, P1-007 e T-001.
- **Esforço:** P

---

#### P1-011 — Autostart é consultado/alterado em todo "Salvar", mesmo sem mudança
- **Severidade:** BAIXA
- **Categoria:** correcao-logica
- **Confiança:** HIPOTESE
- **Local:** `commands.rs:10` (`save_config`), `commands.rs:8` (`get_config`), `main.rs:138-141` (`setup`)
- **Evidência:**
  ```rust
  // commands.rs:10 (formatado)
  if c.autostart { if app.autolaunch().enable().is_err() { ...warns.push("Não foi possível habilitar o início com o Windows.") } }
  else if app.autolaunch().disable().is_err() { ...warns.push("Não foi possível desabilitar o início com o Windows.") }
  ```
  Executa `enable()`/`disable()` **sempre**, independentemente de `old.autostart` ou do estado do SO. Se `disable()` retornar erro quando a entrada não existe (padrão `autostart=false`), todo "Salvar" exibiria o aviso "Não foi possível desabilitar…" para quem nunca ativou o recurso. **Não consegui confirmar esse comportamento do crate `auto-launch`/plugin na documentação** → `HIPOTESE`. Independentemente disso, uma falha de registro (política de grupo, antivírus) gera aviso em salvamentos que não tocaram no autostart.
  Observações (sem achado): `get_config` consultar o SO a cada chamada é barato e o erro é ignorado mantendo o valor armazenado; o `setup` sincronizar `cfg.autostart` com o SO é intencional (a única ressalva, a gravação incondicional no startup, está em P1-001).
- **Reprodução:** em Windows, perfil novo: abrir Configurações → Salvar sem alterar nada. Se aparecer o aviso de autostart, confirmar. Verificar `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` antes/depois.
- **Causa raiz:** efeito colateral aplicado de forma incondicional em vez de "somente se mudou".
- **Impacto:** avisos espúrios, escrita desnecessária no registro; mascara falhas reais de autostart.
- **Correção proposta:** `let atual = app.autolaunch().is_enabled().unwrap_or(old.autostart); if c.autostart != atual { enable/disable }`; tratar `disable()` com erro seguido de `is_enabled()==Ok(false)` como sucesso. Opcional (melhoria): quando `autostart` estiver ativo, re-executar `enable()` no `setup` para atualizar o caminho do `.exe` após reinstalação em outra pasta.
- **Critério de aceite:** salvar sem alterar `autostart` não chama `enable`/`disable` (teste com `trait AutostartBackend` falso) e não produz aviso.
- **Teste a adicionar:** `save_config_does_not_touch_autostart_when_unchanged`, `disable_error_is_ignored_when_already_disabled`.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

---

#### P1-012 — `restore_default_size`: dimensões duplicadas e fluxo excessivo
- **Severidade:** BAIXA
- **Categoria:** correcao-logica
- **Confiança:** PROVAVEL
- **Local:** `commands.rs:12` (`restore_default_size`); `config.rs:37-38` (`Default`)
- **Evidência:** `c.width=500;c.height=700;save_config(app,state,c)` repete as constantes de `AppConfig::default()` (`width: 500`, `height: 700`) e passa por **todo** `save_config` (atalho, autostart, navegação, título). Com a correção de P1-006 (`set_size` só se o tamanho mudou) o botão **deixaria de funcionar** quando o valor na config já é 500×700 mas a janela foi redimensionada por arraste. Além disso, `display(result.config)` no front (`settings.ts:244-246`) descarta edições não salvas do formulário.
- **Reprodução:** não reproduzível aqui. Previsto após P1-006 mal feito: redimensionar por arraste → "Restaurar tamanho" → nada acontece.
- **Causa raiz:** constantes de padrão duplicadas; comando reaproveita um fluxo mais pesado que o necessário.
- **Impacto:** deriva entre padrão e botão; regressão provável ao corrigir P1-006; perda de edições pendentes do formulário.
- **Correção proposta:** `pub const DEFAULT_WIDTH: u32 = 500; pub const DEFAULT_HEIGHT: u32 = 700;` em `config.rs`, usadas por `Default` e por `restore_default_size`; o comando altera só `width/height` e chama `apply_config(..., forcar_tamanho=true)`, sem passar por atalho/autostart. O front deve atualizar apenas os campos de tamanho (Parte 4).
- **Critério de aceite:** `restore_default_size` força o redimensionamento mesmo quando `width/height` já são os padrões; um teste garante `DEFAULT_WIDTH/HEIGHT == AppConfig::default()`.
- **Teste a adicionar:** `restore_default_size_uses_default_constants`, `restore_default_size_forces_resize`.
- **Depende de / Conflita com:** depende de P1-006.
- **Esforço:** P

---

#### P1-013 — Gravação de `config.json` sem serialização, com temporário de nome fixo e fallback `copy` não atômico (risco latente)
- **Severidade:** BAIXA
- **Categoria:** concorrencia
- **Confiança:** HIPOTESE
- **Local:** `config.rs:117-131` (`save`); `window.rs:347` (snapshot salvo após soltar o lock: `let copy=c.clone();drop(c);let _=crate::config::save(...)`)
- **Evidência:**
  ```rust
  // config.rs:119,125-129
  let temp = dir.join("config.json.tmp");
  fs::write(&temp, ...)?;
  fs::rename(&temp, dir.join("config.json")).or_else(|_| {
      fs::copy(&temp, dir.join("config.json"))?;
      fs::remove_file(&temp)
  })
  ```
  Chamadores de `config::save`: `main.rs:141` (setup), `commands.rs:11` (`save_config`), `commands.rs:17-18` (`toggle_*`, via `tray.rs:91-96` `on_menu_event`), `window.rs:347` (via `main.rs:196-198`). **Todos executam na thread principal**: comandos síncronos do Tauri rodam na thread principal (documentação do Tauri), e os handlers de bandeja e de janela são despachados pelo loop de eventos. Portanto **a corrida descrita na pista 6 não ocorre no código atual** (rebaixado de ALTA para latente). O risco passa a existir assim que algum chamador sair da thread principal — o que a correção de P1-007 introduz. Os pontos fracos reais, independentes de threads: sem `fsync` antes do `rename`; o fallback `copy` (usado se o `rename` falhar, p.ex. arquivo aberto por antivírus sem `FILE_SHARE_DELETE`) **trunca e regrava o destino no lugar**; uma queda no meio deixa `config.json` truncado → P1-001.
- **Reprodução:** não reproduzível aqui. Teste de estresse sugerido: 8 threads × 50 `save` simultâneos e verificar `config.json` parseável e igual a um dos snapshots.
- **Causa raiz:** `save` não é serializado, não usa nome temporário único e o fallback não é atômico; o chamador salva um snapshot clonado em vez do estado mais recente.
- **Impacto:** hoje: baixo. Com gravação em segundo plano: arquivo corrompido/snapshot antigo sobrescrevendo o novo.
- **Correção proposta:** `static SAVE_LOCK: Mutex<()>` (ou `AppState::persist(&self)` que, **dentro** do lock de gravação, lê a config atual e grava); temporário único (`config.json.<pid>.<contador>.tmp`); `file.sync_all()` antes do `rename`; em falha do `rename`, tentar de novo com pequeno backoff (3×50 ms) em vez de `copy`; limpar o temporário em erro.
- **Critério de aceite:** teste de estresse com threads passa; nenhum `config.json.tmp` fica no diretório; falha forçada do `rename` não trunca o destino.
- **Teste a adicionar:** `save_is_serialized_across_threads_and_leaves_valid_json`, `save_uses_unique_temp_file_and_cleans_up`.
- **Depende de / Conflita com:** nenhum. **Pré-requisito de P1-007.** Detalhes de I/O são da Parte 2.
- **Esforço:** M

---

#### P1-014 — Heurística `contains("://")` rejeita URL válida cujo esquema só aparece na query
- **Severidade:** BAIXA
- **Categoria:** validacao-entrada
- **Confiança:** PROVAVEL
- **Local:** `config.rs:62-64` (`normalize`)
- **Evidência:**
  ```rust
  if !site.contains("://") { site = format!("https://{site}"); }
  let url = Url::parse(&site).map_err(|_| "URL inválida.".to_owned())?;
  ```
  Entrada `investing.com/economic-calendar?returnTo=https://x` contém `://`, não recebe o prefixo e `Url::parse` falha (URL relativa sem base) → "URL inválida.". Verificados por raciocínio (sem execução) como **corretamente rejeitados**: `file:///x`, `ftp://x`, `javascript:alert(1)` (vira `https://javascript:alert(1)` → porta inválida), `data:text/html,…`, `user:pw@host` com `://`.
- **Reprodução:** não reproduzível aqui; o teste abaixo reproduz.
- **Causa raiz:** o teste "tem esquema?" olha qualquer posição da string, não o início.
- **Impacto:** baixo (URLs desse formato são raras); mensagem confusa.
- **Correção proposta:** considerar que há esquema só se `site` casar `^[A-Za-z][A-Za-z0-9+.-]*://` no início; caso contrário prefixar `https://`.
- **Critério de aceite:** `normalize` aceita `investing.com/x?u=https://y` e continua rejeitando `file:///`, `ftp://` e credenciais.
- **Teste a adicionar:** `normalization_prefixes_https_when_scheme_only_appears_in_query`.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

---

#### P1-015 — Falhas de persistência e de registro descartadas com `let _ =` sem log
- **Severidade:** BAIXA
- **Categoria:** correcao-logica
- **Confiança:** PROVAVEL
- **Local:** `main.rs:141` e `:170`; `commands.rs:9` (`unregister`), `:11` (`main.navigate(url)`), `:17`, `:18` (`let _=config::save`); `window.rs:347` (`let _=crate::config::save`)
- **Evidência (descartes que causam perda de diagnóstico, no escopo da Parte 1):**
  | Local | O que se perde |
  |---|---|
  | `main.rs:141` `let _ = config::save(&dir,&cfg)` | falha de gravação no startup (config não persiste e ninguém sabe) |
  | `main.rs:170` `let _ = ...register(shortcut)` (fallback) | o fallback também pode falhar; nada é registrado e nada é avisado |
  | `commands.rs:17,18` `let _=config::save(...)` (`toggle_*`) | alternar travar/topo "funciona" na sessão mas não persiste |
  | `window.rs:347` `let _=crate::config::save(...)` | falhas repetidas de gravação a cada evento, sem registro |
  | `commands.rs:11` `let _=main.navigate(url)` | URL salva, mas a janela não navega; o usuário recebe "salvo" |
  | `commands.rs:9` `let _=...unregister(s)` | fonte do atalho fantasma (P1-003) |
  A lista dos `let _ =` do projeto inteiro é objeto da Parte 3 (pista 8). Contagem na Parte 1: `commands.rs` 9 (linhas 9, 11×2, 13, 14×2, 15, 17, 18 — ver `grep -no "let _=" commands.rs`), `main.rs` 7 (linhas 65, 67, 74, 75, 141, 170, 194; só 141 e 170 relevantes aqui), `window.rs:307` 5, `:344` 8 e `:347` 1 (os de 307/344 são chamadas de API de janela sem consequência persistente).
- **Reprodução:** não reproduzível aqui. Em Windows: marcar `config.json` como somente leitura e alternar "Sempre no topo" pela bandeja; reiniciar → configuração perdida; `diagnostics.log` sem rastro.
- **Causa raiz:** ausência de um ponto único de persistência que registre falhas.
- **Impacto:** falhas silenciosas de persistência/registro dificultam suporte e escondem P1-001/P1-013 em campo.
- **Correção proposta:** helper `fn persist(state:&AppState)->bool` (usado por todos os chamadores) que executa o `save` e registra `diagnostics::log(dir,"ERRO",...)` com a causa; no `commands.rs:11`, registrar `WARN` se `navigate` falhar e/ou devolver `warning`.
- **Critério de aceite:** com diretório inacessível, cada chamador produz uma linha `ERRO`/`WARN` em `diagnostics.log`.
- **Teste a adicionar:** `persist_logs_error_when_save_fails` (usar diretório inexistente/somente leitura e ler o log).
- **Depende de / Conflita com:** depende de P1-013 (helper de persistência).
- **Esforço:** P

---

### Parte 2 — Integrações Windows, Arquivos e Diagnóstico (I/O)
**Não analisada nesta entrega.** Itens recebidos da Parte 1: P1-013 (detalhes de I/O de `save`), `app_directory` sem `APPDATA` e `create_dir_all(...)?` em `main.rs:133-134` (pista 4 da Parte 2).

### Parte 3 — Camada de Aplicação: Ciclo de Vida, Janelas, Navegação e Bandeja
**Não analisada nesta entrega.** Itens recebidos da Parte 1: `toggle_locked/topmost` não notificam a janela de Configurações (P1-005); `quit_app` sem flush (interação com P1-007); lista global de `let _ =` (P1-015).

### Parte 4 — Interface e Superfície de Entrada Externa
**Não analisada nesta entrega.** Itens recebidos da Parte 1: campo de atalho aceita `Tab/Enter/Esc/Backspace` (P1-004); formulário deve enviar patch/atualizar campos de runtime (P1-005, P1-012); contrato `Number()`/`NaN` (T-002).

### Parte 5 — Infraestrutura: Build, CI/CD, Dependências e Distribuição
**Não analisada nesta entrega.** Item recebido: a CI não executa os testes da Parte 1 com cobertura de comandos/geometria (seção 6).

## 5. Achados Transversais (T-001…) — preliminares (só perspectiva da Parte 1)

#### T-001 — `config.json` tem 4 caminhos de escrita sem funil único
- **Severidade:** MÉDIA · **Categoria:** integridade-de-dados · **Confiança:** PROVAVEL
- **Local:** `main.rs:141` (setup), `commands.rs:11` (`save_config`), `commands.rs:17-18` (`toggle_*` / bandeja, Parte 3), `window.rs:347` (`save_window_state`) + `config.rs:117-131` (I/O, Parte 2).
- **Evidência/impacto:** cada caminho lê/clona a config, modifica e salva por conta própria (P1-005, P1-007, P1-013, P1-015). **Correção:** um `ConfigStore` único (`update(|c| ...)` sob lock → persistir → notificar a UI) usado por todos; resolve de uma vez P1-005 (3), P1-010, P1-013 e P1-015. **Depende de:** P1-010, P1-013. **Esforço:** M. Validar nas Partes 2–3.

#### T-002 — Contrato front↔Rust: `Number()` produz `0`/`NaN`/`1000` e o backend responde de forma inconsistente
- **Severidade:** BAIXA · **Categoria:** validacao-entrada · **Confiança:** PROVAVEL
- **Local:** `settings.ts:174-195` (`read`) × `config.rs:74-82` (`normalize`).
- **Evidência (executado em Node):** `JSON.stringify({width:Number(""), height:Number("abc"), opacity:Number("  "), zoom:Number("1e3")})` → `{"width":0,"height":null,"opacity":0,"zoom":1000}`. No Rust: `0` é "corrigido" por `clamp` (largura 150, opacidade 30) **sem avisar**; `null` não desserializa em `u32` → erro cru do Tauri exibido na UI (`invalidConfig + String(error)`, `settings.ts:213`); `1000` é reduzido a 200 sem aviso. **Correção:** validar/normalizar no front (campo vazio/inválido → erro de campo) e fazer o backend devolver aviso quando aplicar `clamp`. A decidir na Parte 4.

#### T-003 — Mensagens do backend só em pt-BR e acopladas por `includes(...)`
- **Severidade:** BAIXA · **Categoria:** i18n · **Confiança:** PROVAVEL
- **Local:** `config.rs:60,65,71,84`, `commands.rs:9-11` (strings em pt-BR) × `settings.ts:205-207` (`includes("atalho")`, `includes("início")`).
- **Impacto:** com UI em EN/ES, erros de `normalize` aparecem em português; renomear uma palavra quebra o mapeamento. **Correção:** o backend devolve códigos estáveis (`SHORTCUT_CONFLICT`, `AUTOSTART_FAILED`, `INVALID_URL`) e o front traduz. Reforça P1-003/P1-004 (mensagens distintas). A fechar nas Partes 3–4.

## 6. Cobertura de Testes Ausente

Testes existentes da Parte 1 (6, em `config.rs`): `defaults_match_widget_requirements` (:139), `normalization_prefixes_https_and_clamps_values` (:151), `language_is_limited_to_supported_values` (:171), `old_configuration_without_language_keeps_existing_preferences` (:186), `normalization_rejects_non_http_urls_and_credentials` (:196), `corrupt_config_recovers_to_defaults` (:209). Observações: o último só testa JSON sintaticamente inválido (não "campo fora do tipo" — exatamente o gap de P1-001); nenhum teste cobre `save`, `commands.rs`, `main.rs` nem geometria.

| Cenário crítico sem teste | Nome sugerido | Achado |
|---|---|---|
| Campo fora do tipo/limite preserva os demais | `load_keeps_valid_fields_when_one_field_is_out_of_range` | P1-001 |
| JSON inválido gera backup antes de sobrescrever | `load_backs_up_unparsable_config_before_overwriting` | P1-001 |
| Só `site_url` inválida é resetada | `load_resets_only_invalid_site_url` | P1-001 |
| Campo novo ausente não derruba o resto | `load_tolerates_missing_fields_via_struct_default` | P1-001 |
| Atalho não parseável / sem modificador / espaços | `normalize_rejects_unparseable_shortcut`, `normalize_rejects_shortcut_without_ctrl_alt_or_super`, `normalize_trims_and_canonicalizes_shortcut` | P1-004 |
| Mesmo atalho, outra grafia | `change_shortcut_is_noop_for_same_hotkey_different_spelling` | P1-003 |
| Fallback desregistrado ao trocar | `change_shortcut_unregisters_active_fallback` | P1-003 |
| Parse × conflito com mensagens distintas | `change_shortcut_distinguishes_parse_error_from_conflict` | P1-003 |
| Falha de gravação não deixa efeitos no SO | `save_config_rolls_back_shortcut_when_persist_fails` | P1-002 |
| Patch preserva `x/y/locked/always_on_top` | `merge_patch_preserves_runtime_position`, `merge_patch_preserves_locked_and_always_on_top_when_not_in_patch`, `save_config_ignores_client_supplied_xy` | P1-005 |
| Decisão de redimensionar | `apply_config_does_not_resize_when_size_unchanged`, `apply_config_resizes_when_size_changed` | P1-006 |
| Botão restaurar força redimensionamento | `restore_default_size_forces_resize`, `restore_default_size_uses_default_constants` | P1-006/012 |
| Geometria: visibilidade | `rect_with_one_pixel_overlap_is_not_visible`, `rect_visible_on_negative_coordinate_monitor`, `rect_visibility_uses_target_monitor_scale`, `rect_visibility_does_not_overflow_at_i32_max` | P1-009 |
| Geometria: eventos degenerados | `window_state_ignores_zero_size_resize`, `window_state_ignores_sentinel_position`, `window_state_converts_physical_to_logical` | P1-008 |
| Debounce e flush | `window_state_writes_are_debounced`, `window_state_is_flushed_on_exit` | P1-007 |
| `save` concorrente / temp único | `save_is_serialized_across_threads_and_leaves_valid_json`, `save_uses_unique_temp_file_and_cleans_up` | P1-013 |
| Mutex envenenado | `snapshot_recovers_from_poisoned_mutex_without_resetting_to_defaults` | P1-010 |
| Autostart só quando muda | `save_config_does_not_touch_autostart_when_unchanged`, `disable_error_is_ignored_when_already_disabled` | P1-011 |
| URL com `://` só na query | `normalization_prefixes_https_when_scheme_only_appears_in_query` | P1-014 |
| Persistência registra erro | `persist_logs_error_when_save_fails` | P1-015 |

Observação de design: `commands.rs` e `window.rs` concentram lógica de decisão dentro de funções que dependem de `AppHandle`/`WebviewWindow`; para testar, extraia a lógica para funções puras (`merge`, `should_resize`, `is_placeable`, `sanitize_geometry`, `change_shortcut` com `trait` falso).

## 7. Falsos Positivos e Itens Verificados Sem Problema

**Pistas iniciais da Parte 1 — resultado:**

| # | Pista | Resultado |
|---|---|---|
| 1 | `load` descarta tudo por um campo inválido | **Confirmada** (P1-001). Agravante encontrado: o `setup` regrava o arquivo (`main.rs:141`). |
| 2 | `normalize` valida pouco | **Parcial.** Atalho: confirmada (P1-004). `x`/`y`: **refutada como falha própria** — o tipo `Option<i32>` limita o domínio e `restore_position` (`window.rs:307`) já trata coordenadas fora de qualquer monitor (centraliza); o resíduo (overflow, 1 px) é P1-009. `site_url` aceita qualquer host: **comportamento intencional** (URL configurável, README) e **não é achado**; o isolamento da origem remota depende de ACL/capabilities (Parte 4). |
| 3 | Fallback de atalho / `registered_shortcut` | **Confirmada** (P1-003): atalho fantasma, `registered_shortcut` só escrito (4 ocorrências, 0 leituras), `unregister` de atalho nunca registrado, aviso enganoso. |
| 4 | `save_window_state` | **Parcial.** Gravação por evento: confirmada (P1-007). "Fora do lock": **refutada como corrida real** (todos os chamadores na thread principal; P1-013 latente). "Snap-back" com `remember_position=false`: confirmada (P1-006). `Resized` 0×0 e `Moved` −32000: **HIPOTESE** (P1-008). |
| 5 | Estado obsoleto vindo do front | **Confirmada** (P1-005). |
| 6 | Gravação concorrente | **Rebaixada a risco latente** (P1-013, `HIPOTESE`): temp de nome fixo e fallback `copy` existem, mas hoje não há escritores concorrentes. |
| 7 | Mutex envenenado | **Confirmada como padrão, impacto BAIXO** (P1-010): nenhuma seção crítica pode entrar em pânico; `save_config` não grava nesse estado. |
| 8 | Autostart | **Parcial.** `get_config` consulta o SO a cada chamada: **sem problema**. `save_config` chama `enable/disable` sempre: P1-011 (`HIPOTESE`). `setup` sobrescreve `cfg.autostart` e grava: **intencional**; só o efeito de gravar sempre é tratado em P1-001. |
| 9 | `restore_position` com `scale_factor` da janela atual | **Confirmada** (P1-009). Coordenadas negativas: **sem problema**. |

**Verificado sem problema (Parte 1):**
- `normalize` — clamps (`width 150–2000`, `height 100–1600`, `opacity 30–100`, `zoom 50–200`, `auto_reload 0 ou 5–1440`, `corner_radius ≤ 64`), rejeição de esquema não HTTP(S) e de credenciais, idioma limitado a 3 valores (cobertos por testes existentes).
- `javascript:`/`data:` sem `://` → viram `https://javascript:...` e falham no `Url::parse` (rejeitados). `localhost:3000` → `https://localhost:3000/`.
- Campos ausentes do tipo `Option` (`x`,`y`) desserializam como `None`; campos desconhecidos no JSON são ignorados (sem `deny_unknown_fields`).
- `save_config` recusa gravar se o mutex estiver envenenado (`commands.rs:9,11`) — comportamento seguro.
- `main.rs:131-146`: `AppState` é registrado (`app.manage`) antes de `create_main` e antes de qualquer `on_window_event`; `app.state::<AppState>()` em `save_window_state` não pode falhar por estado ausente.
- Eventos `Moved/Resized` emitidos por `restore_position`/`apply_config` durante `create_main` **não** passam por `save_window_state` (o handler só é registrado em `main.rs:186`, depois da criação).
- Segunda instância: o plugin `single-instance` encerra no init do plugin (antes do `setup`), portanto não há escrita concorrente de `config.json` entre processos (**HIPOTESE** forte; confirmar com a Parte 2).
- **Falso positivo de ambiente (não é achado):** `tsc` retornou `TS2307` (×3) e `TS2345` em `settings.ts(172,11)`. O `TS2345` desaparece quando `@tauri-apps/api` está instalado: `invoke<C>(...)` retorna `Promise<C>` e `current` é estreitado a `C` na linha 171 antes de `display(current)`. Reavaliar com `npm install` real (Parte 5).

## 8. Melhorias Opcionais (máx. 10 linhas)

1. `load(dir: &PathBuf)` → `&Path` (clippy `ptr_arg`); `save` idem.
2. `main::toggle` (main.rs:62-70) e `commands::toggle_main_visibility` (commands.rs:14) são duplicatas; manter uma.
3. Adicionar `#[derive(PartialEq)]` a `AppConfig` para só gravar quando mudou.
4. Rodar `cargo fmt` em `commands.rs`/`window.rs` (linhas únicas dificultam revisão e `git blame`).
5. `create_main` usa `.visible(true)` e só depois `restore_position`: criar oculta, posicionar e então `show()` evita "pulo" visual na inicialização.
6. Campo `schema_version` em `config.json` para migrações futuras.
7. Refrescar o caminho do `.exe` no autostart no `setup` quando ativo (pós-reinstalação).
8. `AppState::new(...)` em vez de montar o struct no `setup`.

## 9. Checklist Final de Verificação Pós-Correção (Parte 1)

- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` verde, incluindo todos os testes novos da seção 6.
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` sem avisos novos.
- [ ] `npm run build` verde (`tsc --noEmit && vite build`) após as mudanças de contrato de `save_config` (P1-005).
- [ ] `grep -rn registered_shortcut src-tauri/src` mostra **leituras** além das escritas.
- [ ] `grep -rn "unwrap_or_default" src-tauri/src` não devolve mais `AppConfig::default()` por mutex envenenado.
- [ ] Windows — `config.json` com `"opacity": 300` e `"width": 800`: ao iniciar, `width=800` é mantido e existe `config.json.bad-*`/aviso no `diagnostics.log` (P1-001).
- [ ] Windows — Configurações aberta, mover o widget e travar pela bandeja, Salvar: posição e travamento preservados (P1-005).
- [ ] Windows — `CTRL+SHIFT+X` ocupado por outro app: após reiniciar, o atalho ativo mostrado = o real; trocar de atalho desregistra o fallback; `Tab` e `A` são rejeitados (P1-003/004).
- [ ] Windows — `remember_position=false`: redimensionar e alternar "Sempre no topo" não altera o tamanho (P1-006).
- [ ] Windows — minimizar/restaurar não grava `-32000`/150×100 em `config.json` (P1-008).
- [ ] Windows — arrastar por 2 s: ≤ 2 gravações de `config.json` (Process Monitor) e nenhum `config.json.tmp` residual (P1-007/013).
- [ ] Windows, 2 monitores com DPI diferentes: janela com 1 px visível é reposicionada/encaixada no próximo início (P1-009).
- [ ] Windows, perfil novo: "Salvar" sem mudanças não exibe aviso de autostart (P1-011).
- [ ] Nenhum arquivo fora do escopo foi alterado; documentação (READMEs) atualizada se o comportamento de atalho/posição mudar.
