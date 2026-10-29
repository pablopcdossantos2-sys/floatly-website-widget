use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex, OnceLock,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const HEARTBEAT_TIMEOUT_MS: u64 = 20_000;
const REPEAT_WARNING_MS: u64 = 60_000;
const MAX_LOG_BYTES: u64 = 1_048_576;

static LOG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static LAST_HEARTBEAT_MS: AtomicU64 = AtomicU64::new(0);
static LAST_WARNING_MS: AtomicU64 = AtomicU64::new(0);
static HEARTBEAT_SEEN: AtomicBool = AtomicBool::new(false);
static WATCHDOG_ALERTED: AtomicBool = AtomicBool::new(false);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn utc_timestamp() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = elapsed.as_secs();
    let days = (seconds / 86_400) as i64;
    let day_seconds = (seconds % 86_400) as i64;

    // Convert the Unix day count to a Gregorian date without adding a dependency.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_era - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        day_seconds / 3_600,
        (day_seconds % 3_600) / 60,
        day_seconds % 60,
        elapsed.subsec_millis()
    )
}

pub fn log_path(dir: &Path) -> PathBuf {
    dir.join("diagnostics.log")
}

pub fn log(dir: &Path, level: &str, message: impl AsRef<str>) {
    let timestamp = utc_timestamp();
    let line = format!("[{timestamp}][{level}] {}\n", message.as_ref());

    // Logging should never crash the widget, even when the profile directory is unavailable.
    if let Some(lock) = LOG_LOCK.get_or_init(|| Mutex::new(())).lock().ok() {
        let path = log_path(dir);
        if fs::create_dir_all(dir).is_ok() {
            if fs::metadata(&path).map(|m| m.len() >= MAX_LOG_BYTES).unwrap_or(false) {
                let backup = dir.join("diagnostics.log.1");
                let _ = fs::remove_file(&backup);
                let _ = fs::rename(&path, &backup);
            }
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
                let _ = file.write_all(line.as_bytes());
                let _ = file.flush();
            }
        }
        drop(lock);
    }

    // This is visible when the app is started from a console-attached process.
    // The file above remains the reliable source for GUI-subsystem builds.
    let mut output = std::io::stdout().lock();
    let _ = output.write_all(line.as_bytes());
    let _ = output.flush();
}

pub fn log_navigation(dir: &Path, url: &url::Url) {
    // Do not write query strings or fragments: they can contain session or auth tokens.
    log(
        dir,
        "NAV",
        format!(
            "Navegação solicitada: scheme={} host={} path={}",
            url.scheme(),
            url.host_str().unwrap_or("(sem host)"),
            url.path()
        ),
    );
}

pub fn record_heartbeat(dir: &Path) {
    LAST_HEARTBEAT_MS.store(now_ms(), Ordering::SeqCst);
    let first = !HEARTBEAT_SEEN.swap(true, Ordering::SeqCst);
    let recovered = WATCHDOG_ALERTED.swap(false, Ordering::SeqCst);
    if recovered {
        LAST_WARNING_MS.store(0, Ordering::SeqCst);
        log(
            dir,
            "RECUPERADO",
            "O sinal de atividade do JavaScript/WebView2 voltou a ser recebido.",
        );
    } else if first {
        log(
            dir,
            "INFO",
            "Primeiro sinal de atividade do JavaScript/WebView2 recebido.",
        );
    }
}

pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log(
            &crate::config::app_directory(),
            "PANIC",
            format!("Falha inesperada no processo Rust: {info}"),
        );
        previous(info);
    }));
}

pub fn start_watchdog(app: AppHandle, dir: PathBuf) {
    // If the first page load is still in progress, the initial grace period starts now.
    let now = now_ms();
    let _ = LAST_HEARTBEAT_MS.compare_exchange(0, now, Ordering::SeqCst, Ordering::SeqCst);
    log(
        &dir,
        "INFO",
        format!(
            "Monitor de atividade iniciado. Log persistente: {}",
            log_path(&dir).display()
        ),
    );

    let result = thread::Builder::new()
        .name("widget-webview-watchdog".to_owned())
        .spawn(move || loop {
            thread::sleep(Duration::from_secs(5));
            let now = now_ms();

            // A widget hidden in the system tray is not useful for UI hang detection;
            // reset the grace clock while hidden to avoid false positives from timer throttling.
            if let Some(window) = app.get_webview_window("main") {
                if matches!(window.is_visible(), Ok(false)) {
                    LAST_HEARTBEAT_MS.store(now, Ordering::SeqCst);
                    LAST_WARNING_MS.store(0, Ordering::SeqCst);
                    WATCHDOG_ALERTED.store(false, Ordering::SeqCst);
                    continue;
                }
            }

            let last = LAST_HEARTBEAT_MS.load(Ordering::SeqCst);
            let age = now.saturating_sub(last);
            if age < HEARTBEAT_TIMEOUT_MS {
                continue;
            }

            let previous_warning = LAST_WARNING_MS.load(Ordering::SeqCst);
            if previous_warning != 0 && now.saturating_sub(previous_warning) < REPEAT_WARNING_MS {
                continue;
            }

            let native_status = native_window_status(&app);
            log(
                &dir,
                "WARN",
                format!(
                    "Sem sinal de atividade do JavaScript/WebView2 há aproximadamente {} s. {}. Isso indica uma possível página/renderização travada ou bloqueio de navegação; o log, isoladamente, não prova que outro navegador causou o problema.",
                    age / 1_000,
                    native_status
                ),
            );
            LAST_WARNING_MS.store(now, Ordering::SeqCst);
            WATCHDOG_ALERTED.store(true, Ordering::SeqCst);
        });

    if let Err(error) = result {
        log(
            &dir,
            "ERRO",
            format!("Não foi possível iniciar o monitor de atividade: {error}"),
        );
    }
}

#[cfg(windows)]
fn native_window_status(app: &AppHandle) -> String {
    use windows_sys::Win32::UI::WindowsAndMessaging::IsHungAppWindow;

    let Some(window) = app.get_webview_window("main") else {
        return "A janela principal não foi encontrada".to_owned();
    };
    match window.hwnd() {
        Ok(hwnd) => {
            let hung = unsafe { IsHungAppWindow(hwnd.0) != 0 };
            if hung {
                "O Windows também marca a janela nativa como sem resposta".to_owned()
            } else {
                "A janela nativa ainda responde às mensagens do Windows; o bloqueio pode estar no conteúdo WebView2".to_owned()
            }
        }
        Err(error) => format!("Não foi possível inspecionar a janela nativa: {error}"),
    }
}

#[cfg(not(windows))]
fn native_window_status(_app: &AppHandle) -> String {
    "Verificação nativa de janela indisponível nesta plataforma".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn writes_a_persistent_diagnostic_line() {
        let dir = std::env::temp_dir().join(format!(
            "widget-website-diagnostics-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        log(&dir, "TEST", "diagnostic log test");
        let contents = fs::read_to_string(log_path(&dir)).unwrap();
        assert!(contents.contains("[TEST] diagnostic log test"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn navigation_log_omits_query_and_fragment() {
        let dir = std::env::temp_dir().join(format!(
            "widget-website-navigation-log-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let url = url::Url::parse(
            "https://example.com/login?token=secret-value#private-fragment",
        )
        .unwrap();
        log_navigation(&dir, &url);
        let contents = fs::read_to_string(log_path(&dir)).unwrap();
        assert!(contents.contains("host=example.com path=/login"));
        assert!(!contents.contains("secret-value"));
        assert!(!contents.contains("private-fragment"));
        fs::remove_dir_all(dir).unwrap();
    }
}
