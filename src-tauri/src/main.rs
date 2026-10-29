mod commands;
mod config;
mod diagnostics;
mod tray;
mod window;

use config::AppState;
use std::str::FromStr;
use tauri::{AppHandle, Manager, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg(windows)]
fn webview2_runtime_present() -> bool {
    let bases = ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"];
    bases
        .iter()
        .filter_map(|key| std::env::var_os(key))
        .any(|base| {
            let folder = std::path::PathBuf::from(base)
                .join("Microsoft")
                .join("EdgeWebView")
                .join("Application");
            if folder.join("msedgewebview2.exe").is_file() {
                return true;
            }
            std::fs::read_dir(folder)
                .map(|entries| {
                    entries
                        .flatten()
                        .any(|entry| entry.path().join("msedgewebview2.exe").is_file())
                })
                .unwrap_or(false)
        })
}

#[cfg(windows)]
fn show_message(title: &str, message: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let text: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    let caption: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(windows)]
fn show_webview2_missing_message() {
    show_message(
        "Widget Website — requisito ausente",
        "O Microsoft Edge WebView2 Runtime não foi encontrado.\n\nInstale o Runtime Evergreen oficial e abra o Widget Website novamente:\nhttps://developer.microsoft.com/microsoft-edge/webview2/",
    );
}

fn toggle(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
        }
    }
}

fn show_existing_instance(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn main() {
    diagnostics::install_panic_hook();
    let app_dir = config::app_directory();
    diagnostics::log(
        &app_dir,
        "INFO",
        format!("Widget Website iniciando. PID={}", std::process::id()),
    );

    #[cfg(windows)]
    {
        if !webview2_runtime_present() {
            diagnostics::log(
                &app_dir,
                "FATAL",
                "Microsoft Edge WebView2 Runtime não encontrado nas pastas usuais.",
            );
            show_webview2_missing_message();
            return;
        }
        diagnostics::log(
            &app_dir,
            "INFO",
            "Microsoft Edge WebView2 Runtime detectado.",
        );
    }

    let setup_dir = app_dir.clone();
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            diagnostics::log(
                &config::app_directory(),
                "INFO",
                "Uma segunda instância foi solicitada; trazendo a janela existente para frente.",
            );
            show_existing_instance(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::close_settings
        ])
        .setup(move |app| {
            let dir = setup_dir.clone();
            std::fs::create_dir_all(&dir)?;
            std::fs::create_dir_all(dir.join("webview-data"))?;
            diagnostics::log(&dir, "INFO", "Preparando configuração e janela principal.");

            let mut cfg = config::load(&dir);
            if let Ok(enabled) = app.autolaunch().is_enabled() {
                cfg.autostart = enabled;
            }
            let _ = config::save(&dir, &cfg);
            app.manage(AppState {
                config: std::sync::Mutex::new(cfg.clone()),
                app_dir: dir.clone(),
                registered_shortcut: std::sync::Mutex::new(String::new()),
            });

            let shortcut_text = cfg.global_shortcut.clone();
            let registered = Shortcut::from_str(&shortcut_text)
                .ok()
                .is_some_and(|shortcut| app.global_shortcut().register(shortcut).is_ok());
            if let Ok(mut current) = app.state::<AppState>().registered_shortcut.lock() {
                *current = if registered {
                    shortcut_text
                } else {
                    String::new()
                };
            }
            if !registered {
                diagnostics::log(
                    &dir,
                    "WARN",
                    "O atalho configurado não pôde ser registrado; tentando CTRL+ALT+C.",
                );
                if let Ok(shortcut) = Shortcut::from_str("CTRL+ALT+C") {
                    let _ = app.global_shortcut().register(shortcut);
                }
            }

            let main_window = window::create_main(app.handle(), &cfg, &dir)?;
            diagnostics::log(
                &dir,
                "INFO",
                format!(
                    "Janela principal criada. URL configurada: {}",
                    diagnostics::safe_url_summary(&cfg.site_url)
                ),
            );
            let handle = app.handle().clone();
            let main_for_events = main_window.clone();
            let event_diag_dir = dir.clone();
            main_window.on_window_event(move |event| match event {
                WindowEvent::CloseRequested { api, .. } => {
                    diagnostics::log(
                        &event_diag_dir,
                        "INFO",
                        "Pedido de fechar a janela: mantendo o aplicativo na bandeja.",
                    );
                    api.prevent_close();
                    let _ = main_for_events.hide();
                }
                WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
                    window::save_window_state(&handle, event);
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    diagnostics::log(
                        &event_diag_dir,
                        "INFO",
                        "Mudança de escala/DPI detectada; atualizando os cantos nativos.",
                    );
                    window::refresh_native_corners(&handle);
                }
                WindowEvent::Destroyed => {
                    diagnostics::log(
                        &event_diag_dir,
                        "WARN",
                        "A janela principal foi destruída enquanto o processo estava ativo.",
                    );
                }
                _ => {}
            });

            tray::setup(app.handle())?;
            window::start_auto_reload(app.handle().clone());
            diagnostics::start_watchdog(app.handle().clone(), dir.clone());
            diagnostics::log(&dir, "INFO", "Inicialização concluída.");
            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(error) = result {
        diagnostics::log(
            &app_dir,
            "FATAL",
            format!("O loop principal do Tauri terminou com erro: {error}"),
        );
        #[cfg(windows)]
        show_message(
            "Widget Website — erro ao iniciar",
            &format!(
                "O widget encontrou um erro ao iniciar. Consulte o diagnóstico em:\n{}\n\nDetalhe: {}",
                diagnostics::log_path(&app_dir).display(),
                error
            ),
        );
    }
}
