import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type Language = "pt-BR" | "en" | "es";
type C = {
  site_url: string;
  width: number;
  height: number;
  x: number | null;
  y: number | null;
  remember_position: boolean;
  autostart: boolean;
  always_on_top: boolean;
  opacity: number;
  zoom: number;
  auto_reload_minutes: number;
  show_in_taskbar: boolean;
  locked: boolean;
  global_shortcut: string;
  corner_radius: number;
  language: Language;
};
type S = { config: C; warning: string | null };

const translations: Record<Language, Record<string, string>> = {
  "pt-BR": {
    pageTitle: "Configurações", subtitle: "Personalize a janela do calendário.",
    themeGroup: "Tema da interface", themeLight: "Modo claro", themeDark: "Modo escuro",
    languageSection: "Idioma", languageLabel: "Idioma do programa",
    languageHelp: "Altera o idioma das configurações e dos menus do Widget Website.",
    siteSection: "Site", siteUrl: "URL do site", width: "Largura (px)", height: "Altura (px)",
    windowSection: "Janela", rememberPosition: "Lembrar posição e tamanho",
    rememberPositionHelp: "Reposiciona a janela se o monitor salvo não estiver conectado.",
    alwaysOnTop: "Sempre no topo", alwaysOnTopHelp: "Mantém o widget acima de outras janelas.",
    taskbar: "Mostrar na barra de tarefas", taskbarHelp: "Se desativado, use o ícone da bandeja para reabrir.",
    locked: "Travar mover e redimensionar",
    lockedHelp: "Ative para bloquear o movimento e o redimensionamento. Desative para mover o widget pela aba superior; as bordas transparentes também aceitam arraste.",
    autostart: "Iniciar com o Windows", autostartHelp: "Usa o registro do usuário atual, sem exigir administrador.",
    appearanceSection: "Aparência e atualização", opacity: "Opacidade",
    zoom: "Zoom do navegador embutido",
    zoomHelp: "Ampliação do WebView2 entre 50% e 200%, independente do CSS do site.",
    cornerRadius: "Raio dos cantos (px)", reload: "Recarregar a cada (min)",
    reloadHelp: "0 desliga; valores ativos abaixo de 5 são ajustados para 5.",
    shortcutSection: "Atalho global", shortcutCombination: "Combinação de teclas", shortcutReset: "Restaurar atalho padrão",
    shortcutHelp: "Esse atalho mostra ou oculta o widget mesmo quando você está usando outro programa. Clique no campo e pressione a combinação desejada (ex.: Ctrl + Alt + C).",
    reset: "Restaurar tamanho padrão", close: "Fechar", save: "Salvar alterações",
    saving: "Salvando…", saved: "Configurações salvas.", resetSaved: "Dimensões padrão preenchidas. Clique em Salvar alterações para aplicar.",
    fieldRequired: "Preencha os campos numéricos obrigatórios.", fieldRange: "Use um número inteiro dentro dos limites indicados.",
    siteUrlRequired: "Informe a URL do site.", siteUrlInvalid: "A URL informada é inválida.", siteUrlHttpOnly: "Use uma URL HTTP ou HTTPS válida, sem credenciais.", configReadFailed: "Não foi possível ler a configuração.", configWriteFailed: "Não foi possível atualizar a configuração.", configDiskWriteFailed: "Não foi possível gravar o arquivo de configuração.", shortcutModifierRequired: "Inclua pelo menos um modificador (Ctrl, Alt, Shift ou Win).", shortcutKeyUnsupported: "Essa tecla não é compatível com o atalho global.",
    autostartEnableWarning: "Não foi possível habilitar o início com o Windows.", autostartDisableWarning: "Não foi possível desabilitar o início com o Windows.",
    loadFailed: "Não foi possível ler as configurações: ", invalidConfig: "Não foi possível salvar as configurações: ",
    shortcutWarning: "O novo atalho conflita com outro aplicativo; o atalho anterior foi mantido.",
    autostartWarning: "Não foi possível alterar a configuração de início com o Windows.",
    shortcutDragHint: "Clique no campo e pressione a combinação desejada (ex.: Ctrl + Alt + C)."
  },
  en: {
    pageTitle: "Settings", subtitle: "Customize the calendar window.",
    themeGroup: "Settings appearance", themeLight: "Light mode", themeDark: "Dark mode",
    languageSection: "Language", languageLabel: "Application language",
    languageHelp: "Changes the language of Widget Website settings and menus.",
    siteSection: "Website", siteUrl: "Website URL", width: "Width (px)", height: "Height (px)",
    windowSection: "Window", rememberPosition: "Remember position and size",
    rememberPositionHelp: "Repositions the window if the saved monitor is no longer connected.",
    alwaysOnTop: "Always on top", alwaysOnTopHelp: "Keeps the widget above other windows.",
    taskbar: "Show in taskbar", taskbarHelp: "When disabled, use the system tray icon to show the widget again.",
    locked: "Lock moving and resizing",
    lockedHelp: "Enable this option to lock movement and resizing. Turn it off to move the widget using the top handle; the transparent edges also remain draggable.",
    autostart: "Start with Windows", autostartHelp: "Uses the current user's startup settings; administrator rights are not required.",
    appearanceSection: "Appearance and refresh", opacity: "Opacity",
    zoom: "Embedded browser zoom",
    zoomHelp: "Controls WebView2 zoom from 50% to 200%, independently of the website's CSS.",
    cornerRadius: "Corner radius (px)", reload: "Reload every (min)",
    reloadHelp: "0 disables automatic refresh; active values below 5 are raised to 5.",
    shortcutSection: "Global shortcut", shortcutCombination: "Key combination", shortcutReset: "Restore default shortcut",
    shortcutHelp: "This shortcut shows or hides the widget even while you are using another application. Click the field and press the desired combination (e.g. Ctrl + Alt + C).",
    reset: "Restore default size", close: "Close", save: "Save changes",
    saving: "Saving…", saved: "Settings saved.", resetSaved: "Default dimensions filled in. Click Save changes to apply them.",
    fieldRequired: "Fill in all required numeric fields.", fieldRange: "Enter an integer within the indicated limits.",
    siteUrlRequired: "Enter the website URL.", siteUrlInvalid: "The supplied URL is invalid.", siteUrlHttpOnly: "Use a valid HTTP or HTTPS URL without credentials.", configReadFailed: "Could not read the configuration.", configWriteFailed: "Could not update the configuration.", configDiskWriteFailed: "Could not write the configuration file.", shortcutModifierRequired: "Include at least one modifier (Ctrl, Alt, Shift or Win).", shortcutKeyUnsupported: "This key is not supported for the global shortcut.",
    autostartEnableWarning: "Could not enable startup with Windows.", autostartDisableWarning: "Could not disable startup with Windows.",
    loadFailed: "Could not load settings: ", invalidConfig: "Could not save settings: ",
    shortcutWarning: "The new shortcut conflicts with another application; the previous shortcut was kept.",
    autostartWarning: "Could not change the Windows startup setting.",
    shortcutDragHint: "Click the field and press the desired combination (e.g. Ctrl + Alt + C)."
  },
  es: {
    pageTitle: "Configuración", subtitle: "Personaliza la ventana del calendario.",
    themeGroup: "Tema de la interfaz", themeLight: "Modo claro", themeDark: "Modo oscuro",
    languageSection: "Idioma", languageLabel: "Idioma del programa",
    languageHelp: "Cambia el idioma de la configuración y los menús de Widget Website.",
    siteSection: "Sitio web", siteUrl: "URL del sitio", width: "Ancho (px)", height: "Alto (px)",
    windowSection: "Ventana", rememberPosition: "Recordar posición y tamaño",
    rememberPositionHelp: "Reubica la ventana si el monitor guardado ya no está conectado.",
    alwaysOnTop: "Siempre visible", alwaysOnTopHelp: "Mantiene el widget por encima de otras ventanas.",
    taskbar: "Mostrar en la barra de tareas", taskbarHelp: "Si está desactivado, usa el icono de la bandeja para volver a mostrarlo.",
    locked: "Bloquear movimiento y cambio de tamaño",
    lockedHelp: "Activa esta opción para bloquear el movimiento y el cambio de tamaño. Desactívala para mover el widget desde el tirador superior; los bordes transparentes también permiten arrastrarlo.",
    autostart: "Iniciar con Windows", autostartHelp: "Usa la configuración de inicio del usuario actual, sin permisos de administrador.",
    appearanceSection: "Apariencia y actualización", opacity: "Opacidad",
    zoom: "Zoom del navegador integrado",
    zoomHelp: "Controla el zoom de WebView2 entre el 50 % y el 200 %, independientemente del CSS del sitio.",
    cornerRadius: "Radio de las esquinas (px)", reload: "Actualizar cada (min)",
    reloadHelp: "0 desactiva la actualización automática; los valores activos inferiores a 5 se ajustan a 5.",
    shortcutSection: "Atajo global", shortcutCombination: "Combinación de teclas", shortcutReset: "Restaurar atajo predeterminado",
    shortcutHelp: "Este atajo muestra u oculta el widget aunque estés usando otra aplicación. Haz clic en el campo y pulsa la combinación deseada (p. ej., Ctrl + Alt + C).",
    reset: "Restaurar tamaño predeterminado", close: "Cerrar", save: "Guardar cambios",
    saving: "Guardando…", saved: "Configuración guardada.", resetSaved: "Dimensiones predeterminadas completadas. Pulsa Guardar cambios para aplicarlas.",
    fieldRequired: "Completa los campos numéricos obligatorios.", fieldRange: "Introduce un número entero dentro de los límites indicados.",
    siteUrlRequired: "Introduce la URL del sitio.", siteUrlInvalid: "La URL indicada no es válida.", siteUrlHttpOnly: "Usa una URL HTTP o HTTPS válida y sin credenciales.", configReadFailed: "No se pudo leer la configuración.", configWriteFailed: "No se pudo actualizar la configuración.", configDiskWriteFailed: "No se pudo escribir el archivo de configuración.", shortcutModifierRequired: "Incluye al menos un modificador (Ctrl, Alt, Shift o Win).", shortcutKeyUnsupported: "Esta tecla no es compatible con el atajo global.",
    autostartEnableWarning: "No se pudo activar el inicio con Windows.", autostartDisableWarning: "No se pudo desactivar el inicio con Windows.",
    loadFailed: "No se pudo leer la configuración: ", invalidConfig: "No se pudo guardar la configuración: ",
    shortcutWarning: "El nuevo atajo entra en conflicto con otra aplicación; se mantuvo el atajo anterior.",
    autostartWarning: "No se pudo cambiar la opción de inicio con Windows.",
    shortcutDragHint: "Haz clic en el campo y pulsa la combinación deseada (p. ej., Ctrl + Alt + C)."
  }
};

const form = document.querySelector<HTMLFormElement>("#settings-form")!;
const msg = document.querySelector<HTMLDivElement>("#message")!;
const ids = [
  "site_url", "width", "height", "remember_position", "autostart", "always_on_top",
  "opacity", "zoom", "auto_reload_minutes", "show_in_taskbar", "locked",
  "global_shortcut", "corner_radius", "language"
];
let current: C | null = null;
let activeLanguage: Language = "pt-BR";
let forceResizeOnSave = false;
const dirtyFields = new Set<string>();
const DEFAULT_WIDGET_WIDTH = 350;
const DEFAULT_WIDGET_HEIGHT = 550;
type SettingsTheme = "light" | "dark";
const themePreferenceKey = "widget-website-settings-theme";

function setSettingsTheme(theme: SettingsTheme, persist = true) {
  document.body.dataset.theme = theme;
  const lightButton = document.getElementById("theme-light") as HTMLButtonElement;
  const darkButton = document.getElementById("theme-dark") as HTMLButtonElement;
  lightButton.setAttribute("aria-pressed", String(theme === "light"));
  darkButton.setAttribute("aria-pressed", String(theme === "dark"));
  if (persist) {
    try { localStorage.setItem(themePreferenceKey, theme); } catch { /* Storage may be unavailable. */ }
  }
}

function f<T extends HTMLInputElement | HTMLSelectElement = HTMLInputElement>(id: string): T {
  return document.getElementById(id) as T;
}
function t(key: string): string {
  return translations[activeLanguage][key] ?? translations["pt-BR"][key] ?? key;
}
const backendErrorKeys: Record<string, string> = {
  "config.site_url_required": "siteUrlRequired",
  "config.site_url_invalid": "siteUrlInvalid",
  "config.site_url_http_only": "siteUrlHttpOnly",
  "config.shortcut_modifier_required": "shortcutModifierRequired",
  "config.shortcut_key_unsupported": "shortcutKeyUnsupported",
  "config.read_failed": "configReadFailed",
  "config.write_failed": "configWriteFailed",
  "config.disk_write_failed": "configDiskWriteFailed"
};
function translateBackendError(error: unknown): string {
  const code = error instanceof Error ? error.message : String(error);
  const key = backendErrorKeys[code];
  return key ? t(key) : code;
}
function say(message: string, kind = "") {
  msg.textContent = message;
  msg.className = "message " + kind;
}
function applyLanguage(language: string) {
  activeLanguage = language === "en" || language === "es" ? language : "pt-BR";
  document.documentElement.lang = activeLanguage;
  document.title = activeLanguage === "en"
    ? "Settings — Widget Website"
    : activeLanguage === "es" ? "Configuración — Widget Website" : "Configurações — Widget Website";
  document.querySelectorAll<HTMLElement>("[data-i18n]").forEach((element) => {
    const key = element.dataset.i18n;
    if (key) element.textContent = t(key);
  });
  document.querySelectorAll<HTMLElement>("[data-i18n-aria-label]").forEach((element) => {
    const key = element.dataset.i18nAriaLabel;
    if (key) element.setAttribute("aria-label", t(key));
  });
}
function displayField(config: C, id: string) {
  const element = f(id);
  const value = config[id as keyof C];
  if (element instanceof HTMLInputElement && element.type === "checkbox") {
    element.checked = Boolean(value);
  } else {
    element.value = String(value ?? "");
  }
  if (id === "opacity") document.getElementById("opacity-value")!.textContent = config.opacity + "%";
  if (id === "zoom") document.getElementById("zoom-value")!.textContent = config.zoom + "%";
  if (id === "language") applyLanguage(config.language);
}
function display(config: C) {
  for (const id of ids) displayField(config, id);
}
async function load() {
  current = await invoke<C>("get_config");
  display(current);
}
function readInt(id: string, min: number, max: number): number {
  const raw = f(id).value.trim();
  if (!raw) throw new Error("FIELD_REQUIRED");
  const parsed = Number(raw);
  if (!Number.isInteger(parsed) || parsed < min || parsed > max) throw new Error("FIELD_RANGE");
  return parsed;
}
function read(): C {
  const value = (id: string) => f(id).value;
  const checked = (id: string) => f(id).checked;
  return {
    site_url: value("site_url"),
    width: readInt("width", 150, 2000),
    height: readInt("height", 100, 1600),
    x: current?.x ?? null,
    y: current?.y ?? null,
    remember_position: checked("remember_position"),
    autostart: checked("autostart"),
    always_on_top: checked("always_on_top"),
    opacity: Number(value("opacity")),
    zoom: Number(value("zoom")),
    auto_reload_minutes: readInt("auto_reload_minutes", 0, 1440),
    show_in_taskbar: checked("show_in_taskbar"),
    locked: checked("locked"),
    global_shortcut: value("global_shortcut"),
    corner_radius: readInt("corner_radius", 0, 64),
    language: f<HTMLSelectElement>("language").value as Language
  };
}

type ConfigPatch = Partial<Omit<C, "x" | "y">>;
function readPatch(): ConfigPatch {
  if (!current) throw new Error("CONFIG_NOT_LOADED");
  const candidate = read();
  const patch: ConfigPatch = {};
  if (candidate.site_url !== current.site_url) patch.site_url = candidate.site_url;
  if (candidate.width !== current.width) patch.width = candidate.width;
  if (candidate.height !== current.height) patch.height = candidate.height;
  if (candidate.remember_position !== current.remember_position) patch.remember_position = candidate.remember_position;
  if (candidate.autostart !== current.autostart) patch.autostart = candidate.autostart;
  if (candidate.always_on_top !== current.always_on_top) patch.always_on_top = candidate.always_on_top;
  if (candidate.opacity !== current.opacity) patch.opacity = candidate.opacity;
  if (candidate.zoom !== current.zoom) patch.zoom = candidate.zoom;
  if (candidate.auto_reload_minutes !== current.auto_reload_minutes) patch.auto_reload_minutes = candidate.auto_reload_minutes;
  if (candidate.show_in_taskbar !== current.show_in_taskbar) patch.show_in_taskbar = candidate.show_in_taskbar;
  if (candidate.locked !== current.locked) patch.locked = candidate.locked;
  if (candidate.global_shortcut !== current.global_shortcut) patch.global_shortcut = candidate.global_shortcut;
  if (candidate.corner_radius !== current.corner_radius) patch.corner_radius = candidate.corner_radius;
  if (candidate.language !== current.language) patch.language = candidate.language;
  return patch;
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  say(t("saving"));
  try {
    const result = await invoke<S>("save_config", { config: readPatch(), forceResize: forceResizeOnSave });
    current = result.config;
    forceResizeOnSave = false;
    dirtyFields.clear();
    display(result.config);
    if (result.warning) {
      const warningKeys: Record<string, string> = {
        shortcut_conflict: "shortcutWarning",
        autostart_enable_failed: "autostartEnableWarning",
        autostart_disable_failed: "autostartDisableWarning"
      };
      const warning = result.warning.split(/\s+/).map((code) => t(warningKeys[code] ?? "shortcutWarning")).join(" ");
      say(warning, "warning");
    } else {
      say(t("saved"));
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (message === "FIELD_REQUIRED") say(t("fieldRequired"), "error");
    else if (message === "FIELD_RANGE") say(t("fieldRange"), "error");
    else if (message === "CONFIG_NOT_LOADED") say(t("loadFailed") + t("configReadFailed"), "error");
    else {
      const key = backendErrorKeys[message];
      say(key ? t(key) : t("invalidConfig") + translateBackendError(error), "error");
    }
  }
});

f<HTMLSelectElement>("language").addEventListener("change", () => {
  applyLanguage(f<HTMLSelectElement>("language").value);
  say("");
});
f("opacity").addEventListener("input", (event) => {
  document.getElementById("opacity-value")!.textContent = (event.target as HTMLInputElement).value + "%";
});
f("zoom").addEventListener("input", (event) => {
  document.getElementById("zoom-value")!.textContent = (event.target as HTMLInputElement).value + "%";
});
function shortcutKeyFromCode(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^Numpad[0-9]$/.test(code)) return "NUMPAD" + code.slice(6);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  const named: Record<string, string> = {
    Escape: "ESC", Enter: "ENTER", Space: "SPACE", Backspace: "BACKSPACE",
    Delete: "DELETE", Insert: "INSERT", Home: "HOME", End: "END",
    PageUp: "PAGEUP", PageDown: "PAGEDOWN", ArrowUp: "UP", ArrowDown: "DOWN",
    ArrowLeft: "LEFT", ArrowRight: "RIGHT"
  };
  return named[code] ?? null;
}
f("global_shortcut").addEventListener("keydown", (event) => {
  if (event.key === "Tab") return;
  if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) return;
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("CTRL");
  if (event.altKey) parts.push("ALT");
  if (event.shiftKey) parts.push("SHIFT");
  if (event.metaKey) parts.push("SUPER");
  if (parts.length === 0) {
    event.preventDefault();
    say(t("shortcutModifierRequired"), "warning");
    return;
  }
  const primary = shortcutKeyFromCode(event.code);
  if (!primary) {
    event.preventDefault();
    say(t("shortcutKeyUnsupported"), "warning");
    return;
  }
  event.preventDefault();
  parts.push(primary);
  f("global_shortcut").value = parts.join("+");
  say("");
});
document.getElementById("reset")!.addEventListener("click", () => {
  f("width").value = String(DEFAULT_WIDGET_WIDTH);
  f("height").value = String(DEFAULT_WIDGET_HEIGHT);
  forceResizeOnSave = true;
  dirtyFields.add("width");
  dirtyFields.add("height");
  say(t("resetSaved"));
});
document.getElementById("close")!.addEventListener("click", () => {
  void invoke("close_settings");
});

document.getElementById("theme-light")!.addEventListener("click", () => setSettingsTheme("light"));
document.getElementById("theme-dark")!.addEventListener("click", () => setSettingsTheme("dark"));
document.getElementById("reset-shortcut")!.addEventListener("click", () => {
  f("global_shortcut").value = "CTRL+ALT+C";
  dirtyFields.add("global_shortcut");
  say("");
});
try {
  setSettingsTheme(localStorage.getItem(themePreferenceKey) === "dark" ? "dark" : "light", false);
} catch {
  setSettingsTheme("light", false);
}

form.addEventListener("input", (event) => {
  const target = event.target;
  if (target instanceof HTMLElement && target.id) dirtyFields.add(target.id);
});
form.addEventListener("change", (event) => {
  const target = event.target;
  if (target instanceof HTMLElement && target.id) dirtyFields.add(target.id);
});
void listen<C>("config-changed", (event) => {
  current = event.payload;
  for (const id of ids) {
    if (!dirtyFields.has(id)) displayField(event.payload, id);
  }
}).catch((error) => {
  say(t("loadFailed") + String(error), "error");
});

void load().catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  const key = backendErrorKeys[message];
  say(key ? t(key) : t("loadFailed") + translateBackendError(error), "error");
});
