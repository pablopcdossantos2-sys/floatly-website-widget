use crate::{commands, config::AppState, diagnostics, i18n};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

fn create_menu(app: &AppHandle, language: &str) -> tauri::Result<Menu<tauri::Wry>> {
    let locked = app.state::<AppState>().snapshot().locked;
    let text = i18n::tray_labels(language, locked);
    let toggle = MenuItem::with_id(app, "show-hide", text.show_hide, true, None::<&str>)?;
    let reload = MenuItem::with_id(app, "reload", text.reload, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", text.settings, true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let lock = MenuItem::with_id(app, "locked", text.lock, true, None::<&str>)?;
    let top = MenuItem::with_id(app, "top", text.topmost, true, None::<&str>)?;
    let login = MenuItem::with_id(app, "login", text.login, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", text.quit, true, None::<&str>)?;
    Menu::with_items(
        app,
        &[
            &toggle, &reload, &settings, &separator, &lock, &top, &login, &quit,
        ],
    )
}

pub fn update_language(app: &AppHandle, language: &str) {
    if let Some(tray) = app.tray_by_id("widget-website-tray") {
        match create_menu(app, language) {
            Ok(menu) => {
                if let Err(error) = tray.set_menu(Some(menu)) {
                    diagnostics::log(
                        &app.state::<AppState>().app_dir,
                        "WARN",
                        format!("Não foi possível atualizar o menu da bandeja: {error}"),
                    );
                }
            }
            Err(error) => diagnostics::log(
                &app.state::<AppState>().app_dir,
                "WARN",
                format!("Não foi possível reconstruir o menu da bandeja: {error}"),
            ),
        }
        let text = i18n::tray_labels(language, false);
        if let Err(error) = tray.set_tooltip(Some(text.tooltip)) {
            diagnostics::log(
                &app.state::<AppState>().app_dir,
                "WARN",
                format!("Não foi possível atualizar a dica da bandeja: {error}"),
            );
        }
    }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let language = app.state::<AppState>().snapshot().language;
    let menu = create_menu(app, &language)?;
    let text = i18n::tray_labels(&language, false);
    let mut builder = TrayIconBuilder::with_id("widget-website-tray")
        .menu(&menu)
        .tooltip(text.tooltip);
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
