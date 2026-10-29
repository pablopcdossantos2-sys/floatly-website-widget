use serde::{Deserialize,Serialize};
use std::{fs,path::PathBuf,sync::Mutex};
use url::Url;
pub const DEFAULT_SITE:&str="https://br.investing.com/economic-calendar";
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AppConfig{pub site_url:String,pub width:u32,pub height:u32,pub x:Option<i32>,pub y:Option<i32>,pub remember_position:bool,pub autostart:bool,pub always_on_top:bool,pub opacity:u8,pub zoom:u16,pub auto_reload_minutes:u16,pub show_in_taskbar:bool,pub locked:bool,pub global_shortcut:String,pub corner_radius:u8}
impl Default for AppConfig{fn default()->Self{Self{site_url:DEFAULT_SITE.into(),width:500,height:700,x:None,y:None,remember_position:true,autostart:false,always_on_top:false,opacity:100,zoom:100,auto_reload_minutes:0,show_in_taskbar:false,locked:false,global_shortcut:"CTRL+ALT+C".into(),corner_radius:12}}}
impl AppConfig{pub fn normalize(&mut self)->Result<(),String>{let mut s=self.site_url.trim().to_owned();if s.is_empty(){return Err("Informe a URL do site.".into())}if !s.contains("://"){s=format!("https://{s}")}let u=Url::parse(&s).map_err(|_|"URL inválida.".to_owned())?;if !["http","https"].contains(&u.scheme())||u.host_str().is_none()||!u.username().is_empty()||u.password().is_some(){return Err("Use uma URL HTTP/HTTPS válida, sem credenciais.".into())}self.site_url=u.to_string();self.width=self.width.clamp(150,2000);self.height=self.height.clamp(100,1600);self.opacity=self.opacity.clamp(30,100);self.zoom=self.zoom.clamp(50,200);if self.auto_reload_minutes>0&&self.auto_reload_minutes<5{self.auto_reload_minutes=5}self.auto_reload_minutes=self.auto_reload_minutes.min(1440);self.corner_radius=self.corner_radius.min(64);if self.global_shortcut.trim().is_empty(){return Err("Informe um atalho global.".into())}Ok(())}}
pub struct AppState{pub config:Mutex<AppConfig>,pub app_dir:PathBuf,pub registered_shortcut:Mutex<String>}
pub fn app_directory()->PathBuf{std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(||std::env::current_dir().unwrap_or_else(|_|PathBuf::from("."))).join("WidgetWebsite")}
pub fn load(dir:&PathBuf)->AppConfig{let mut c=fs::read(dir.join("config.json")).ok().and_then(|b|serde_json::from_slice::<AppConfig>(&b).ok()).unwrap_or_default();if c.normalize().is_err(){c=AppConfig::default()}c}
pub fn save(dir:&PathBuf,c:&AppConfig)->Result<(),String>{fs::create_dir_all(dir).map_err(|e|e.to_string())?;let tmp=dir.join("config.json.tmp");fs::write(&tmp,serde_json::to_vec_pretty(c).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;fs::rename(&tmp,dir.join("config.json")).or_else(|_|{fs::copy(&tmp,dir.join("config.json"))?;fs::remove_file(&tmp)}).map_err(|e|e.to_string())}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn defaults_match_widget_requirements() {
        let c = AppConfig::default();
        assert_eq!(c.site_url, DEFAULT_SITE);
        assert_eq!(c.width, 500);
        assert_eq!(c.height, 700);
        assert_eq!(c.zoom, 100);
        assert_eq!(c.opacity, 100);
        assert_eq!(c.global_shortcut, "CTRL+ALT+C");
    }

    #[test]
    fn normalization_prefixes_https_and_clamps_values() {
        let mut c = AppConfig::default();
        c.site_url = "investing.com".into();
        c.width = 1;
        c.height = 9999;
        c.opacity = 0;
        c.zoom = 250;
        c.auto_reload_minutes = 1;
        c.corner_radius = 100;
        c.normalize().unwrap();
        assert_eq!(c.site_url, "https://investing.com/");
        assert_eq!(c.width, 150);
        assert_eq!(c.height, 1600);
        assert_eq!(c.opacity, 30);
        assert_eq!(c.zoom, 200);
        assert_eq!(c.auto_reload_minutes, 5);
        assert_eq!(c.corner_radius, 64);
    }

    #[test]
    fn normalization_rejects_non_http_urls_and_credentials() {
        for invalid in ["file:///private/file", "ftp://example.com", "https://user:secret@example.com"] {
            let mut c = AppConfig::default();
            c.site_url = invalid.into();
            assert!(c.normalize().is_err(), "accepted invalid URL: {invalid}");
        }
    }

    #[test]
    fn corrupt_config_recovers_to_defaults() {
        let dir = std::env::temp_dir().join(format!("widget-website-config-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("config.json"), b"{ this is not valid json").unwrap();
        let config = load(&dir);
        assert_eq!(config.site_url, DEFAULT_SITE);
        assert_eq!(config.width, 500);
        fs::remove_dir_all(&dir).unwrap();
    }
}
