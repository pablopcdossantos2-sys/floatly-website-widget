use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex, MutexGuard,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri_plugin_global_shortcut::Shortcut;
use url::Url;

pub const DEFAULT_SITE: &str = "";
pub const DEFAULT_WIDTH: u32 = 350;
pub const DEFAULT_HEIGHT: u32 = 550;

static SAVE_LOCK: Mutex<()> = Mutex::new(());
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);
static CONFIG_POISON_WARNING_LOGGED: AtomicBool = AtomicBool::new(false);
static REGISTERED_SHORTCUT_POISON_WARNING_LOGGED: AtomicBool = AtomicBool::new(false);

const RESERVED_SHORTCUTS: &[&str] = &[
    "CTRL+C", "CTRL+V", "CTRL+X", "CTRL+Z", "CTRL+Y", "CTRL+A", "CTRL+S",
    "CTRL+F", "CTRL+P", "CTRL+W", "CTRL+T", "CTRL+N",
    "CTRL+ESC", "CTRL+ESCAPE", "CTRL+SHIFT+ESC", "CTRL+SHIFT+ESCAPE",
    "ALT+F4", "ALT+TAB", "ALT+ESC", "ALT+ESCAPE", "ALT+SPACE",
    "SUPER+L", "SUPER+D", "SUPER+E", "SUPER+R", "SUPER+TAB",
];

fn default_language() -> String {
    "pt-BR".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub site_url: String,
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub remember_position: bool,
    pub autostart: bool,
    pub always_on_top: bool,
    pub zoom: u16,
    pub auto_reload_minutes: u16,
    pub show_in_taskbar: bool,
    pub locked: bool,
    pub global_shortcut: String,
    pub corner_radius: u8,
    /// Defaults to Brazilian Portuguese for configuration files created by older releases.
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            site_url: DEFAULT_SITE.into(),
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            x: None,
            y: None,
            remember_position: true,
            autostart: false,
            always_on_top: false,
            zoom: 80,
            auto_reload_minutes: 0,
            show_in_taskbar: false,
            locked: false,
            global_shortcut: "CTRL+ALT+C".into(),
            corner_radius: 12,
            language: default_language(),
        }
    }
}

impl AppConfig {
    pub fn normalize(&mut self) -> Result<(), String> {
        let mut site = self.site_url.trim().to_owned();
        if site.is_empty() {
            self.site_url.clear();
        } else {
            let has_scheme = site.split_once("://").is_some_and(|(scheme, _)| {
                let mut chars = scheme.chars();
                chars
                    .next()
                    .is_some_and(|first| first.is_ascii_alphabetic())
                    && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-'))
            });
            if !has_scheme {
                site = format!("https://{site}");
            }
            let url = Url::parse(&site).map_err(|_| "config.site_url_invalid".to_owned())?;
            if !["http", "https"].contains(&url.scheme())
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err("config.site_url_http_only".into());
            }
            self.site_url = url.to_string();
        }
        self.width = self.width.clamp(150, 2000);
        self.height = self.height.clamp(100, 1600);
        self.zoom = self.zoom.clamp(50, 200);
        if self.auto_reload_minutes > 0 && self.auto_reload_minutes < 5 {
            self.auto_reload_minutes = 5;
        }
        self.auto_reload_minutes = self.auto_reload_minutes.min(1440);
        self.corner_radius = self.corner_radius.min(64);
        let raw = self.global_shortcut.trim().to_ascii_uppercase();
        let parts: Vec<String> = raw.split('+').map(|part| part.trim().to_owned()).collect();
        if parts.len() < 2 || parts.iter().any(String::is_empty) {
            return Err("config.shortcut_modifier_required".into());
        }
        let key = parts.last().cloned().unwrap_or_default();
        if matches!(
            key.as_str(),
            "CTRL" | "CONTROL" | "ALT" | "SHIFT" | "SUPER" | "WIN" | "META"
        ) {
            return Err("config.shortcut_key_unsupported".into());
        }
        let mut modifiers = Vec::new();
        for part in &parts[..parts.len() - 1] {
            let canonical = match part.as_str() {
                "CTRL" | "CONTROL" => "CTRL",
                "ALT" => "ALT",
                "SHIFT" => "SHIFT",
                "SUPER" | "WIN" | "META" => "SUPER",
                _ => return Err("config.shortcut_key_unsupported".into()),
            };
            if !modifiers.contains(&canonical) {
                modifiers.push(canonical);
            }
        }
        if !modifiers
            .iter()
            .any(|modifier| matches!(*modifier, "CTRL" | "ALT" | "SUPER"))
        {
            return Err("config.shortcut_modifier_required".into());
        }
        modifiers.sort_by_key(|modifier| match *modifier {
            "CTRL" => 0,
            "ALT" => 1,
            "SHIFT" => 2,
            "SUPER" => 3,
            _ => 4,
        });
        let shortcut = format!("{}+{}", modifiers.join("+"), key);
        if RESERVED_SHORTCUTS.contains(&shortcut.as_str()) {
            return Err("config.shortcut_reserved".into());
        }
        Shortcut::from_str(&shortcut).map_err(|_| "config.shortcut_key_unsupported".to_owned())?;
        self.global_shortcut = shortcut;
        if !["pt-BR", "en", "es"].contains(&self.language.as_str()) {
            self.language = default_language();
        }
        Ok(())
    }
}

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub app_dir: PathBuf,
    pub registered_shortcut: Mutex<String>,
    /// Incremented for each geometry update; the worker persists after a quiet interval.
    pub geometry_save_revision: AtomicU64,
    pub geometry_save_worker_running: AtomicBool,
    /// Per-process token accepted only by the minimal remote heartbeat command.
    pub heartbeat_token: String,
    /// Set before an explicit app exit, so a destroyed main window is not misreported as a crash.
    pub shutdown_requested: AtomicBool,
}

impl AppState {
    fn log_poison_recovery(&self) {
        if !CONFIG_POISON_WARNING_LOGGED.swap(true, Ordering::Relaxed) {
            crate::diagnostics::log(
                &self.app_dir,
                "WARN",
                "O mutex da configuração foi envenenado; recuperando o último estado armazenado.",
            );
        }
    }

    /// Lock the current configuration, recovering the last valid state after a panic.
    pub fn lock_config(&self) -> MutexGuard<'_, AppConfig> {
        match self.config.lock() {
            Ok(config) => config,
            Err(poisoned) => {
                self.config.clear_poison();
                self.log_poison_recovery();
                poisoned.into_inner()
            }
        }
    }

    /// Lock the active shortcut, recovering after a panic instead of disabling future saves.
    pub fn lock_registered_shortcut(&self) -> MutexGuard<'_, String> {
        match self.registered_shortcut.lock() {
            Ok(shortcut) => shortcut,
            Err(poisoned) => {
                self.registered_shortcut.clear_poison();
                if !REGISTERED_SHORTCUT_POISON_WARNING_LOGGED.swap(true, Ordering::Relaxed) {
                    crate::diagnostics::log(
                        &self.app_dir,
                        "WARN",
                        "O mutex do atalho registrado foi envenenado; recuperando o último valor armazenado.",
                    );
                }
                poisoned.into_inner()
            }
        }
    }

    /// Read the latest configuration even if an earlier panic poisoned the mutex.
    pub fn snapshot(&self) -> AppConfig {
        self.lock_config().clone()
    }

    /// Persist while holding the config lock so a debounced writer cannot commit a stale snapshot.
    pub fn persist_current(&self) -> Result<(), String> {
        let config = self.lock_config();
        save(&self.app_dir, &config)
    }
}

fn roaming_directory(
    appdata: Option<PathBuf>,
    local_appdata: Option<PathBuf>,
    user_profile: Option<PathBuf>,
    temp_dir: PathBuf,
) -> PathBuf {
    appdata
        .or_else(|| {
            local_appdata.map(|path| {
                path.parent()
                    .unwrap_or(path.as_path())
                    .join("Roaming")
            })
        })
        .or_else(|| user_profile.map(|path| path.join("AppData").join("Roaming")))
        .unwrap_or_else(|| temp_dir.join("Floatly").join("Roaming"))
}

pub fn app_directory() -> PathBuf {
    roaming_directory(
        std::env::var_os("APPDATA").map(PathBuf::from),
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
        std::env::var_os("USERPROFILE").map(PathBuf::from),
        std::env::temp_dir(),
    )
    .join("WidgetWebsite")
}

#[cfg(test)]
pub fn load(dir: &Path) -> AppConfig {
    load_with_warnings(dir).0
}

/// Loads a configuration field-by-field so one malformed value does not discard every
/// other preference. The boolean indicates whether startup may safely persist the result.
pub fn load_with_warnings(dir: &Path) -> (AppConfig, Vec<String>, bool) {
    let path = dir.join("config.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (AppConfig::default(), Vec::new(), true);
        }
        Err(error) => {
            return (
                AppConfig::default(),
                vec![format!("Não foi possível ler config.json: {error}")],
                false,
            );
        }
    };
    let value: serde_json::Value = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(value) if value.is_object() => value,
        _ => {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let backup_counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let backup = dir.join(format!(
                "config.json.bad-{timestamp}-{}-{backup_counter}",
                std::process::id()
            ));
            match fs::rename(&path, &backup) {
                Ok(()) => {
                    return (
                        AppConfig::default(),
                        vec![format!(
                            "JSON inválido; configuração original preservada em {}",
                            backup.display()
                        )],
                        true,
                    );
                }
                Err(error) => {
                    return (
                        AppConfig::default(),
                        vec![format!(
                            "JSON inválido e não foi possível criar backup de config.json: {error}"
                        )],
                        false,
                    );
                }
            }
        }
    };

    let mut warnings = Vec::new();
    let mut merged =
        serde_json::to_value(AppConfig::default()).expect("AppConfig default serializes");
    if let Some(saved) = value.as_object() {
        for (key, candidate_value) in saved {
            let previous = match merged
                .as_object()
                .and_then(|base| base.get::<str>(key.as_str()))
                .cloned()
            {
                Some(previous) => previous,
                None if key == "opacity" => {
                    warnings.push("Configuração obsoleta removida: opacity.".into());
                    continue;
                }
                None => continue,
            };
            if let Some(base) = merged.as_object_mut() {
                base.insert(key.clone(), candidate_value.clone());
            }
            if serde_json::from_value::<AppConfig>(merged.clone()).is_err() {
                if let Some(base) = merged.as_object_mut() {
                    base.insert(key.clone(), previous);
                }
                warnings.push(format!(
                    "Campo inválido em config.json: {key}; usando o valor padrão."
                ));
            }
        }
    }
    let mut config = serde_json::from_value::<AppConfig>(merged).unwrap_or_default();
    let before_normalize = config.clone();
    let mut normalization_retries = 0;
    loop {
        match config.normalize() {
            Ok(()) => break,
            Err(error) if error.starts_with("config.site_url") => {
                config.site_url = AppConfig::default().site_url;
                warnings
                    .push("Campo inválido em config.json: site_url; usando o valor padrão.".into());
            }
            Err(error) if error.starts_with("config.shortcut") => {
                config.global_shortcut = AppConfig::default().global_shortcut;
                warnings.push(
                    "Campo inválido em config.json: global_shortcut; usando o valor padrão.".into(),
                );
            }
            Err(error) => {
                warnings.push(format!(
                    "Configuração inválida; recuperando padrões: {error}"
                ));
                config = AppConfig::default();
                break;
            }
        }
        normalization_retries += 1;
        if normalization_retries >= 3 {
            warnings.push(
                "A configuração permaneceu inválida após várias tentativas; usando os padrões."
                    .into(),
            );
            config = AppConfig::default();
            break;
        }
    }
    if before_normalize.site_url != config.site_url {
        warnings.push("Valor de site_url normalizado.".into());
    }
    if before_normalize.global_shortcut != config.global_shortcut {
        warnings.push("Valor de global_shortcut normalizado.".into());
    }
    if before_normalize.language != config.language {
        warnings.push("Valor de language inválido; usando o idioma padrão.".into());
    }
    if before_normalize.zoom != config.zoom {
        warnings.push("Valor de zoom ajustado aos limites suportados.".into());
    }
    if before_normalize.width != config.width {
        warnings.push("Valor de width ajustado aos limites suportados.".into());
    }
    if before_normalize.height != config.height {
        warnings.push("Valor de height ajustado aos limites suportados.".into());
    }
    if before_normalize.auto_reload_minutes != config.auto_reload_minutes {
        warnings.push("Valor de auto_reload_minutes ajustado aos limites suportados.".into());
    }
    if before_normalize.corner_radius != config.corner_radius {
        warnings.push("Valor de corner_radius ajustado aos limites suportados.".into());
    }
    (config, warnings, true)
}

pub fn save(dir: &Path, config: &AppConfig) -> Result<(), String> {
    let _guard = SAVE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(config).map_err(|error| error.to_string())?;
    let target = dir.join("config.json");
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp = dir.join(format!(
        "config.json.{}.{}.tmp",
        std::process::id(),
        counter
    ));
    let write_result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| error.to_string())?;
        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    let mut last_error = None;
    for attempt in 0..3 {
        match fs::rename(&temp, &target) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                if attempt < 2 {
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
    }
    let _ = fs::remove_file(&temp);
    Err(last_error
        .map(|error| error.to_string())
        .unwrap_or_else(|| "Não foi possível substituir config.json.".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn defaults_match_widget_requirements() {
        let config = AppConfig::default();
        assert_eq!(config.site_url, DEFAULT_SITE);
        assert_eq!(config.width, DEFAULT_WIDTH);
        assert_eq!(config.height, DEFAULT_HEIGHT);
        assert_eq!(config.zoom, 80);
        assert_eq!(config.global_shortcut, "CTRL+ALT+C");
        assert_eq!(config.language, "pt-BR");
    }

    #[test]
    fn empty_site_url_is_allowed_until_user_configures_widget() {
        let mut config = AppConfig::default();
        assert!(config.site_url.is_empty());
        config.normalize().unwrap();
        assert!(config.site_url.is_empty());
    }

    #[test]
    fn normalization_prefixes_https_and_clamps_values() {
        let mut config = AppConfig {
            site_url: "example.com".into(),
            width: 1,
            height: 9999,
            zoom: 250,
            auto_reload_minutes: 1,
            corner_radius: 100,
            ..AppConfig::default()
        };
        config.normalize().unwrap();
        assert_eq!(config.site_url, "https://example.com/");
        assert_eq!(config.width, 150);
        assert_eq!(config.height, 1600);
        assert_eq!(config.zoom, 200);
        assert_eq!(config.auto_reload_minutes, 5);
        assert_eq!(config.corner_radius, 64);
    }

    #[test]
    fn language_is_limited_to_supported_values() {
        for language in ["pt-BR", "en", "es"] {
            let mut config = AppConfig {
                language: language.to_owned(),
                ..AppConfig::default()
            };
            config.normalize().unwrap();
            assert_eq!(config.language, language);
        }

        let mut config = AppConfig {
            language: "fr".into(),
            ..AppConfig::default()
        };
        config.normalize().unwrap();
        assert_eq!(config.language, "pt-BR");
    }

    #[test]
    fn shortcut_requires_a_modifier_and_a_supported_key() {
        let mut config = AppConfig {
            global_shortcut: "A".into(),
            ..AppConfig::default()
        };
        assert_eq!(
            config.normalize().unwrap_err(),
            "config.shortcut_modifier_required"
        );

        let mut config = AppConfig {
            global_shortcut: "CTRL+ALT+C".into(),
            ..AppConfig::default()
        };
        config.normalize().unwrap();
        assert_eq!(config.global_shortcut, "CTRL+ALT+C");

        let mut config = AppConfig {
            global_shortcut: "CTRL+ALT+NOT_A_REAL_KEY".into(),
            ..AppConfig::default()
        };
        assert_eq!(
            config.normalize().unwrap_err(),
            "config.shortcut_key_unsupported"
        );
    }

    #[test]
    fn old_configuration_without_language_keeps_existing_preferences() {
        let mut value = serde_json::to_value(AppConfig::default()).unwrap();
        value.as_object_mut().unwrap().remove("language");
        value["width"] = serde_json::json!(777);
        let config: AppConfig = serde_json::from_value(value).unwrap();
        assert_eq!(config.language, "pt-BR");
        assert_eq!(config.width, 777);
    }

    #[test]
    fn normalization_rejects_non_http_urls_and_credentials() {
        for invalid in [
            "file:///private/file",
            "ftp://example.com",
            "https://user:secret@example.com",
        ] {
            let mut config = AppConfig {
                site_url: invalid.into(),
                ..AppConfig::default()
            };
            assert!(
                config.normalize().is_err(),
                "accepted invalid URL: {invalid}"
            );
        }
    }

    #[test]
    fn corrupt_config_recovers_to_defaults() {
        let dir =
            std::env::temp_dir().join(format!("widget-website-config-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("config.json"), b"{ this is not valid json").unwrap();
        let config = load(&dir);
        assert_eq!(config.site_url, DEFAULT_SITE);
        assert_eq!(config.width, DEFAULT_WIDTH);
        assert!(fs::read_dir(&dir).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("config.json.bad-")
        }));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_removes_legacy_opacity_preference_without_discarding_valid_fields() {
        let dir = std::env::temp_dir().join(format!(
            "widget-website-legacy-opacity-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("config.json"),
            r#"{"width":800,"opacity":65,"language":"en"}"#,
        )
        .unwrap();
        let (config, warnings, safe_to_persist) = load_with_warnings(&dir);
        assert_eq!(config.width, 800);
        assert_eq!(config.language, "en");
        assert!(warnings.iter().any(|warning| warning.contains("opacity")));
        assert!(safe_to_persist);
        let serialized = serde_json::to_value(&config).unwrap();
        assert!(!serialized.as_object().unwrap().contains_key("opacity"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_resets_only_invalid_site_url() {
        let dir =
            std::env::temp_dir().join(format!("widget-website-invalid-url-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("config.json"),
            r#"{"site_url":"file:///invalid","width":860,"language":"es"}"#,
        )
        .unwrap();
        let (config, warnings, _) = load_with_warnings(&dir);
        assert_eq!(config.site_url, DEFAULT_SITE);
        assert_eq!(config.width, 860);
        assert_eq!(config.language, "es");
        assert!(warnings.iter().any(|warning| warning.contains("site_url")));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_tolerates_missing_fields_via_struct_default() {
        let config: AppConfig = serde_json::from_str(r#"{"width":777}"#).unwrap();
        assert_eq!(config.width, 777);
        assert_eq!(config.height, DEFAULT_HEIGHT);
        assert_eq!(config.site_url, DEFAULT_SITE);
    }

    #[test]
    fn app_directory_uses_stable_profile_fallbacks_without_current_dir() {
        assert_eq!(
            roaming_directory(
                Some(PathBuf::from(r"C:\\Users\\Test\\AppData\\Roaming")),
                Some(PathBuf::from(r"C:\\Users\\Test\\AppData\\Local")),
                Some(PathBuf::from(r"C:\\Users\\Test")),
                PathBuf::from(r"C:\\Temp")
            ),
            PathBuf::from(r"C:\\Users\\Test\\AppData\\Roaming")
        );
        assert_eq!(
            roaming_directory(
                None,
                Some(PathBuf::from(r"C:\\Users\\Test\\AppData\\Local")),
                None,
                PathBuf::from(r"C:\\Temp")
            ),
            PathBuf::from(r"C:\\Users\\Test\\AppData\\Roaming")
        );
        assert_eq!(
            roaming_directory(None, None, None, PathBuf::from(r"C:\\Temp")),
            PathBuf::from(r"C:\\Temp\\Floatly\\Roaming")
        );
    }

    #[test]
    fn normalization_rejects_reserved_system_shortcuts() {
        for shortcut in [
            "CTRL+C", "ctrl+v", "CTRL+SHIFT+ESC", "ALT+F4", "ALT+TAB", "SUPER+L",
        ] {
            let mut config = AppConfig {
                global_shortcut: shortcut.into(),
                ..AppConfig::default()
            };
            assert_eq!(
                config.normalize().unwrap_err(),
                "config.shortcut_reserved",
                "accepted reserved shortcut: {shortcut}"
            );
        }
        let mut allowed = AppConfig {
            global_shortcut: "CTRL+ALT+C".into(),
            ..AppConfig::default()
        };
        allowed.normalize().unwrap();
    }

    #[test]
    fn normalization_prefixes_https_when_scheme_only_appears_in_query() {
        let mut config = AppConfig {
            site_url: "example.com/x?u=https://example.org".into(),
            ..AppConfig::default()
        };
        config.normalize().unwrap();
        assert_eq!(
            config.site_url,
            "https://example.com/x?u=https://example.org"
        );
    }

    #[test]
    fn normalization_rejects_shift_only_shortcuts_and_canonicalizes_modifiers() {
        for shortcut in ["SHIFT+A", "TAB", "", "CTRL+"] {
            let mut config = AppConfig {
                global_shortcut: shortcut.into(),
                ..AppConfig::default()
            };
            assert!(config.normalize().is_err(), "accepted shortcut: {shortcut}");
        }
        let mut config = AppConfig {
            global_shortcut: " ctrl+alt+c ".into(),
            ..AppConfig::default()
        };
        config.normalize().unwrap();
        assert_eq!(config.global_shortcut, "CTRL+ALT+C");
        let mut reversed = AppConfig {
            global_shortcut: "ALT+CTRL+C".into(),
            ..AppConfig::default()
        };
        reversed.normalize().unwrap();
        assert_eq!(reversed.global_shortcut, "CTRL+ALT+C");
    }

    #[test]
    fn load_recovers_multiple_invalid_fields_without_losing_valid_preferences() {
        let dir = std::env::temp_dir().join(format!(
            "widget-website-multiple-invalid-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("config.json"),
            r#"{"site_url":"file:///invalid","global_shortcut":"SHIFT+A","width":920,"language":"en"}"#,
        ).unwrap();
        let (config, warnings, _) = load_with_warnings(&dir);
        assert_eq!(config.site_url, DEFAULT_SITE);
        assert_eq!(config.global_shortcut, "CTRL+ALT+C");
        assert_eq!(config.width, 920);
        assert_eq!(config.language, "en");
        assert!(warnings.iter().any(|warning| warning.contains("site_url")));
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("global_shortcut")));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn snapshot_recovers_config_from_a_poisoned_mutex() {
        let dir = std::env::temp_dir().join(format!(
            "widget-website-poisoned-state-{}",
            std::process::id()
        ));
        let state = AppState {
            config: Mutex::new(AppConfig {
                width: 777,
                ..AppConfig::default()
            }),
            app_dir: dir.clone(),
            registered_shortcut: Mutex::new(String::new()),
            geometry_save_revision: AtomicU64::new(0),
            geometry_save_worker_running: AtomicBool::new(false),
            heartbeat_token: "test-token".into(),
            shutdown_requested: AtomicBool::new(false),
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut config = state.config.lock().unwrap();
            config.width = 888;
            panic!("intentionally poison the test mutex");
        }));
        assert!(result.is_err());
        assert_eq!(state.snapshot().width, 888);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn save_uses_unique_temp_files_and_leaves_valid_json() {
        let dir =
            std::env::temp_dir().join(format!("widget-website-atomic-save-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let config = AppConfig {
            width: 811,
            ..AppConfig::default()
        };
        save(&dir, &config).unwrap();
        let parsed: AppConfig =
            serde_json::from_slice(&fs::read(dir.join("config.json")).unwrap()).unwrap();
        assert_eq!(parsed.width, 811);
        assert!(!fs::read_dir(&dir).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")));
        fs::remove_dir_all(&dir).unwrap();
    }
}
