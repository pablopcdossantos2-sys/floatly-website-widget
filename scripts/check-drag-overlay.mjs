import fs from "node:fs";

const windowSource = fs.readFileSync(new URL("../src-tauri/src/window.rs", import.meta.url), "utf8");
const init = windowSource.match(/const INIT: &str = r#"(.*?)"#;/s)?.[1];

if (!init) throw new Error("Could not extract the main WebView initialization script.");

const checks = [
  ["visible blue frame has inline CSSOM styles", init.includes("border:'4px solid #1683ff'") && init.includes("element.style.setProperty(property,value)")],
  ["drag grip has a visible inline-styled label", init.includes("handle.textContent='⋮⋮'") && init.includes("handle.style.setProperty('background'")],
  ["drag requires a trusted primary-button gesture", init.includes("if(!event.isTrusted||event.button!==0)return;")],
  ["drag action uses the per-window token", init.includes("widget-action://'+ACTION_TOKEN")],
  ["lock state disables drag hit targets", init.includes("edge.style.setProperty('pointer-events',locked?'none':'auto')") && init.includes("handle.style.setProperty('pointer-events',locked?'none':'auto')")],
  ["Rust validates lock state before processing window actions", windowSource.includes("Ação de janela ignorada porque mover/redimensionar está travado")]
];

const failed = checks.filter(([, passed]) => !passed).map(([name]) => name);
if (failed.length) throw new Error("Drag overlay check failed: " + failed.join("; "));

console.log("Drag overlay smoke check passed: visible inline-styled frame/grip, trusted gesture, token authorization and lock enforcement.");
