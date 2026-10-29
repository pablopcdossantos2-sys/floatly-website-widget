use crate::{
    config::{self, AppConfig, AppState},
    window,
};
use serde::Serialize;
use std::str::FromStr;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
#[derive(Serialize)]
pub struct SaveResult {
    pub config: AppConfig,
    pub warning: Option<String>,
}
/// The remote calendar can only use this command to report liveness.
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
    if let Ok(v) = app.autolaunch().is_enabled() {
        c.autostart = v
    }
    c
}
#[tauri::command]
pub fn save_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
    force_resize: Option<bool>,
) -> Result<SaveResult, String> {
    let mut c = config;
    c.normalize()?;
    let old = state
        .config
        .lock()
        .map_err(|_| "config.read_failed".to_owned())?
        .clone();
    c.x = old.x;
    c.y = old.y;
    let requested = c.clone();

    // Persist the candidate before registering hotkeys or changing Windows startup.
    config::save(&state.app_dir, &requested)
        .map_err(|_| "config.disk_write_failed".to_owned())?;

    let mut warns = Vec::new();
    let active_shortcut = state
        .registered_shortcut
        .lock()
        .map_err(|_| "config.write_failed".to_owned())?
        .clone();
    let previous_active_shortcut = active_shortcut.clone();
    let mut shortcut_changed = false;
    if c.global_shortcut != active_shortcut {
        match Shortcut::from_str(&c.global_shortcut) {
            Ok(new_shortcut) => match app.global_shortcut().register(new_shortcut.clone()) {
                Ok(()) => {
                    let unregister_result = if active_shortcut.is_empty() {
                        Ok(())
                    } else {
                        Shortcut::from_str(&active_shortcut)
                            .map_err(|error| error.to_string())
                            .and_then(|old_shortcut| {
                                app.global_shortcut().unregister(old_shortcut).map_err(|error| error.to_string())
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
                        if let Ok(mut registered) = state.registered_shortcut.lock() {
                            *registered = c.global_shortcut.clone();
                        }
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

    let previous_autostart = app.autolaunch().is_enabled().unwrap_or(old.autostart);
    let mut autostart_changed = false;
    if c.autostart != previous_autostart {
        let result = if c.autostart {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        };
        if result.is_ok() {
            autostart_changed = app.autolaunch().is_enabled().unwrap_or(c.autostart) != previous_autostart;
        } else {
            let actual = app.autolaunch().is_enabled().unwrap_or(previous_autostart);
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

    let corrected = c.global_shortcut != requested.global_shortcut || c.autostart != requested.autostart;
    if corrected {
        if let Err(error) = config::save(&state.app_dir, &c) {
            crate::diagnostics::log(
                &state.app_dir,
                "ERRO",
                format!("Não foi possível persistir a configuração corrigida após uma falha de integração: {error}"),
            );
            // Best-effort rollback: do not leave the runtime on a new hotkey if saving
            // the corrected configuration failed.
            if shortcut_changed {
                if let Ok(new_shortcut) = Shortcut::from_str(&c.global_shortcut) {
                    let _ = app.global_shortcut().unregister(new_shortcut);
                }
                if let Ok(old_shortcut) = Shortcut::from_str(&previous_active_shortcut) {
                    let _ = app.global_shortcut().register(old_shortcut);
                }
                if let Ok(mut registered) = state.registered_shortcut.lock() {
                    *registered = previous_active_shortcut;
                }
            }
            if autostart_changed {
                let _ = if previous_autostart {
                    app.autolaunch().enable()
                } else {
                    app.autolaunch().disable()
                };
            }
            let _ = config::save(&state.app_dir, &old);
            return Err("config.disk_write_failed".to_owned());
        }
    }

    {
        let mut stored = state
            .config
            .lock()
            .map_err(|_| "config.write_failed".to_owned())?;
        *stored = c.clone();
    }
    let resize = force_resize.unwrap_or(false) || c.width != old.width || c.height != old.height;
    window::apply_config(&app, &c, resize);
    crate::tray::update_language(&app, &c.language);
    let _ = app.emit("config-changed", &c);
    if let Some(settings) = app.get_webview_window("settings") {
        let title = match c.language.as_str() {
            "en" => "Settings — Widget Website",
            "es" => "Configuración — Widget Website",
            _ => "Configurações — Widget Website",
        };
        let _ = settings.set_title(title);
    }
    if old.site_url != c.site_url {
        if let (Some(main), Ok(url)) =
            (app.get_webview_window("main"), url::Url::parse(&c.site_url))
        {
            if let Err(error) = main.navigate(url) {
                crate::diagnostics::log(
                    &state.app_dir,
                    "WARN",
                    format!("A configuração foi salva, mas a navegação para o novo site falhou: {error}"),
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
pub fn close_settings(app: AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }
}
pub fn toggle_main_visibility(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
        }
    }
}
pub fn reload_site(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.eval("location.reload()");
    }
}
pub fn quit_app(app: AppHandle) {
    window::flush_pending_config(&app);
    app.exit(0)
}
pub fn toggle_locked(app: &AppHandle) {
    let s = app.state::<AppState>();
    let next = {
        let mut c = match s.config.lock() {
            Ok(v) => v,
            Err(_) => return,
        };
        c.locked = !c.locked;
        let _ = config::save(&s.app_dir, &c);
        c.clone()
    };
    window::apply_config(app, &next, false);
    crate::tray::update_language(app, &next.language);
    let _ = app.emit("config-changed", &next);
}
pub fn toggle_topmost(app: &AppHandle) {
    let s = app.state::<AppState>();
    let next = {
        let mut c = match s.config.lock() {
            Ok(v) => v,
            Err(_) => return,
        };
        c.always_on_top = !c.always_on_top;
        let _ = config::save(&s.app_dir, &c);
        c.clone()
    };
    window::apply_config(app, &next, false);
    let _ = app.emit("config-changed", &next);
}
pub fn show_settings(app: &AppHandle) {
    window::open_settings(app)
}
pub fn show_login(app: &AppHandle) {
    window::open_login_from_tray(app)
}
