import fs from "node:fs";

const windowSource = fs.readFileSync(new URL("../src-tauri/src/window.rs", import.meta.url), "utf8");
const init = windowSource.match(/const INIT: &str = r#"(.*?)"#;/s)?.[1];

if (!init) throw new Error("Could not extract the main WebView initialization script.");

const frameRule = init.match(/#__widget_drag_frame\{([^}]*)\}/)?.[1] ?? "";
const hoverRule = init.match(/#__widget_drag_frame:hover\{([^}]*)\}/)?.[1] ?? "";
const lockedRule = init.match(/#__widget_drag_frame\.__widget_drag_locked\{([^}]*)\}/)?.[1] ?? "";

const checks = [
  ["frame stays visually transparent but keeps its hit-target geometry",
    init.includes("border:'4px solid transparent'") &&
    frameRule.includes("border:4px solid transparent") &&
    frameRule.includes("box-shadow:none") &&
    frameRule.includes("pointer-events:none")],
  ["hover and locked states never reveal a visible frame",
    hoverRule.includes("border-color:transparent") && hoverRule.includes("box-shadow:none") &&
    lockedRule.includes("border-color:transparent") && lockedRule.includes("box-shadow:none")],
  ["top handle remains visible, labelled and draggable",
    init.includes("handle.textContent='⋮⋮'") &&
    init.includes("handle.style.setProperty('background'") &&
    init.includes("handle.addEventListener('pointerdown',dragWindow)")],
  ["all four transparent perimeter drag targets remain interactive",
    init.includes("for(const side of ['top','bottom','left','right'])") &&
    init.includes("edge.className='__widget_drag_edge __widget_drag_edge--'+side;") &&
    init.includes("edge.addEventListener('pointerdown',dragWindow)") &&
    init.includes("'pointer-events':'auto'")],
  ["drag requires a trusted primary-button gesture", init.includes("if(!event.isTrusted||event.button!==0)return;")],
  ["drag action uses the per-window token", init.includes("widget-action://'+ACTION_TOKEN")],
  ["lock state disables drag hit targets",
    init.includes("edge.style.setProperty('pointer-events',locked?'none':'auto')") &&
    init.includes("handle.style.setProperty('pointer-events',locked?'none':'auto')")],
  ["Rust validates lock state before processing window actions",
    windowSource.includes("Ação de janela ignorada porque mover/redimensionar está travado")]
];

const failed = checks.filter(([, passed]) => !passed).map(([name]) => name);
if (failed.length) throw new Error("Drag overlay check failed: " + failed.join("; "));

console.log("Drag overlay smoke check passed: invisible frame, interactive transparent edges, draggable top handle, trusted gesture, token authorization and lock enforcement.");
