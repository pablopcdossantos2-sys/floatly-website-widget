import { invoke } from "@tauri-apps/api/core";

type Language = "pt-BR" | "en" | "es";
type Cfg = { site_url: string; language?: string; [key: string]: unknown };
type MessageKey =
  | "title" | "configureTitle" | "configureMessage" | "configureButton"
  | "checking" | "opening" | "failed" | "retry" | "footer"
  | "settingsFailed" | "configFailed";

const status = document.querySelector<HTMLParagraphElement>("#status")!;
const retry = document.querySelector<HTMLButtonElement>("#retry")!;
const title = document.querySelector<HTMLHeadingElement>(".status-panel h1")!;
const footer = document.querySelector<HTMLElement>(".status-panel small")!;

const translations: Record<Language, Record<MessageKey, string>> = {
  "pt-BR": {
    title: "Floatly",
    configureTitle: "Escolha um site",
    configureMessage: "Informe o endereço de um site nas Configurações para mantê-lo visível nesta janela.",
    configureButton: "Escolher site",
    checking: "Verificando a conexão com o site…",
    opening: "Abrindo o site…",
    failed: "Não foi possível acessar o site configurado. Verifique a conexão e tente novamente.",
    retry: "Tentar novamente",
    footer: "Um site à sua escolha em uma janela flutuante",
    settingsFailed: "Não foi possível abrir as Configurações. Tente pelo menu da bandeja do sistema.",
    configFailed: "Não foi possível carregar a configuração. Abra as Configurações e informe o endereço do site."
  },
  en: {
    title: "Floatly",
    configureTitle: "Choose a website",
    configureMessage: "Enter a website address in Settings to keep it visible in this window.",
    configureButton: "Choose website",
    checking: "Checking the website connection…",
    opening: "Opening website…",
    failed: "Could not reach the configured website. Check your connection and try again.",
    retry: "Try again",
    footer: "A website of your choice in a floating window",
    settingsFailed: "Could not open Settings. Try using the system tray menu.",
    configFailed: "Could not load the configuration. Open Settings and enter a website address."
  },
  es: {
    title: "Floatly",
    configureTitle: "Elige un sitio web",
    configureMessage: "Introduce la dirección de un sitio web en Configuración para mantenerlo visible en esta ventana.",
    configureButton: "Elegir sitio web",
    checking: "Comprobando la conexión con el sitio web…",
    opening: "Abriendo el sitio web…",
    failed: "No se pudo acceder al sitio configurado. Comprueba la conexión e inténtalo de nuevo.",
    retry: "Reintentar",
    footer: "Un sitio web a tu elección en una ventana flotante",
    settingsFailed: "No se pudo abrir Configuración. Inténtalo desde el menú de la bandeja del sistema.",
    configFailed: "No se pudo cargar la configuración. Abre Configuración e introduce la dirección del sitio web."
  }
};

let language: Language = "pt-BR";
let target = "";
let timer: number | undefined;
let navigating = false;
let needsConfiguration = false;

function text(key: MessageKey): string {
  return translations[language][key];
}

function applyLanguage(value?: string) {
  language = value === "en" || value === "es" ? value : "pt-BR";
  document.documentElement.lang = language;
  title.textContent = text("title");
  footer.textContent = text("footer");
}

function showSetup(message?: string) {
  needsConfiguration = true;
  navigating = false;
  title.textContent = text("configureTitle");
  status.textContent = message ?? text("configureMessage");
  retry.textContent = text("configureButton");
  retry.hidden = false;
}

function failed() {
  needsConfiguration = false;
  status.textContent = text("failed");
  retry.textContent = text("retry");
  retry.hidden = false;
  if (timer === undefined) {
    timer = window.setInterval(() => void attempt(true), 30000);
  }
}

async function openSettings() {
  try {
    await invoke("open_settings");
  } catch {
    status.textContent = text("settingsFailed");
  }
}

async function attempt(quiet = false) {
  if (navigating) return;
  if (!target) {
    showSetup();
    return;
  }
  if (!quiet) {
    status.textContent = text("checking");
    retry.hidden = true;
  }
  try {
    const controller = new AbortController();
    const timeout = window.setTimeout(() => controller.abort(), 8000);
    try {
      await fetch(new URL("/favicon.ico", target).href, {
        mode: "no-cors",
        cache: "no-store",
        signal: controller.signal
      });
    } finally {
      window.clearTimeout(timeout);
    }
    navigating = true;
    status.textContent = text("opening");
    location.replace(target);
  } catch {
    navigating = false;
    failed();
  }
}

retry.addEventListener("click", () => {
  if (needsConfiguration) void openSettings();
  else void attempt();
});

void (async () => {
  try {
    const config = await invoke<Cfg>("get_config");
    applyLanguage(config.language);
    target = (config.site_url ?? "").trim();
    if (!target) {
      showSetup();
      return;
    }
    void attempt();
  } catch {
    applyLanguage("pt-BR");
    showSetup(text("configFailed"));
  }
})();
