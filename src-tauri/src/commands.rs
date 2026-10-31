use crate::{
    config::{self, AppConfig, AppState},
    window,
};
use serde::Serialize;
use std::str::FromStr;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
#[derive(Serialize)]
pub struct SaveResult {
    pub config: AppConfig,
    pub warning: Option<String>,
}
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct ConfigPatch {
    pub site_url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub remember_position: Option<bool>,
    pub autostart: Option<bool>,
    pub always_on_top: Option<bool>,
    pub zoom: Option<u16>,
    pub auto_reload_minutes: Option<u16>,
    pub show_in_taskbar: Option<bool>,
    pub locked: Option<bool>,
    pub global_shortcut: Option<String>,
    pub corner_radius: Option<u8>,
    pub language: Option<String>,
}

impl ConfigPatch {
    fn apply_to(&self, mut config: AppConfig) -> AppConfig {
        if let Some(value) = &self.site_url {
            config.site_url = value.clone();
        }
        if let Some(value) = self.width {
            config.width = value;
        }
        if let Some(value) = self.height {
            config.height = value;
        }
        if let Some(value) = self.remember_position {
            config.remember_position = value;
        }
        if let Some(value) = self.autostart {
            config.autostart = value;
        }
        if let Some(value) = self.always_on_top {
            config.always_on_top = value;
        }
        if let Some(value) = self.zoom {
            config.zoom = value;
        }
        if let Some(value) = self.auto_reload_minutes {
            config.auto_reload_minutes = value;
        }
        if let Some(value) = self.show_in_taskbar {
            config.show_in_taskbar = value;
        }
        if let Some(value) = self.locked {
            config.locked = value;
        }
        if let Some(value) = &self.global_shortcut {
            config.global_shortcut = value.clone();
        }
        if let Some(value) = self.corner_radius {
            config.corner_radius = value;
        }
        if let Some(value) = &self.language {
            config.language = value.clone();
        }
        config
    }
}

#[cfg(test)]
mod patch_tests {
    use super::*;

    #[test]
    fn config_patch_preserves_runtime_state_not_in_the_patch() {
        let latest = AppConfig {
            x: Some(10),
            y: Some(20),
            locked: true,
            always_on_top: true,
            width: 900,
            ..AppConfig::default()
        };
        let patch = ConfigPatch {
            zoom: Some(125),
            ..ConfigPatch::default()
        };
        let result = patch.apply_to(latest);
        assert_eq!(result.x, Some(10));
        assert_eq!(result.y, Some(20));
        assert!(result.locked);
        assert!(result.always_on_top);
        assert_eq!(result.width, 900);
        assert_eq!(result.zoom, 125);
    }

    #[test]
    fn empty_config_patch_does_not_change_anything() {
        let latest = AppConfig {
            width: 777,
            locked: true,
            ..AppConfig::default()
        };
        assert_eq!(
            ConfigPatch::default().apply_to(latest.clone()).width,
            latest.width
        );
        assert_eq!(
            ConfigPatch::default().apply_to(latest.clone()).locked,
            latest.locked
        );
    }
}

/// The configured remote website can only use this command to report liveness.
#[tauri::command]
pub fn heartbeat(token: String, state: State<'_, AppState>) -> Result<(), String> {
    if token != state.heartbeat_token {
        return Err("heartbeat.invalid_token".to_owned());
    }
    crate::diagnostics::record_heartbeat(&state.app_dir);
    Ok(())
}

#[tauri::command]
pub fn get_config(app: AppHandle, state: State<'_, AppState>) -> AppConfig {
    let mut c = state.snapshot();
    if let Ok(enabled) = crate::autostart::is_enabled(&app) {
        c.autostart = enabled;
    }
    c
}
#[tauri::command]
pub fn save_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: ConfigPatch,
    force_resize: Option<bool>,
) -> Result<SaveResult, String> {
    let patch = config;
    let actual_autostart = if patch.autostart.is_none() {
        crate::autostart::is_enabled(&app).ok()
    } else {
        None
    };
    let active_shortcut = state.lock_registered_shortcut().clone();
    let previous_active_shortcut = active_shortcut.clone();
    let (old, requested) = {
        let mut stored = state.lock_config();
        let old = stored.clone();
        let mut requested = patch.apply_to(old.clone());
        if patch.autostart.is_none() {
            if let Some(enabled) = actual_autostart {
                requested.autostart = enabled;
            }
        }
        requested.normalize()?;
        *stored = requested.clone();

        // Hold the state lock while persisting so the debounced geometry writer
        // cannot later overwrite this newer configuration with an old snapshot.
        if let Err(error) = config::save(&state.app_dir, &stored) {
            *stored = old.clone();
            crate::diagnostics::log(
                &state.app_dir,
                "ERRO",
                format!("Não foi possível persistir as configurações antes de aplicar integrações: {error}"),
            );
            return Err("config.disk_write_failed".to_owned());
        }
        (old, requested)
    };
    let mut c = requested.clone();

    let mut warns = Vec::new();
    let mut shortcut_changed = false;
    if c.global_shortcut != active_shortcut {
        match Shortcut::from_str(&c.global_shortcut) {
            Ok(new_shortcut) => match app.global_shortcut().register(new_shortcut) {
                Ok(()) => {
                    let unregister_result = if active_shortcut.is_empty() {
                        Ok(())
                    } else {
                        Shortcut::from_str(&active_shortcut)
                            .map_err(|error| error.to_string())
                            .and_then(|old_shortcut| {
                                app.global_shortcut()
                                    .unregister(old_shortcut)
                                    .map_err(|error| error.to_string())
                            })
                    };
                    if let Err(error) = unregister_result {
                        let _ = app.global_shortcut().unregister(new_shortcut);
                        crate::diagnostics::log(
                            &state.app_dir,
                            "WARN",
                            format!("Não foi possível substituir o atalho ativo {active_shortcut}: {error}"),
                        );
                        c.global_shortcut = old.global_shortcut.clone();
                        warns.push("shortcut_conflict".to_owned());
                    } else {
                        shortcut_changed = true;
                        *state.lock_registered_shortcut() = c.global_shortcut.clone();
                    }
                }
                Err(error) => {
                    crate::diagnostics::log(
                        &state.app_dir,
                        "WARN",
                        format!("Não foi possível registrar o novo atalho: {error}"),
                    );
                    c.global_shortcut = old.global_shortcut.clone();
                    warns.push("shortcut_conflict".to_owned());
                }
            },
            Err(error) => {
                crate::diagnostics::log(
                    &state.app_dir,
                    "WARN",
                    format!("O atalho não pôde ser interpretado: {error}"),
                );
                c.global_shortcut = old.global_shortcut.clone();
                warns.push("shortcut_conflict".to_owned());
            }
        }
    }

    let previous_autostart = crate::autostart::is_enabled(&app).unwrap_or(old.autostart);
    let mut autostart_changed = false;
    if c.autostart != previous_autostart {
        match crate::autostart::set_enabled(&app, c.autostart) {
            Ok(actual) => {
                autostart_changed = actual != previous_autostart;
                if actual != c.autostart {
                    c.autostart = actual;
                    warns.push(if requested.autostart {
                        "autostart_enable_failed".to_owned()
                    } else {
                        "autostart_disable_failed".to_owned()
                    });
                }
            }
            Err(error) => {
                crate::diagnostics::log(
                    &state.app_dir,
                    "WARN",
                    format!("Não foi possível alterar a inicialização automática: {error}"),
                );
                let actual = crate::autostart::is_enabled(&app).unwrap_or(previous_autostart);
                if actual != c.autostart {
                    c.autostart = actual;
                    warns.push(if requested.autostart {
                        "autostart_enable_failed".to_owned()
                    } else {
                        "autostart_disable_failed".to_owned()
                    });
                }
            }
        }
    }

    let corrected =
        c.global_shortcut != requested.global_shortcut || c.autostart != requested.autostart;
    if corrected {
        let correction_result = {
            let mut stored = state.lock_config();
            *stored = c.clone();
            match config::save(&state.app_dir, &stored) {
                Ok(()) => Ok(()),
                Err(error) => {
                    *stored = old.clone();
                    Err(error)
                }
            }
        };
        if let Err(error) = correction_result {
            crate::diagnostics::log(
                &state.app_dir,
                "ERRO",
                format!("Não foi possível persistir a configuração corrigida após uma falha de integração: {error}"),
            );
            if shortcut_changed {
                if let Ok(new_shortcut) = Shortcut::from_str(&requested.global_shortcut) {
                    let _ = app.global_shortcut().unregister(new_shortcut);
                }
                if let Ok(old_shortcut) = Shortcut::from_str(&previous_active_shortcut) {
                    let _ = app.global_shortcut().register(old_shortcut);
                }
                *state.lock_registered_shortcut() = previous_active_shortcut;
            }
            if autostart_changed {
                let _ = crate::autostart::set_enabled(&app, previous_autostart);
            }
            if let Err(rollback_error) = config::save(&state.app_dir, &old) {
                crate::diagnostics::log(
                    &state.app_dir,
                    "ERRO",
                    format!("Falha adicional ao restaurar config.json: {rollback_error}"),
                );
            }
            return Err("config.disk_write_failed".to_owned());
        }
    }
    let resize = force_resize.unwrap_or(false) || c.width != old.width || c.height != old.height;
    window::apply_config(&app, &c, resize);
    crate::tray::update_language(&app, &c.language);
    let _ = app.emit_to("settings", "config-changed", &c);
    if let Some(settings) = app.get_webview_window("settings") {
        let title = crate::i18n::settings_title(&c.language);
        let _ = settings.set_title(title);
    }
    if old.site_url != c.site_url {
        if let Some(main) = app.get_webview_window("main") {
            let target_url = if c.site_url.is_empty() {
                Some(window::local_start_url())
            } else {
                url::Url::parse(&c.site_url).ok()
            };
            if let Some(url) = target_url {
                if let Err(error) = main.navigate(url) {
                    crate::diagnostics::log(
                        &state.app_dir,
                        "WARN",
                        format!("A configuração foi salva, mas a navegação para o novo destino falhou: {error}"),
                    );
                }
            } else {
                crate::diagnostics::log(
                    &state.app_dir,
                    "ERRO",
                    "A configuração foi salva, mas o endereço do site não pôde ser convertido em URL.",
                );
            }
        }
    }
    Ok(SaveResult {
        config: c,
        warning: if warns.is_empty() {
            None
        } else {
            Some(warns.join(" "))
        },
    })
}
#[tauri::command]
pub fn open_settings(app: AppHandle) {
    window::open_settings(&app);
}

#[tauri::command]
pub fn close_settings(app: AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }
}
pub fn toggle_main_visibility(app: AppHandle) {
    window::toggle_main_visibility(&app)
}
pub fn reload_site(app: AppHandle) {
    let dir = app.state::<AppState>().app_dir.clone();
    if let Some(window) = app.get_webview_window("main") {
        let _ = crate::diagnostics::log_err(
            &dir,
            "Não foi possível recarregar o site pela bandeja",
            window.eval("location.reload()"),
        );
    } else {
        crate::diagnostics::log(
            &dir,
            "WARN",
            "A solicitação de recarga foi ignorada porque a janela principal não existe.",
        );
    }
}
pub fn quit_app(app: AppHandle) {
    app.state::<AppState>()
        .shutdown_requested
        .store(true, std::sync::atomic::Ordering::SeqCst);
    window::flush_pending_config(&app);
    app.exit(0)
}
pub fn toggle_locked(app: &AppHandle) {
    let s = app.state::<AppState>();
    let next = {
        let mut c = s.lock_config();
        let previous = c.locked;
        c.locked = !c.locked;
        if let Err(error) = config::save(&s.app_dir, &c) {
            c.locked = previous;
            crate::diagnostics::log(
                &s.app_dir,
                "ERRO",
                format!("Falha ao persistir a opção de travamento: {error}"),
            );
        }
        c.clone()
    };
    window::apply_config(app, &next, false);
    crate::tray::update_language(app, &next.language);
    let _ = app.emit_to("settings", "config-changed", &next);
}
pub fn toggle_topmost(app: &AppHandle) {
    let s = app.state::<AppState>();
    let next = {
        let mut c = s.lock_config();
        let previous = c.always_on_top;
        c.always_on_top = !c.always_on_top;
        if let Err(error) = config::save(&s.app_dir, &c) {
            c.always_on_top = previous;
            crate::diagnostics::log(
                &s.app_dir,
                "ERRO",
                format!("Falha ao persistir a opção Sempre no topo: {error}"),
            );
        }
        c.clone()
    };
    window::apply_config(app, &next, false);
    let _ = app.emit_to("settings", "config-changed", &next);
}
pub fn show_settings(app: &AppHandle) {
    window::open_settings(app)
}
pub fn show_login(app: &AppHandle) {
    if app
        .state::<AppState>()
        .snapshot()
        .site_url
        .trim()
        .is_empty()
    {
        window::open_settings(app);
    } else {
        window::open_login_from_tray(app)
    }
}
