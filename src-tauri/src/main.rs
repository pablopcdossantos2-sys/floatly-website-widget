mod commands;mod config;mod tray;mod window;
use config::AppState;
use std::str::FromStr;
use tauri::{AppHandle,Manager,WindowEvent};
use tauri_plugin_autostart::{MacosLauncher,ManagerExt};
use tauri_plugin_global_shortcut::{GlobalShortcutExt,Shortcut,ShortcutState};

#[cfg(windows)]
fn webview2_runtime_present()->bool{
 let bases=["ProgramFiles(x86)","ProgramFiles","LOCALAPPDATA"];
 bases.iter().filter_map(|key|std::env::var_os(key)).any(|base|{
  let folder=std::path::PathBuf::from(base).join("Microsoft").join("EdgeWebView").join("Application");
  if folder.join("msedgewebview2.exe").is_file(){return true}
  std::fs::read_dir(folder).map(|entries|entries.flatten().any(|entry|entry.path().join("msedgewebview2.exe").is_file())).unwrap_or(false)
 })
}
#[cfg(windows)]
fn show_webview2_missing_message(){
 use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW,MB_ICONERROR,MB_OK};
 let message="O Microsoft Edge WebView2 Runtime não foi encontrado.\\n\\nInstale o Runtime Evergreen oficial e abra o Widget Website novamente:\\nhttps://developer.microsoft.com/microsoft-edge/webview2/";
 let title="Widget Website — requisito ausente";
 let text:Vec<u16>=message.encode_utf16().chain(std::iter::once(0)).collect();
 let caption:Vec<u16>=title.encode_utf16().chain(std::iter::once(0)).collect();
 unsafe{MessageBoxW(std::ptr::null_mut(),text.as_ptr(),caption.as_ptr(),MB_OK|MB_ICONERROR);}
}

fn toggle(app:&AppHandle){if let Some(w)=app.get_webview_window("main"){if w.is_visible().unwrap_or(false){let _=w.hide();}else{let _=w.show();}}}
fn show_existing_instance(app:&AppHandle){if let Some(w)=app.get_webview_window("main"){let _=w.show();let _=w.set_focus();}}
fn main(){
 #[cfg(windows)]
 if !webview2_runtime_present(){show_webview2_missing_message();return;}
 tauri::Builder::default()
.plugin(tauri_plugin_single_instance::init(|app,_,_|show_existing_instance(app)))
.plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent,None))
.plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app,_,event|{if event.state()==ShortcutState::Pressed{toggle(app)}}).build())
.invoke_handler(tauri::generate_handler![commands::get_config,commands::save_config,commands::restore_default_size,commands::close_settings,commands::toggle_main_visibility,commands::reload_site,commands::quit_app])
.setup(|app|{let dir=config::app_directory();std::fs::create_dir_all(&dir)?;std::fs::create_dir_all(dir.join("webview-data"))?;let mut cfg=config::load(&dir);if let Ok(v)=app.autolaunch().is_enabled(){cfg.autostart=v}let _=config::save(&dir,&cfg);app.manage(AppState{config:std::sync::Mutex::new(cfg.clone()),app_dir:dir.clone(),registered_shortcut:std::sync::Mutex::new(String::new())});let text=cfg.global_shortcut.clone();let registered=Shortcut::from_str(&text).ok().is_some_and(|s|app.global_shortcut().register(s).is_ok());if let Ok(mut r)=app.state::<AppState>().registered_shortcut.lock(){*r=if registered{text}else{String::new()};}if !registered{if let Ok(s)=Shortcut::from_str("CTRL+ALT+C"){let _=app.global_shortcut().register(s);}}let main=window::create_main(app.handle(),&cfg,&dir)?;let handle=app.handle().clone();let main_for_events=main.clone();main.on_window_event(move|e|match e{WindowEvent::CloseRequested{api,..}=>{api.prevent_close();let _=main_for_events.hide();},WindowEvent::Moved(_)|WindowEvent::Resized(_)=>window::save_window_state(&handle,e),_=>{}});tray::setup(app.handle())?;window::start_auto_reload(app.handle().clone());Ok(())})
.run(tauri::generate_context!()).expect("erro ao iniciar Widget Website");}