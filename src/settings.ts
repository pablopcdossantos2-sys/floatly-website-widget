import { invoke } from "@tauri-apps/api/core";

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
    lockedHelp: "Ative para bloquear o movimento e o redimensionamento. Desative para habilitar o arraste pela borda azul ao redor do widget.",
    autostart: "Iniciar com o Windows", autostartHelp: "Usa o registro do usuário atual, sem exigir administrador.",
    appearanceSection: "Aparência e atualização", opacity: "Opacidade",
    zoom: "Zoom do navegador embutido",
    zoomHelp: "Ampliação do WebView2 entre 50% e 200%, independente do CSS do site.",
    cornerRadius: "Raio dos cantos (px)", reload: "Recarregar a cada (min)",
    reloadHelp: "0 desliga; valores ativos abaixo de 5 são ajustados para 5.",
    shortcutSection: "Atalho global", shortcutCombination: "Combinação de teclas",
    shortcutHelp: "Esse atalho mostra ou oculta o widget mesmo quando você está usando outro programa. Clique no campo e pressione a combinação desejada (ex.: Ctrl + Alt + C).",
    reset: "Restaurar tamanho padrão", close: "Fechar", save: "Salvar alterações",
    saving: "Salvando…", saved: "Configurações salvas.", resetSaved: "Tamanho padrão restaurado.",
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
    lockedHelp: "Enable this option to lock movement and resizing. Turn it off to move the widget using its blue border.",
    autostart: "Start with Windows", autostartHelp: "Uses the current user's startup settings; administrator rights are not required.",
    appearanceSection: "Appearance and refresh", opacity: "Opacity",
    zoom: "Embedded browser zoom",
    zoomHelp: "Controls WebView2 zoom from 50% to 200%, independently of the website's CSS.",
    cornerRadius: "Corner radius (px)", reload: "Reload every (min)",
    reloadHelp: "0 disables automatic refresh; active values below 5 are raised to 5.",
    shortcutSection: "Global shortcut", shortcutCombination: "Key combination",
    shortcutHelp: "This shortcut shows or hides the widget even while you are using another application. Click the field and press the desired combination (e.g. Ctrl + Alt + C).",
    reset: "Restore default size", close: "Close", save: "Save changes",
    saving: "Saving…", saved: "Settings saved.", resetSaved: "Default size restored.",
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
    lockedHelp: "Activa esta opción para bloquear el movimiento y el cambio de tamaño. Desactívala para mover el widget desde el borde azul.",
    autostart: "Iniciar con Windows", autostartHelp: "Usa la configuración de inicio del usuario actual, sin permisos de administrador.",
    appearanceSection: "Apariencia y actualización", opacity: "Opacidad",
    zoom: "Zoom del navegador integrado",
    zoomHelp: "Controla el zoom de WebView2 entre el 50 % y el 200 %, independientemente del CSS del sitio.",
    cornerRadius: "Radio de las esquinas (px)", reload: "Actualizar cada (min)",
    reloadHelp: "0 desactiva la actualización automática; los valores activos inferiores a 5 se ajustan a 5.",
    shortcutSection: "Atajo global", shortcutCombination: "Combinación de teclas",
    shortcutHelp: "Este atajo muestra u oculta el widget aunque estés usando otra aplicación. Haz clic en el campo y pulsa la combinación deseada (p. ej., Ctrl + Alt + C).",
    reset: "Restaurar tamaño predeterminado", close: "Cerrar", save: "Guardar cambios",
    saving: "Guardando…", saved: "Configuración guardada.", resetSaved: "Se restauró el tamaño predeterminado.",
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
function display(config: C) {
  for (const id of ids) {
    const element = f(id);
    const value = config[id as keyof C];
    if (element instanceof HTMLInputElement && element.type === "checkbox") {
      element.checked = Boolean(value);
    } else {
      element.value = String(value ?? "");
    }
  }
  document.getElementById("opacity-value")!.textContent = config.opacity + "%";
  document.getElementById("zoom-value")!.textContent = config.zoom + "%";
  applyLanguage(config.language);
}
async function load() {
  current = await invoke<C>("get_config");
  display(current);
}
function read(): C {
  const value = (id: string) => f(id).value;
  const checked = (id: string) => f(id).checked;
  return {
    site_url: value("site_url"),
    width: Number(value("width")),
    height: Number(value("height")),
    x: current?.x ?? null,
    y: current?.y ?? null,
    remember_position: checked("remember_position"),
    autostart: checked("autostart"),
    always_on_top: checked("always_on_top"),
    opacity: Number(value("opacity")),
    zoom: Number(value("zoom")),
    auto_reload_minutes: Number(value("auto_reload_minutes")),
    show_in_taskbar: checked("show_in_taskbar"),
    locked: checked("locked"),
    global_shortcut: value("global_shortcut"),
    corner_radius: Number(value("corner_radius")),
    language: f<HTMLSelectElement>("language").value as Language
  };
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  say(t("saving"));
  try {
    const result = await invoke<S>("save_config", { config: read() });
    current = result.config;
    display(result.config);
    if (result.warning) {
      const warning = result.warning.includes("atalho")
        ? t("shortcutWarning")
        : result.warning.includes("início") ? t("autostartWarning") : result.warning;
      say(warning, "warning");
    } else {
      say(t("saved"));
    }
  } catch (error) {
    say(t("invalidConfig") + String(error), "error");
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
f("global_shortcut").addEventListener("keydown", (event) => {
  event.preventDefault();
  if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) return;
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("CTRL");
  if (event.altKey) parts.push("ALT");
  if (event.shiftKey) parts.push("SHIFT");
  if (event.metaKey) parts.push("SUPER");
  const map: Record<string, string> = {
    Escape: "ESC", " ": "SPACE", ArrowUp: "UP", ArrowDown: "DOWN",
    ArrowLeft: "LEFT", ArrowRight: "RIGHT", PageUp: "PAGEUP", PageDown: "PAGEDOWN"
  };
  parts.push(event.key.length === 1 ? event.key.toUpperCase() : (map[event.key] || event.key.toUpperCase()));
  f("global_shortcut").value = parts.join("+");
});
document.getElementById("reset")!.addEventListener("click", async () => {
  try {
    const result = await invoke<S>("restore_default_size");
    current = result.config;
    display(result.config);
    say(result.warning || t("resetSaved"), result.warning ? "warning" : "");
  } catch (error) {
    say(String(error), "error");
  }
});
document.getElementById("close")!.addEventListener("click", () => {
  void invoke("close_settings");
});

document.getElementById("theme-light")!.addEventListener("click", () => setSettingsTheme("light"));
document.getElementById("theme-dark")!.addEventListener("click", () => setSettingsTheme("dark"));
try {
  setSettingsTheme(localStorage.getItem(themePreferenceKey) === "dark" ? "dark" : "light", false);
} catch {
  setSettingsTheme("light", false);
}

void load().catch((error) => say(t("loadFailed") + String(error), "error"));
