import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = process.cwd();
const readJson = async (path) => JSON.parse(await readFile(resolve(root, path), "utf8"));
const fail = (message) => {
  console.error(`[check:msix-config] ERRO: ${message}`);
  process.exitCode = 1;
};

const [tauri, cargo, pkg, manifest] = await Promise.all([
  readJson("src-tauri/tauri.conf.json"),
  readFile(resolve(root, "src-tauri/Cargo.toml"), "utf8"),
  readJson("package.json"),
  readFile(resolve(root, "packaging/msix/Package.appxmanifest.template.xml"), "utf8"),
]);

if (tauri.productName !== "Floatly") fail("productName do Tauri deve ser Floatly.");
if (tauri.version !== pkg.version) fail("package.json e tauri.conf.json devem usar a mesma versão.");
if (!cargo.includes(`version = "${tauri.version}"`)) fail("A versão do pacote Rust deve corresponder à versão do Tauri.");
if (!manifest.includes('Name="__PACKAGE_IDENTITY_NAME__"')) fail("O manifesto deve manter o marcador de identidade do Partner Center.");
if (!manifest.includes('Publisher="__PACKAGE_PUBLISHER__"')) fail("O manifesto deve manter o marcador Publisher do Partner Center.");
if (!manifest.includes('Version="__PACKAGE_VERSION__"')) fail("O manifesto deve receber a versão MSIX de quatro componentes no build.");
if (!manifest.includes('Executable="$targetnametoken$.exe"')) fail("O manifesto deve usar o token de executável resolvido pelo winapp.");
if (!manifest.includes('Category="windows.startupTask"')) fail("A extensão StartupTask do MSIX está ausente.");
if (!manifest.includes('TaskId="FloatlyStartup"')) fail("O TaskId do StartupTask deve corresponder ao código Rust.");
if (!manifest.includes('Name="runFullTrust"')) fail("O manifesto precisa declarar runFullTrust para o processo desktop Tauri.");
for (const asset of ["Assets\\StoreLogo.png", "Assets\\Square150x150Logo.png", "Assets\\Square44x44Logo.png", "Assets\\Wide310x150Logo.png"]) {
  if (!manifest.includes(asset)) fail(`O manifesto não referencia o ativo obrigatório ${asset}.`);
}
if (!manifest.includes("uap10:RuntimeBehavior") || !manifest.includes("packagedClassicApp")) {
  fail("O manifesto deve configurar o aplicativo desktop empacotado como packagedClassicApp.");
}
if (!pkg.scripts?.["build:msix"] || !pkg.scripts?.["check:msix-config"]) {
  fail("package.json deve expor os comandos build:msix e check:msix-config.");
}

if (process.exitCode) {
  process.exit(process.exitCode);
}
console.log("[check:msix-config] Configuração base MSIX, manifesto, identidade dinâmica e StartupTask OK.");
