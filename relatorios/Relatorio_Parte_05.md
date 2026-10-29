# Relatório de Análise e Auditoria de Código — Widget Website (Tauri 2 / Windows)

> **Escopo desta versão:** somente a **Parte 5 — Infraestrutura: Build, CI/CD, Dependências e Distribuição**.
> As Partes 1 a 4 ainda **não foram analisadas** e aparecem como pendentes na seção 4. Os achados transversais que dependem delas estão registrados na seção 5 apenas como referência cruzada.

## 0. Instruções para a IA/Dev que fará as correções
- Corrija UM achado por vez, na ordem da seção 3.
- Após cada alteração, rode: `cargo test --manifest-path src-tauri/Cargo.toml` e `npm run build`. Para achados de CI, valide com um `workflow_dispatch` em runner Windows.
- Adicione/ajuste os testes indicados; não remova testes sem justificativa.
- Em caso de conflito, siga o campo "Depende de / Conflita com".
- Achados `HIPOTESE` ou `PROVAVEL` exigem confirmação no ambiente indicado (runner Windows, instalador real) antes da correção definitiva.
- Este relatório **não modificou nenhum arquivo** do repositório.

## 1. Resumo Executivo
- **Ambiente analisado:** `widget-website-main.zip` (33 arquivos, v0.1.0). Análise estática de `.github/workflows/build-windows.yml`, `package.json`, `tsconfig.json`, `vite.config.ts`, `src-tauri/Cargo.toml`, `build.rs`, `tauri.conf.json`, `.gitignore`, `icons/widget.svg` e dos 3 READMEs.
- **Execução:** não foi possível rodar `npm install`, `npm audit`, `cargo test`, `cargo audit` nem o build (sandbox sem rede e sem Rust/Cargo; app é Windows-only). **Linha de base de testes: não obtida.** Nenhum achado abaixo é `CONFIRMADO` por execução; os `CONFIRMADO` são fatos verificados por leitura do arquivo.
- **Verificação externa feita:** as tags das GitHub Actions usadas **existem** (`checkout v7.0.1`, `setup-node v7`, `cache v6`, `upload-artifact v7`), portanto não são erro. Foram consultados avisos de segurança do Vite, o comportamento do desinstalador NSIS do Tauri e a checagem de versões do Tauri CLI.
- **Totais (Parte 5):** 11 achados — **ALTA: 1 · MÉDIA: 4 · BAIXA: 6**. Nenhum CRÍTICO. Nenhum segredo, chave ou credencial no repositório (busca feita; só há referências a `secrets.*` no workflow).
- **Top 5 riscos:**
  1. **P5-001** Sem lockfiles + versões flutuantes: o build pode quebrar sem mudança de código (Tauri CLI recusa crate e pacote npm em minor diferente) e não é reproduzível.
  2. **P5-002** Cache de `src-tauri/target` com chave só de `Cargo.toml`: pode devolver instaladores antigos ao artefato final.
  3. **P5-003** O desinstalador NSIS não apaga os dados reais do app (`%APPDATA%\WidgetWebsite`), nem a sessão de login do WebView2.
  4. **P5-004** O build de release roda com o certificado de assinatura já instalado e com dependências sem versão fixa.
  5. **P5-005** Sem auditoria automatizada de dependências (`npm audit`, `cargo audit`, Dependabot) nem lint/typecheck separado no CI.

## 2. Ambiente e Linha de Base

**Ferramentas disponíveis no sandbox:** `node v22.22.2`, `npm 10.9.7`, `python3`. **Ausentes:** `cargo`, `rustc`, `jq`. Rede: desativada.

**Comandos executados e resultado**

| Comando | Resultado |
|---|---|
| `unzip -l widget-website-main.zip` | 33 arquivos; **sem** `package-lock.json`, `Cargo.lock`, `LICENSE`, `.github/dependabot.yml`, `rust-toolchain.toml`, `.nvmrc` |
| validação JSON (`python3 json.load`) de `package.json`, `tsconfig.json`, `tauri.conf.json`, `capabilities/*.json` | todos válidos |
| conferência de versões | `0.1.0` em `package.json`, `Cargo.toml:3` e `tauri.conf.json:4` (consistentes hoje) |
| busca por segredos (`secret|token|password|api_key|BEGIN … PRIVATE`) fora de `*.md` | apenas `secrets.WINDOWS_PFX_*` no workflow e testes de URL em `config.rs`/`diagnostics.rs` (dados de teste); **nenhum segredo real** |
| `npm install --no-audit --no-fund` | **falhou**: `ENOTFOUND registry.npmjs.org` (sem rede) → `node_modules` não criado; `tsc`, `vite build` e `npm audit` **não puderam rodar** |
| `cargo test …`, `cargo audit`, `cargo clippy` | **não executados**: Rust ausente |

**Verificações externas (pesquisa web, não execução):**
- Existência das tags de Actions usadas: confirmada para `actions/checkout v7.0.1`, `actions/setup-node v7` (7.0.0), `actions/cache v6` (6.1.0), `actions/upload-artifact v7` (7.0.1).
- Avisos do Vite: `CVE-2025-58751`/`CVE-2025-58752` corrigidos em 6.3.6; `CVE-2025-62522` corrigido em 6.4.1.
- Tauri CLI: `tauri build` falha se o crate `tauri` e `@tauri-apps/api` não estiverem na mesma versão major/minor (mensagem "Found version mismatched Tauri packages…"); há relato público de CI quebrado exatamente por `tauri = "2"` + npm em minor anterior.
- Desinstalador NSIS do Tauri: a opção "Delete the application data" remove `$APPDATA\${BUNDLEID}` e `$LOCALAPPDATA\${BUNDLEID}` (identificador do bundle).

**Limitações:** sem Windows real e sem rede, não foi possível confirmar o resultado de nenhum job do CI, o comportamento do instalador nem `npm audit`/`cargo audit`. Quem corrigir deve repetir as verificações marcadas como "não reproduzível neste ambiente".

## 3. Plano de Correção Ordenado

| Ordem | ID | Severidade | Esforço (P/M/G) | Arquivo(s) Afetado(s) | Depende de | Resumo (1 linha) |
|---|---|---|---|---|---|---|
| 1 | P5-001 | ALTA | M | `package.json`, `src-tauri/Cargo.toml`, `build-windows.yml:34,37,70-77` (+ novos `package-lock.json`, `Cargo.lock`) | — | Sem lockfiles e com versões flutuantes: build quebra ou muda sem alteração de código |
| 2 | P5-002 | MÉDIA | P | `build-windows.yml:24-33,78-84` | P5-001 | Cache de `target/` inclui instaladores antigos e usa chave inadequada |
| 3 | P5-004 | MÉDIA | M | `build-windows.yml:38-77` | P5-001 | Build de release roda com certificado de assinatura instalado |
| 4 | P5-005 | MÉDIA | M | `build-windows.yml`, `package.json`, novo `.github/dependabot.yml` | P5-001 | CI sem auditoria de dependências, lint ou typecheck separado |
| 5 | P5-003 | MÉDIA | M | `tauri.conf.json:8`, `config.rs:99-104`, instalador NSIS | — | Desinstalador não remove dados reais do app nem a entrada de autostart |
| 6 | P5-006 | BAIXA | P | `build-windows.yml:17-25,78` | — | Actions de terceiros por tag mutável em job que assina |
| 7 | P5-007 | BAIXA | P | `package.json:devDependencies`, `vite.config.ts` | P5-001 | Piso `vite ^6.3.5` abaixo das versões corrigidas |
| 8 | P5-008 | BAIXA | P | `.gitignore` | — | Falta ignorar `*.pfx`, `*.p12`, `.env`, `src-tauri/gen/` |
| 9 | P5-009 | BAIXA | P | raiz, `Cargo.toml`, `package.json`, `tauri.conf.json` | — | Sem `LICENSE`, metadados placeholder e versão triplicada à mão |
| 10 | P5-010 | BAIXA | P | `README*.md`, `main.rs:179` | — | READMEs divergem entre si e do código |
| 11 | P5-011 | BAIXA | P | `tauri.conf.json:8` | — | Instalador só em PT-BR, app é trilíngue |

## 4. Achados por Parte

### Parte 1 — Núcleo: Configuração, Estado e Regras de Negócio
*Não analisada nesta etapa.*

### Parte 2 — Integrações Windows, Arquivos e Diagnóstico (I/O)
*Não analisada nesta etapa.*

### Parte 3 — Camada de Aplicação: Ciclo de Vida, Janelas, Navegação e Bandeja
*Não analisada nesta etapa.*

### Parte 4 — Interface e Superfície de Entrada Externa
*Não analisada nesta etapa.*

### Parte 5 — Infraestrutura: Build, CI/CD, Dependências e Distribuição

#### P5-001 — Sem lockfiles e com versões flutuantes: build não reproduzível e sujeito a quebra espontânea
- **Severidade:** ALTA
- **Categoria:** `build-ci`
- **Confiança:** PROVAVEL
- **Local:** `package.json` (`devDependencies`/`dependencies`), `src-tauri/Cargo.toml:9,12-15`, `.github/workflows/build-windows.yml:34` (`npm install`), `:37` (`cargo test`), `:70-77` (`tauri build`)
- **Evidência:**
  - A listagem do ZIP não contém `package-lock.json` nem `Cargo.lock` (nem `rust-toolchain.toml`/`.nvmrc`).
  - `Cargo.toml`: `tauri-build = { version = "2" }`, `tauri = { version = "2", … }`, `tauri-plugin-autostart = "2"`, `tauri-plugin-single-instance = "2"`, `tauri-plugin-global-shortcut = "2"`.
  - `package.json`: `"@tauri-apps/cli": "^2.9.5"`, `"vite": "^6.3.5"`, `"@tauri-apps/api": "^2.8.0"`, `"typescript": "~5.8.3"`.
  - O workflow usa `npm install` (linha 34), não `npm ci`, e `cargo` sem `--locked`.
  - Pesquisa externa: o Tauri CLI aborta `tauri build` quando o crate `tauri` e `@tauri-apps/api` diferem em major/minor ("Found version mismatched Tauri packages…"). Há relato público de CI quebrado por `tauri = "2"` resolver 2.11.0 enquanto o npm ainda estava em 2.10.1.
- **Reprodução:** não reproduzível neste ambiente (sem rede, sem Cargo). Para reproduzir: em runner limpo, rodar o workflow em dois momentos diferentes e comparar `npm ls @tauri-apps/api` com `cargo tree -p tauri --depth 0`; ou, após `npm install`, rodar `npx tauri info` e observar o aviso de pacotes com versão divergente.
- **Causa raiz:** ausência de lockfiles versionados e de instalação com versões travadas; cada execução resolve as dependências do zero (crates.io e npm são publicados em momentos diferentes).
- **Impacto:** o CI e qualquer build local podem falhar de um dia para o outro sem mudança de código; dois builds do mesmo commit podem gerar binários diferentes; fica impossível auditar exatamente o que foi enviado ao instalador assinado.
- **Correção proposta:**
  1. Gerar e **versionar** `package-lock.json` (`npm install --package-lock-only`) e `src-tauri/Cargo.lock` (`cargo generate-lockfile`; aplicativos devem versionar `Cargo.lock`). Garanta que o par `tauri` (crate) × `@tauri-apps/api`/`@tauri-apps/cli` esteja na **mesma minor** no momento do commit.
  2. No workflow: trocar `npm install` por `npm ci`; usar `cargo test --locked` e `npm run tauri build -- --locked` (ou `cargo build --locked` no passo de build, conforme o CLI aceitar) para falhar se o lockfile estiver desatualizado.
  3. Opcional: fixar a toolchain Rust (`rust-toolchain.toml` com canal/versão) e usar `.nvmrc`/`engines` (ver P5-009).
- **Critério de aceite:** (a) os dois lockfiles estão no repositório; (b) o workflow usa `npm ci` e `--locked` e passa em runner limpo; (c) alterar uma dependência sem atualizar o lockfile faz o CI falhar; (d) `npx tauri info` não mostra pacotes com versão divergente.
- **Teste a adicionar:** passo de CI "verificar versões Tauri alinhadas" (script que compara minor de `tauri` em `Cargo.lock` com `@tauri-apps/api` em `package-lock.json` e falha se diferirem).
- **Depende de / Conflita com:** nenhum.
- **Esforço:** M

#### P5-002 — Cache de `src-tauri/target` com chave inadequada pode incluir instaladores antigos no artefato final
- **Severidade:** MÉDIA
- **Categoria:** `build-ci`
- **Confiança:** PROVAVEL
- **Local:** `.github/workflows/build-windows.yml:24-33` (cache) e `:78-84` (upload)
- **Evidência:**
  ```yaml
  path: |
    ~/.cargo/registry
    ~/.cargo/git
    src-tauri/target
  key: windows-cargo-${{ hashFiles('src-tauri/Cargo.toml') }}
  restore-keys: |
    windows-cargo-
  ...
  path: |
    src-tauri/target/release/bundle/nsis/*.exe
    src-tauri/target/release/widget-website.exe
  ```
- **Reprodução:** não reproduzível neste ambiente. Passos: (1) rodar o workflow em `main` com versão `0.1.0` (o cache é salvo ao fim do job, contendo `target/release/bundle/nsis/*0.1.0*.exe`); (2) subir a versão para `0.2.0` em `Cargo.toml`/`tauri.conf.json` (o hash do `Cargo.toml` muda → *cache miss* na chave exata → `restore-keys` restaura o cache anterior pelo prefixo `windows-cargo-`); (3) inspecionar o artefato `widget-website-windows-x64`: o glob `bundle/nsis/*.exe` tende a incluir o instalador antigo e o novo.
- **Causa raiz:** (a) o cache inclui toda a pasta `target/`, inclusive `release/bundle/`; (b) a chave depende só do `Cargo.toml` (o `Cargo.lock` não existe, e `src/` não entra), e `restore-keys` aceita qualquer cache anterior; (c) o upload usa glob sem filtrar pela versão atual; (d) pelo comportamento padrão de `actions/cache`, com *hit* exato o cache **não é regravado**, então ele fica congelado no estado da primeira execução daquela chave.
- **Impacto:** o artefato pode conter um instalador de versão anterior (possivelmente sem assinatura ou com código antigo) ao lado do novo, levando a distribuição do arquivo errado; além disso o cache cresce e fica obsoleto.
- **Correção proposta:**
  1. Cachear apenas `~/.cargo/registry`, `~/.cargo/git` (ou usar `Swatinem/rust-cache`, que já trata `target/`), e **não** `src-tauri/target/release/bundle`.
  2. Chave do cache baseada em `hashFiles('**/Cargo.lock')` (depende de P5-001) e sistema operacional/versão do Rust.
  3. Antes do `tauri build`, limpar saídas anteriores: `Remove-Item -Recurse -Force src-tauri/target/release/bundle -ErrorAction SilentlyContinue`.
  4. No upload, restringir ao arquivo da versão atual (ex.: `*_${{ version }}_x64-setup.exe`) ou manter só um `.exe` esperado.
- **Critério de aceite:** após um bump de versão, o artefato contém **somente** o instalador e o executável da versão atual (verificável listando o artefato); a chave de cache muda quando `Cargo.lock` muda.
- **Teste a adicionar:** passo final do job que falha se `nsis/*.exe` tiver mais de um arquivo (`if ((Get-ChildItem src-tauri/target/release/bundle/nsis/*.exe).Count -ne 1) { exit 1 }`).
- **Depende de / Conflita com:** depende de P5-001 (usa `Cargo.lock` na chave).
- **Esforço:** P

#### P5-003 — O desinstalador não remove os dados reais do app (config, log e sessão de login do WebView2) nem a entrada de autostart
- **Severidade:** MÉDIA
- **Categoria:** `compatibilidade-plataforma`
- **Confiança:** PROVAVEL (a parte do autostart é HIPOTESE)
- **Local:** `src-tauri/tauri.conf.json:5,8` (`identifier`, `bundle.windows.nsis`); `src-tauri/src/config.rs:99-104` (`app_directory`); `src-tauri/src/main.rs:112` (plugin `autostart`)
- **Evidência:**
  - `identifier` = `com.pablopcdossantos.widgetwebsite`.
  - `config::app_directory()` usa `%APPDATA%\WidgetWebsite` (config, `diagnostics.log`, `webview-data\`, `settings-data\`), **não** o identificador.
  - O instalador NSIS do Tauri, ao marcar "Delete the application data", executa `RmDir /r "$APPDATA\${BUNDLEID}"` e `RmDir /r "$LOCALAPPDATA\${BUNDLEID}"` (confirmado no histórico do `installer.nsi` do `tauri-bundler`). Nenhum desses caminhos coincide com `%APPDATA%\WidgetWebsite`.
  - O README informa que a sessão persistente do WebView2 (cookies de login do Investing.com) fica em `%APPDATA%\WidgetWebsite\webview-data`.
- **Reprodução:** não reproduzível neste ambiente. Em Windows: instalar o `.exe`, abrir o app, fazer login, ativar "Iniciar com o Windows"; desinstalar marcando "Delete the application data"; verificar `dir %APPDATA%\WidgetWebsite` e `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- **Causa raiz:** a pasta de dados do app foi definida manualmente e é independente do identificador que o desinstalador usa; não há *hook* NSIS para remover a pasta nem o valor de autostart.
- **Impacto:** a opção de apagar dados do desinstalador não faz o que promete; a sessão autenticada de um site financeiro, o log (que pode conter hosts e caminhos visitados) e as preferências ficam no disco após a desinstalação. Se o autostart estiver ativo, pode sobrar uma entrada em `Run` apontando para um executável removido (HIPOTESE: confirmar se o instalador/plugin não limpa o valor).
- **Correção proposta:**
  1. Preferida: criar um *hook* NSIS (`bundle.windows.nsis.installerHooks`) com `NSIS_HOOK_POSTUNINSTALL` que, quando `$DeleteAppDataCheckboxState = 1`, remove `$APPDATA\WidgetWebsite`; e remove o valor de autostart de `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` (confirmar o nome do valor gravado pelo plugin).
  2. Alternativa: mover os dados para o diretório baseado no identificador (`app.path().app_data_dir()`), com migração da pasta antiga na primeira execução (impacta Partes 1 e 2).
  3. Documentar no README onde ficam os dados e como removê-los.
- **Critério de aceite:** após desinstalar com a caixa marcada, `%APPDATA%\WidgetWebsite` não existe e não há valor de autostart do app em `HKCU\...\Run`; com a caixa desmarcada, os dados são preservados.
- **Teste a adicionar:** roteiro manual de instalação/desinstalação em Windows 10/11 (checklist da seção 9) e, se houver runner, script PowerShell que instala silenciosamente (`/S`), cria dados, desinstala e confere a limpeza.
- **Depende de / Conflita com:** a alternativa 2 conflita com mudanças de caminho nas Partes 1 e 2; decidir após analisá-las.
- **Esforço:** M

#### P5-004 — O build de release roda com o certificado de assinatura já instalado e com dependências sem versão fixa
- **Severidade:** MÉDIA
- **Categoria:** `seguranca`
- **Confiança:** PROVAVEL (a ordem dos passos é fato; o impacto é um risco, não uma exploração demonstrada)
- **Local:** `.github/workflows/build-windows.yml:34-37` (instalação e testes antes do certificado), `:38-69` (importa PFX para `Cert:\CurrentUser\My`), `:70-77` (`tauri build`)
- **Evidência:** `npm install` (linha 34) e `cargo test` (linha 37) rodam **antes** do certificado ser importado (bom). Porém `tauri build` (linhas 70-77) compila de novo todo o grafo de crates em perfil *release* — incluindo `build.rs` e *proc-macros* de dependências transitivas — **depois** de `Import-PfxCertificate` (linha 53) e com `TAURI_WINDOWS_SIGNING_CONFIG` definido. Sem `Cargo.lock` (P5-001), as versões desses crates são resolvidas no próprio momento do build.
- **Reprodução:** não aplicável (risco de cadeia de suprimentos). Verificação estática: ordem dos passos nas linhas citadas; inexistência de lockfile na listagem do ZIP.
- **Causa raiz:** assinatura embutida no mesmo job/passo que compila código de terceiros não travado.
- **Impacto:** uma dependência comprometida, ao executar seu `build.rs`, teria o mesmo usuário do runner e poderia usar o certificado instalado para assinar binários arbitrários com a identidade do editor. Probabilidade baixa, impacto alto (reputação do certificado/SmartScreen).
- **Correção proposta:**
  1. Primeiro, travar dependências (P5-001) e adicionar auditoria (P5-005).
  2. Separar em dois jobs: `build` (sem acesso ao certificado, gera instalador não assinado) e `sign` (baixa o artefato e só executa `signtool`, em job com `environment` protegido e aprovação manual).
  3. Se mantiver a assinatura integrada, importar o PFX com `-Exportable:$false` (padrão) e remover o certificado do repositório de certificados ao final (`Remove-Item Cert:\CurrentUser\My\<thumbprint>` em passo com `if: always()`).
- **Critério de aceite:** nenhum passo que compila ou executa código de dependências roda com o certificado presente; o passo de limpeza roda mesmo em falha; o instalador final continua assinado e o artefato verificado com `signtool verify /pa`.
- **Teste a adicionar:** passo `signtool verify /pa /v <instalador>` ao final do job de assinatura (falha se não assinado quando os segredos existem).
- **Depende de / Conflita com:** depende de P5-001; complementa P5-005 e P5-006.
- **Esforço:** M

#### P5-005 — CI sem auditoria de dependências, lint ou verificação de tipos separada
- **Severidade:** MÉDIA
- **Categoria:** `dependencias-licencas`
- **Confiança:** CONFIRMADO (ausência observada nos arquivos)
- **Local:** `.github/workflows/build-windows.yml` (todo o job); `package.json:scripts`; ausência de `.github/dependabot.yml`
- **Evidência:** o job executa apenas `npm install`, `npm run icons`, `cargo test` e `tauri build`. `package.json` define só `dev`, `build`, `icons`, `tauri` (sem `test`, `lint`, `audit`). Não há `npm audit`, `cargo audit`/`cargo deny`, `cargo clippy`, `cargo fmt --check`, nem arquivo do Dependabot. O `tsc --noEmit` roda apenas dentro de `npm run build`, disparado por `beforeBuildCommand` **depois** dos testes e já no passo de empacotamento.
- **Reprodução:** `grep -n -E "audit|clippy|fmt|dependabot|lint" .github/workflows/build-windows.yml package.json` retorna vazio; `unzip -l` não lista `dependabot.yml`.
- **Causa raiz:** pipeline focado em compilar o instalador; sem etapas de qualidade nem monitoramento de vulnerabilidades.
- **Impacto:** vulnerabilidades em crates/pacotes npm (o app embute WebView2 e navega em site externo) e regressões de tipo/estilo só seriam notadas manualmente; erros de TypeScript só aparecem no passo mais lento.
- **Correção proposta:**
  1. Adicionar `.github/dependabot.yml` para `npm`, `cargo` (diretório `/src-tauri`) e `github-actions`.
  2. No workflow, antes do build: `npx tsc --noEmit`, `cargo clippy --locked -- -D warnings`, `cargo fmt --check`, `npm audit --audit-level=high` e `cargo audit` (ou `cargo deny check`).
  3. Adicionar scripts `typecheck` e `audit` em `package.json`.
- **Critério de aceite:** o CI falha com um pacote com vulnerabilidade alta conhecida, com erro de tipo e com aviso do clippy; o Dependabot abre PRs de atualização.
- **Teste a adicionar:** os próprios passos de CI acima (não há função de teste a criar).
- **Depende de / Conflita com:** depende de P5-001 (`--locked`, `npm ci`).
- **Esforço:** M

#### P5-006 — Actions de terceiros referenciadas por tag mutável em job que lida com certificado
- **Severidade:** BAIXA
- **Categoria:** `build-ci`
- **Confiança:** CONFIRMADO (fato); impacto HIPOTESE
- **Local:** `.github/workflows/build-windows.yml:17,18,21,25,78`
- **Evidência:** `actions/checkout@v7.0.1`, `actions/setup-node@v7`, `dtolnay/rust-toolchain@stable`, `actions/cache@v6`, `actions/upload-artifact@v7`. **As tags existem** (verificado), portanto não há erro de versão. `v7`, `v6` e `stable` são referências móveis; `dtolnay/rust-toolchain` pertence a terceiro (não é da organização `actions`).
- **Reprodução:** não aplicável.
- **Causa raiz:** uso de tags/branches em vez de SHAs de commit.
- **Impacto:** se uma tag móvel for reapontada para código malicioso, o código roda no mesmo job que depois importa o certificado (ver P5-004). Risco baixo para ações oficiais, um pouco maior para a de terceiro.
- **Correção proposta:** fixar por SHA completo com comentário da versão (`uses: dtolnay/rust-toolchain@<sha> # stable`) e deixar o Dependabot atualizar (P5-005).
- **Critério de aceite:** todas as linhas `uses:` terminam em SHA de 40 caracteres.
- **Teste a adicionar:** verificação no CI (`grep -E "uses: .+@[^0-9a-f]"` falha se achar referência não fixada).
- **Depende de / Conflita com:** depende de P5-005 para manutenção automática.
- **Esforço:** P

#### P5-007 — Piso de versão do Vite abaixo das versões com correção de segurança
- **Severidade:** BAIXA
- **Categoria:** `dependencias-licencas`
- **Confiança:** PROVAVEL
- **Local:** `package.json` (`"vite": "^6.3.5"`), `vite.config.ts:3`, `package.json` script `dev` (`--host 127.0.0.1`)
- **Evidência:** pesquisa externa: `CVE-2025-58751` e `CVE-2025-58752` (arquivos servidos fora de `server.fs`) corrigidos em **6.3.6**; `CVE-2025-62522` (bypass de `server.fs.deny` com `\` no Windows) corrigido em **6.4.1**. O intervalo `^6.3.5` ainda permite 6.3.5 e 6.4.0. Os avisos afetam o servidor de desenvolvimento apenas quando exposto à rede; aqui `server.host` é `127.0.0.1` (`vite.config.ts` e script `dev`), logo a condição de exploração não se aplica. Sem lockfile (P5-001), uma instalação nova tende a obter a última 6.x, o que mitiga na prática.
- **Reprodução:** não reproduzível (sem rede para `npm audit`). Verificar com `npm ls vite` e `npm audit` após gerar o lockfile.
- **Causa raiz:** piso do intervalo semver desatualizado.
- **Impacto:** baixo (ambiente de desenvolvimento local; sem exposição à rede), mas o `package.json` declara compatibilidade com versões vulneráveis.
- **Correção proposta:** elevar para `"vite": "^6.4.1"` (ou migrar para a série corrente após testar) e manter `server.host` em loopback.
- **Critério de aceite:** `npm audit` sem avisos para `vite`; `npm ls vite` ≥ 6.4.1.
- **Teste a adicionar:** passo `npm audit --audit-level=high` (P5-005).
- **Depende de / Conflita com:** P5-001, P5-005.
- **Esforço:** P

#### P5-008 — `.gitignore` incompleto
- **Severidade:** BAIXA
- **Categoria:** `seguranca`
- **Confiança:** CONFIRMADO (conteúdo do arquivo); item `src-tauri/gen` é HIPOTESE
- **Local:** `.gitignore:1-5`
- **Evidência:** o arquivo ignora apenas `node_modules/`, `dist/`, `src-tauri/target/`, `src-tauri/icons/`, `*.log`. O fluxo de assinatura do projeto trabalha com um `.pfx` (README e workflow), e `capabilities/*.json` referenciam `../gen/schemas/desktop-schema.json`, pasta gerada pelo Tauri no build.
- **Reprodução:** `git check-ignore -v meu.pfx .env src-tauri/gen/schemas/x.json` não retorna correspondência.
- **Causa raiz:** lista mínima de ignorados.
- **Impacto:** risco de commitar por engano um certificado/senha local ou arquivos gerados.
- **Correção proposta:** acrescentar `*.pfx`, `*.p12`, `*.key`, `*.pem`, `.env`, `.env.*` e `src-tauri/gen/` (confirmar se o Tauri gera essa pasta no build e se deve ser versionada).
- **Critério de aceite:** `git check-ignore` reconhece os padrões acima.
- **Teste a adicionar:** nenhum (verificação manual).
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

#### P5-009 — Sem `LICENSE`, metadados com valor genérico e versão repetida em três arquivos
- **Severidade:** BAIXA
- **Categoria:** `dependencias-licencas`
- **Confiança:** CONFIRMADO
- **Local:** raiz do repositório; `src-tauri/Cargo.toml:3-6`; `package.json`; `src-tauri/tauri.conf.json:4`
- **Evidência:** não há `LICENSE`; `Cargo.toml` tem `authors = ["Widget Website"]` e nenhum `license`/`rust-version`; `package.json` não tem `license` nem `engines`; o README exige "Rust estável 1.90 ou mais recente" e "Node.js LTS", mas nada no repositório impõe isso (o CI usa `stable` e Node 22). A versão `0.1.0` aparece em `package.json`, `Cargo.toml:3` e `tauri.conf.json:4` (hoje consistentes, mantidas à mão).
- **Reprodução:** `unzip -l widget-website-main.zip | grep -i license` retorna vazio.
- **Causa raiz:** metadados do projeto não preenchidos.
- **Impacto:** sem licença, terceiros não têm permissão clara de uso; a versão pode divergir entre os três arquivos e gerar nome de instalador e versão exibida inconsistentes; ambientes com Rust/Node antigos falham tarde e com erros pouco claros.
- **Correção proposta:** adicionar `LICENSE` e os campos `license` (Cargo/npm); definir `rust-version` (confirmar o valor real necessário — o "1.90" do README não foi verificado por execução) e `engines.node`; deixar `tauri.conf.json` sem `version` (o Tauri usa a do `Cargo.toml`) ou validar a igualdade no CI.
- **Critério de aceite:** `LICENSE` presente; `cargo metadata` mostra `license` e `rust-version`; CI falha se as versões divergirem.
- **Teste a adicionar:** passo de CI que compara as três versões.
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

#### P5-010 — Documentação divergente entre os READMEs e o código
- **Severidade:** BAIXA
- **Categoria:** `documentacao`
- **Confiança:** CONFIRMADO
- **Local:** `README.md:63` e `README.md:77`; `README.en.md:68` / `README.es.md:68` (diagnóstico); seções de testes e estrutura dos três READMEs; `src-tauri/src/main.rs:177-181`
- **Evidência:**
  1. Os três READMEs afirmam que as URLs são registradas **sem query string nem fragmento**, mas `main.rs:179-180` registra `cfg.site_url` completo (`"Janela principal criada. URL configurada: {}"`).
  2. A seção "Estrutura" do `README.md` lista "configuração, janela, comandos e bandeja" e **omite** `diagnostics.rs`; `README.en.md` e `README.es.md` o incluem.
  3. A lista de testes do CI no `README.md:70` não menciona a compatibilidade com configuração antiga sem idioma, que `README.en.md` e `README.es.md` citam.
  4. `README.md` exige "Node.js LTS"; os outros dois dizem apenas "Node.js and npm".
- **Reprodução:** `grep -n "URL configurada" src-tauri/src/main.rs`; `grep -n "src-tauri/src" README*.md`.
- **Causa raiz:** READMEs mantidos manualmente em três idiomas, sem checagem de consistência com o código.
- **Impacto:** o item 1 é uma promessa de privacidade não cumprida em um caminho (a URL configurada pode conter parâmetros); os demais geram confusão.
- **Correção proposta:** corrigir o código para registrar a URL sem query/fragmento (reutilizando a lógica de `log_navigation`) — correção em `main.rs`, a validar na Parte 2 — **ou** ajustar os três READMEs; alinhar os itens 2 a 4 nos três idiomas.
- **Critério de aceite:** nenhum log contém query/fragmento da URL configurada e os três READMEs têm o mesmo conteúdo factual.
- **Teste a adicionar:** `fn config_url_log_omits_query_and_fragment()` em `main.rs`/`diagnostics.rs`, configurando `https://x.com/p?token=abc#frag` e checando que `token=abc` e `frag` não aparecem no log.
- **Depende de / Conflita com:** relacionado ao achado de logs da Parte 2 (T-001).
- **Esforço:** P

#### P5-011 — Instalador NSIS disponível apenas em português
- **Severidade:** BAIXA
- **Categoria:** `i18n`
- **Confiança:** CONFIRMADO
- **Local:** `src-tauri/tauri.conf.json:8`
- **Evidência:** `"nsis":{"displayLanguageSelector":false,"languages":["PortugueseBR"]}`, enquanto o aplicativo oferece pt-BR, en e es.
- **Reprodução:** leitura do arquivo.
- **Causa raiz:** configuração do instalador não acompanhou a internacionalização do app.
- **Impacto:** usuários de inglês/espanhol veem o instalador e o desinstalador em português.
- **Correção proposta:** `"languages": ["PortugueseBR","English","Spanish"]` com `displayLanguageSelector: true` (ou deixar o instalador escolher pelo idioma do sistema).
- **Critério de aceite:** o instalador gerado oferece os três idiomas.
- **Teste a adicionar:** verificação manual no instalador (seção 9).
- **Depende de / Conflita com:** nenhum.
- **Esforço:** P

## 5. Achados Transversais (T-001…)

> Referências cruzadas a confirmar quando as Partes 1 a 4 forem analisadas.

- **T-001 (pendente — Parte 2):** o README promete log sem query string; `main.rs:179` contradiz (ver P5-010). A correção de código pertence à Parte 2.
- **T-002 (pendente — Partes 3 e 4):** a política de navegação e as permissões (`capabilities/*.json`, `withGlobalTauri: true`, `csp: null`) definem o que a página remota consegue fazer; a revisão ficou para a Parte 4. Como `build.rs` usa apenas `tauri_build::build()`, confirmar se os comandos do app estão restritos por ACL. **Não** avaliado nesta etapa.
- **T-003 (Parte 5 × Partes 1 e 2):** a pasta de dados fixa em `%APPDATA%\WidgetWebsite` (P5-003) afeta o desinstalador e deve ser decidida junto com a revisão de `config::app_directory`.

## 6. Cobertura de Testes Ausente (Parte 5)

| Cenário crítico sem teste | Função/passo sugerido |
|---|---|
| Crate `tauri` e `@tauri-apps/api` na mesma minor | passo de CI "versões Tauri alinhadas" (P5-001) |
| Artefato contém só o instalador da versão atual | passo `Count -ne 1` em `nsis/*.exe` (P5-002) |
| Instalador assinado quando há segredos | `signtool verify /pa /v` (P5-004) |
| Versões iguais em `package.json`, `Cargo.toml`, `tauri.conf.json` | passo de CI de consistência (P5-009) |
| Log da URL configurada sem query/fragmento | `config_url_log_omits_query_and_fragment` (P5-010) |
| Instalação/desinstalação limpa (dados e autostart) | roteiro manual/script PowerShell (P5-003) |
| Verificação de tipos e lint isolados | `npx tsc --noEmit`, `cargo clippy`, `cargo fmt --check` (P5-005) |

Observação: o front-end TypeScript não tem nenhum teste automatizado; o back-end tem ~13 testes `#[test]` identificados por leitura (`config.rs` 6, `diagnostics.rs` 4, `window.rs` 3), **não executados** aqui.

## 7. Falsos Positivos e Itens Verificados Sem Problema

- **Tags das Actions (`checkout@v7.0.1`, `setup-node@v7`, `cache@v6`, `upload-artifact@v7`):** existem; não são erro de versão (restam apenas a observação P5-006).
- **`permissions: contents: read` (linhas 11-12):** privilégio mínimo adequado; o job não escreve no repositório.
- **Segredos:** `WINDOWS_PFX_*` só são lidos pelo passo de assinatura (variáveis de ambiente do passo), não são impressos e o PFX temporário é apagado (linha 68). Em `pull_request` o passo é ignorado (linha 39), portanto PRs de forks não têm acesso a eles. Se a importação falhar antes da linha 68, o arquivo fica em `RUNNER_TEMP`, que é descartado com o runner efêmero — sem achado.
- **`ConvertTo-SecureString -AsPlainText -Force` (linha 52):** necessário para `Import-PfxCertificate` com senha vinda de variável de ambiente; o valor é mascarado pelo GitHub como segredo. Sem achado.
- **`timestampUrl = "http://timestamp.digicert.com"` (linha 62):** servidores de carimbo de tempo RFC 3161 são usados por HTTP por convenção; a integridade vem da assinatura do carimbo, não do transporte. Não é vulnerabilidade.
- **Caminho do arquivo de configuração de assinatura (`--config "$env:TAURI_WINDOWS_SIGNING_CONFIG"`, linhas 73-77):** o Tauri CLI aceita caminho para JSON em `--config`; as chaves `certificateThumbprint`, `digestAlgorithm` e `timestampUrl` são as documentadas. Sem achado.
- **`exit 0` no ramo sem segredos (linha 47):** encerra o passo com sucesso, comportamento documentado nos READMEs; o artefato não assinado é comunicado por `Write-Warning`.
- **Nome do executável no upload (`widget-website.exe`):** coincide com `name = "widget-website"` em `Cargo.toml:2`; `if-no-files-found: error` evita artefato vazio.
- **Ícones gerados no CI (`src-tauri/icons/` ignorado + `npm run icons`):** coerente com `bundle.icon` (`32x32.png`, `128x128.png`, `icon.ico`) e documentado no README; sem achado.
- **`tsconfig.json` e `vite.config.ts`:** `allowImportingTsExtensions` exige `noEmit: true`, que está definido; as duas entradas HTML (`index.html`, `settings.html`) estão em `rollupOptions.input`. Sem problema.
- **Versões:** `0.1.0` consistente nos três arquivos hoje (apenas risco de divergência futura — P5-009).
- **`concurrency` com `cancel-in-progress: true`:** cancela execuções antigas da mesma ref; aceitável (um push novo em `main` cancela o build anterior, inclusive um build assinado em andamento).
- **WebView2 no instalador:** `webviewInstallMode` não definido usa o padrão do Tauri (baixa o *bootstrapper*), coerente com o app que avisa quando o Runtime não existe. Sem achado.
- **Itens NÃO verificados por falta de Windows/rede/Cargo (não considerar aprovados):**
  - se `cargo test` (linha 37) compila antes de existir `../dist` — pela lógica do Tauri em modo de desenvolvimento (`devUrl` definido) não deve exigir, mas **não foi confirmado**; confirmar com um `workflow_dispatch`;
  - se o `tauri build` do CLI ativa o recurso `custom-protocol` automaticamente (o `Cargo.toml` não declara `[features]`); confirmar com um build de release que o app carrega `tauri.localhost` e não `devUrl`;
  - resultado de `npm audit`, `cargo audit` e das licenças transitivas (`cargo deny`/`cargo license`);
  - que o "Rust 1.90" do README seja de fato o mínimo necessário.

## 8. Melhorias Opcionais
- Fixar o runner (`windows-2025` etc.) em vez de `windows-latest` para builds reprodutíveis.
- Adicionar `timeout-minutes` ao job (hoje 360 min por padrão).
- Publicar o instalador em um *GitHub Release* (hoje só artefato do workflow) e gerar *checksums* SHA-256.
- Avaliar assinatura via serviço em nuvem (Azure Trusted Signing) em vez de PFX no repositório de segredos.
- Remover `targets: x86_64-pc-windows-msvc` redundante no runner Windows (já é o alvo padrão).
- Fixar `typescript ~5.8.3`/`@tauri-apps/cli` com política de atualização (Dependabot agrupado).
- Gerar um arquivo de licenças de terceiros (`cargo about`/`license-checker`) para distribuir com o instalador.

## 9. Checklist Final de Verificação Pós-Correção
- [ ] `package-lock.json` e `src-tauri/Cargo.lock` versionados; CI usa `npm ci` e `--locked` (P5-001).
- [ ] `npx tauri info` sem pacotes Tauri com versão divergente (P5-001).
- [ ] Após bump de versão, o artefato do CI contém um único instalador e o executável da versão atual (P5-002).
- [ ] Desinstalar com "Delete the application data" remove `%APPDATA%\WidgetWebsite` e o autostart; sem marcar, preserva (P5-003).
- [ ] Nenhum passo que compila dependências roda com o certificado instalado; `signtool verify /pa` passa (P5-004).
- [ ] `npm audit`, `cargo audit`, `clippy`, `fmt --check`, `tsc --noEmit` rodando no CI; Dependabot ativo (P5-005, P5-007).
- [ ] Todas as `uses:` fixadas por SHA (P5-006).
- [ ] `.gitignore` cobre `*.pfx`, `*.p12`, `.env`, `src-tauri/gen/` (P5-008).
- [ ] `LICENSE`, `license`, `rust-version`, `engines` presentes; versões verificadas no CI (P5-009).
- [ ] READMEs consistentes entre si e com o código; log sem query da URL configurada (P5-010).
- [ ] Instalador oferece pt-BR, en e es (P5-011).
- [ ] **Pendente:** análise das Partes 1 a 4 e do item "não verificado" da seção 7.
