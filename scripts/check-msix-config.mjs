import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = process.cwd();
const readJson = async (path) => JSON.parse(await readFile(resolve(root, path), "utf8"));
const fail = (message) => {
  console.error(`[check:msix-config] ERRO: ${message}`);
  process.exitCode = 1;
};

const [tauri, cargo, pkg, packageLock, cargoLock, manifest] = await Promise.all([
  readJson("src-tauri/tauri.conf.json"),
  readFile(resolve(root, "src-tauri/Cargo.toml"), "utf8"),
  readJson("package.json"),
  readJson("package-lock.json"),
  readFile(resolve(root, "src-tauri/Cargo.lock"), "utf8"),
  readFile(resolve(root, "packaging/msix/Package.appxmanifest.template.xml"), "utf8"),
]);

if (tauri.productName !== "Floatly") fail("productName do Tauri deve ser Floatly.");
if (tauri.version !== pkg.version) fail("package.json e tauri.conf.json devem usar a mesma versão.");
const versionParts = String(tauri.version).split(".").map(Number);
if (versionParts.length !== 3 || versionParts.some((part) => !Number.isInteger(part) || part < 0 || part > 65535)) {
  fail("A versão do aplicativo deve ter três componentes numéricos entre 0 e 65535.");
} else if (versionParts[0] === 0) {
  fail("A versão do pacote MSIX não pode começar com 0. Use, por exemplo, 1.0.0.");
}
const cargoPackage = cargo.match(/^\[package\]\s*([\s\S]*?)(?=^\[|$)/m)?.[1] ?? "";
const cargoPackageVersion = cargoPackage.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? "";
const cargoLockPackageVersion = cargoLock.match(/\[\[package\]\]\s*name = "widget-website"\s*version = "([^"]+)"/)?.[1] ?? "";
if (cargoPackageVersion !== tauri.version) fail("Cargo.toml e tauri.conf.json devem usar a mesma versão.");
if (pkg.version !== packageLock.version || pkg.version !== packageLock.packages?.[""]?.version) {
  fail("package.json e package-lock.json devem usar a mesma versão, inclusive em packages[\"\"] .");
}
if (cargoLockPackageVersion !== pkg.version) fail("A versão do pacote widget-website em Cargo.lock deve corresponder à versão do aplicativo.");

const targetDeviceFamily = manifest.match(/<TargetDeviceFamily[^>]*>/i)?.[0] ?? "";
const minVersion = targetDeviceFamily.match(/MinVersion="([^"]+)"/i)?.[1] ?? "";
const minVersionParts = minVersion.split(".").map(Number);
const requiredMinVersion = [10, 0, 19041, 0];
const minVersionIsValid =
  minVersionParts.length === 4 &&
  minVersionParts.every((part) => Number.isInteger(part) && part >= 0);
let minVersionComparison = 0;
if (minVersionIsValid) {
  for (let index = 0; index < requiredMinVersion.length; index += 1) {
    if (minVersionParts[index] !== requiredMinVersion[index]) {
      minVersionComparison = minVersionParts[index] < requiredMinVersion[index] ? -1 : 1;
      break;
    }
  }
}
if (!minVersionIsValid || minVersionComparison < 0) {
  fail(`TargetDeviceFamily MinVersion must be 10.0.19041.0 or newer for the declared uap10 features (found "${minVersion || "missing"}").`);
}
if (!manifest.includes("xmlns:uap10=") || !manifest.includes("uap10:PackageIntegrity") || !manifest.includes("uap10:RuntimeBehavior")) {
  fail("The manifest uses uap10 features; keep their namespace and declarations aligned with the minimum Windows build.");
}
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
