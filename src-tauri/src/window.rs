use crate::config::{AppConfig, AppState};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
};
use tauri::{
    utils::config::WebviewUrl,
    webview::{NewWindowResponse, PageLoadEvent, WebviewWindowBuilder},
    AppHandle, Emitter, Listener, LogicalSize, Manager, PhysicalPosition, PhysicalSize,
    WebviewWindow, WindowEvent,
};
use url::Url;
static LOGIN_ID: AtomicU64 = AtomicU64::new(1);
const INIT: &str = r#"(function boot(){
const localTauriOrigin=(location.protocol==='tauri:'&&location.host==='localhost')||(location.protocol==='http:'&&['tauri.localhost','127.0.0.1','localhost'].includes(location.host));
if(!(/^https?:$/.test(location.protocol)||localTauriOrigin)||window.top!==window)return;
if(!document.documentElement){document.addEventListener('DOMContentLoaded',boot,{once:true});return;}
let UI_LANGUAGE='__WIDGET_LANGUAGE__';
let DRAG_LOCKED='__WIDGET_LOCKED__'==='true';
const ACTION_TOKEN='__WIDGET_ACTION_TOKEN__';
const HEARTBEAT_TOKEN='__WIDGET_HEARTBEAT_TOKEN__';
const UI_COPY={
 'pt-BR':{dragTitle:'Arraste a borda azul para mover o widget.',dragLabel:'Mover o widget pela borda azul',offlineTitle:'Sem conexão com o calendário',offlineMessage:'O site não respondeu. Verifique a internet e tente novamente.',retry:'Tentar novamente'},
 en:{dragTitle:'Drag the blue border to move the widget.',dragLabel:'Move the widget using the blue border',offlineTitle:'Calendar connection unavailable',offlineMessage:'The website did not respond. Check your connection and try again.',retry:'Try again'},
 es:{dragTitle:'Arrastra el borde azul para mover el widget.',dragLabel:'Mover el widget desde el borde azul',offlineTitle:'Sin conexión con el calendario',offlineMessage:'El sitio no respondió. Comprueba la conexión e inténtalo de nuevo.',retry:'Reintentar'}
};
function setUiLanguage(language){UI_LANGUAGE=UI_COPY[language]?language:'pt-BR';const copy=UI_COPY[UI_LANGUAGE];const drag=document.getElementById('__widget_drag');if(drag){drag.title=copy.dragTitle;drag.setAttribute('aria-label',copy.dragLabel)}document.querySelectorAll('#__widget_drag_frame .__widget_drag_edge').forEach(edge=>{edge.title=copy.dragTitle;edge.setAttribute('aria-label',copy.dragLabel)});const overlay=document.getElementById('__widget_offline');if(overlay){const heading=overlay.querySelector('h2');const paragraph=overlay.querySelector('p');const button=overlay.querySelector('button');if(heading)heading.textContent=copy.offlineTitle;if(paragraph)paragraph.textContent=copy.offlineMessage;if(button)button.textContent=copy.retry}}
window.__widgetSetLanguage=setUiLanguage;
const style=document.createElement('style');
style.textContent=`
#__widget_drag{position:fixed;z-index:2147483647;left:50%;transform:translateX(-50%);top:8px;width:78px;height:20px;display:flex;align-items:center;justify-content:center;background:rgba(33,52,75,.22);border:1px solid rgba(255,255,255,.72);border-radius:999px;opacity:.78;cursor:move;transition:background .12s,opacity .12s,box-shadow .12s;box-shadow:0 1px 4px rgba(0,0,0,.14);user-select:none;-webkit-user-select:none;touch-action:none}
#__widget_drag::before{content:'⋮⋮';font:700 13px/1 Segoe UI,Arial,sans-serif;letter-spacing:2px;color:rgba(25,45,69,.96)}
#__widget_drag:hover{opacity:1;background:rgba(24,105,210,.88);box-shadow:0 2px 8px rgba(0,0,0,.2)}
#__widget_drag.__widget_drag_locked{opacity:.28;cursor:not-allowed;pointer-events:none}
#__widget_drag_frame{position:fixed;z-index:2147483644;inset:10px;border:4px solid #1683ff;border-radius:8px;box-sizing:border-box;pointer-events:none;box-shadow:0 0 0 1px rgba(255,255,255,.9),0 0 8px rgba(22,131,255,.38),inset 0 0 0 1px rgba(255,255,255,.45);transition:border-color .12s,box-shadow .12s}
#__widget_drag_frame:hover{border-color:#005fd4;box-shadow:0 0 0 1px rgba(255,255,255,.96),0 0 10px rgba(22,131,255,.52),inset 0 0 0 1px rgba(255,255,255,.55)}
#__widget_drag_frame.__widget_drag_locked{border-color:rgba(115,126,139,.6);box-shadow:none}
#__widget_drag_frame.__widget_drag_locked .__widget_drag_edge{pointer-events:none;cursor:default}
#__widget_drag_frame .__widget_drag_edge{position:absolute;display:block;pointer-events:auto;background:transparent;touch-action:none;user-select:none;-webkit-user-select:none}
#__widget_drag_frame .__widget_drag_edge:hover{background:rgba(24,105,210,.16)}
#__widget_drag_frame .__widget_drag_edge--top{top:-3px;left:0;right:0;height:9px;cursor:move}
#__widget_drag_frame .__widget_drag_edge--bottom{bottom:-3px;left:0;right:0;height:9px;cursor:move}
#__widget_drag_frame .__widget_drag_edge--left{left:-3px;top:0;bottom:0;width:9px;cursor:move}
#__widget_drag_frame .__widget_drag_edge--right{right:-3px;top:0;bottom:0;width:9px;cursor:move}
#__widget_offline{position:fixed;z-index:2147483646;inset:0;background:rgba(248,250,253,.98);color:#203247;display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center;font:14px Segoe UI,Arial,sans-serif;padding:24px}
#__widget_offline button{border:0;background:#1769d2;color:#fff;border-radius:8px;padding:10px 14px}
html{overflow-y:auto!important}
html,body,*{scrollbar-width:thin;scrollbar-color:transparent transparent}
html::-webkit-scrollbar,body::-webkit-scrollbar,*::-webkit-scrollbar{width:9px;height:9px}
html::-webkit-scrollbar-track,body::-webkit-scrollbar-track,*::-webkit-scrollbar-track{background:transparent}
html::-webkit-scrollbar-thumb,body::-webkit-scrollbar-thumb,*::-webkit-scrollbar-thumb{background-color:transparent;border:2px solid transparent;border-radius:9px;background-clip:padding-box;transition:background-color .16s ease}
html.__widget_scroll_active,html.__widget_scroll_active body,html.__widget_scroll_active *{scrollbar-color:rgba(100,116,139,.82) transparent!important}
html.__widget_scroll_active::-webkit-scrollbar-thumb,html.__widget_scroll_active body::-webkit-scrollbar-thumb,html.__widget_scroll_active *::-webkit-scrollbar-thumb{background-color:rgba(100,116,139,.78)!important}
`;
(document.head||document.documentElement).appendChild(style);
function action(p){location.href='widget-action://'+ACTION_TOKEN+'/'+p}
function setUiLocked(locked){
 DRAG_LOCKED=Boolean(locked);
 const frame=document.getElementById('__widget_drag_frame');
 const handle=document.getElementById('__widget_drag');
 if(frame)frame.classList.toggle('__widget_drag_locked',DRAG_LOCKED);
 if(handle)handle.classList.toggle('__widget_drag_locked',DRAG_LOCKED);
}
window.__widgetSetLocked=setUiLocked;
function dragWindow(event){
 if(!event.isTrusted||event.button!==0)return;
 if(DRAG_LOCKED){event.preventDefault();event.stopPropagation();if(typeof event.stopImmediatePropagation==='function')event.stopImmediatePropagation();return;}
 event.preventDefault();
 event.stopPropagation();
 if(typeof event.stopImmediatePropagation==='function')event.stopImmediatePropagation();
 action('drag');
}
function install(){
 if(!document.documentElement)return;
 if(!document.getElementById('__widget_drag_frame')){
  const frame=document.createElement('div');frame.id='__widget_drag_frame';frame.setAttribute('aria-label','Borda para mover o widget');if(DRAG_LOCKED)frame.classList.add('__widget_drag_locked');
  for(const side of ['top','bottom','left','right']){
   const edge=document.createElement('div');edge.className='__widget_drag_edge __widget_drag_edge--'+side;edge.title=UI_COPY[UI_LANGUAGE]?.dragTitle||UI_COPY['pt-BR'].dragTitle;edge.setAttribute('aria-label',UI_COPY[UI_LANGUAGE]?.dragLabel||UI_COPY['pt-BR'].dragLabel);edge.addEventListener('pointerdown',dragWindow);frame.appendChild(edge);
  }
  document.documentElement.appendChild(frame);
 }
 if(!document.body||document.getElementById('__widget_drag'))return;
 const d=document.createElement('div');d.id='__widget_drag';const copy=UI_COPY[UI_LANGUAGE]||UI_COPY['pt-BR'];d.title=copy.dragTitle;d.setAttribute('aria-label',copy.dragLabel);if(DRAG_LOCKED)d.classList.add('__widget_drag_locked');d.addEventListener('pointerdown',dragWindow);document.body.appendChild(d);
}
if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',install,{once:true});else install();
const dragObserver=new MutationObserver(()=>install());
dragObserver.observe(document.documentElement,{childList:true,subtree:true});
document.addEventListener('pointerdown',e=>{
 if(!e.isTrusted||e.button!==0||DRAG_LOCKED)return;
 const x=e.clientX,y=e.clientY,w=innerWidth,h=innerHeight;
 if(x<6||x>w-6||y<6||y>h-6){
  const ns=y<6?'north':y>h-6?'south':'';
  const ew=x<6?'west':x>w-6?'east':'';
  const dir=ns&&ew?ns+'-'+ew:(ns||ew);
  if(dir){e.preventDefault();action('resize/'+dir)}
 }
},true);
document.addEventListener('contextmenu',e=>{
 const target=e.target;
 if(target instanceof Element&&target.closest('input,textarea,select,[contenteditable=""],[contenteditable="true"]'))return;
 if(window.getSelection()&&window.getSelection().toString())return;
 e.preventDefault();
},true);
document.addEventListener('keydown',e=>{
 const target=e.target;
 if(target instanceof Element&&target.closest('input,textarea,select,[contenteditable=""],[contenteditable="true"]'))return;
 if((e.ctrlKey||e.metaKey)&&['l','t','n','w','p','j','u','s'].includes(e.key.toLowerCase())){e.preventDefault();e.stopImmediatePropagation()}
 else if(e.key==='F5'){e.preventDefault();location.reload()}
},true);
let scrollHideTimer=0;
function revealScrollbar(){const root=document.documentElement;if(!root)return;root.classList.add('__widget_scroll_active');if(scrollHideTimer)clearTimeout(scrollHideTimer);scrollHideTimer=setTimeout(()=>root.classList.remove('__widget_scroll_active'),1200)}
document.addEventListener('wheel',revealScrollbar,{passive:true,capture:true});
document.addEventListener('scroll',revealScrollbar,{passive:true,capture:true});
document.addEventListener('pointermove',e=>{if(innerWidth-e.clientX<=24)revealScrollbar()},{passive:true});
async function check(){try{const c=new AbortController(),t=setTimeout(()=>c.abort(),7000);try{await fetch(new URL('/favicon.ico',location.origin),{mode:'no-cors',cache:'no-store',signal:c.signal})}finally{clearTimeout(t)}document.getElementById('__widget_offline')?.remove()}catch(_){if(!document.getElementById('__widget_offline')){const b=document.createElement('div');b.id='__widget_offline';const copy=UI_COPY[UI_LANGUAGE]||UI_COPY['pt-BR'];b.innerHTML='<div style="font-size:30px">⌁</div><h2></h2><p></p><button type="button"></button>';b.querySelector('h2').textContent=copy.offlineTitle;b.querySelector('p').textContent=copy.offlineMessage;b.querySelector('button').textContent=copy.retry;(document.body||document.documentElement).appendChild(b);b.querySelector('button').addEventListener('click',()=>location.reload())}}
}
addEventListener('offline',check);
setInterval(check,30000);
function heartbeat(){try{const events=window.__TAURI__&&window.__TAURI__.event;if(events&&typeof events.emit==='function')events.emit('widget-heartbeat',HEARTBEAT_TOKEN).catch(()=>{})}catch(_){}}
setTimeout(heartbeat,1000);
setInterval(heartbeat,5000)
})();"#;
const LOGIN_INIT: &str = r#"(function(){
if(window.top!==window||!/^https?:$/.test(location.protocol))return;
const UI_LANGUAGE='__WIDGET_LANGUAGE__';
const CLOSE_TOKEN='__WIDGET_LOGIN_CLOSE_TOKEN__';
const TEXT=UI_LANGUAGE==='en'
?{login:'Sign in',back:'Back',reload:'Reload',address:'Website address',go:'Go',close:'Close',invalidAddress:'Enter a valid HTTP or HTTPS address.'}
:UI_LANGUAGE==='es'
?{login:'Iniciar sesión',back:'Volver',reload:'Actualizar',address:'Dirección del sitio',go:'Ir',close:'Cerrar',invalidAddress:'Introduce una dirección HTTP o HTTPS válida.'}
:{login:'Login',back:'Voltar',reload:'Recarregar',address:'Endereço do site',go:'Ir',close:'Fechar',invalidAddress:'Informe um endereço HTTP ou HTTPS válido.'};
function addToolbar(){
 if(!document.documentElement||document.getElementById('__widget_login_bar'))return;
 const style=document.createElement('style');
 style.textContent='#__widget_login_bar{position:fixed;z-index:2147483647;left:0;right:0;top:0;height:46px;display:flex;align-items:center;gap:6px;padding:6px 9px;background:#edf2f8;border-bottom:1px solid #cad5e3;box-shadow:0 2px 8px #152b451a;font:12px Segoe UI,Arial,sans-serif;color:#213247}#__widget_login_bar button{flex:none;border:1px solid #d0d9e5;background:#fff;color:#23364d;border-radius:6px;padding:6px 9px;font:600 12px Segoe UI,Arial,sans-serif;cursor:pointer}#__widget_login_bar input{flex:1;min-width:30px;border:1px solid #cbd6e3;border-radius:6px;padding:7px 9px;background:#fff;color:#24364a;font:12px Segoe UI,Arial,sans-serif;outline:none}#__widget_login_bar .label{font-weight:700;white-space:nowrap}';
 (document.head||document.documentElement).appendChild(style);
 const bar=document.createElement('div');bar.id='__widget_login_bar';
 const label=document.createElement('span');label.className='label';label.textContent=TEXT.login;
 const back=document.createElement('button');back.type='button';back.textContent='←';back.title=TEXT.back;back.setAttribute('aria-label',TEXT.back);
 back.addEventListener('click',function(){history.back()});
 const reload=document.createElement('button');reload.type='button';reload.textContent='↻';reload.title=TEXT.reload;
 reload.addEventListener('click',function(){location.reload()});
 const address=document.createElement('input');address.type='text';address.value=location.href;address.setAttribute('aria-label',TEXT.address);
 address.addEventListener('focus',function(){address.select()});
 function go(){let v=address.value.trim();try{const u=new URL(v.includes('://')?v:'https://'+v);if(u.protocol!=='https:'&&u.protocol!=='http:')throw new Error('scheme');location.href=u.href}catch(_){address.setCustomValidity(TEXT.invalidAddress);address.reportValidity();address.setCustomValidity('')}}
 address.addEventListener('keydown',function(e){if(e.key==='Enter'){e.preventDefault();go()}});
 const button=document.createElement('button');button.type='button';button.textContent=TEXT.go;button.addEventListener('click',go);
 const close=document.createElement('button');close.type='button';close.textContent=TEXT.close;close.addEventListener('click',function(){location.href='widget-login-action://'+CLOSE_TOKEN+'/close'});
 bar.append(label,back,address,button,reload,close);document.documentElement.appendChild(bar);
 document.documentElement.style.setProperty('padding-top','46px','important');document.documentElement.style.setProperty('scroll-padding-top','50px','important');
}
if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',addToolbar,{once:true});else addToolbar();
})();"#;

fn allowed(host: &str, c: &AppConfig) -> bool {
    let target = Url::parse(&c.site_url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_ascii_lowercase));
    let h = host.to_ascii_lowercase();
    let site = target
        .as_ref()
        .is_some_and(|t| h == *t || h.ends_with(&format!(".{t}")));
    let auth = h == "accounts.google.com"
        || h == "appleid.apple.com"
        || h == "login.microsoftonline.com"
        || h == "facebook.com"
        || h.ends_with(".facebook.com")
        || h == "google.com"
        || h.ends_with(".google.com")
        || h == "investing.com"
        || h.ends_with(".investing.com");
    site || auth
}
fn is_auth_page(host: &str, path: &str, c: &AppConfig) -> bool {
    let h = host.to_ascii_lowercase();
    let p = path.to_ascii_lowercase();
    let configured = Url::parse(&c.site_url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_ascii_lowercase));
    let same_site = configured
        .as_ref()
        .is_some_and(|t| h == *t || h.ends_with(&format!(".{t}")));
    let provider = h == "accounts.google.com"
        || h == "google.com"
        || h.ends_with(".google.com")
        || h == "facebook.com"
        || h.ends_with(".facebook.com")
        || h == "appleid.apple.com"
        || h == "login.microsoftonline.com"
        || h == "login.live.com";
    let investing_auth = (h == "investing.com" || h.ends_with(".investing.com")) && !same_site;
    let login_path = [
        "/login",
        "/log-in",
        "/signin",
        "/sign-in",
        "/account",
        "/accounts",
        "/auth",
        "/oauth",
        "/callback",
    ]
    .iter()
    .any(|part| p.contains(part));
    provider || investing_auth || (same_site && login_path)
}
fn is_internal_app_url(url: &Url) -> bool {
    if url.scheme() == "tauri" {
        return true;
    }
    let host = url.host_str().unwrap_or("");
    (url.scheme() == "http" && host == "tauri.localhost")
        || (cfg!(debug_assertions) && host == "127.0.0.1")
}

fn is_configured_page(url: &Url, c: &AppConfig) -> bool {
    let Ok(target) = Url::parse(&c.site_url) else {
        return false;
    };
    let host = url.host_str().unwrap_or("").to_ascii_lowercase();
    let target_host = target.host_str().unwrap_or("").to_ascii_lowercase();
    let same = host == target_host || host.ends_with(&format!(".{target_host}"));
    let target_path = target.path().trim_end_matches('/');
    let current_path = url.path().trim_end_matches('/');
    same && (target_path.is_empty()
        || target_path == "/"
        || current_path == target_path
        || current_path.starts_with(&format!("{target_path}/")))
}
fn authorized_widget_action<'a>(url: &'a Url, expected_token: &str) -> Option<&'a str> {
    if url.scheme() != "widget-action"
        || url.host_str() != Some(expected_token)
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return None;
    }
    let action = url.path().trim_matches('/');
    if action == "drag" {
        return Some(action);
    }
    let direction = action.strip_prefix("resize/")?;
    [
        "north",
        "south",
        "east",
        "west",
        "north-east",
        "north-west",
        "south-east",
        "south-west",
    ]
    .contains(&direction)
    .then_some(action)
}

pub fn create_main(
    app: &AppHandle,
    c: &AppConfig,
    dir: &std::path::Path,
) -> tauri::Result<WebviewWindow> {
    let data = dir.join("webview-data");
    std::fs::create_dir_all(&data).ok();
    let diag_dir = dir.to_path_buf();
    let build_diag_dir = dir.to_path_buf();
    let popup_diag_dir = dir.to_path_buf();
    let nav_app = app.clone();
    let popup_app = app.clone();
    let action_token = uuid::Uuid::new_v4().simple().to_string();
    let heartbeat_token = uuid::Uuid::new_v4().simple().to_string();
    let init_script = INIT
        .replace("__WIDGET_LANGUAGE__", &c.language)
        .replace("__WIDGET_LOCKED__", &c.locked.to_string())
        .replace("__WIDGET_ACTION_TOKEN__", &action_token)
        .replace("__WIDGET_HEARTBEAT_TOKEN__", &heartbeat_token);
    let navigation_action_token = action_token.clone();
    let heartbeat_token = heartbeat_token.clone();
    let heartbeat_dir = dir.to_path_buf();
    let _heartbeat_listener = app.listen("widget-heartbeat", move |event| {
        let payload = event.payload();
        let value = serde_json::from_str::<String>(payload).unwrap_or_else(|_| payload.to_owned());
        if value == heartbeat_token {
            crate::diagnostics::record_heartbeat(&heartbeat_dir);
        }
    });
    let page_load_app = app.clone();
    let w=WebviewWindowBuilder::new(app,"main",WebviewUrl::App("index.html".into())).title("Widget Website").inner_size(c.width as f64,c.height as f64).min_inner_size(150.0,100.0).decorations(false).transparent(true).shadow(true).resizable(!c.locked).always_on_top(c.always_on_top).skip_taskbar(!c.show_in_taskbar).visible(true).data_directory(data).devtools(cfg!(debug_assertions)).zoom_hotkeys_enabled(false).initialization_script(init_script)
.on_page_load(move|window,payload|{
    if payload.event()!=PageLoadEvent::Finished{return;}
    let url=payload.url();
    if url.scheme()!="https"&&url.scheme()!="http"{return;}
    let config=page_load_app.state::<AppState>().config.lock().map(|value|value.clone()).unwrap_or_default();
    let language=serde_json::to_string(&config.language).unwrap_or_else(|_|"\"pt-BR\"".to_owned());
    let script=format!("(function(){{function apply(){{const root=document.documentElement;if(!root)return;root.style.opacity='{}';root.style.borderRadius='{}px';if(typeof window.__widgetSetLanguage==='function')window.__widgetSetLanguage({});if(typeof window.__widgetSetLocked==='function')window.__widgetSetLocked({});}}if(document.documentElement)apply();else document.addEventListener('DOMContentLoaded',apply,{{once:true}});}})();",config.opacity as f64/100.0,config.corner_radius,language,config.locked);
    let _=window.eval(&script);
})
.on_navigation(move|u|{
if is_internal_app_url(&u){
    if u.host_str()==Some("tauri.localhost"){
        crate::diagnostics::log(&diag_dir,"INFO","Origem interna do Tauri autorizada; carregando a interface local.");
    }
    return true;
}
if u.scheme()=="widget-action"{
    let action=match authorized_widget_action(&u,&navigation_action_token) {
        Some(action)=>action,
        None=>{
            crate::diagnostics::log(&diag_dir,"WARN","Comando de janela ignorado: token ou ação inválida.");
            return false;
        }
    };
    let state=nav_app.state::<AppState>();
    let locked=state.config.lock().map(|config|config.locked).unwrap_or(true);
    if locked {
        crate::diagnostics::log(&diag_dir,"WARN",format!("Ação de janela ignorada porque mover/redimensionar está travado: {action}"));
        return false;
    }
    if let Some(window)=nav_app.get_webview_window("main") {
        if action=="drag" {
            crate::diagnostics::log(&diag_dir,"INFO","Comando de movimento autenticado recebido; iniciando arraste nativo do Windows.");
            if let Err(error)=begin_native_drag(&window) {
                crate::diagnostics::log(&diag_dir,"ERRO",format!("O arraste nativo falhou: {error}"));
                let _=window.start_dragging();
            }
        } else if let Some(direction)=action.strip_prefix("resize/") {
            if ["north","south","east","west","north-east","north-west","south-east","south-west"].contains(&direction) {
                begin_native_resize(&window,direction);
            } else {
                crate::diagnostics::log(&diag_dir,"WARN","Comando de redimensionamento ignorado: direção inválida.");
            }
        } else {
            crate::diagnostics::log(&diag_dir,"WARN","Comando de janela ignorado: ação inválida.");
        }
    } else {
        crate::diagnostics::log(&diag_dir,"ERRO","Comando de movimento recebido sem a janela principal disponível.");
    }
    return false
}
if u.scheme()!="https"&&u.scheme()!="http"{crate::diagnostics::log(&diag_dir,"WARN",format!("Navegação bloqueada por esquema não HTTP(S): {}",u.scheme()));return false}
crate::diagnostics::log_navigation(&diag_dir,&u);
let cfg=nav_app.state::<AppState>().config.lock().map(|v|v.clone()).unwrap_or_default();let host=u.host_str().unwrap_or("");let target=Url::parse(&cfg.site_url).ok().and_then(|v|v.host_str().map(str::to_owned)).unwrap_or_default();
if host==target||host.ends_with(&format!(".{target}")){return true}
if allowed(host,&cfg){crate::diagnostics::log(&diag_dir,"INFO",format!("Abrindo fluxo de autenticação em janela secundária (host={host})."));let a=nav_app.clone();let dest=u.clone();thread::spawn(move||{let _=open_login(&a,dest);});return false}
crate::diagnostics::log(&diag_dir,"INFO",format!("Encaminhando link externo ao navegador padrão (host={host})."));let _=open::that(u.as_str());false
})
.on_new_window(move|u,_|{if u.scheme()=="http"||u.scheme()=="https"{crate::diagnostics::log_navigation(&popup_diag_dir,&u);let cfg=popup_app.state::<AppState>().config.lock().map(|v|v.clone()).unwrap_or_default();if allowed(u.host_str().unwrap_or(""),&cfg){let a=popup_app.clone();thread::spawn(move||{let _=open_login(&a,u);});}else{let _=open::that(u.as_str());}}NewWindowResponse::Deny}).build().map_err(|error|{crate::diagnostics::log(&build_diag_dir,"ERRO",format!("Falha ao criar a janela WebView2 principal: {error}"));error})?;
    restore_position(&w, c);
    apply_config(app, c);
    Ok(w)
}
#[cfg(windows)]
fn begin_native_drag(window: &WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, SendMessageW, HTCAPTION, WM_NCLBUTTONDOWN,
    };

    let raw_hwnd = window.hwnd().map_err(|error| error.to_string())?;
    let mut cursor: POINT = unsafe { std::mem::zeroed() };
    if unsafe { GetCursorPos(&mut cursor) } == 0 {
        return Err("Não foi possível obter a posição do ponteiro do mouse.".into());
    }
    let packed_position = ((cursor.x as u16 as u32) | ((cursor.y as u16 as u32) << 16)) as isize;

    unsafe {
        ReleaseCapture();
        SendMessageW(
            raw_hwnd.0,
            WM_NCLBUTTONDOWN,
            HTCAPTION as usize,
            packed_position,
        );
    }
    Ok(())
}

#[cfg(not(windows))]
fn begin_native_drag(window: &WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|error| error.to_string())
}

#[cfg(windows)]
fn begin_native_resize(window: &WebviewWindow, direction: &str) {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageW, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT,
        HTTOPRIGHT, WM_NCLBUTTONDOWN,
    };
    let hit_test = match direction {
        "north" => HTTOP,
        "south" => HTBOTTOM,
        "east" => HTRIGHT,
        "west" => HTLEFT,
        "north-east" => HTTOPRIGHT,
        "north-west" => HTTOPLEFT,
        "south-east" => HTBOTTOMRIGHT,
        "south-west" => HTBOTTOMLEFT,
        _ => return,
    };
    if let Ok(raw_hwnd) = window.hwnd() {
        let hwnd = raw_hwnd.0;
        unsafe {
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, hit_test as usize, 0);
        }
    }
}
#[cfg(not(windows))]
fn begin_native_resize(_window: &WebviewWindow, _direction: &str) {}

fn open_login(app: &AppHandle, url: Url) -> tauri::Result<()> {
    let id = LOGIN_ID.fetch_add(1, Ordering::Relaxed);
    let label = format!("login-{id}");
    let close_label = label.clone();
    let close_token = uuid::Uuid::new_v4().simple().to_string();
    let navigation_close_token = close_token.clone();
    let state = app.state::<AppState>();
    let diag_dir = state.app_dir.clone();
    let language = state
        .config
        .lock()
        .map(|config| config.language.clone())
        .unwrap_or_else(|_| "pt-BR".to_owned());
    let dir = state.app_dir.join("webview-data");
    std::fs::create_dir_all(&dir).ok();
    crate::diagnostics::log(
        &diag_dir,
        "INFO",
        format!(
            "Abrindo janela de autenticação (host={}).",
            url.host_str().unwrap_or("sem host")
        ),
    );
    let nav = app.clone();
    let pop = app.clone();
    let close_app = app.clone();
    let nav_seen_auth = Arc::new(AtomicBool::new(false));
    let popup_seen_auth = Arc::clone(&nav_seen_auth);
    let host = url.host_str().unwrap_or("Login").to_owned();
    let login_title = match language.as_str() {
        "en" => "Sign in",
        "es" => "Iniciar sesión",
        _ => "Entrar",
    };
    WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))
        .title(format!("{login_title} — {host}"))
        .inner_size(960.0, 760.0)
        .min_inner_size(600.0, 450.0)
        .decorations(true)
        .resizable(true)
        .skip_taskbar(false)
        .data_directory(dir)
        .devtools(cfg!(debug_assertions))
        .zoom_hotkeys_enabled(false)
        .initialization_script(
            LOGIN_INIT
                .replace("__WIDGET_LANGUAGE__", &language)
                .replace("__WIDGET_LOGIN_CLOSE_TOKEN__", &close_token),
        )
        .on_navigation(move |u| {
            if u.scheme() == "widget-login-action" {
                if u.host_str() == Some(navigation_close_token.as_str())
                    && u.path() == "/close"
                    && u.query().is_none()
                    && u.fragment().is_none()
                {
                    if let Some(w) = nav.get_webview_window(&close_label) {
                        let _ = w.close();
                    }
                } else {
                    crate::diagnostics::log(
                        &diag_dir,
                        "WARN",
                        "Pedido de fechamento de login ignorado: token ou ação inválida.",
                    );
                }
                return false;
            }
            if u.scheme() != "http" && u.scheme() != "https" {
                crate::diagnostics::log(
                    &diag_dir,
                    "WARN",
                    format!("Navegação de login bloqueada por esquema: {}", u.scheme()),
                );
                return false;
            }
            crate::diagnostics::log_navigation(&diag_dir, &u);
            let c = nav
                .state::<AppState>()
                .config
                .lock()
                .map(|v| v.clone())
                .unwrap_or_default();
            let host = u.host_str().unwrap_or("");
            if is_auth_page(host, u.path(), &c) {
                nav_seen_auth.store(true, Ordering::SeqCst);
                return true;
            }
            if nav_seen_auth.load(Ordering::SeqCst) && is_configured_page(&u, &c) {
                let a = close_app.clone();
                let label = close_label.clone();
                thread::spawn(move || {
                    if let Some(w) = a.get_webview_window(&label) {
                        let _ = w.close();
                    }
                });
                return false;
            }
            if allowed(host, &c) {
                true
            } else {
                let _ = open::that(u.as_str());
                false
            }
        })
        .on_new_window(move |u, _| {
            if u.scheme() == "http" || u.scheme() == "https" {
                let c = pop
                    .state::<AppState>()
                    .config
                    .lock()
                    .map(|v| v.clone())
                    .unwrap_or_default();
                if allowed(u.host_str().unwrap_or(""), &c) {
                    if is_auth_page(u.host_str().unwrap_or(""), u.path(), &c) {
                        popup_seen_auth.store(true, Ordering::SeqCst);
                    }
                    let a = pop.clone();
                    thread::spawn(move || {
                        let _ = open_login(&a, u);
                    });
                } else {
                    let _ = open::that(u.as_str());
                }
            }
            NewWindowResponse::Deny
        })
        .build()?;
    Ok(())
}
fn restore_position(w: &WebviewWindow, c: &AppConfig) {
    if !c.remember_position {
        let _ = w.center();
        return;
    }
    let (Some(x), Some(y)) = (c.x, c.y) else {
        let _ = w.center();
        return;
    };
    let Ok(monitors) = w.available_monitors() else {
        let _ = w.center();
        return;
    };
    let scale = w.scale_factor().unwrap_or(1.0);
    let s = PhysicalSize::new(
        (c.width as f64 * scale) as u32,
        (c.height as f64 * scale) as u32,
    );
    let visible = monitors.iter().any(|m| {
        let p = m.position();
        let z = m.size();
        (x + s.width as i32).min(p.x + z.width as i32) > x.max(p.x)
            && (y + s.height as i32).min(p.y + z.height as i32) > y.max(p.y)
    });
    if visible {
        let _ = w.set_position(PhysicalPosition::new(x, y));
    } else {
        let _ = w.center();
    }
}
#[cfg(windows)]
fn apply_native_corners(window: &WebviewWindow, radius: u8) {
    use std::ffi::c_void;
    use windows_sys::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE,
    };
    use windows_sys::Win32::Graphics::Gdi::{CreateRoundRectRgn, DeleteObject, SetWindowRgn};
    if let Ok(raw_hwnd) = window.hwnd() {
        let hwnd = raw_hwnd.0;
        let preference: u32 = if radius == 0 { 1 } else { 2 };
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                &preference as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
        if let Ok(size) = window.inner_size() {
            let scale = window.scale_factor().unwrap_or(1.0);
            let w = size.width.min(i32::MAX as u32) as i32;
            let h = size.height.min(i32::MAX as u32) as i32;
            let r = ((radius as f64 * scale).round() as i32).clamp(0, w.min(h) / 2);
            unsafe {
                if r == 0 {
                    let _ = SetWindowRgn(hwnd, std::ptr::null_mut(), 1);
                } else {
                    let region = CreateRoundRectRgn(0, 0, w + 1, h + 1, r * 2, r * 2);
                    if !region.is_null() && SetWindowRgn(hwnd, region, 1) == 0 {
                        // SetWindowRgn takes ownership only when it succeeds.
                        let _ = DeleteObject(region);
                    }
                }
            }
        }
    }
}
#[cfg(not(windows))]
fn apply_native_corners(_window: &WebviewWindow, _radius: u8) {}

pub fn refresh_native_corners(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let radius = app
            .state::<AppState>()
            .config
            .lock()
            .map(|c| c.corner_radius)
            .unwrap_or(12);
        apply_native_corners(&window, radius);
    }
}
pub fn apply_config(app: &AppHandle, c: &AppConfig) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_always_on_top(c.always_on_top);
        let _ = w.set_skip_taskbar(!c.show_in_taskbar);
        let _ = w.set_resizable(!c.locked);
        let _ = w.set_min_size(Some(LogicalSize::new(150.0, 100.0)));
        let _ = w.set_size(LogicalSize::new(c.width as f64, c.height as f64));
        let _ = w.set_zoom(c.zoom as f64 / 100.0);
        let language =
            serde_json::to_string(&c.language).unwrap_or_else(|_| "\"pt-BR\"".to_owned());
        let script=format!("document.documentElement.style.opacity='{}';document.documentElement.style.borderRadius='{}px';if(typeof window.__widgetSetLanguage==='function')window.__widgetSetLanguage({});if(typeof window.__widgetSetLocked==='function')window.__widgetSetLocked({});",c.opacity as f64/100.0,c.corner_radius,language,c.locked);
        let _ = w.eval(&script);
        let _ = w.set_shadow(c.corner_radius > 0);
        apply_native_corners(&w, c.corner_radius);
    }
}
pub fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let a = app.clone();
    thread::spawn(move || {
        let language = a
            .state::<AppState>()
            .config
            .lock()
            .map(|config| config.language.clone())
            .unwrap_or_else(|_| "pt-BR".to_owned());
        let title = match language.as_str() {
            "en" => "Settings — Widget Website",
            "es" => "Configuración — Widget Website",
            _ => "Configurações — Widget Website",
        };
        let dir = a.state::<AppState>().app_dir.join("settings-data");
        std::fs::create_dir_all(&dir).ok();
        let _ = WebviewWindowBuilder::new(&a, "settings", WebviewUrl::App("settings.html".into()))
            .title(title)
            .inner_size(760.0, 850.0)
            .min_inner_size(520.0, 550.0)
            .resizable(true)
            .data_directory(dir)
            .build();
    });
}
pub fn open_login_from_tray(app: &AppHandle) {
    let text = app
        .state::<AppState>()
        .config
        .lock()
        .map(|c| c.site_url.clone())
        .unwrap_or_else(|_| crate::config::DEFAULT_SITE.to_owned());
    if let Ok(url) = Url::parse(&text) {
        let a = app.clone();
        thread::spawn(move || {
            let _ = open_login(&a, url);
        });
    }
}
pub fn save_window_state(app: &AppHandle, e: &WindowEvent) {
    let state = app.state::<AppState>();
    let scale = app
        .get_webview_window("main")
        .and_then(|w| w.scale_factor().ok())
        .unwrap_or(1.0);
    let mut c = match state.config.lock() {
        Ok(v) => v,
        Err(_) => return,
    };
    if !c.remember_position {
        return;
    }
    match e {
        WindowEvent::Moved(p) => {
            c.x = Some(p.x);
            c.y = Some(p.y)
        }
        WindowEvent::Resized(z) => {
            c.width = ((z.width as f64 / scale).round() as u32).clamp(150, 2000);
            c.height = ((z.height as f64 / scale).round() as u32).clamp(100, 1600)
        }
        _ => return,
    }
    let copy = c.clone();
    drop(c);
    let _ = crate::config::save(&state.app_dir, &copy);
    let _ = app.emit("config-changed", &copy);
    if let Some(window) = app.get_webview_window("main") {
        apply_native_corners(&window, copy.corner_radius);
    }
}
pub fn start_auto_reload(app: AppHandle) {
    thread::spawn(move || {
        let mut seconds: u32 = 0;
        loop {
            thread::sleep(std::time::Duration::from_secs(10));
            let mins = app
                .state::<AppState>()
                .config
                .lock()
                .map(|c| c.auto_reload_minutes)
                .unwrap_or(0) as u32;
            if mins == 0 {
                seconds = 0;
                continue;
            }
            seconds += 10;
            if seconds >= mins * 60 {
                seconds = 0;
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.eval("location.reload()");
                }
            }
        }
    });
}
#[cfg(test)]
mod internal_app_navigation_tests {
    use super::*;

    #[test]
    fn widget_action_requires_the_current_random_token() {
        let token = "7f9a12345678";
        let valid = Url::parse(&format!("widget-action://{token}/drag")).unwrap();
        let invalid = Url::parse("widget-action://drag").unwrap();
        assert_eq!(authorized_widget_action(&valid, token), Some("drag"));
        assert_eq!(authorized_widget_action(&invalid, token), None);
        assert_eq!(authorized_widget_action(&valid, "different-token"), None);
    }

    #[test]
    fn widget_action_accepts_only_known_resize_directions() {
        let token = "7f9a12345678";
        let valid = Url::parse(&format!("widget-action://{token}/resize/north-west")).unwrap();
        let invalid = Url::parse(&format!("widget-action://{token}/resize/center")).unwrap();
        let query = Url::parse(&format!("widget-action://{token}/drag?bypass=true")).unwrap();
        assert_eq!(
            authorized_widget_action(&valid, token),
            Some("resize/north-west")
        );
        assert_eq!(authorized_widget_action(&invalid, token), None);
        assert_eq!(authorized_widget_action(&query, token), None);
    }

    #[test]
    fn allows_tauri_windows_asset_origin() {
        let url = Url::parse("http://tauri.localhost/").unwrap();
        assert!(is_internal_app_url(&url));
    }

    #[test]
    fn does_not_treat_investing_as_an_internal_app_origin() {
        let url = Url::parse("https://br.investing.com/economic-calendar").unwrap();
        assert!(!is_internal_app_url(&url));
    }

    #[test]
    fn allows_tauri_protocol() {
        let url = Url::parse("tauri://localhost/").unwrap();
        assert!(is_internal_app_url(&url));
    }
}
