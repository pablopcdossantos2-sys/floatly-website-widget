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
const versionParts = String(tauri.version).split(".").map(Number);
if (versionParts.length !== 3 || versionParts.some((part) => !Number.isInteger(part) || part < 0 || part > 65535)) {
  fail("A versão do aplicativo deve ter três componentes numéricos entre 0 e 65535.");
} else if (versionParts[0] === 0) {
  fail("A versão do pacote MSIX não pode começar com 0. Use, por exemplo, 1.0.0.");
}
if (!cargo.includes(`version = "${tauri.version}"`)) fail("A versão do pacote Rust deve corresponder à versão do Tauri.");
const targetDeviceFamily = manifest.match(/<TargetDeviceFamily\\b[^>]*>/s)?.[0] ?? "";
const minVersion = targetDeviceFamily.match(/\\bMinVersion="([^"]+)"/)?.[1] ?? "";
const versionTuple = (value) => value.split(".").map((part) => Number(part));
const minimumSupported = [10, 0, 19041, 0];
const declaredMinimum = versionTuple(minVersion);
if (
  declaredMinimum.length !== 4 ||
  declaredMinimum.some((part) => !Number.isInteger(part) || part < 0) ||
  declaredMinimum.some((part, index) => part !== minimumSupported[index] && declaredMinimum.slice(0, index).every((value, previous) => value === minimumSupported[previous]) && part < minimumSupported[index]) ||
  declaredMinimum.findIndex((part, index) => part !== minimumSupported[index]) !== -1 &&
    (() => {
      for (let index = 0; index < 4; index += 1) {
        if (declaredMinimum[index] !== minimumSupported[index]) return declaredMinimum[index] < minimumSupported[index];
      }
      return false;
    })()
) {
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
