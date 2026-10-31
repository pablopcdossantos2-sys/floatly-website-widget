# Relatório de análise — Floatly (`floatly-website-widget-main`) — **PARTE 4 de 4**

> **Parte executada neste relatório: Parte 4 — Build, CI/CD, empacotamento, dependências, documentação e distribuição.**
> As Partes 1, 2 e 3 **não** foram auditadas aqui (seus arquivos foram lidos apenas de forma pontual, para confirmar interfaces com a Parte 4 — ver seção 3).
> Data da análise: 31/10/2026. Modo: somente leitura (nenhum arquivo do projeto foi alterado).
> Público-alvo: outra IA que aplicará as correções. Cada achado é autossuficiente.

---

## 1. Resumo executivo

A infraestrutura de build do Floatly está **bem acima da média** para um projeto desse porte: ações fixadas por SHA, `persist-credentials: false`, `permissions: contents: read`, separação entre job de build (sem segredos) e job de assinatura, limpeza do PFX, verificação do subsistema PE, validação do MSIX com identidade de teste, `--locked` em todos os comandos Cargo, entradas de `workflow_dispatch` passadas por `env:` (sem injeção de expressão). Os 5 scripts `check:*` **passaram** na execução local. Não foram encontrados achados Críticos nem Altos *confirmados*.

Os problemas reais concentram-se em: (a) um risco de **rejeição/falha de validação do MSIX** por `MinVersion` incompatível com o namespace `uap10`; (b) o **job de assinatura acessível a PRs**; (c) **acoplamento frágil** entre Dependabot, `check:tauri-versions` e `check:acl`; (d) uma **provável divergência no nome da entrada de inicialização automática** (`Run`) removida pelo desinstalador; (e) **documentação/Política de Privacidade que não cobrem o canal MSIX** (dados em `%APPDATA%` virtualizado) nem os backups `config.json.bad-*`.

**Nota de risco da Parte 4: MÉDIO.**

### Contagem de achados

| Parte | Crítica | Alta | Média | Baixa | Info | Total |
|---|---|---|---|---|---|---|
| Parte 1 | — | — | — | — | — | não auditada |
| Parte 2 | — | — | — | — | — | não auditada |
| Parte 3 | — | — | — | — | — | não auditada |
| **Parte 4** | 0 | 0 | 9 | 8 | 1 | **18** |

### Os 5 achados mais importantes

1. **FLY-P4-001** — `MinVersion="10.0.18362.0"` no manifesto MSIX, mas o manifesto usa `uap10:*` (introduzido no Windows 10 2004, build 19041): risco de falha em `winapp pack`/validação do Partner Center.
2. **FLY-P4-002** — `sign-and-publish` roda em `pull_request` e recebe `WINDOWS_PFX_*`: um PR de branch do próprio repositório pode alterar o workflow e exfiltrar o certificado.
3. **FLY-P4-005** — O desinstalador apaga as entradas `Run` `widget-website`/`Widget Website`, mas o plugin de autostart provavelmente grava como `Floatly` (productName): a entrada fica órfã e o README promete remoção.
4. **FLY-P4-003 / FLY-P4-004** — Dependabot separa npm e cargo (PRs que falham em `check:tauri-versions`) e `check:acl` trava a capability remota em `http://*` + `https://*`, impedindo qualquer endurecimento vindo da Parte 3 sem editar o script.
5. **FLY-P4-012** — Política de Privacidade/SUPPORT/README descrevem somente o canal NSIS; no MSIX os dados em `%APPDATA%\WidgetWebsite` são (provavelmente) virtualizados, e os backups `config.json.bad-*` não são mencionados.

---

## 2. Escopo, premissas e limitações

**Lido integralmente (Parte 4):** `.github/workflows/build-windows.yml` (243 linhas), `.github/workflows/build-msix.yml` (144), `.github/dependabot.yml`, `scripts/*` (6 arquivos), `packaging/msix/Package.appxmanifest.template.xml`, `src-tauri/windows/hooks.nsh`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `rust-toolchain.toml`, `.gitignore`, `package.json`, `tsconfig.json`, `vite.config.ts`, `index.html`, `README.md`, `README-en.md`, `README.es.md`, `PRIVACY_POLICY.md` (seções PT/EN/ES: PT lida por inteiro, EN/ES conferidas por estrutura e datas), `SUPPORT.md`, `GUIA_PUBLICACAO_MICROSOFT_STORE.md`, `MICROSOFT_STORE_LISTING_DRAFT.md`, `MICROSOFT_STORE_READINESS.md`. Lockfiles (`package-lock.json`, `Cargo.lock`) foram **inspecionados por consulta programática** (versões e coerência), não lidos linha a linha.

**Lido de forma pontual (Partes 1–3, só para confirmar interfaces):** trechos de `main.rs`, `autostart.rs`, `config.rs`, `diagnostics.rs`, `window.rs`, `main.ts`, `settings.ts`, `settings.html`, `styles.css`, capabilities.

### Execução de comandos (seção 7 do prompt)

| Comando | Resultado |
|---|---|
| `npm run check:tauri-versions` | ✅ passou (`api 2.12.4`, `cli 2.12.4`, crate `tauri 2.12.5`, `tauri-build 2.7.4`) |
| `npm run check:acl` | ✅ passou |
| `npm run check:drag-overlay` | ✅ passou |
| `npm run check:contrast` | ✅ passou |
| `npm run check:msix-config` | ✅ passou |
| `npm ci` | ❌ **não concluiu**: sem acesso ao registry (`getaddrinfo ENOTFOUND registry.npmjs.org`; npm: "Exit handler never called"). `node_modules` ficou **incompleto** |
| `npm run typecheck` / `npm run build` | ⚠️ **inconclusivo — não executado de forma válida.** Os 7 erros TS (`TS2307` para `@tauri-apps/api/*` e `vite`, e `TS2345`/`TS7006` em `settings.ts:200/267/361/366`) são **consequência da instalação incompleta** (os tipos de `invoke`/`listen` viram `any`). **Não são achados.** |
| `npm audit` | ❌ não executado (sem rede) |
| `cargo fmt` / `cargo test` / `cargo clippy` | ❌ não executados: alvo `x86_64-pc-windows-msvc`, sem Windows nem dependências do crate (sem rede) |
| `cargo audit` | ❌ não executado (ferramenta ausente, sem rede) |
| Verificação dos SHAs das ações no GitHub | ❌ não possível (acesso GitHub não habilitado na sessão; não contornado) |
| Download de fontes de crates (p.ex. `tauri-plugin-autostart`) | ❌ bloqueado pelo proxy (403); não contornado |
| Validação sintática YAML (3 arquivos) | ✅ válidos |

**Não verificado por falta de Windows/rede:** existência real das versões futuras (`actions/*` v7/v8, `cargo-audit 0.22.2`, `setup-WinAppCli v0.2/v0.7.1`), flags do `winapp` CLI, comportamento do Partner Center, virtualização de `%APPDATA%` no MSIX, nome real da entrada `Run` do plugin.

---

## 3. Mapa das 4 partes (delimitações) e interfaces

| Nº | Parte | Arquivos | Status neste relatório |
|---|---|---|---|
| **1** | Núcleo Rust: ciclo de vida, configuração, comandos IPC e diagnóstico | `src-tauri/src/main.rs`, `config.rs`, `commands.rs`, `diagnostics.rs`, `autostart.rs`, `src-tauri/Cargo.toml`, `build.rs` | Não executada |
| **2** | Janela, navegação, login e script injetado | `src-tauri/src/window.rs` (1.768 linhas), `tray.rs`, `i18n.rs` | Não executada |
| **3** | Interface local (frontend), IPC e superfície de segurança | `index.html`, `settings.html`, `src/main.ts`, `src/settings.ts`, `src/styles.css`, `vite.config.ts`, `tsconfig.json`, `package.json` (scripts/deps de runtime), `src-tauri/tauri.conf.json` (seção `app`/`security`), `src-tauri/capabilities/*.json` | Não executada |
| **4** | **Build, CI/CD, empacotamento, dependências, documentação e distribuição** | `.github/workflows/*.yml`, `.github/dependabot.yml`, `scripts/*`, `packaging/msix/Package.appxmanifest.template.xml`, `src-tauri/windows/hooks.nsh`, `src-tauri/tauri.conf.json` (seção `bundle`), `src-tauri/Cargo.toml`, `Cargo.lock`, `package.json`/`package-lock.json`, `rust-toolchain.toml`, `.gitignore`, `README*.md`, `PRIVACY_POLICY.md`, `SUPPORT.md`, `GUIA_PUBLICACAO_MICROSOFT_STORE.md`, `MICROSOFT_STORE_*.md` | ✅ **Executada (esta)** |

**Arquivos compartilhados entre partes** (o prompt os lista em mais de uma): `Cargo.toml` (P1 e P4), `tauri.conf.json` (P3 = `app`/`security`; P4 = `bundle`), `package.json` (P3 = scripts/deps de runtime; P4 = completo). Aqui analisei apenas o recorte da Parte 4.

**Interfaces Parte 4 ↔ outras partes (pendentes para as respectivas auditorias):**

| Interface | Parte(s) | Ponto de contato |
|---|---|---|
| `check-acl.mjs` trava `remote.urls == ["http://*","https://*"]` | 3 | FLY-P4-004 — qualquer endurecimento da capability exige alterar o script junto |
| `check-drag-overlay.mjs` e `check-acl.mjs` leem `window.rs` (`INIT`, mensagem de log em PT) e `main.ts` por regex/`includes` | 2, 3 | FLY-P4-010 |
| `hooks.nsh` e docs referenciam `WidgetWebsite` (= `config.rs:204-207`) | 1 | FLY-P4-005, 012, 017 |
| Nome da entrada `Run` gerada por `tauri_plugin_autostart::init(...)` (`main.rs:131-134`) | 1 | FLY-P4-005 |
| `app_directory()` com fallback `current_dir()` (H8) e virtualização do `%APPDATA%` sob MSIX | 1 | FLY-P4-012 |
| Poda dos `config.json.bad-*` (`config.rs:239-260`) — **não verificado** se há limpeza | 1 | FLY-P4-012 |
| Features de `windows-sys`/`windows` declaradas em `Cargo.toml` × usos em `window.rs`/`diagnostics.rs`/`main.rs` | 1, 2 | Verificado: todas as 6 features de `windows-sys` têm uso (ver seção 9) |
| `npm run icons` necessário antes de `cargo test` | 1 | FLY-P4-014 |

---

## 4. Índice de achados

| ID | Título | Parte | Severidade | Confiança | Esforço | Depende de |
|---|---|---|---|---|---|---|
| FLY-P4-001 | `MinVersion` do MSIX incompatível com `uap10` | 4 | Média | Hipótese | P | — |
| FLY-P4-002 | Job `sign-and-publish` executa em `pull_request` com acesso a segredos | 4 | Média | Confirmado | P | — |
| FLY-P4-003 | Dependabot separa npm e cargo → PRs quebram `check:tauri-versions` (H9) | 4 | Média | Provável | P | — |
| FLY-P4-004 | `check:acl` fixa `http://*`/`https://*` na capability remota | 4 (3) | Média | Confirmado | P | Parte 3 (decisão) |
| FLY-P4-005 | Entrada `Run` do autostart pode ficar órfã após desinstalar (H14) | 4 (1) | Média | Hipótese | P | — |
| FLY-P4-006 | Procedimento de versão omite `package-lock.json`/`Cargo.lock` | 4 | Baixa | Confirmado | P | — |
| FLY-P4-007 | Assinatura: exe embutido sem assinatura; timestamp via HTTP | 4 | Baixa | Confirmado | M | — |
| FLY-P4-008 | CI frágil/lento: `cargo install cargo-audit` a cada execução e auditorias como gate de build | 4 | Média | Confirmado | M | — |
| FLY-P4-009 | CI: sem `timeout-minutes`, cache incompleto, trabalho duplicado | 4 | Baixa | Confirmado | P | FLY-P4-008 |
| FLY-P4-010 | Scripts `check-*.mjs` frágeis por regex/`includes` (H11) | 4 (2, 3) | Média | Confirmado | M | — |
| FLY-P4-011 | `build-msix.ps1`: `SHA256SUMS.txt` fora do formato, sem BOM, sem validação do Publisher | 4 | Baixa | Confirmado | P | — |
| FLY-P4-012 | Docs/Privacidade não cobrem canal MSIX e backups `config.json.bad-*` (H12 parcial) | 4 (1) | Média | Provável | M | — |
| FLY-P4-013 | READMEs divergentes entre idiomas e erros de formatação | 4 | Baixa | Confirmado | P | — |
| FLY-P4-014 | `src-tauri/icons/` ignorado: `cargo test` em clone limpo exige `npm run icons` (H10) | 4 (1) | Baixa | Provável | P | — |
| FLY-P4-015 | Dependência do runtime VC++ não verificada (NSIS e MSIX) | 4 | Média | Hipótese | M | — |
| FLY-P4-016 | Fixação de ações: comentários/versões inconsistentes e não verificáveis | 4 | Baixa | Hipótese | P | — |
| FLY-P4-017 | Nomes marca × interno: acoplamentos não protegidos (H14) | 4 (1) | Info | Confirmado | P | — |
| FLY-P4-018 | Pendências menores agregadas (licença, metadados, i18n do manifesto, marca em exemplo) | 4 | Baixa | Confirmado | P | — |

---

## 5. Achados detalhados — Parte 4

### FLY-P4-001 — `MinVersion` do MSIX incompatível com o namespace `uap10`

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Média (sobe para Alta se `winapp pack` ou o Partner Center recusarem o pacote) |
| Categoria | Build/CI |
| Confiança | Hipótese — depende da documentação do namespace `uap10` (introduzido no Windows 10 versão 2004, build 19041). **Não verificado nesta sessão (sem rede).** |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `packaging/msix/Package.appxmanifest.template.xml:6,19-21,25-28,42-43` |
| Depende de | — |

**Evidência**
```xml
xmlns:uap10="http://schemas.microsoft.com/appx/manifest/uap/windows10/10"
...
<uap10:PackageIntegrity><uap10:Content Enforcement="on" /></uap10:PackageIntegrity>
...
<TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.18362.0" MaxVersionTested="10.0.26100.0" />
...
uap10:TrustLevel="mediumIL"
uap10:RuntimeBehavior="packagedClassicApp">
```

**Descrição e causa raiz** — O manifesto declara suporte mínimo ao Windows 10 1903 (18362), mas usa elementos/atributos do contrato `uap10` (`PackageIntegrity`, `TrustLevel`, `RuntimeBehavior`) que, segundo a documentação do esquema, exigem Windows 10 2004 (10.0.19041.0). Manifestos que usam recursos acima do `MinVersion` costumam ser recusados por `MakeAppx`/validação do Store ou, pior, instalar e falhar em SOs antigos.

**Impacto / cenário** — `npm run build:msix` pode falhar no `winapp pack`, ou o Partner Center pode reprovar a validação do pacote. Nada disso é detectado hoje pelos checks (`check:msix-config` não compara versões).

**Solução proposta**
1. **Antes de editar**, o corretor deve confirmar na documentação oficial do manifesto (`uap10:RuntimeBehavior`/`PackageIntegrity`) a versão mínima exigida.
2. Se confirmado (19041), subir `MinVersion` para `10.0.19041.0`. Windows 10 1903/1909 estão fora de suporte; **decisão humana** só se o autor quiser manter esses SOs (nesse caso, remover os atributos `uap10`, o que muda o comportamento do pacote — não recomendado).
3. Atualizar docs: README (3 idiomas, "Windows 10/11 x64") para "Windows 10 versão 2004 (19041) ou superior" no canal Store.
4. Adicionar a verificação em `scripts/check-msix-config.mjs`.
```diff
-      MinVersion="10.0.18362.0"
+      MinVersion="10.0.19041.0"
```
```diff
 // scripts/check-msix-config.mjs (antes do bloco final `if (process.exitCode)`)
+const minVersion = /MinVersion="(\d+)\.(\d+)\.(\d+)\.(\d+)"/.exec(manifest)?.slice(1).map(Number);
+const compare = (a, b) => { for (let i = 0; i < 4; i += 1) { if (a[i] !== b[i]) return a[i] - b[i]; } return 0; };
+if (!minVersion) fail("O manifesto precisa declarar MinVersion em TargetDeviceFamily.");
+else if (manifest.includes("uap10:") && compare(minVersion, [10, 0, 19041, 0]) < 0) {
+  fail("O manifesto usa uap10:*, que exige MinVersion >= 10.0.19041.0.");
+}
```

**Arquivos a alterar** — `packaging/msix/Package.appxmanifest.template.xml`, `scripts/check-msix-config.mjs`, `README.md`, `README-en.md`, `README.es.md`.

**Critérios de aceite** — `npm run check:msix-config` passa com 19041 e **falha** se `MinVersion` for reduzido; `winapp pack` (Windows) gera o `.msix`; upload no Partner Center sem erro de manifesto.

**Como verificar** — Em Windows: `npm run build:msix` com identidade de teste; inspecionar `AppxManifest.xml` dentro do `.msix`. Teste negativo: reduzir `MinVersion` temporariamente e ver `check:msix-config` falhar.

**Cuidados** — Não alterar `Identity`/placeholders. Manter `MaxVersionTested`.

---

### FLY-P4-002 — `sign-and-publish` executa em `pull_request` com acesso a segredos de assinatura

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Média |
| Categoria | Segurança |
| Confiança | Confirmado (configuração); impacto depende de quem tem permissão de escrita no repositório |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `.github/workflows/build-windows.yml:23-39` (trigger `pull_request`), `:184-196` |
| Depende de | — |

**Evidência**
```yaml
on:
  pull_request:
    branches: [main]
...
  sign-and-publish:
    needs: build
    runs-on: windows-2025
    steps:
      ...
        env:
          WINDOWS_PFX_BASE64: ${{ secrets.WINDOWS_PFX_BASE64 }}
          WINDOWS_PFX_PASSWORD: ${{ secrets.WINDOWS_PFX_PASSWORD }}
```

**Descrição e causa raiz** — O job que injeta os segredos não tem condição de evento. Em PRs originados de **branches do mesmo repositório**, o workflow executado é o do commit do PR e os segredos estão disponíveis; quem puder abrir um PR pode editar o passo de assinatura e exfiltrar o PFX/senha. (PRs de forks e do Dependabot não recebem esses segredos, então caem no caminho "sem assinatura".)

**Impacto / cenário** — Projeto individual: risco baixo hoje, mas o certificado (quando existir) é o ativo mais sensível do repositório e a correção é trivial.

**Solução proposta** (recomendada: condição de evento + ambiente protegido; alternativa descartada: remover o `pull_request` — perde a validação de PRs)
```diff
   sign-and-publish:
     needs: build
+    if: github.event_name != 'pull_request'
     runs-on: windows-2025
+    timeout-minutes: 20
+    permissions:
+      contents: read
+    # Opcional (recomendado quando houver certificado): mover os segredos para um Environment com aprovação.
+    # environment: release-signing
     steps:
```

**Arquivos a alterar** — `.github/workflows/build-windows.yml`.

**Critérios de aceite** — Em PR, o job `sign-and-publish` aparece como *skipped*; em push para `main` e `workflow_dispatch` ele roda normalmente; o artefato `widget-website-build` (não assinado) continua sendo produzido pelo job `build` em PRs.

**Como verificar** — Abrir PR de teste e conferir a aba Actions; rodar `workflow_dispatch` e confirmar o artefato `widget-website-windows-x64`.

**Cuidados** — Se for adotado `environment:`, criar o ambiente no GitHub e migrar os segredos (ação humana). Não mexer na lógica de limpeza do PFX (linhas 227-234).

---

### FLY-P4-003 — Dependabot separa npm e cargo → PRs que quebram `check:tauri-versions` (H9)

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Média |
| Categoria | Build/CI · Dependência |
| Confiança | Provável (configuração confirmada; que a divergência de minor quebre o build do Tauri é comportamento conhecido do CLI, **não verificado** aqui) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `.github/dependabot.yml:3-20`; `scripts/check-tauri-versions.mjs:21-26` |
| Depende de | — |

**Evidência**
```yaml
- package-ecosystem: npm
  groups: { npm-minor-and-patch: { update-types: [minor, patch] } }
- package-ecosystem: cargo
  directory: /src-tauri
  groups: { cargo-minor-and-patch: { update-types: [minor, patch] } }
```
```js
const minors = [majorMinor(npmVersions.api), majorMinor(npmVersions.cli), majorMinor(rustTauri)];
if (new Set(minors).size !== 1) throw new Error("Tauri JS API, CLI and Rust crate are not on the same major/minor: ...");
```

**Descrição e causa raiz** — O Dependabot não cruza ecossistemas: quando sai Tauri 2.13, abre um PR npm (`@tauri-apps/api`+`cli`) e outro cargo (`tauri`). Cada PR isolado deixa os minors divergentes e **falha** em `check:tauri-versions`; nenhum dos dois pode ser mesclado sozinho.

**Solução proposta** — Impedir atualizações *minor/major* automáticas dos pacotes Tauri (patches continuam, pois o check compara só `major.minor`) e documentar o procedimento manual conjunto.
```diff
 - package-ecosystem: npm
   directory: /
   schedule:
     interval: weekly
   groups:
     npm-minor-and-patch:
       update-types:
         - minor
         - patch
+  ignore:
+    - dependency-name: "@tauri-apps/*"
+      update-types: ["version-update:semver-minor", "version-update:semver-major"]
 - package-ecosystem: cargo
   directory: /src-tauri
   ...
+  ignore:
+    - dependency-name: "tauri"
+      update-types: ["version-update:semver-minor", "version-update:semver-major"]
+    - dependency-name: "tauri-build"
+      update-types: ["version-update:semver-minor", "version-update:semver-major"]
+    - dependency-name: "tauri-plugin-*"
+      update-types: ["version-update:semver-minor", "version-update:semver-major"]
```
(O bloco `ignore:` fica no mesmo nível de `groups:` dentro de cada item de `updates`.) Acrescentar ao `GUIA_PUBLICACAO_MICROSOFT_STORE.md` ou a um `CONTRIBUTING`/README técnico a rotina: `npm i @tauri-apps/api@X @tauri-apps/cli@X && cargo update -p tauri -p tauri-build --manifest-path src-tauri/Cargo.toml && npm run check:tauri-versions`.

*Alternativa descartada:* afrouxar o check (aceitar minors diferentes) — remove a proteção contra o erro de "pacotes Tauri com versões divergentes" do CLI.

**Arquivos a alterar** — `.github/dependabot.yml`, documentação de procedimento.

**Critérios de aceite** — O Dependabot (Insights → Dependency graph → Dependabot) não propõe mais PRs minor/major para os pacotes Tauri; PRs de patch continuam passando em `check:tauri-versions`.

**Como verificar** — Validar o YAML (`python -c "import yaml;yaml.safe_load(open('.github/dependabot.yml'))"`) e conferir o log do Dependabot após a primeira execução.

**Cuidados** — `plugins` oficiais (`tauri-plugin-*`) têm numeração própria; ignorar só minor/major evita saltos, mas lembrar de atualizá-los manualmente junto com o Tauri.

---

### FLY-P4-004 — `check:acl` fixa `http://*`/`https://*` na capability remota

| Campo | Valor |
|---|---|
| Parte | 4 (interface com Parte 3) |
| Severidade | Média |
| Categoria | Segurança · Testes |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `scripts/check-acl.mjs:29`; `src-tauri/capabilities/main-remote-heartbeat.json:8-13` |
| Depende de | Decisão da Parte 3 (H2) |

**Evidência**
```js
if (JSON.stringify(remoteCapability.remote?.urls) !== JSON.stringify(["http://*", "https://*"]) || ... ) throw new Error("Remote sites may invoke only the dedicated heartbeat command using HTTP(S) URL patterns.");
```

**Descrição e causa raiz** — O check exige **exatamente** os dois curingas. Qualquer redução de superfície (por exemplo, restringir ao host configurado, ou só `https://*`) **quebra o CI**, desincentivando o endurecimento apontado na hipótese H2 da Parte 3. O check protege o conjunto de *permissões* (correto), mas também "congela" o escopo de URLs (indesejado).

**Solução proposta** — Substituir a igualdade exata por asserções semânticas (permissões e janela fixas; URLs: ao menos um padrão http(s) válido).
```diff
-if (JSON.stringify(remoteCapability.remote?.urls) !== JSON.stringify(["http://*", "https://*"]) || JSON.stringify(remoteCapability.permissions) !== JSON.stringify(["allow-heartbeat"]) || remoteCapability.permissions.some((p) => p !== "allow-heartbeat")) throw new Error("Remote sites may invoke only the dedicated heartbeat command using HTTP(S) URL patterns.");
+const remoteUrls = remoteCapability.remote?.urls;
+if (!Array.isArray(remoteUrls) || remoteUrls.length === 0 || remoteUrls.some((u) => typeof u !== "string" || !/^https?:\/\/\S+$/.test(u))) throw new Error("A capability remota deve declarar ao menos um padrão http(s) válido.");
+if (JSON.stringify(remoteCapability.windows) !== JSON.stringify(["main"])) throw new Error("A capability remota deve valer somente para a janela main.");
+if (JSON.stringify(remoteCapability.permissions) !== JSON.stringify(["allow-heartbeat"])) throw new Error("Sites remotos só podem invocar o comando heartbeat.");
```
Aplicar **junto** com a decisão da Parte 3 sobre o escopo de URLs (esta mudança, sozinha, **não** altera a capability).

**Arquivos a alterar** — `scripts/check-acl.mjs`.

**Critérios de aceite** — `npm run check:acl` passa com a capability atual **e** com uma versão reduzida (por exemplo, `["https://*"]`); falha se `permissions` incluir qualquer coisa além de `allow-heartbeat` ou se `windows` ≠ `["main"]`.

**Como verificar** — Testes com `node --test` (ver seção 8) usando fixtures de capability.

**Cuidados** — Não relaxar as demais asserções (comandos × `build.rs` × `generate_handler!`).

---

### FLY-P4-005 — Entrada `Run` do autostart pode ficar órfã após desinstalar (H14)

| Campo | Valor |
|---|---|
| Parte | 4 (interface com Parte 1) |
| Severidade | Média |
| Categoria | Corretude · Documentação |
| Confiança | Hipótese — o nome do valor `Run` é definido pelo plugin `tauri-plugin-autostart` (usa `package_info().name`); **não foi possível ler o código-fonte do plugin** (rede bloqueada). Há indício de que seja o `productName` ("Floatly"), não o nome do crate. |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `src-tauri/windows/hooks.nsh:2-3,12-13`; `src-tauri/src/main.rs:131-134`; README (3 idiomas, seção "Desinstalação e dados") |
| Depende de | — |

**Evidência**
```nsis
; Tauri uses Cargo package name "widget-website" for the default autostart entry.
...
DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "widget-website"
DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Widget Website"
```
```rust
.plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
```

**Descrição e causa raiz** — O hook assume que a entrada se chama `widget-website` (nome do crate). Se o plugin usar o `productName` (`Floatly`), o desinstalador **não** remove a entrada correta; o Windows passa a mostrar um item de inicialização quebrado, e o README afirma o contrário ("A entrada de início automático do Windows é removida sempre que o aplicativo é desinstalado").

**Impacto / cenário** — Usuário NSIS ativa "Iniciar com o Windows", desinstala; o item `Floatly` permanece em `HKCU\...\Run` apontando para um `.exe` inexistente.

**Solução proposta**
1. **Verificar primeiro** (Windows): instalar o NSIS, ativar "Iniciar com o Windows" nas Configurações e executar `reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run"`; anotar o nome real do valor.
2. Remover **todos** os nomes plausíveis (manter os antigos por migração):
```diff
-    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "widget-website"
-    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Widget Website"
+    ; O plugin de autostart nomeia a entrada conforme o app; remover o nome atual e os legados.
+    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Floatly"
+    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "widget-website"
+    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Widget Website"
```
3. Corrigir o comentário do hook com o nome confirmado no passo 1 (em pt-BR).

**Arquivos a alterar** — `src-tauri/windows/hooks.nsh`; (se o nome divergir do documentado) READMEs.

**Critérios de aceite** — Após instalar → ativar autostart → desinstalar (sem `UpdateMode`), `reg query` não lista mais nenhuma entrada do Floatly; em **atualização** (`UpdateMode = 1`) a entrada é preservada.

**Como verificar** — Procedimento manual acima em VM Windows (não automatizável no CI atual).

**Cuidados** — Não remover entradas quando `$UpdateMode = 1`. O hook `Floatly` só afeta o próprio usuário (`HKCU`). *Opcional:* só apagar `Floatly` se o dado apontar para `widget-website.exe` (exige `StrStr` no NSIS) — não necessário para o objetivo.

---

### FLY-P4-006 — Procedimento de versão omite `package-lock.json` e `Cargo.lock`

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Baixa |
| Categoria | Documentação · Build/CI |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `GUIA_PUBLICACAO_MICROSOFT_STORE.md:164-167`; `scripts/check-tauri-versions.mjs:5-10`; `scripts/check-msix-config.mjs:26` |
| Depende de | — |

**Evidência** — Guia: *"Atualize a versão de forma consistente em package.json, src-tauri/tauri.conf.json e src-tauri/Cargo.toml."* O check compara `package.json`, `package-lock.json` (raiz `.version`), `Cargo.toml` e `tauri.conf.json`, mas **não** `Cargo.lock` (`widget-website`, hoje `1.0.0` em `Cargo.lock:4420-4421`) nem `packages[""]` do lock npm.

**Descrição e causa raiz** — Ao subir a versão, `Cargo.lock` precisa mudar também; como todos os comandos usam `--locked`, o build falha com erro pouco óbvio. Além disso, `check-msix-config.mjs:26` usa `cargo.includes('version = "1.0.0"')`, que casa com **qualquer** linha contendo essa substring (p.ex. uma dependência futura `version = "1.0.0"`) — falso negativo.

**Solução proposta**
1. Atualizar o guia (passo 10): listar `package-lock.json` (`npm version <v> --no-git-tag-version` já cobre `package.json` e lock), `Cargo.toml`, `tauri.conf.json` e `Cargo.lock` (`cargo update -w --manifest-path src-tauri/Cargo.toml` ou `cargo check`).
2. Fortalecer os checks:
```diff
 // scripts/check-tauri-versions.mjs
-const packageCargoVersion = /^version = "([^"]+)"/m.exec(cargoToml)?.[1];
+const packageStart = cargoToml.indexOf("[package]");
+const packageEnd = cargoToml.indexOf("\n[", packageStart + 1);
+const packageCargoVersion = /^version = "([^"]+)"/m.exec(cargoToml.slice(packageStart, packageEnd < 0 ? undefined : packageEnd))?.[1];
...
+// após a definição de cargoLockVersion():
+const selfVersions = [packageLock.packages?.[""]?.version, cargoLockVersion("widget-website")];
+if (selfVersions.some((v) => v !== packageJson.version)) {
+  throw new Error("package-lock.json (packages[\"\"]) ou Cargo.lock com versão diferente de package.json: " + JSON.stringify({ esperado: packageJson.version, encontrado: selfVersions }));
+}
```
```diff
 // scripts/check-msix-config.mjs:26 — usar a mesma extração da seção [package]
-if (!cargo.includes(`version = "${tauri.version}"`)) fail("A versão do pacote Rust deve corresponder à versão do Tauri.");
+const pkgStart = cargo.indexOf("[package]");
+const pkgEnd = cargo.indexOf("\n[", pkgStart + 1);
+const cargoVersion = /^version = "([^"]+)"/m.exec(cargo.slice(pkgStart, pkgEnd < 0 ? undefined : pkgEnd))?.[1];
+if (cargoVersion !== tauri.version) fail("A versão do pacote Rust deve corresponder à versão do Tauri.");
```

**Arquivos a alterar** — `GUIA_PUBLICACAO_MICROSOFT_STORE.md`, `scripts/check-tauri-versions.mjs`, `scripts/check-msix-config.mjs`.

**Critérios de aceite** — `npm run check:tauri-versions` e `check:msix-config` passam hoje; falham se `Cargo.lock` ou `packages[""]` divergirem (teste com fixture).

**Como verificar** — `node --test` (seção 8) com fixtures alterando só `Cargo.lock`.

**Cuidados** — Manter a função `cargoLockVersion` como está (não depende de ordem dos pacotes).

---

### FLY-P4-007 — Assinatura: executável embutido sem assinatura e timestamp via HTTP

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Baixa |
| Categoria | Segurança · Build/CI |
| Confiança | Confirmado (e já **documentado** no README) |
| Esforço | M |
| Risco de regressão | Médio |
| Local | `build-windows.yml:112-115` (empacotamento NSIS), `:220-221` (`signtool … /tr http://timestamp.digicert.com`); `README.md:63` |
| Depende de | — |

**Evidência**
```powershell
& $signtool.FullName sign /s My /sha1 $thumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $file
```

**Descrição e causa raiz** — (1) O instalador e o `.exe` publicado separadamente são assinados *depois* do `tauri build`; o `.exe` **dentro** do instalador (o que realmente roda após instalar) permanece sem assinatura, o que pode manter reputação/SmartScreen e falsos positivos de antivírus no binário instalado. (2) O timestamp usa HTTP; o protocolo RFC 3161 é autenticado pela assinatura da própria resposta, então o risco é baixo, mas HTTPS é preferível quando disponível.

**Solução proposta**
- **Decisão humana** (a arquitetura atual evita expor o certificado ao job de build, escolha deliberada): *(a)* manter e continuar documentando (recomendado enquanto não houver certificado); *(b)* quando houver certificado, assinar o `.exe` **antes** do NSIS via `bundle.windows.signCommand` do Tauri, mantendo o PFX em ambiente protegido (ver FLY-P4-002).
- Timestamp: trocar para a URL HTTPS **somente após confirmar** que o servidor da DigiCert a oferece; senão manter HTTP.

**Arquivos a alterar** — `build-windows.yml` (e `tauri.conf.json` `bundle.windows` se opção *b*), `README*.md`.

**Critérios de aceite** — Opção *b*: `signtool verify /pa` passa no `.exe` extraído do instalador.

**Como verificar** — Instalar o NSIS assinado, `Get-AuthenticodeSignature "$env:LOCALAPPDATA\Floatly\widget-website.exe"`.

**Cuidados** — Não registrar o PFX no repositório; manter a limpeza do certificado no `finally`.

---

### FLY-P4-008 — CI frágil/lento: `cargo install cargo-audit` a cada execução e auditorias como gate de build

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Média |
| Categoria | Build/CI |
| Confiança | Confirmado |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | `build-windows.yml:88-89,96-102`; `build-msix.yml:107-108,113-119` |
| Depende de | — |

**Evidência**
```yaml
- name: Audit npm dependencies
  run: npm audit --audit-level=high
- name: Install and run RustSec audit
  run: |
    cargo install cargo-audit --version 0.22.2 --locked
    cargo audit --file src-tauri/Cargo.lock
```

**Descrição e causa raiz** — (1) `cargo install` **compila** o `cargo-audit` do zero em cada job (minutos, e depende do crates.io estar disponível) em dois workflows que rodam no mesmo push. (2) `npm audit`/`cargo audit` como etapas **bloqueantes** fazem o build e a geração do MSIX falharem por um advisory novo **sem relação com a mudança** — inclusive quando só se corrige um typo no código. (3) A versão `0.22.2` não foi verificada (sem rede).

**Solução proposta** (sem desativar nenhuma verificação — regra 8.4)
1. Cachear o binário para evitar recompilar:
```diff
+      - name: Cache cargo-audit
+        id: cache-cargo-audit
+        uses: actions/cache@3edfce9056124e459a23f683a21433670d47daca # v6.2.0 (manter o SHA atual)
+        with:
+          path: ~/.cargo/bin/cargo-audit.exe
+          key: cargo-audit-0.22.2-${{ runner.os }}
       - name: Install and run RustSec audit
         shell: pwsh
         run: |
-          cargo install cargo-audit --version 0.22.2 --locked
-          if ($LASTEXITCODE -ne 0) { throw "Could not install cargo-audit." }
+          if ("${{ steps.cache-cargo-audit.outputs.cache-hit }}" -ne "true") {
+            cargo install cargo-audit --version 0.22.2 --locked
+            if ($LASTEXITCODE -ne 0) { throw "Could not install cargo-audit." }
+          }
           cargo audit --file src-tauri/Cargo.lock
```
2. **Decisão humana:** adicionar `.github/workflows/audit.yml` agendado (semanal + `workflow_dispatch`) para detectar advisories novos sem relação com commits; manter as etapas de auditoria nos workflows de build (ou movê-las só para os caminhos que alteram lockfiles). Recomendação: manter no build **e** agendar.

**Arquivos a alterar** — `build-windows.yml`, `build-msix.yml`, (novo) `.github/workflows/audit.yml`.

**Critérios de aceite** — Segunda execução do workflow usa o cache (sem `cargo install`); o job de auditoria agendado aparece em Actions.

**Como verificar** — Rodar o workflow duas vezes e comparar o tempo do passo.

**Cuidados** — Qualquer nova ação (por exemplo, para instalar ferramentas) deve ser fixada por SHA **verificado** pelo corretor.

---

### FLY-P4-009 — CI: sem `timeout-minutes`, cache incompleto e trabalho duplicado

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Baixa |
| Categoria | Build/CI · Desempenho |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `build-windows.yml:44,64-70,184`; `build-msix.yml:47,74-80,85-86`; `scripts/build-msix.ps1:64` |
| Depende de | FLY-P4-008 |

**Evidência** — Nenhum job define `timeout-minutes` (padrão 360 min). O cache cobre apenas `~/.cargo/registry` e `~/.cargo/git` (compilação do Tauri/`windows` a cada execução), com chaves **diferentes** em cada workflow (`windows-cargo-…` e `floatly-msix-cargo-…`) e **sem** `restore-keys`. Ambos os workflows disparam no mesmo push para `main` e repetem os mesmos checks; `npm run icons` roda duas vezes no fluxo MSIX (`build-msix.yml:85-86` e `build-msix.ps1:64`).

**Solução proposta**
```diff
 jobs:
   build:
     runs-on: windows-2025
+    timeout-minutes: 60
 ...
-          key: windows-cargo-${{ runner.os }}-${{ hashFiles('src-tauri/Cargo.lock') }}
+          key: cargo-${{ runner.os }}-${{ hashFiles('src-tauri/Cargo.lock') }}
+          restore-keys: |
+            cargo-${{ runner.os }}-
```
(idem em `build-msix.yml`, com **a mesma** chave para compartilhar o cache). Remover o passo redundante `Generate Windows icons required by Tauri tests` **apenas se** a ordem dos passos continuar garantindo que os ícones existam antes de `cargo test` (hoje são necessários ali; o script os regenera depois — manter se houver dúvida). *Opcional (decisão humana):* extrair os checks comuns para um workflow reutilizável.

**Arquivos a alterar** — `build-windows.yml`, `build-msix.yml`.

**Critérios de aceite** — Jobs com `timeout-minutes`; cache restaurado entre os dois workflows (log "Cache restored from key").

**Como verificar** — Duas execuções consecutivas e comparação de duração.

**Cuidados** — Não cachear `target/` sem avaliar o tamanho (limite de cache do GitHub).

---

### FLY-P4-010 — Scripts `check-*.mjs` frágeis por regex/`includes` (H11)

| Campo | Valor |
|---|---|
| Parte | 4 (interface com Partes 2 e 3) |
| Severidade | Média |
| Categoria | Testes · Manutenibilidade |
| Confiança | Confirmado |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | ver tabela |
| Depende de | — |

| Script:linha | Fragilidade | Efeito |
|---|---|---|
| `check-acl.mjs:32` | `startupSource.includes('let target = "";')` | Reformatação (`let target: string = ""`, aspas simples) → falso erro |
| `check-acl.mjs:33` | `settingsSource.includes("opacity")` em **todo** o arquivo (inclui comentários/strings) | Qualquer palavra "opacity" (p.ex. num comentário ou CSS inline) quebra o CI |
| `check-acl.mjs:14-16,17-19` | `generate_handler!\s*\[([\s\S]*?)\]` pega até o **primeiro** `]` | Atributos/arrays dentro da lista cortam a captura → falso negativo |
| `check-drag-overlay.mjs:4` | `const INIT: &str = r#"(.*?)"#;` (lazy até o primeiro `"#;`) | Um `"#;` dentro do JS injetado trunca o script analisado |
| `check-drag-overlay.mjs:14-36` | ~20 `includes()` de trechos literais do JS minificado e da **mensagem de log em português** (`:36`) | Renomear variável/mensagem quebra o CI; passar não prova comportamento |
| `check-tauri-versions.mjs:2-5` | `readFileSync("package.json")` relativo ao **cwd** (os demais usam `import.meta.url` ou `process.cwd()` de forma diferente) | Falha se executado fora da raiz |
| `check-tauri-versions.mjs:6` | `/^version = "…"/m` no `Cargo.toml` inteiro | Pode capturar `version =` de uma tabela `[dependencies.x]` |
| `check-theme-contrast.mjs:4-6` | Regex do seletor **sem âncora** (`.settings-page\s*\{`) e só cores hex | Pode casar `body.settings-page {`/regra aninhada; tokens `rgb()/hsl()/color-mix()` falham (ao menos de forma explícita) |
| `check-msix-config.mjs:26` | `cargo.includes('version = "1.0.0"')` | Ver FLY-P4-006 |

**Descrição e causa raiz** — Os checks verificam **texto**, não **comportamento**; servem como "smoke tests", mas acoplam o CI à formatação do código das Partes 1–3.

**Solução proposta** (incremental, sem reescrita)
1. Criar `scripts/lib.mjs` com `repoFile(relPath)` baseado em `import.meta.url` e `stripComments(source)` (remove `//…`, `/*…*/`); usar em todos os scripts.
2. `check-acl.mjs:32-33`: testar por regex tolerante e **sem comentários**:
```diff
-if (!startupSource.includes('let target = "";' ) || startupSource.includes("investing.com")) throw ...
+const startupCode = stripComments(startupSource);
+if (!/let\s+target\s*(?::\s*string)?\s*=\s*(["'`])\1/.test(startupCode) || /investing\.com/i.test(startupCode)) throw ...
```
```diff
-if (configSource.includes("pub opacity: u8") || settingsSource.includes("opacity") || ...
+const settingsCode = stripComments(settingsSource);
+if (/pub\s+opacity\s*:/.test(configSource) || /\bopacity\b/.test(settingsCode) || ...
```
3. `check-tauri-versions.mjs`: usar `repoFile()` e a extração da seção `[package]` (FLY-P4-006).
4. `check-theme-contrast.mjs`: ancorar o seletor: `new RegExp("(?:^|[}\\s])" + escaped + "\\s*\\{([^}]+)\\}")` e ignorar tokens não-hex com erro claro (já falha com "Missing … token").
5. `check-drag-overlay.mjs`: manter, mas **migrar a asserção da linha 36** para um teste Rust (`#[cfg(test)]` em `window.rs`) que valide o bloqueio por travamento — **dependência da Parte 2** (registrar como interface); até lá, apontar a mensagem em uma `const` compartilhada.
6. Adicionar testes dos próprios checks (seção 8).

**Arquivos a alterar** — `scripts/*.mjs`, novo `scripts/lib.mjs`.

**Critérios de aceite** — Os 5 checks continuam passando; reformatar `main.ts` (aspas/tipo) não quebra `check:acl`; um teste negativo prova que cada check falha quando deveria.

**Como verificar** — `node --test scripts/` (seção 8).

**Cuidados** — Não enfraquecer o que cada check protege; não remover checks.

---

### FLY-P4-011 — `build-msix.ps1`: `SHA256SUMS.txt` fora do formato, sem BOM, sem validação do Publisher

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Baixa |
| Categoria | Build/CI · Manutenibilidade |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `scripts/build-msix.ps1:119-127,32-34`; `package.json:18`; `build-msix.yml:140-143`; `GUIA_PUBLICACAO_MICROSOFT_STORE.md:47-49` |
| Depende de | — |

**Evidência**
```powershell
@("Floatly MSIX package","Version: $packageVersion",...,"SHA256: $hash","File: $outputFileName") | Set-Content -Path (Join-Path $outputRoot "SHA256SUMS.txt") -Encoding UTF8
```
O arquivo é UTF-8 **sem BOM** com caracteres acentuados (`à á ã ç é ê í ó`, verificado). `package.json:18` executa `powershell` (Windows PowerShell 5.1, que lê arquivos sem BOM como ANSI): as mensagens em português saem com mojibake localmente (não quebra a análise sintática: nenhum caractere gera aspas tipográficas); o CI usa `pwsh` e não é afetado.

**Descrição e causa raiz** — (1) `SHA256SUMS.txt` não está no formato `HASH *arquivo`, então `sha256sum -c`/`Get-FileHash` comparativos não funcionam; o nome é enganoso. (2) Encoding sem BOM. (3) `PackagePublisher` não é validado (`^CN=`), só o `Name`.

**Solução proposta**
```diff
-    @(
-        "Floatly MSIX package", ...
-    ) | Set-Content -Path (Join-Path $outputRoot "SHA256SUMS.txt") -Encoding UTF8
+    "$($hash.ToLower()) *$outputFileName" | Set-Content -Path (Join-Path $outputRoot "SHA256SUMS.txt") -Encoding ascii
+    @("Floatly MSIX package","Version: $packageVersion","Architecture: x64","Package identity: $PackageIdentityName","SHA256: $hash","File: $outputFileName") |
+        Set-Content -Path (Join-Path $outputRoot "BUILD-INFO.txt") -Encoding UTF8
```
```diff
+if ($PackagePublisher -notmatch "^CN=.+") {
+    Stop-Build "PackagePublisher deve ser o valor X.500 do Partner Center (começa com CN=)."
+}
```
- Salvar `build-msix.ps1` como **UTF-8 com BOM** (ou manter ASCII nas mensagens).
- `build-msix.yml:140-143`: incluir `dist/msix/BUILD-INFO.txt` no `path:` do upload.
- `GUIA…:47-49`: descrever os dois arquivos.

**Arquivos a alterar** — `scripts/build-msix.ps1`, `.github/workflows/build-msix.yml`, `GUIA_PUBLICACAO_MICROSOFT_STORE.md`.

**Critérios de aceite** — `sha256sum -c SHA256SUMS.txt` (ou equivalente PowerShell) valida o `.msix`; `npm run build:msix` imprime acentos corretamente no PowerShell 5.1.

**Como verificar** — Execução em Windows; o passo "Validate unsigned MSIX packaging" do `build-windows.yml` continua passando.

**Cuidados** — O workflow `build-windows.yml:173` usa `Get-FileHash` próprio; não depende do arquivo.

---

### FLY-P4-012 — Docs/Privacidade não cobrem canal MSIX e backups `config.json.bad-*` (H12 parcial)

| Campo | Valor |
|---|---|
| Parte | 4 (interface com Parte 1) |
| Severidade | Média |
| Categoria | Documentação (conformidade com a Store) |
| Confiança | Provável — a virtualização de escrita em `%APPDATA%` para apps empacotados *full trust* é comportamento do MSIX **não verificado aqui**; o backup `config.json.bad-*` é Confirmado (`config.rs:239-260`) |
| Esforço | M |
| Risco de regressão | Baixo |
| Local | `PRIVACY_POLICY.md` seções 3.1/3.2/6/7 (PT: linhas 41, 55, 67, 116-118; EN/ES equivalentes); `SUPPORT.md:26,68,108`; `README*.md` (Desinstalação/Idioma); `README.es.md:94`; `MICROSOFT_STORE_READINESS.md:30-36` |
| Depende de | — |

**Evidência**
```text
PRIVACY_POLICY.md:67  A pasta de dados do WebView2 fica normalmente em %APPDATA%\WidgetWebsite\webview-data.
PRIVACY_POLICY.md:117 Remover todos os dados do aplicativo: ao desinstalar, selecionar a opção do instalador ...
README.es.md:94       Si el archivo de configuración está dañado, se reemplaza por valores predeterminados.
```
```rust
// config.rs:239-244 — o arquivo inválido é preservado como backup
let backup = dir.join(format!("config.json.bad-{timestamp}-{}-{backup_counter}", ...));
match fs::rename(&path, &backup) {
```

**Descrição e causa raiz**
1. **Canal MSIX:** o pacote usa `packagedClassicApp` (`Package.appxmanifest.template.xml:42-43`) e `app_directory()` grava em `%APPDATA%\WidgetWebsite` (`config.rs:204-207`). Em apps empacotados, escritas em `AppData` costumam ser redirecionadas para a pasta privada do pacote (`%LOCALAPPDATA%\Packages\<PFN>\LocalCache\Roaming\…`); o usuário **não** encontrará `%APPDATA%\WidgetWebsite` no Explorer, e a exclusão ocorre ao desinstalar pela Store. A Política, o SUPPORT e os READMEs só descrevem a opção do instalador NSIS e caminhos "normais".
2. **Backups de configuração corrompida:** `config.json.bad-*` guardam a configuração anterior (URL do site e preferências) e não são citados na Política (item de "dados locais") nem em "Remover manualmente os dados"; o README.es diz apenas que o arquivo "se reemplaza". **Não verificado** (Parte 1) se esses backups são podados.
3. **Divergências de diagnóstico:** `SUPPORT.md` orienta enviar `diagnostics.log` por caminho `%APPDATA%…` (mesmo problema no MSIX). (As alegações de **sanitização de URL**, **rotação** e **ausência de telemetria** da Política *conferem* com o código — ver seção 9.)

**Solução proposta**
1. Validar em Windows: instalar o MSIX, abrir/fechar o app e localizar os dados (`Get-ChildItem "$env:LOCALAPPDATA\Packages" -Filter "*Floatly*"`; `…\LocalCache\Roaming\WidgetWebsite`). Registrar o resultado no `MICROSOFT_STORE_READINESS.md` (item de validação).
2. Se confirmado, acrescentar (nos 3 idiomas) à Política (seção 3.1/6/7), SUPPORT e READMEs algo como (PT-BR):
> *"Na versão instalada pela Microsoft Store (MSIX), o Windows pode redirecionar a pasta de dados para a área privada do pacote (`%LOCALAPPDATA%\Packages\<nome do pacote>\LocalCache\Roaming\WidgetWebsite`). Nesse caso, os dados são removidos ao desinstalar o aplicativo pelo Windows (Configurações → Aplicativos), sem necessidade de limpeza manual."*
3. Mencionar na Política que, se o arquivo de configuração for inválido, uma cópia (`config.json.bad-*`) é mantida na pasta de dados; ajustar `README.es.md:94` ("se reemplaza por valores predeterminados y se conserva una copia del archivo original").
4. Atualizar a data "Última atualização" nas 3 versões da Política.

**Arquivos a alterar** — `PRIVACY_POLICY.md` (PT/EN/ES), `SUPPORT.md` (PT/EN/ES), `README.md`, `README-en.md`, `README.es.md`, `MICROSOFT_STORE_READINESS.md`.

**Critérios de aceite** — Os três idiomas descrevem o canal MSIX e os backups; a Política continua afirmando apenas o que o código faz.

**Como verificar** — Revisão cruzada com `config.rs` e `diagnostics.rs`; teste manual descrito.

**Cuidados** — Política é documento legal: **não** alterar o nome do controlador/contato; manter os 3 idiomas sincronizados.

---

### FLY-P4-013 — READMEs divergentes entre idiomas e erros de formatação

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Baixa |
| Categoria | Documentação · i18n |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `README.md:3,54,77`; `README-en.md:3,58,81`; `README.es.md:3,79-99,103` |
| Depende de | — |

**Evidência / problemas**
1. **Nomes de arquivo inconsistentes:** `README.md` (PT), `README-en.md`, `README.es.md`.
2. **Conteúdo divergente:** apenas o `README.es.md` contém "Diagnóstico de bloqueos", "Privacidad y soporte", "Privacidad y limitaciones" e "Estructura"; PT e EN **não** têm essas seções. O `README.es.md:99` inclui uma **lista de pendências de teste interno** ("Es necesario probar… escalas del 125 %/150 %…") — conteúdo de desenvolvimento em documento de usuário.
3. **Barras duplicadas em código inline** (`` `%APPDATA%\\WidgetWebsite` ``): `README.md:54,77`, `README-en.md:81`, `README.es.md:103` — dentro de crases a barra **não** é escapada; renderiza `\\`.
4. Requisitos: "Rust estável 1.90 ou mais recente" × `rust-toolchain.toml` fixa `1.90.0` (rustup instala essa versão automaticamente).

**Solução proposta**
- Renomear `README-en.md` → `README.en.md` e atualizar os links (`README.md:3`, `README.es.md:3`, `README.en.md:3`, e demais referências — `grep -rn "README-en"`).
- Portar para PT e EN as seções "Diagnóstico", "Privacidade e suporte", "Privacidade e limitações" e "Estrutura" (conteúdo verificado contra o código: heartbeat de 5 s/limite de 20 s e rotação de ~1 MB conferem — `diagnostics.rs:14,16`; `window.rs:172-174`), **removendo** a lista de pendências de teste do ES (mover para `MICROSOFT_STORE_READINESS.md`).
- Corrigir `\\` → `\` dentro de crases.
- Ajustar requisito de Rust ("o `rust-toolchain.toml` seleciona 1.90.0 automaticamente com rustup").

**Arquivos a alterar** — os 3 READMEs, `MICROSOFT_STORE_READINESS.md`.

**Critérios de aceite** — Os 3 READMEs têm as mesmas seções; nenhum link quebrado (`grep -rn "README-en.md"` vazio); caminhos renderizam com uma barra.

**Como verificar** — Conferir links e renderização (pré-visualização Markdown).

**Cuidados** — i18n: PT-BR, EN, ES mantidos em paridade.

---

### FLY-P4-014 — `src-tauri/icons/` ignorado: `cargo test` em clone limpo exige `npm run icons` (H10)

| Campo | Valor |
|---|---|
| Parte | 4 (interface com Parte 1) |
| Severidade | Baixa |
| Categoria | Build/CI · Documentação |
| Confiança | Provável (o `build.rs`/`tauri-build` valida os ícones; o comentário do CI confirma a necessidade; **não executado** em Windows) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `.gitignore:4,13`; `src-tauri/tauri.conf.json:29-33`; `README.md:26-30`; `build-msix.yml:85` |
| Depende de | — |

**Evidência**
```gitignore
src-tauri/icons/
src-tauri/gen/
```
```yaml
- name: Generate Windows icons required by Tauri tests   # build-msix.yml:85
```

**Descrição e causa raiz** — `tauri.conf.json` referencia `icons/32x32.png`, `icons/128x128.png`, `icons/icon.ico`, mas o diretório é gerado (não versionado). Em um clone limpo, `cargo test`/`clippy`/`tauri dev` falham até rodar `npm run icons`. O README cobre `tauri dev`/`tauri build`, **não** `cargo test`/`clippy`. (`src-tauri/gen/` ignorado é coerente; o `$schema` das capabilities aponta para `../gen/schemas/…`, que só existe após o primeiro build — apenas afeta o editor.)

**Solução proposta** (recomendada: manter ignorado e tornar o fluxo explícito; alternativa descartada: versionar os ícones gerados — ruído de diff binário)
```diff
   "scripts": {
+    "test:rust": "npm run icons && cargo test --manifest-path src-tauri/Cargo.toml --locked",
```
e acrescentar ao README (3 idiomas), seção Desenvolvimento: *"Para rodar `cargo test`/`cargo clippy`, execute antes `npm run icons` (ou use `npm run test:rust`)."*

**Arquivos a alterar** — `package.json` (scripts), `README*.md`.

**Critérios de aceite** — Em clone limpo no Windows: `npm ci && npm run test:rust` passa.

**Como verificar** — Execução em clone limpo.

**Cuidados** — Não alterar `build.rs`; não remover `icons/` do `.gitignore`.

---

### FLY-P4-015 — Dependência do runtime VC++ não verificada (NSIS e MSIX)

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Média |
| Categoria | Build/CI · Robustez |
| Confiança | Hipótese — **não verificado** se o binário liga o CRT dinamicamente (`vcruntime140.dll`) |
| Esforço | M |
| Risco de regressão | Médio |
| Local | `src-tauri/Cargo.toml` (sem `.cargo/config.toml` nem `[profile.release]`); `build-windows.yml:112-129`; `Package.appxmanifest.template.xml:24-29` (sem `PackageDependency`) |
| Depende de | — |

**Descrição e causa raiz** — Rust no alvo MSVC liga por padrão o CRT dinamicamente. Se o binário depender de `vcruntime140.dll`/`msvcp140.dll`, uma máquina limpa sem o *Visual C++ Redistributable* não inicia o app. O MSIX não embute nem declara dependência (`Microsoft.VCLibs.140.00.UWPDesktop`); o NSIS também não instala o redistribuível. O CI só confere o subsistema PE (GUI), não as DLLs importadas.

**Solução proposta**
1. **Verificar:** após o build, no Windows, listar importações do `widget-website.exe` (`dumpbin /dependents`, disponível nos Build Tools) e **adicionar ao CI** um passo que falhe se aparecerem `vcruntime140*.dll`/`msvcp140*.dll`/`ucrtbase` não esperados.
2. Se confirmado, escolher (decisão humana): *(a)* CRT estático (`src-tauri/.cargo/config.toml` com `[target.x86_64-pc-windows-msvc] rustflags = ["-C", "target-feature=+crt-static"]`) — recomendado, simples; ou *(b)* declarar `PackageDependency` do VCLibs no MSIX (e instalar o redistribuível no NSIS).
3. Alternativa para (a) sem arquivo novo: `RUSTFLAGS` no CI — **não** recomendada (o build local divergiria do CI).

**Arquivos a alterar** — (condicional) `src-tauri/.cargo/config.toml`, `build-windows.yml`/`build-msix.yml` (passo de verificação).

**Critérios de aceite** — O passo de CI aprova quando o `.exe` não importa DLLs de runtime VC++ fora do conjunto permitido; instalação em VM Windows limpa abre o app.

**Como verificar** — VM Windows 10/11 limpa; `dumpbin /dependents`.

**Cuidados** — `crt-static` altera o tamanho do binário e pode interagir com dependências nativas; rodar a suíte completa depois.

---

### FLY-P4-016 — Fixação de ações: comentários/versões inconsistentes e não verificáveis

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Baixa |
| Categoria | Build/CI · Dependência |
| Confiança | Hipótese (SHAs/versões **não verificáveis** nesta sessão) |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `build-windows.yml:47,50,54,60,65,176,188`; `build-msix.yml:52,57,63,70,75,137` |
| Depende de | — |

**Evidência**
```yaml
- uses: dtolnay/rust-toolchain@683b588c… # pinned rust-toolchain action
- uses: actions/cache@3edfce90… # actions/cache v6.2.0
- uses: actions/upload-artifact@cf430e03… # actions/upload-artifact v7.0.2      (build-windows)
- uses: actions/upload-artifact@cf430e03… # v7.0.2                              (build-msix)
- uses: actions/download-artifact@9000827c… # actions/download-artifact v8.0.2
```

**Descrição e causa raiz** — (1) Todos os `uses:` estão fixados por SHA (boa prática; **nenhum sem versão**). (2) O comentário de `dtolnay/rust-toolchain` não traz versão/ref, e o do `actions/cache` usa formato diferente dos demais; o Dependabot (`github-actions`) depende do comentário `# vX.Y.Z` para atualizar de forma limpa. (3) `upload-artifact` v7 × `download-artifact` v8: majors diferentes — conferir compatibilidade. (4) **Nenhum SHA/versão foi validado** contra o repositório oficial.

**Solução proposta**
1. O corretor deve validar cada SHA: `git ls-remote --tags https://github.com/<owner>/<repo>` (ou `gh api repos/<owner>/<repo>/git/ref/tags/<tag>`) e confirmar que o SHA é o da tag comentada.
2. Padronizar comentários para `# vX.Y.Z` (e, para `dtolnay/rust-toolchain`, citar a ref/tag do SHA).
3. Conferir a matriz de compatibilidade `upload-artifact`/`download-artifact` na documentação das ações; alinhar majors se exigido.

**Arquivos a alterar** — `build-windows.yml`, `build-msix.yml`.

**Critérios de aceite** — Todos os comentários no formato `# vX.Y.Z`; cada SHA confere com a tag.

**Como verificar** — Comandos acima; execução do workflow.

**Cuidados** — Nunca trocar SHA por tag móvel.

---

### FLY-P4-017 — Nomes marca × interno: acoplamentos não protegidos (H14)

| Campo | Valor |
|---|---|
| Parte | 4 (interface com Parte 1) |
| Severidade | Info |
| Categoria | Manutenibilidade |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | `Cargo.toml:2`, `package.json:2`, `tauri.conf.json:5`, `config.rs:207`, `hooks.nsh:8-9`, workflows (`widget-website-*`), `build-msix.ps1:70,83,112` |
| Depende de | — |

**Análise** — Nomes em uso: `Floatly` (produto/MSIX), `widget-website` (crate, npm, `.exe`, artefatos, `concurrency`), `WidgetWebsite` (pasta de dados), `com.pablopcdossantos.widgetwebsite` (identifier), `pablopcdsantos.Floatly` (identidade Store). A pasta de dados é referenciada em **código** (`config.rs`), **NSIS** (`hooks.nsh`) e **documentação**; mudar um sem os outros apaga/perde dados. Renomear o executável (`mainBinaryName`) alteraria o caminho gravado na entrada de inicialização de instalações existentes. **Não é necessário renomear** (regra 6): o README já explica a pasta herdada.

**Recomendação (leve, sem renomeação)** — Proteger o acoplamento por teste: novo `scripts/check-data-dir.mjs` (`npm run check:data-dir`) que compare a pasta em `config.rs` (regex `\.join\("([^"]+)"\)` na função `app_directory`) com as ocorrências em `hooks.nsh`, e rode no CI junto aos demais checks (ver seção 8). Se, no futuro, a pasta for renomeada, será necessária migração de dados (cópia + mantém `WidgetWebsite` como fallback), fora do escopo atual.

**Arquivos a alterar** — (opcional) `scripts/check-data-dir.mjs`, `package.json`, workflows.

**Critérios de aceite** — O check falha se `hooks.nsh` e `config.rs` divergirem na pasta.

---

### FLY-P4-018 — Pendências menores agregadas

| Campo | Valor |
|---|---|
| Parte | 4 |
| Severidade | Baixa |
| Categoria | Documentação · Manutenibilidade |
| Confiança | Confirmado |
| Esforço | P |
| Risco de regressão | Baixo |
| Local | ver lista |
| Depende de | — |

1. **Sem arquivo de licença** na raiz (nem campo `license` no `Cargo.toml`): decisão do autor (a ausência de licença significa "todos os direitos reservados"). Considerar `LICENSE` e um aviso de terceiros (Tauri, WebView2).
2. **Metadados do instalador:** `tauri.conf.json` `bundle` não define `publisher`, `copyright`, `shortDescription`/`category` — aparecem vazios/derivados no instalador NSIS.
3. **MSIX `Description`** (`Package.appxmanifest.template.xml:47`) fixa em português, embora declare 3 `Resource Language`. Sem `resources.pri`/`ms-resource`, usuários EN/ES veem o texto em PT (apenas no Windows/tooltip).
4. **Termos de busca** (`MICROSOFT_STORE_LISTING_DRAFT.md:89-93`) incluem "WebView2" (nome de produto da Microsoft); avaliar substituir por termos genéricos.
5. **Exemplos com marca de terceiros** (`README*.md`, seção "Exemplo de uso: Investing.com"): o próprio projeto afirma não endossar terceiros; considerar exemplo neutro (`https://example.com`).
6. **`SUPPORT.md`/`PRIVACY_POLICY.md` expõem e-mail pessoal**: necessário para suporte/LGPD (mantido); o **link Lattes** nos READMEs é opcional.
7. **Grafias divergentes** do identificador do autor: `pablopcdsantos` (Store/MSIX), `pablopcdossantos` (identifier/GitHub/`authors`), `pablopcsantos` (e-mail) — apenas conferir se são intencionais (a identidade da Store é fixa do Partner Center).
8. **Cargo.toml** sem `[profile.release]` (binário maior que o necessário; `opt-level`/`lto`/`strip`) — melhoria opcional; **não** usar `panic = "abort"` sem revisar o *panic hook* (Parte 1).

**Arquivos a alterar** — conforme cada item (decisões do autor nos itens 1, 4, 5 e 7).

**Critérios de aceite** — Itens tratados ou explicitamente rejeitados no registro de decisões.

---

## 6. Achados transversais

| Tema | Partes | Ordem de correção |
|---|---|---|
| Escopo da capability remota × `check:acl` | 3 → 4 | (1) decidir o escopo na Parte 3; (2) aplicar FLY-P4-004 no mesmo PR |
| Strings do `INIT` (e mensagens de log em PT) lidas por `check-drag-overlay`/`check-acl` | 2 → 4 | Qualquer alteração em `window.rs` (INIT, mensagem de "mover/redimensionar travado") exige revisar `check-drag-overlay.mjs`; ideal: migrar asserção para teste Rust (FLY-P4-010) |
| Pasta `WidgetWebsite` e nome da entrada `Run` | 1 ↔ 4 | FLY-P4-005 (verificação) → FLY-P4-012/017 (docs e check) |
| MSIX × `%APPDATA%` virtualizado × H8 (`current_dir()` fallback) | 1 ↔ 4 | A Parte 1 decide o comportamento de `app_directory()`; FLY-P4-012 documenta o resultado final |
| `icons` gerados × testes Rust | 1 ↔ 4 | FLY-P4-014 |

---

## 7. Plano de correção ordenado

**Convenção:** dentro de cada onda, itens agrupados por arquivo; em arquivos com várias edições, aplicar **de baixo para cima** (números de linha mudam).

### Onda 1 — pré-requisitos e riscos de rejeição/segurança

| Ordem | ID | Arquivo(s) | Esforço | Como validar | Paralelo? |
|---|---|---|---|---|---|
| 1 | FLY-P4-001 | `Package.appxmanifest.template.xml`, `check-msix-config.mjs`, READMEs | P | `npm run check:msix-config`; `build:msix` (Windows) | Sim (com 2, 3) |
| 2 | FLY-P4-002 | `build-windows.yml` (job `sign-and-publish`) | P | PR de teste: job *skipped* | Sim |
| 3 | FLY-P4-003 | `dependabot.yml` (+ doc de procedimento) | P | YAML válido; log do Dependabot | Sim |
| 4 | FLY-P4-005 | `hooks.nsh` (**exige verificação manual antes**) | P | `reg query` pós-desinstalação | Não (depende da verificação em Windows) |

### Onda 2 — médias

| Ordem | ID | Arquivo(s) | Esforço | Como validar |
|---|---|---|---|---|
| 5 | FLY-P4-010 (+ `scripts/lib.mjs`) | `scripts/*.mjs` | M | 5 `check:*` verdes + `node --test scripts/` |
| 6 | FLY-P4-006 | `check-tauri-versions.mjs`, `check-msix-config.mjs`, guia | P | checks + fixtures |
| 7 | FLY-P4-004 | `check-acl.mjs` — **junto com a decisão da Parte 3** | P | `check:acl` com capability atual e reduzida |
| 8 | FLY-P4-008 | `build-windows.yml`, `build-msix.yml`, (novo) `audit.yml` | M | 2ª execução usa cache |
| 9 | FLY-P4-012 | `PRIVACY_POLICY.md`, `SUPPORT.md`, READMEs, `READINESS` — **após** validar o caminho de dados no MSIX | M | revisão cruzada com `config.rs` |
| 10 | FLY-P4-015 | verificação em Windows → condicional | M | `dumpbin /dependents` no CI |

### Onda 3 — baixas e documentação

FLY-P4-009, FLY-P4-011, FLY-P4-013, FLY-P4-014, FLY-P4-016, FLY-P4-007 (decisão humana), FLY-P4-017 (opcional), FLY-P4-018 (decisões do autor).

**Conflitos de edição a evitar:** `build-windows.yml` recebe 002, 008, 009, 016 (e 007) → aplicar em um único passe; `build-msix.yml` recebe 008, 009, 011, 016; `check-msix-config.mjs` recebe 001 e 006; `check-tauri-versions.mjs` recebe 006 e 010.

**Exigem decisão humana:** 001 (confirmar versão mínima/impacto em Windows 1903/1909), 007 (assinar exe antes do NSIS), 008 (workflow de auditoria agendado), 015 (CRT estático × dependência VCLibs), 018 (licença, termos de busca, exemplo de terceiros).

---

## 8. Testes a adicionar

| Teste (nome sugerido) | Tipo | O que cobre |
|---|---|---|
| `scripts/check-tauri-versions.test.mjs` → `fails_when_cargo_lock_version_differs` | `node --test` | FLY-P4-006: `Cargo.lock`/`packages[""]` ≠ `package.json` |
| `scripts/check-msix-config.test.mjs` → `fails_when_minversion_below_uap10` | `node --test` | FLY-P4-001 |
| `scripts/check-msix-config.test.mjs` → `reads_version_from_package_section_only` | `node --test` | FLY-P4-006 |
| `scripts/check-acl.test.mjs` → `accepts_narrowed_remote_urls` / `rejects_extra_remote_permission` / `rejects_remote_window_other_than_main` | `node --test` | FLY-P4-004 |
| `scripts/check-acl.test.mjs` → `ignores_opacity_in_comments` | `node --test` | FLY-P4-010 |
| `scripts/check-theme-contrast.test.mjs` → `anchors_selector_and_rejects_non_hex_tokens` | `node --test` | FLY-P4-010 |
| `scripts/check-data-dir.mjs` (novo check) | script `check:*` | FLY-P4-017: pasta de dados em `config.rs` = `hooks.nsh` |
| Passo de CI "Verify PE imports" (PowerShell + `dumpbin /dependents`) | CI | FLY-P4-015 |
| Passo de CI `actionlint` (opcional; ação fixada por SHA **verificado**) | CI | Sintaxe/expressões dos workflows |
| Teste Rust (Parte 2) que valide o bloqueio de ação com mover/redimensionar travado | `#[cfg(test)]` | Substitui a asserção textual de `check-drag-overlay.mjs:36` |

Adicionar `"test:scripts": "node --test scripts/"` ao `package.json` e um passo correspondente nos dois workflows.

---

## 9. Itens descartados (hipóteses refutadas ou sem achado)

| Hipótese / ponto | Resultado | Evidência |
|---|---|---|
| "`sign-and-publish` herda permissões amplas" | **Refutado** (apenas melhoria opcional em FLY-P4-002) | Workflow tem `permissions: contents: read` no topo (`build-windows.yml:41-42`); não há `permissions` por job que as eleve; `download-artifact` não exige mais |
| H13 — versões `typescript ~7.0.2`, `vite ^8.3.5`, `windows 0.62.2`, `windows-future 0.3.2` "inexistentes" | **Sem achado de incoerência**; existência no registry **não verificada** (sem rede) | `package-lock.json`: `typescript 7.0.2`, `vite 8.3.5`, `rolldown 1.2.15` (engines `^20.19.0 \|\| >=22.12.0`, compatível com `engines.node >=22.12.0`); `Cargo.lock`: `windows 0.62.2`, `windows-core 0.62.2`, `windows-future 0.3.2`, `windows-sys 0.61.2` coerentes com `Cargo.toml` |
| H12 — Política diverge do comportamento real (logs, sanitização, telemetria) | **Refutado quanto ao núcleo**; permanecem lacunas de canal MSIX e backups (FLY-P4-012) | `sanitize_url` (`diagnostics.rs:209-228`) remove credenciais/query/fragmento e limita o caminho a 96 caracteres; rotação: `MAX_LOG_BYTES = 1 MiB`, `MAX_LOG_BACKUPS = 3` (`:16,20`); verificação `/favicon.ico` a cada 30 s existe (`window.rs:168-171`, `main.ts:89,115`); sem telemetria no código |
| Features de `windows-sys` declaradas e não usadas | **Refutado** | `Foundation` (`POINT`, `CloseHandle`, `HWND`), `UI_WindowsAndMessaging`, `UI_Input_KeyboardAndMouse`, `Graphics_Dwm`, `Graphics_Gdi`, `System_Diagnostics_ToolHelp` têm uso (`window.rs`, `diagnostics.rs`, `main.rs`) |
| Versões do app inconsistentes entre arquivos | **Refutado** | `1.0.0` em `package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock`, `tauri.conf.json` |
| Alinhamento API/CLI/crate do Tauri | **OK hoje** | npm `2.12.4`/`2.12.4`, crate `2.12.5` (mesmo major.minor) |
| Injeção de expressão em `build-msix.yml` via `inputs` | **Refutado** | Entradas passadas por `env:` e lidas como variáveis (`build-msix.yml:130-132`), não interpoladas no script |
| `$LASTEXITCODE` não propagado no passo MSIX | **Refutado** | O *wrapper* `pwsh` do Actions encerra com o código do último comando nativo; o script usa `throw` e `Invoke-Checked` |
| Verificação do subsistema PE com *offset* errado | **Refutado** | Subsystem em `OptionalHeader + 68` para PE32 e PE32+ (`build-windows.yml:127`) |
| `Get-Command winapp` com `IsNullOrWhiteSpace` | **Refutado** | `ApplicationInfo.ToString()` retorna o nome; `$null` → vazio → erro correto |
| `hooks.nsh` apagar dados fora do escopo | **Refutado** | Só remove `$APPDATA\WidgetWebsite` e `$LOCALAPPDATA\WidgetWebsite`, sob `$UpdateMode <> 1` e com o checkbox marcado; `webview-data` está dentro de `WidgetWebsite` (`window.rs:493`); a pasta do *identifier* é tratada pelo modelo NSIS do Tauri (**não verificado** aqui) |
| Workflows com sintaxe inválida / `uses:` sem versão | **Refutado** | YAML válido; todos os `uses:` fixados por SHA |
| Erros de `npm run typecheck` | **Não são achados** | Provêm de instalação npm incompleta (ver seção 2) |

---

## 10. Pontos não verificados / perguntas em aberto

1. **Autor:** manter suporte a Windows 10 1903/1909? (FLY-P4-001)
2. **Autor:** nome real da entrada `Run` na instalação NSIS (FLY-P4-005) — exige Windows.
3. **Windows:** localização real dos dados do app no MSIX (FLY-P4-012) e se `winapp pack` aceita o manifesto atual (FLY-P4-001).
4. **Windows:** importações do `widget-website.exe` (VC++ runtime) (FLY-P4-015).
5. **Rede/GitHub:** validade dos SHAs e versões das ações; existência de `cargo-audit 0.22.2`, `setup-WinAppCli v0.2/v0.7.1`; compatibilidade `upload/download-artifact` v7/v8 (FLY-P4-016, FLY-P4-008).
6. **Resultados de `npm audit`, `cargo audit`, `cargo fmt/test/clippy`, `typecheck`, `vite build`:** não executados (sem rede/Windows).
7. **Parte 1:** há poda dos `config.json.bad-*`? (FLY-P4-012)
8. **Store:** o Partner Center pode questionar aplicativos que apenas encapsulam sites; o texto de certificação (`GUIA…:125-135`) já cobre `runFullTrust` e a ausência de site predefinido — avaliar reforço, se houver pedido de esclarecimento. **Hipótese não verificada.**
9. **Decisões de assinatura (FLY-P4-007) e de auditoria agendada (FLY-P4-008).**

---

## 11. Instruções operacionais para a IA corretora

- Aplicar **um achado por vez**, na ordem da seção 7; após cada um, rodar os comandos pertinentes (`npm run check:*`, `npm run typecheck`, `cargo fmt/test/clippy` quando o achado tocar Rust) e só avançar se passarem.
- Mudar **somente** o que o achado pede; não refatorar por conta própria.
- **Reler o trecho atual** antes de editar (os números de linha deste relatório mudam após cada edição; em `build-windows.yml` aplique de baixo para cima).
- Adicionar o teste sugerido junto com a correção (seção 8).
- Manter comentários/logs em pt-BR e atualizar **os três idiomas** (PT-BR, EN, ES) ao alterar README, Política, SUPPORT ou textos de interface.
- Se uma correção mexer em `generate_handler!`, `build.rs` ou capabilities, atualizar os três **juntos** e conferir `npm run check:acl`. (Os achados da Parte 4 não exigem isso, exceto a decisão da Parte 3 ligada a FLY-P4-004.)
- **Nunca** incluir segredos; **nunca** desativar `check:*`, `clippy -D warnings`, `npm audit` ou `cargo audit` para "fazer passar".
- Itens marcados **Hipótese** (001, 005, 015, 016) exigem a **verificação indicada antes** de editar; se não for possível verificar, registrar "não verificado" e **não aplicar**.
- Itens que exigem decisão humana (seção 7) devem ser apresentados ao autor com as opções e a recomendação, sem aplicar por conta própria.
- Ao terminar cada achado, registrar: ID, arquivos alterados, resultado dos comandos e eventual desvio em relação ao proposto.

---

## 12. Prompt de execução sugerido (pronto para colar)

> Você é um(a) engenheiro(a) sênior de Rust/Tauri 2/CI-CD no Windows. Leia integralmente `RELATORIO_ANALISE_FLOATLY_PARTE_4.md` (Parte 4 do Floatly) e corrija os achados **na ordem da seção 7**, um por vez. Para cada achado: releia o trecho atual, aplique somente a mudança proposta, adicione o teste indicado, rode os comandos pertinentes (`npm run check:*`, `npm run typecheck`, e `cargo fmt/test/clippy` se tocar Rust) e só avance se passarem. Não desative verificações, não inclua segredos, mantenha comentários em pt-BR e os textos em PT-BR/EN/ES. Para achados com confiança **Hipótese** (FLY-P4-001, 005, 015, 016), execute primeiro a verificação descrita e, se não puder, registre "não verificado" e não aplique. Para os itens que exigem decisão humana (001, 007, 008, 015, 018), apresente as opções e aguarde. Ao final de cada achado, informe ID, arquivos alterados, resultado dos comandos e desvios.

*Fim do relatório da Parte 4.*
