mod commands;mod config;mod tray;mod window;
use config::AppState;
use std::str::FromStr;
use tauri::{AppHandle,Manager,WindowEvent};
use tauri_plugin_autostart::{MacosLauncher,ManagerExt};
use tauri_plugin_global_shortcut::{GlobalShortcutExt,Shortcut,ShortcutState};
fn toggle(app:&AppHandle){if let Some(w)=app.get_webview_window("main"){if w.is_visible().unwrap_or(false){let _=w.hide();}else{let _=w.show();}}}
fn main(){tauri::Builder::default()
.plugin(tauri_plugin_single_instance::init(|app,_,_|toggle(app)))
.plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent,None))
.plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app,_,event|{if event.state()==ShortcutState::Pressed{toggle(app)}}).build())
.invoke_handler(tauri::generate_handler![commands::get_config,commands::save_config,commands::restore_defaults,commands::close_settings,commands::toggle_main_visibility,commands::reload_site,commands::quit_app])
.setup(|app|{let dir=config::app_directory();std::fs::create_dir_all(&dir)?;std::fs::create_dir_all(dir.join("webview-data"))?;let mut cfg=config::load(&dir);if let Ok(v)=app.autolaunch().is_enabled(){cfg.autostart=v}let _=config::save(&dir,&cfg);app.manage(AppState{config:std::sync::Mutex::new(cfg.clone()),app_dir:dir.clone(),registered_shortcut:std::sync::Mutex::new(String::new())});let text=cfg.global_shortcut.clone();let registered=Shortcut::from_str(&text).ok().is_some_and(|s|app.global_shortcut().register(s).is_ok());if let Ok(mut r)=app.state::<AppState>().registered_shortcut.lock(){*r=if registered{text}else{String::new()};}if !registered{if let Ok(s)=Shortcut::from_str("CTRL+ALT+C"){let _=app.global_shortcut().register(s);}}let main=window::create_main(app.handle(),&cfg,&dir)?;let handle=app.handle().clone();main.on_window_event(move|e|if matches!(e,WindowEvent::Moved(_)|WindowEvent::Resized(_)){window::save_window_state(&handle,e)});tray::setup(app.handle())?;window::start_auto_reload(app.handle().clone());Ok(())})
.run(tauri::generate_context!()).expect("erro ao iniciar Widget Website");}