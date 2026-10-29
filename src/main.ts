import { invoke } from "@tauri-apps/api/core";

type Language = "pt-BR" | "en" | "es";
type Cfg = { site_url: string; language?: string; [key: string]: unknown };

const status = document.querySelector<HTMLParagraphElement>("#status")!;
const retry = document.querySelector<HTMLButtonElement>("#retry")!;
const title = document.querySelector<HTMLHeadingElement>(".status-panel h1")!;
const footer = document.querySelector<HTMLElement>(".status-panel small")!;
const translations: Record<Language, { title: string; checking: string; opening: string; failed: string; retry: string; footer: string }> = {
  "pt-BR": {
    title: "Calendário econômico",
    checking: "Verificando a conexão com o site…",
    opening: "Abrindo o calendário…",
    failed: "Não foi possível acessar o site. Verifique a conexão e tente novamente.",
    retry: "Tentar novamente",
    footer: "Widget Website · sessão persistente do WebView2"
  },
  en: {
    title: "Economic calendar",
    checking: "Checking the website connection…",
    opening: "Opening the calendar…",
    failed: "Could not access the website. Check your connection and try again.",
    retry: "Try again",
    footer: "Widget Website · persistent WebView2 session"
  },
  es: {
    title: "Calendario económico",
    checking: "Comprobando la conexión con el sitio…",
    opening: "Abriendo el calendario…",
    failed: "No se pudo acceder al sitio. Comprueba la conexión e inténtalo de nuevo.",
    retry: "Reintentar",
    footer: "Widget Website · sesión persistente de WebView2"
  }
};

let language: Language = "pt-BR";
let timer: number | undefined;
let target = "https://br.investing.com/economic-calendar";
let navigating = false;

function text(key: "title" | "checking" | "opening" | "failed" | "retry" | "footer"): string {
  return translations[language][key];
}
function applyLanguage(value?: string) {
  language = value === "en" || value === "es" ? value : "pt-BR";
  document.documentElement.lang = language;
  title.textContent = text("title");
  footer.textContent = text("footer");
  retry.textContent = text("retry");
}

function failed() {
  status.textContent = text("failed");
  retry.hidden = false;
  if (timer === undefined) timer = window.setInterval(() => void attempt(true), 30000);
}

async function attempt(quiet = false) {
  if (navigating) return;
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

retry.addEventListener("click", () => void attempt());

void (async () => {
  try {
    const config = await invoke<Cfg>("get_config");
    if (config.site_url) target = config.site_url;
    applyLanguage(config.language);
  } catch {
    applyLanguage("pt-BR");
  }
  void attempt();
})();
