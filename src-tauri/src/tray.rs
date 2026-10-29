use crate::{commands, config::AppState};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

fn labels(language: &str, locked: bool) -> [&'static str; 8] {
    match language {
        "en" => [
            "Show / hide",
            "Reload page",
            "Settings",
            if locked {
                "Unlock moving / resizing"
            } else {
                "Lock moving / resizing"
            },
            "Toggle always on top",
            "Sign in / login window",
            "Quit",
            "Widget Website — Economic Calendar",
        ],
        "es" => [
            "Mostrar / ocultar",
            "Actualizar página",
            "Configuración",
            if locked {
                "Desbloquear movimiento / tamaño"
            } else {
                "Bloquear movimiento / tamaño"
            },
            "Alternar siempre visible",
            "Iniciar sesión",
            "Salir",
            "Widget Website — Calendario económico",
        ],
        _ => [
            "Mostrar / ocultar",
            "Recarregar página",
            "Configurações",
            if locked {
                "Travar / destravar (travado)"
            } else {
                "Travar / destravar (destravado)"
            },
            "Alternar sempre no topo",
            "Entrar / mostrar janela de login",
            "Sair",
            "Widget Website — Calendário Econômico",
        ],
    }
}

fn create_menu(app: &AppHandle, language: &str) -> tauri::Result<Menu<tauri::Wry>> {
    let locked = app.state::<AppState>().snapshot().locked;
    let text = labels(language, locked);
    let toggle = MenuItem::with_id(app, "show-hide", text[0], true, None::<&str>)?;
    let reload = MenuItem::with_id(app, "reload", text[1], true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", text[2], true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let lock = MenuItem::with_id(app, "locked", text[3], true, None::<&str>)?;
    let top = MenuItem::with_id(app, "top", text[4], true, None::<&str>)?;
    let login = MenuItem::with_id(app, "login", text[5], true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", text[6], true, None::<&str>)?;
    Menu::with_items(
        app,
        &[
            &toggle, &reload, &settings, &separator, &lock, &top, &login, &quit,
        ],
    )
}

pub fn update_language(app: &AppHandle, language: &str) {
    if let Some(tray) = app.tray_by_id("widget-website-tray") {
        if let Ok(menu) = create_menu(app, language) {
            let _ = tray.set_menu(Some(menu));
        }
        let text = labels(language, false);
        let _ = tray.set_tooltip(Some(text[7]));
    }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let language = app.state::<AppState>().snapshot().language;
    let menu = create_menu(app, &language)?;
    let text = labels(&language, false);
    let mut builder = TrayIconBuilder::with_id("widget-website-tray")
        .menu(&menu)
        .tooltip(text[7]);
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    builder
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show-hide" => commands::toggle_main_visibility(app.clone()),
            "reload" => commands::reload_site(app.clone()),
            "settings" => commands::show_settings(app),
            "locked" => commands::toggle_locked(app),
            "top" => commands::toggle_topmost(app),
            "login" => commands::show_login(app),
            "quit" => commands::quit_app(app.clone()),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                commands::toggle_main_visibility(tray.app_handle().clone());
            }
        })
        .build(app)?;
    Ok(())
}
