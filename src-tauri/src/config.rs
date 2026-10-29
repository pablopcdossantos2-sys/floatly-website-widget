use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, str::FromStr, sync::Mutex};
use tauri_plugin_global_shortcut::Shortcut;
use url::Url;

pub const DEFAULT_SITE: &str = "https://br.investing.com/economic-calendar";

fn default_language() -> String {
    "pt-BR".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub site_url: String,
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub remember_position: bool,
    pub autostart: bool,
    pub always_on_top: bool,
    pub opacity: u8,
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
            width: 500,
            height: 700,
            x: None,
            y: None,
            remember_position: true,
            autostart: false,
            always_on_top: false,
            opacity: 100,
            zoom: 100,
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
            return Err("config.site_url_required".into());
        }
        if !site.contains("://") {
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
        self.width = self.width.clamp(150, 2000);
        self.height = self.height.clamp(100, 1600);
        self.opacity = self.opacity.clamp(30, 100);
        self.zoom = self.zoom.clamp(50, 200);
        if self.auto_reload_minutes > 0 && self.auto_reload_minutes < 5 {
            self.auto_reload_minutes = 5;
        }
        self.auto_reload_minutes = self.auto_reload_minutes.min(1440);
        self.corner_radius = self.corner_radius.min(64);
        let shortcut = self.global_shortcut.trim().to_ascii_uppercase();
        let parts: Vec<&str> = shortcut.split('+').map(str::trim).collect();
        let has_modifier = parts.iter().any(|part| {
            matches!(*part, "CTRL" | "CONTROL" | "ALT" | "SHIFT" | "SUPER" | "WIN" | "META")
        });
        if !has_modifier || parts.len() < 2 || parts.last().is_none_or(|key| key.is_empty()) {
            return Err("config.shortcut_modifier_required".into());
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
}

pub fn app_directory() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("WidgetWebsite")
}

pub fn load(dir: &PathBuf) -> AppConfig {
    let mut config = fs::read(dir.join("config.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<AppConfig>(&bytes).ok())
        .unwrap_or_default();
    if config.normalize().is_err() {
        config = AppConfig::default();
    }
    config
}

pub fn save(dir: &PathBuf, config: &AppConfig) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let temp = dir.join("config.json.tmp");
    fs::write(
        &temp,
        serde_json::to_vec_pretty(config).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::rename(&temp, dir.join("config.json"))
        .or_else(|_| {
            fs::copy(&temp, dir.join("config.json"))?;
            fs::remove_file(&temp)
        })
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn defaults_match_widget_requirements() {
        let config = AppConfig::default();
        assert_eq!(config.site_url, DEFAULT_SITE);
        assert_eq!(config.width, 500);
        assert_eq!(config.height, 700);
        assert_eq!(config.zoom, 100);
        assert_eq!(config.opacity, 100);
        assert_eq!(config.global_shortcut, "CTRL+ALT+C");
        assert_eq!(config.language, "pt-BR");
    }

    #[test]
    fn normalization_prefixes_https_and_clamps_values() {
        let mut config = AppConfig::default();
        config.site_url = "investing.com".into();
        config.width = 1;
        config.height = 9999;
        config.opacity = 0;
        config.zoom = 250;
        config.auto_reload_minutes = 1;
        config.corner_radius = 100;
        config.normalize().unwrap();
        assert_eq!(config.site_url, "https://investing.com/");
        assert_eq!(config.width, 150);
        assert_eq!(config.height, 1600);
        assert_eq!(config.opacity, 30);
        assert_eq!(config.zoom, 200);
        assert_eq!(config.auto_reload_minutes, 5);
        assert_eq!(config.corner_radius, 64);
    }

    #[test]
    fn language_is_limited_to_supported_values() {
        for language in ["pt-BR", "en", "es"] {
            let mut config = AppConfig::default();
            config.language = language.to_owned();
            config.normalize().unwrap();
            assert_eq!(config.language, language);
        }

        let mut config = AppConfig::default();
        config.language = "fr".into();
        config.normalize().unwrap();
        assert_eq!(config.language, "pt-BR");
    }

    #[test]
    fn shortcut_requires_a_modifier_and_a_supported_key() {
        let mut config = AppConfig::default();
        config.global_shortcut = "A".into();
        assert_eq!(config.normalize().unwrap_err(), "config.shortcut_modifier_required");

        let mut config = AppConfig::default();
        config.global_shortcut = "CTRL+ALT+C".into();
        config.normalize().unwrap();
        assert_eq!(config.global_shortcut, "CTRL+ALT+C");

        let mut config = AppConfig::default();
        config.global_shortcut = "CTRL+ALT+NOT_A_REAL_KEY".into();
        assert_eq!(config.normalize().unwrap_err(), "config.shortcut_key_unsupported");
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
            let mut config = AppConfig::default();
            config.site_url = invalid.into();
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
        assert_eq!(config.width, 500);
        fs::remove_dir_all(&dir).unwrap();
    }
}
