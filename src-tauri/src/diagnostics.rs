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
const MAX_LOG_HARD_BYTES: u64 = MAX_LOG_BYTES * 4;
const MAX_LOG_LINE_CHARS: usize = 1_024;
const MAX_LOG_LINES_PER_SECOND: u32 = 20;
const MAX_LOG_BACKUPS: usize = 3;

static LOG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static LOG_RATE_LOCK: OnceLock<Mutex<(u64, u32)>> = OnceLock::new();
static WATCHDOG_STATE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
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
    format_utc_timestamp(elapsed.as_secs(), elapsed.subsec_millis())
}

fn format_utc_timestamp(seconds: u64, millis: u32) -> String {
    let days = (seconds / 86_400) as i64;
    let day_seconds = (seconds % 86_400) as i64;

    // Convert Unix days to a Gregorian date (Howard Hinnant's civil_from_days).
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year,
        month,
        day,
        day_seconds / 3_600,
        (day_seconds % 3_600) / 60,
        day_seconds % 60,
        millis
    )
}

pub fn log_path(dir: &Path) -> PathBuf {
    dir.join("diagnostics.log")
}

fn sanitize_log_message(message: &str) -> String {
    message
        .chars()
        .map(|character| if character.is_control() { ' ' } else { character })
        .take(MAX_LOG_LINE_CHARS)
        .collect()
}

fn should_mirror_to_stdout() -> bool {
    cfg!(debug_assertions)
}

fn allow_log_line(level: &str) -> bool {
    if matches!(level, "ERRO" | "FATAL" | "PANIC") {
        return true;
    }
    let second = now_ms() / 1_000;
    let mut state = LOG_RATE_LOCK
        .get_or_init(|| Mutex::new((0, 0)))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if state.0 != second {
        *state = (second, 0);
    }
    if state.1 >= MAX_LOG_LINES_PER_SECOND {
        return false;
    }
    state.1 += 1;
    true
}

fn prune_log_backups(dir: &Path) {
    let mut backups = fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let suffix = name.strip_prefix("diagnostics.log.")?;
            if suffix.is_empty() || !suffix.chars().all(|c| c.is_ascii_digit() || c == '.') {
                return None;
            }
            Some((name, entry.path()))
        })
        .collect::<Vec<_>>();
    backups.sort_by(|left, right| left.0.cmp(&right.0));
    let excess = backups.len().saturating_sub(MAX_LOG_BACKUPS);
    for (_, path) in backups.into_iter().take(excess) {
        let _ = fs::remove_file(path);
    }
}

/// Returns false only when the file is beyond its hard cap and cannot be safely reset.
fn rotate_log_if_needed(dir: &Path, path: &Path) -> bool {
    let size = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return true,
        Err(_) => return true,
    };
    if size < MAX_LOG_BYTES {
        return true;
    }
    let backup = dir.join(format!("diagnostics.log.{}.{}", now_ms(), std::process::id()));
    match fs::rename(path, &backup) {
        Ok(()) => {
            prune_log_backups(dir);
            true
        }
        Err(error) if size >= MAX_LOG_HARD_BYTES => {
            if let Ok(mut file) = OpenOptions::new().write(true).truncate(true).open(path) {
                let warning = format!(
                    "[{}][WARN] A rotação do log falhou ({error}); o log atual foi reiniciado para respeitar o limite máximo.\n",
                    utc_timestamp()
                );
                if file.write_all(warning.as_bytes()).is_ok() {
                    let _ = file.flush();
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        Err(_) => true,
    }
}

pub fn log(dir: &Path, level: &str, message: impl AsRef<str>) {
    if !allow_log_line(level) {
        return;
    }
    let timestamp = utc_timestamp();
    let safe_message = sanitize_log_message(message.as_ref());
    let line = format!("[{timestamp}][{level}] {safe_message}\n");

    // Logging should never crash the widget, even when the profile directory is unavailable.
    let lock = LOG_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let path = log_path(dir);
    if fs::create_dir_all(dir).is_ok() && rotate_log_if_needed(dir, &path) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }
    drop(lock);

    if should_mirror_to_stdout() {
        let stdout = std::io::stdout();
        let mut output = stdout.lock();
        let _ = output.write_all(line.as_bytes());
        let _ = output.flush();
    }
}

pub fn sanitize_url(url: &url::Url) -> String {
    // Never include credentials, query strings, fragments, or an unbounded path.
    let path: String = url.path().chars().take(96).collect();
    let path = if url.path().chars().count() > 96 {
        format!("{path}…")
    } else {
        path
    };
    let port = url.port().map(|port| format!(":{port}")).unwrap_or_default();
    format!(
        "scheme={} host={}{} path={}",
        url.scheme(),
        url.host_str().unwrap_or("(sem host)"),
        port,
        sanitize_log_message(&path)
    )
}

pub fn log_navigation(dir: &Path, url: &url::Url) {
    log(dir, "NAV", format!("Navegação solicitada: {}", sanitize_url(url)));
}

pub fn safe_url_summary(raw_url: &str) -> String {
    match url::Url::parse(raw_url) {
        Ok(url) => sanitize_url(&url),
        Err(_) => "(URL inválida; detalhes omitidos)".to_owned(),
    }
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

    let error_log_dir = dir.clone();
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
            &error_log_dir,
            "ERRO",
            format!("Não foi possível iniciar o monitor de atividade: {error}"),
        );
    }
}

#[cfg(windows)]
fn native_window_status(app: &AppHandle) -> String {
    use windows_sys::Win32::UI::WindowsAndMessaging::IsHungAppWindow;

    let window_status = if let Some(window) = app.get_webview_window("main") {
        match window.hwnd() {
            Ok(hwnd) => {
                let hung = unsafe { IsHungAppWindow(hwnd.0) != 0 };
                if hung {
                    "O Windows também marca a janela nativa como sem resposta".to_owned()
                } else {
                    "A janela nativa ainda responde às mensagens do Windows".to_owned()
                }
            }
            Err(error) => format!("Não foi possível inspecionar a janela nativa: {error}"),
        }
    } else {
        "A janela principal não foi encontrada".to_owned()
    };

    format!(
        "{window_status}. {}",
        webview2_descendant_process_status(std::process::id())
    )
}

#[cfg(windows)]
fn webview2_descendant_process_status(app_pid: u32) -> String {
    use std::collections::HashSet;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot as isize == -1 {
        return "O Windows não permitiu consultar a árvore de processos".to_owned();
    }

    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    let mut processes: Vec<(u32, u32, String)> = Vec::new();
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;

    while has_entry {
        let length = entry
            .szExeFile
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(entry.szExeFile.len());
        let name = String::from_utf16_lossy(&entry.szExeFile[..length]);
        processes.push((entry.th32ProcessID, entry.th32ParentProcessID, name));
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }
    unsafe {
        let _ = CloseHandle(snapshot);
    }

    if processes.is_empty() {
        return "A lista de processos retornada pelo Windows estava vazia".to_owned();
    }

    // Include descendants of the widget, such as WebView2's browser and renderer processes.
    let mut descendants = HashSet::new();
    descendants.insert(app_pid);
    loop {
        let mut changed = false;
        for (pid, parent_pid, _) in &processes {
            if descendants.contains(parent_pid) && descendants.insert(*pid) {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let count = processes
        .iter()
        .filter(|(pid, _, name)| {
            *pid != app_pid
                && descendants.contains(pid)
                && name.eq_ignore_ascii_case("msedgewebview2.exe")
        })
        .count();

    if count > 0 {
        format!("Foram encontrados {count} processo(s) msedgewebview2.exe na árvore do widget")
    } else {
        "Nenhum msedgewebview2.exe descendente do widget foi encontrado; isso pode indicar que o processo WebView2 terminou ou foi bloqueado".to_owned()
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
    fn formats_utc_timestamp_from_unix_epoch() {
        assert_eq!(format_utc_timestamp(0, 123), "1970-01-01T00:00:00.123Z");
    }

    #[test]
    fn formats_leap_day_and_modern_year_correctly() {
        assert_eq!(
            format_utc_timestamp(951_782_400, 0),
            "2000-02-29T00:00:00.000Z"
        );
        assert_eq!(
            format_utc_timestamp(1_793_291_247, 251),
            "2026-10-29T16:27:27.251Z"
        );
    }

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
    fn configured_url_summary_omits_query_and_fragment() {
        let summary =
            safe_url_summary("https://example.com/login?token=secret-value#private-fragment");
        assert_eq!(summary, "scheme=https host=example.com path=/login");
        assert!(!summary.contains("secret-value"));
        assert!(!summary.contains("private-fragment"));
    }

    #[test]
    fn navigation_log_omits_query_and_fragment() {
        let dir = std::env::temp_dir().join(format!(
            "widget-website-navigation-log-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let url = url::Url::parse("https://example.com/login?token=secret-value#private-fragment")
            .unwrap();
        log_navigation(&dir, &url);
        let contents = fs::read_to_string(log_path(&dir)).unwrap();
        assert!(contents.contains("host=example.com path=/login"));
        assert!(!contents.contains("secret-value"));
        assert!(!contents.contains("private-fragment"));
        fs::remove_dir_all(dir).unwrap();
    }
}
