import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (name) => fs.readFileSync(path.join(root, name), "utf8");
const fail = (message) => {
  console.error("Microsoft Store configuration check failed: " + message);
  process.exit(1);
};

const pkg = JSON.parse(read("package.json"));
const tauri = JSON.parse(read("src-tauri/tauri.conf.json"));
const store = JSON.parse(read("src-tauri/tauri.microsoftstore.conf.json"));
const remote = JSON.parse(read("src-tauri/capabilities/main-remote-heartbeat.json"));
const cargo = read("src-tauri/Cargo.toml");
const config = read("src-tauri/src/config.rs");
const startup = read("src/main.ts");
const settings = read("src/settings.ts");
const settingsHtml = read("settings.html");

const cargoVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (!pkg.version || pkg.version !== tauri.version || pkg.version !== cargoVersion) {
  fail("package.json, tauri.conf.json and Cargo.toml versions must match.");
}
if (!tauri.productName || tauri.productName.trim() !== "Widget Website") {
  fail("The product name must stay generic; do not use a third-party website's brand.");
}
if (!tauri.identifier || !/^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)+$/.test(tauri.identifier)) {
  fail("The Tauri bundle identifier is missing or invalid.");
}
if (!Array.isArray(tauri.bundle?.targets) || !tauri.bundle.targets.includes("nsis")) {
  fail("The current Microsoft Store build script expects the NSIS target.");
}
if (store.bundle?.windows?.webviewInstallMode?.type !== "offlineInstaller") {
  fail("The Store-specific Tauri config must use WebView2 offlineInstaller.");
}
if (!remote.remote?.urls?.includes("https://*") ||
    !remote.remote?.urls?.includes("http://*") ||
    JSON.stringify(remote.permissions) !== JSON.stringify(["allow-heartbeat"])) {
  fail("Remote website capability must allow only the authenticated heartbeat command on HTTP(S) sites.");
}
if (!/pub const DEFAULT_SITE: &str = "";/m.test(config) || startup.includes("investing.com")) {
  fail("A new installation must not preconfigure a third-party website URL.");
}
if (config.includes("pub opacity: u8") || settings.includes("opacity") ||
    settingsHtml.includes('id="opacity"') || settingsHtml.includes('id="opacity-value"')) {
  fail("The removed page opacity setting has reappeared.");
}
if (/calendário econômico|economic calendar|calendario economico/i.test(cargo.match(/^description\s*=\s*"([^"]+)"/m)?.[1] ?? "")) {
  fail("Cargo package description is still calendar-specific.");
}

if (!tauri.bundle.publisher) {
  console.warn("MANUAL ACTION REQUIRED: set bundle.publisher to the developer's real publisher name before producing the final Store package.");
}
console.log("Microsoft Store configuration checks passed. The publisher identity and signing certificate still require human configuration.");
