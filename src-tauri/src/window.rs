use crate::config::{AppConfig, AppState};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use tauri::{
    utils::config::WebviewUrl,
    webview::{NewWindowResponse, PageLoadEvent, WebviewWindowBuilder},
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewWindow,
    WindowEvent,
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
function setInlineStyles(element, values){
 for(const [property,value] of Object.entries(values))element.style.setProperty(property,value);
}
function applyDragLockedUi(){
 const locked=DRAG_LOCKED,frame=document.getElementById('__widget_drag_frame'),handle=document.getElementById('__widget_drag');
 if(frame){
  frame.classList.toggle('__widget_drag_locked',locked);
  frame.style.setProperty('border-color',locked?'rgba(115,126,139,.72)':'#1683ff');
  frame.style.setProperty('box-shadow',locked?'none':'0 0 0 1px rgba(255,255,255,.9),0 0 8px rgba(22,131,255,.38),inset 0 0 0 1px rgba(255,255,255,.45)');
  frame.querySelectorAll('.__widget_drag_edge').forEach(edge=>{
   edge.style.setProperty('pointer-events',locked?'none':'auto');
   edge.style.setProperty('cursor',locked?'default':'move');
  });
 }
 if(handle){
  handle.classList.toggle('__widget_drag_locked',locked);
  handle.style.setProperty('opacity',locked?'.28':'.95');
  handle.style.setProperty('pointer-events',locked?'none':'auto');
  handle.style.setProperty('cursor',locked?'not-allowed':'move');
 }
}
function setUiLocked(locked){DRAG_LOCKED=Boolean(locked);applyDragLockedUi()}
window.__widgetSetLocked=setUiLocked;
function dragWindow(event){
 if(!event.isTrusted||event.button!==0)return;
 if(DRAG_LOCKED){event.preventDefault();event.stopPropagation();if(typeof event.stopImmediatePropagation==='function')event.stopImmediatePropagation();return;}
 event.preventDefault();event.stopPropagation();
 if(typeof event.stopImmediatePropagation==='function')event.stopImmediatePropagation();
 action('drag');
}
function install(){
 if(!document.documentElement)return;
 let frame=document.getElementById('__widget_drag_frame');
 if(!frame){
  frame=document.createElement('div');frame.id='__widget_drag_frame';
  frame.setAttribute('aria-label','Borda para mover o widget');
  setInlineStyles(frame,{
   position:'fixed','z-index':'2147483646',top:'10px',right:'10px',bottom:'10px',left:'10px',
   border:'4px solid #1683ff','border-radius':'8px','box-sizing':'border-box','pointer-events':'none',
   'box-shadow':'0 0 0 1px rgba(255,255,255,.9),0 0 8px rgba(22,131,255,.38),inset 0 0 0 1px rgba(255,255,255,.45)',
   transition:'border-color .12s,box-shadow .12s'
  });
  const positions={
   top:{top:'-3px',left:'0',right:'0',height:'9px'},
   bottom:{bottom:'-3px',left:'0',right:'0',height:'9px'},
   left:{left:'-3px',top:'0',bottom:'0',width:'9px'},
   right:{right:'-3px',top:'0',bottom:'0',width:'9px'}
  };
  for(const side of ['top','bottom','left','right']){
   const edge=document.createElement('div');edge.className='__widget_drag_edge __widget_drag_edge--'+side;
   edge.title=UI_COPY[UI_LANGUAGE]?.dragTitle||UI_COPY['pt-BR'].dragTitle;
   edge.setAttribute('aria-label',UI_COPY[UI_LANGUAGE]?.dragLabel||UI_COPY['pt-BR'].dragLabel);
   setInlineStyles(edge,Object.assign({
    position:'absolute',display:'block','pointer-events':'auto',background:'transparent',
    'touch-action':'none','user-select':'none','-webkit-user-select':'none',cursor:'move'
   },positions[side]));
   edge.addEventListener('pointerdown',dragWindow);frame.appendChild(edge);
  }
  document.documentElement.appendChild(frame);
 }
 if(document.body&&!document.getElementById('__widget_drag')){
  const handle=document.createElement('div');handle.id='__widget_drag';handle.textContent='⋮⋮';
  const copy=UI_COPY[UI_LANGUAGE]||UI_COPY['pt-BR'];handle.title=copy.dragTitle;
  handle.setAttribute('aria-label',copy.dragLabel);
  setInlineStyles(handle,{
   position:'fixed','z-index':'2147483647',left:'50%',transform:'translateX(-50%)',top:'8px',
   width:'78px',height:'20px',display:'flex','align-items':'center','justify-content':'center',
   background:'rgba(33,52,75,.42)',border:'1px solid rgba(255,255,255,.9)','border-radius':'999px',
   opacity:'.95',cursor:'move',transition:'background .12s,opacity .12s,box-shadow .12s',
   'box-shadow':'0 1px 4px rgba(0,0,0,.28)','user-select':'none','-webkit-user-select':'none',
   'touch-action':'none',font:'700 13px/1 Segoe UI,Arial,sans-serif','letter-spacing':'2px',color:'#fff'
  });
  handle.addEventListener('pointerenter',()=>{if(!DRAG_LOCKED){handle.style.setProperty('background','rgba(24,105,210,.96)');handle.style.setProperty('opacity','1')}});
  handle.addEventListener('pointerleave',()=>{if(!DRAG_LOCKED){handle.style.setProperty('background','rgba(33,52,75,.42)');handle.style.setProperty('opacity','.95')}});
  handle.addEventListener('pointerdown',dragWindow);document.body.appendChild(handle);
 }
 applyDragLockedUi();
}
if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',install,{once:true});else install();
const dragObserver=new MutationObserver(()=>install());
dragObserver.observe(document.documentElement,{childList:true,subtree:true});
document.addEventListener('pointerdown',e=>{
 if(!e.isTrusted||e.button!==0||DRAG_LOCKED)return;
 const x=e.clientX,y=e.clientY,w=innerWidth,h=innerHeight;
 const band=6/(window.devicePixelRatio||1);
 if(x<band||x>w-band||y<band||y>h-band){
  const ns=y<band?'north':y>h-band?'south':'';
  const ew=x<band?'west':x>w-band?'east':'';
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
function heartbeat(){try{const core=window.__TAURI__&&window.__TAURI__.core;if(core&&typeof core.invoke==='function')core.invoke('heartbeat',{token:HEARTBEAT_TOKEN}).catch(()=>{})}catch(_){}}
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavAction {
    Allow,
    OpenLogin,
    External,
    Block,
}

const LOGIN_SEGMENTS: &[&str] = &[
    "login", "log-in", "signin", "sign-in", "account", "accounts", "auth", "oauth", "callback",
];

fn normalized_host(host: &str) -> String {
    host.trim().trim_end_matches('.').to_ascii_lowercase()
}

/// Match an exact host or a real DNS subdomain, never a text-only suffix.
fn host_matches(host: &str, target: &str) -> bool {
    let host = normalized_host(host);
    let target = normalized_host(target);
    if host.is_empty() || target.is_empty() {
        return false;
    }
    if host == target {
        return true;
    }
    // IP literals and single host labels must not be treated as registrable domains.
    if host.parse::<std::net::IpAddr>().is_ok() || target.parse::<std::net::IpAddr>().is_ok() {
        return false;
    }
    host.ends_with(&format!(".{target}"))
}

fn configured_host(c: &AppConfig) -> Option<String> {
    Url::parse(&c.site_url)
        .ok()
        .and_then(|url| url.host_str().map(normalized_host))
}

fn is_auth_provider(host: &str, path: &str) -> bool {
    let host = normalized_host(host);
    match host.as_str() {
        "accounts.google.com"
        | "appleid.apple.com"
        | "login.microsoftonline.com"
        | "login.live.com" => true,
        "facebook.com" | "www.facebook.com" | "m.facebook.com" => {
            let path = path.to_ascii_lowercase();
            let segments = path.split('/').filter(|segment| !segment.is_empty()).collect::<Vec<_>>();
            segments.iter().any(|segment| {
                matches!(*segment, "login" | "log-in" | "signin" | "sign-in" | "dialog" | "oauth" | "authorize")
            })
        }
        _ => false,
    }
}

fn is_login_path(path: &str) -> bool {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .any(|segment| LOGIN_SEGMENTS.contains(&segment.to_ascii_lowercase().as_str()))
}

fn is_auth_page(host: &str, path: &str, c: &AppConfig) -> bool {
    if is_auth_provider(host, path) {
        return true;
    }
    configured_host(c).is_some_and(|target| host_matches(host, &target) && is_login_path(path))
}

fn is_configured_page(url: &Url, c: &AppConfig) -> bool {
    let Some(target_host) = configured_host(c) else {
        return false;
    };
    let host = url.host_str().unwrap_or("");
    host_matches(host, &target_host) && !is_auth_page(host, url.path(), c)
}

fn is_exact_configured_page(url: &Url, c: &AppConfig) -> bool {
    let Ok(target) = Url::parse(&c.site_url) else {
        return false;
    };
    host_matches(url.host_str().unwrap_or(""), target.host_str().unwrap_or(""))
        && url.path().trim_end_matches('/') == target.path().trim_end_matches('/')
}

fn decide_main(url: &Url, c: &AppConfig) -> NavAction {
    if url.scheme() != "http" && url.scheme() != "https" {
        return NavAction::Block;
    }
    let host = url.host_str().unwrap_or("");
    if is_auth_provider(host, url.path()) {
        return NavAction::OpenLogin;
    }
    if let Some(target) = configured_host(c) {
        if host_matches(host, &target) {
            if is_login_path(url.path()) && !is_exact_configured_page(url, c) {
                return NavAction::OpenLogin;
            }
            return NavAction::Allow;
        }
    }
    NavAction::External
}

fn decide_login(url: &Url, c: &AppConfig) -> NavAction {
    if url.scheme() != "http" && url.scheme() != "https" {
        return NavAction::Block;
    }
    let host = url.host_str().unwrap_or("");
    if is_auth_provider(host, url.path())
        || configured_host(c).is_some_and(|target| host_matches(host, &target))
    {
        NavAction::Allow
    } else {
        NavAction::External
    }
}

fn should_close_login(seen_auth: bool, url: &Url, c: &AppConfig) -> bool {
    seen_auth && is_configured_page(url, c)
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

#[derive(Debug, Default)]
struct ExternalOpenState {
    last_open_ms: u64,
    last_url: String,
}

fn external_open_allowed(state: &mut ExternalOpenState, url: &str, now_ms: u64) -> bool {
    let elapsed = now_ms.saturating_sub(state.last_open_ms);
    if state.last_open_ms != 0 && elapsed < 2_000 {
        return false;
    }
    if state.last_url == url && state.last_open_ms != 0 && elapsed < 10_000 {
        return false;
    }
    state.last_open_ms = now_ms;
    state.last_url = url.to_owned();
    true
}

static EXTERNAL_OPEN_STATE: std::sync::OnceLock<Mutex<ExternalOpenState>> = std::sync::OnceLock::new();
static LOGIN_WINDOW_LOCK: Mutex<()> = Mutex::new(());

fn open_external_limited(dir: std::path::PathBuf, url: String) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64;
    let allowed = {
        let mut state = EXTERNAL_OPEN_STATE
            .get_or_init(|| Mutex::new(ExternalOpenState::default()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        external_open_allowed(&mut state, &url, now)
    };
    if !allowed {
        crate::diagnostics::log(&dir, "WARN", "Abertura externa descartada pelo limite contra navegações repetidas.");
        return;
    }
    thread::spawn(move || {
        if let Err(error) = open::that(&url) {
            crate::diagnostics::log(&dir, "ERRO", format!("Não foi possível abrir o link no navegador padrão: {error}"));
        }
    });
}

fn open_login_limited(app: AppHandle, url: Url, dir: std::path::PathBuf) {
    thread::spawn(move || {
        if let Err(error) = open_login(&app, url) {
            crate::diagnostics::log(&dir, "ERRO", format!("Não foi possível abrir/reutilizar a janela de autenticação: {error}"));
        }
    });
}

pub fn create_main(
    app: &AppHandle,
    c: &AppConfig,
    dir: &std::path::Path,
) -> tauri::Result<WebviewWindow> {
    let data = dir.join("webview-data");
    let diag_dir = dir.to_path_buf();
    if let Err(error) = std::fs::create_dir_all(&data) {
        crate::diagnostics::log(&diag_dir, "ERRO", format!("Não foi possível criar o diretório de dados do WebView2 principal: {error}"));
    }
    let build_diag_dir = dir.to_path_buf();
    let popup_diag_dir = dir.to_path_buf();
    let page_load_diag_dir = dir.to_path_buf();
    let nav_app = app.clone();
    let popup_app = app.clone();
    let action_token = uuid::Uuid::new_v4().simple().to_string();
    let heartbeat_token = app.state::<AppState>().heartbeat_token.clone();
    let init_script = INIT
        .replace("__WIDGET_LANGUAGE__", &c.language)
        .replace("__WIDGET_LOCKED__", &c.locked.to_string())
        .replace("__WIDGET_ACTION_TOKEN__", &action_token)
        .replace("__WIDGET_HEARTBEAT_TOKEN__", &heartbeat_token);
    let navigation_action_token = action_token.clone();
    let page_load_app = app.clone();
    let w=WebviewWindowBuilder::new(app,"main",WebviewUrl::App("index.html".into())).title("Widget Website").inner_size(c.width as f64,c.height as f64).min_inner_size(150.0,100.0).decorations(false).transparent(true).shadow(true).resizable(!c.locked).always_on_top(c.always_on_top).skip_taskbar(!c.show_in_taskbar).visible(true).data_directory(data).devtools(cfg!(debug_assertions)).zoom_hotkeys_enabled(false).initialization_script(init_script)
.on_page_load(move|window,payload|{
    if payload.event()!=PageLoadEvent::Finished{return;}
    let url=payload.url();
    if url.scheme()!="https"&&url.scheme()!="http"{return;}
    let config=page_load_app.state::<AppState>().snapshot();
    let language=serde_json::to_string(&config.language).unwrap_or_else(|_|"\"pt-BR\"".to_owned());
    let script=format!("(function(){{function apply(){{const root=document.documentElement;if(!root)return;root.style.opacity='{}';root.style.borderRadius='{}px';if(typeof window.__widgetSetLanguage==='function')window.__widgetSetLanguage({});if(typeof window.__widgetSetLocked==='function')window.__widgetSetLocked({});}}if(document.documentElement)apply();else document.addEventListener('DOMContentLoaded',apply,{{once:true}});}})();",config.opacity as f64/100.0,config.corner_radius,language,config.locked);
    if let Err(error)=window.eval(&script) {
        crate::diagnostics::log(&page_load_diag_dir,"WARN",format!("Não foi possível aplicar idioma/aparência à página carregada: {error}"));
    }
})
.on_navigation(move|u|{
if is_internal_app_url(u){
    if u.host_str()==Some("tauri.localhost"){
        crate::diagnostics::log(&diag_dir,"INFO","Origem interna do Tauri autorizada; carregando a interface local.");
    }
    return true;
}
if u.scheme()=="widget-action"{
    let action=match authorized_widget_action(u,&navigation_action_token) {
        Some(action)=>action,
        None=>{
            crate::diagnostics::log(&diag_dir,"WARN","Comando de janela ignorado: token ou ação inválida.");
            return false;
        }
    };
    let state=nav_app.state::<AppState>();
    let locked=state.snapshot().locked;
    if locked {
        crate::diagnostics::log(&diag_dir,"WARN",format!("Ação de janela ignorada porque mover/redimensionar está travado: {action}"));
        return false;
    }
    if let Some(window)=nav_app.get_webview_window("main") {
        if action=="drag" {
            crate::diagnostics::log(&diag_dir,"INFO","Comando de movimento autenticado recebido; iniciando arraste nativo do Windows.");
            if let Err(error)=begin_native_drag(&window) {
                crate::diagnostics::log(&diag_dir,"ERRO",format!("O arraste nativo falhou: {error}"));
                if let Err(fallback_error) = window.start_dragging() {
                    crate::diagnostics::log(&diag_dir, "ERRO", format!("O arraste nativo e a alternativa do Tauri falharam: {fallback_error}"));
                }
            }
        } else if let Some(direction)=action.strip_prefix("resize/") {
            if ["north","south","east","west","north-east","north-west","south-east","south-west"].contains(&direction) {
                if let Err(error)=begin_native_resize(&window,direction) { crate::diagnostics::log(&diag_dir,"WARN",format!("Redimensionamento nativo não iniciado: {error}")); }
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
crate::diagnostics::log_navigation(&diag_dir,u);
let cfg=nav_app.state::<AppState>().snapshot();
match decide_main(u,&cfg) {
    NavAction::Allow => true,
    NavAction::OpenLogin => {
        crate::diagnostics::log(&diag_dir,"INFO",format!("Abrindo fluxo de autenticação (host={}).",u.host_str().unwrap_or("sem host")));
        open_login_limited(nav_app.clone(),u.clone(),diag_dir.clone());
        false
    }
    NavAction::External => {
        crate::diagnostics::log(&diag_dir,"INFO",format!("Encaminhando link externo ao navegador padrão (host={}).",u.host_str().unwrap_or("sem host")));
        open_external_limited(diag_dir.clone(),u.as_str().to_owned());
        false
    }
    NavAction::Block => {
        crate::diagnostics::log(&diag_dir,"WARN",format!("Navegação bloqueada por política: {}",crate::diagnostics::sanitize_url(u)));
        false
    }
}
})
.on_new_window(move|u,_|{
    crate::diagnostics::log_navigation(&popup_diag_dir,&u);
    let cfg=popup_app.state::<AppState>().snapshot();
    match decide_main(&u,&cfg) {
        NavAction::OpenLogin => open_login_limited(popup_app.clone(),u,popup_diag_dir.clone()),
        NavAction::Allow | NavAction::External => open_external_limited(popup_diag_dir.clone(),u.as_str().to_owned()),
        NavAction::Block => crate::diagnostics::log(&popup_diag_dir,"WARN","Pop-up bloqueado por esquema não HTTP(S)."),
    }
    NewWindowResponse::Deny
}).build().map_err(|error|{crate::diagnostics::log(&build_diag_dir,"ERRO",format!("Falha ao criar a janela WebView2 principal: {error}"));error})?;
    restore_position(&w, c);
    apply_config(app, c, true);
    Ok(w)
}
fn pack_lparam(x: i32, y: i32) -> isize {
    ((x as u16 as u32) | ((y as u16 as u32) << 16)) as isize
}

#[cfg(windows)]
fn begin_native_drag(window: &WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, ReleaseCapture, VK_LBUTTON,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, PostMessageW, HTCAPTION, WM_NCLBUTTONDOWN,
    };

    if unsafe { GetAsyncKeyState(VK_LBUTTON as i32) } as u16 & 0x8000 == 0 {
        return Err("Arraste cancelado porque o botão esquerdo do mouse já foi solto.".into());
    }
    let raw_hwnd = window.hwnd().map_err(|error| error.to_string())?;
    let mut cursor: POINT = unsafe { std::mem::zeroed() };
    if unsafe { GetCursorPos(&mut cursor) } == 0 {
        return Err("Não foi possível obter a posição do ponteiro do mouse.".into());
    }
    let packed_position = pack_lparam(cursor.x, cursor.y);
    unsafe {
        ReleaseCapture();
        if PostMessageW(raw_hwnd.0, WM_NCLBUTTONDOWN, HTCAPTION as usize, packed_position) == 0 {
            return Err("O Windows não aceitou o início do arraste nativo.".into());
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn begin_native_drag(window: &WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|error| error.to_string())
}

#[cfg(windows)]
fn begin_native_resize(window: &WebviewWindow, direction: &str) -> Result<(), String> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, ReleaseCapture, VK_LBUTTON,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, PostMessageW, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTLEFT, HTRIGHT, HTTOP,
        HTTOPLEFT, HTTOPRIGHT, WM_NCLBUTTONDOWN,
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
        _ => return Err("Direção de redimensionamento inválida.".into()),
    };
    if unsafe { GetAsyncKeyState(VK_LBUTTON as i32) } as u16 & 0x8000 == 0 {
        return Err("Redimensionamento cancelado porque o botão esquerdo do mouse já foi solto.".into());
    }
    let raw_hwnd = window.hwnd().map_err(|error| error.to_string())?;
    let mut cursor: POINT = unsafe { std::mem::zeroed() };
    if unsafe { GetCursorPos(&mut cursor) } == 0 {
        return Err("Não foi possível obter a posição do ponteiro do mouse.".into());
    }
    let packed_position = pack_lparam(cursor.x, cursor.y);
    unsafe {
        ReleaseCapture();
        if PostMessageW(raw_hwnd.0, WM_NCLBUTTONDOWN, hit_test as usize, packed_position) == 0 {
            return Err("O Windows não aceitou o início do redimensionamento nativo.".into());
        }
    }
    Ok(())
}
#[cfg(not(windows))]
fn begin_native_resize(_window: &WebviewWindow, _direction: &str) -> Result<(), String> { Ok(()) }

fn close_login_and_reload(app: &AppHandle, label: &str, dir: &std::path::Path, reason: &str) {
    crate::diagnostics::log(dir, "INFO", format!("Fechando a janela de autenticação ({reason})."));
    if let Some(window) = app.get_webview_window(label) {
        if let Err(error) = window.close() {
            crate::diagnostics::log(dir, "ERRO", format!("Não foi possível fechar a janela de autenticação: {error}"));
        }
    }
}

fn open_login(app: &AppHandle, url: Url) -> tauri::Result<()> {
    let _creation_guard = LOGIN_WINDOW_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    if let Some((label, existing)) = app
        .webview_windows()
        .into_iter()
        .find(|(label, _)| label.starts_with("login-"))
    {
        let app_dir = app.state::<AppState>().app_dir.clone();
        crate::diagnostics::log(&app_dir, "INFO", format!("Reutilizando a janela de autenticação existente ({label})."));
        existing.navigate(url).map_err(|error| {
            crate::diagnostics::log(&app_dir, "ERRO", format!("Não foi possível navegar na janela de autenticação existente: {error}"));
            error
        })?;
        for (name, result) in [
            ("restaurar", existing.unminimize().map_err(|error| error.to_string())),
            ("exibir", existing.show().map_err(|error| error.to_string())),
            ("focar", existing.set_focus().map_err(|error| error.to_string())),
        ] {
            if let Err(error) = result {
                crate::diagnostics::log(&app_dir, "WARN", format!("Não foi possível {name} a janela de autenticação: {error}"));
            }
        }
        return Ok(());
    }

    let id = LOGIN_ID.fetch_add(1, Ordering::Relaxed);
    let label = format!("login-{id}");
    let close_label = label.clone();
    let close_token = uuid::Uuid::new_v4().simple().to_string();
    let navigation_close_token = close_token.clone();
    let state = app.state::<AppState>();
    let diag_dir = state.app_dir.clone();
    let initial_config = state.snapshot();
    let language = initial_config.language.clone();
    let initially_authenticating = is_auth_page(url.host_str().unwrap_or(""), url.path(), &initial_config);
    let data_dir = state.app_dir.join("webview-data");
    if let Err(error) = std::fs::create_dir_all(&data_dir) {
        crate::diagnostics::log(&diag_dir, "ERRO", format!("Não foi possível preparar os dados da janela de autenticação: {error}"));
        return Err(error.into());
    }
    crate::diagnostics::log(&diag_dir, "INFO", format!("Abrindo janela de autenticação (host={}).", url.host_str().unwrap_or("sem host")));

    let nav = app.clone();
    let popup = app.clone();
    let close_event_app = app.clone();
    let nav_diag_dir = diag_dir.clone();
    let popup_diag_dir = diag_dir.clone();
    let close_event_dir = diag_dir.clone();
    let nav_seen_auth = Arc::new(AtomicBool::new(initially_authenticating));
    let popup_seen_auth = Arc::clone(&nav_seen_auth);
    let host = url.host_str().unwrap_or("Login").to_owned();
    let login_title = crate::i18n::login_title(&language);

    WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))
        .title(format!("{login_title} — {host}"))
        .inner_size(960.0, 760.0)
        .min_inner_size(600.0, 450.0)
        .decorations(true)
        .resizable(true)
        .skip_taskbar(false)
        .data_directory(data_dir)
        .devtools(cfg!(debug_assertions))
        .zoom_hotkeys_enabled(false)
        .initialization_script(
            LOGIN_INIT
                .replace("__WIDGET_LANGUAGE__", &language)
                .replace("__WIDGET_LOGIN_CLOSE_TOKEN__", &close_token),
        )
        .on_navigation(move |url| {
            if url.scheme() == "widget-login-action" {
                if url.host_str() == Some(navigation_close_token.as_str())
                    && url.path() == "/close"
                    && url.query().is_none()
                    && url.fragment().is_none()
                {
                    close_login_and_reload(&nav, &close_label, &nav_diag_dir, "solicitação pelo botão Fechar");
                } else {
                    crate::diagnostics::log(&nav_diag_dir, "WARN", "Pedido de fechamento de login ignorado: token ou ação inválida.");
                }
                return false;
            }

            crate::diagnostics::log_navigation(&nav_diag_dir, url);
            let config = nav.state::<AppState>().snapshot();
            if is_auth_page(url.host_str().unwrap_or(""), url.path(), &config) {
                nav_seen_auth.store(true, Ordering::SeqCst);
            }
            if should_close_login(nav_seen_auth.load(Ordering::SeqCst), url, &config) {
                close_login_and_reload(&nav, &close_label, &nav_diag_dir, "retorno ao site após autenticação");
                return false;
            }

            match decide_login(url, &config) {
                NavAction::Allow | NavAction::OpenLogin => true,
                NavAction::External => {
                    crate::diagnostics::log(&nav_diag_dir, "INFO", format!("Encaminhando navegação externa da janela de autenticação (host={}).", url.host_str().unwrap_or("sem host")));
                    open_external_limited(nav_diag_dir.clone(), url.as_str().to_owned());
                    false
                }
                NavAction::Block => {
                    crate::diagnostics::log(&nav_diag_dir, "WARN", "Navegação bloqueada na janela de autenticação por esquema não HTTP(S).");
                    false
                }
            }
        })
        .on_new_window(move |url, _| {
            crate::diagnostics::log_navigation(&popup_diag_dir, &url);
            let config = popup.state::<AppState>().snapshot();
            match decide_login(&url, &config) {
                NavAction::Allow | NavAction::OpenLogin => {
                    if is_auth_page(url.host_str().unwrap_or(""), url.path(), &config) {
                        popup_seen_auth.store(true, Ordering::SeqCst);
                    }
                    open_login_limited(popup.clone(), url, popup_diag_dir.clone());
                }
                NavAction::External => open_external_limited(popup_diag_dir.clone(), url.as_str().to_owned()),
                NavAction::Block => crate::diagnostics::log(&popup_diag_dir, "WARN", "Pop-up de autenticação bloqueado por esquema não HTTP(S)."),
            }
            NewWindowResponse::Deny
        })
        .on_window_event(move |event| {
            if matches!(event, WindowEvent::CloseRequested { .. }) {
                crate::diagnostics::log(&close_event_dir, "INFO", "Janela de autenticação fechada; solicitando atualização da janela principal.");
                if let Some(main) = close_event_app.get_webview_window("main") {
                    if let Err(error) = main.eval("location.reload()") {
                        crate::diagnostics::log(&close_event_dir, "ERRO", format!("Não foi possível recarregar a janela principal após fechar a autenticação: {error}"));
                    }
                }
            }
        })
        .build()?;
    Ok(())
}

pub fn open_login_from_tray(app: &AppHandle) {
    let config = app.state::<AppState>().snapshot();
    match Url::parse(&config.site_url) {
        Ok(url) => open_login_limited(app.clone(), url, app.state::<AppState>().app_dir.clone()),
        Err(error) => crate::diagnostics::log(&app.state::<AppState>().app_dir, "ERRO", format!("A URL configurada não pôde ser usada para abrir a autenticação: {error}")),
    }
}

#[derive(Clone, Copy, Debug)]
struct MonitorBounds {
    x: i64,
    y: i64,
    width: i64,
    height: i64,
    scale_factor: f64,
}

/// Returns true only when a useful portion of the window is visible on one monitor.
/// Coordinates and intersection arithmetic use i64 to avoid overflow at extreme positions.
fn is_placeable(x: i32, y: i32, width: u32, height: u32, monitors: &[MonitorBounds]) -> bool {
    if width == 0 || height == 0 {
        return false;
    }
    monitors.iter().any(|monitor| {
        let scale = if monitor.scale_factor.is_finite() && monitor.scale_factor > 0.0 {
            monitor.scale_factor
        } else {
            1.0
        };
        let physical_width = ((width as f64 * scale).round() as i64).max(1);
        let physical_height = ((height as f64 * scale).round() as i64).max(1);
        let left = i64::from(x);
        let top = i64::from(y);
        let right = left + physical_width;
        let bottom = top + physical_height;
        let visible_width = right.min(monitor.x + monitor.width) - left.max(monitor.x);
        let visible_height = bottom.min(monitor.y + monitor.height) - top.max(monitor.y);
        visible_width >= 80.min(physical_width) && visible_height >= 40.min(physical_height)
    })
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
    let bounds = monitors.iter().map(|monitor| {
        let position = monitor.position();
        let size = monitor.size();
        MonitorBounds {
            x: i64::from(position.x),
            y: i64::from(position.y),
            width: i64::from(size.width),
            height: i64::from(size.height),
            scale_factor: monitor.scale_factor(),
        }
    }).collect::<Vec<_>>();
    if is_placeable(x, y, c.width, c.height, &bounds) {
        let _ = w.set_position(PhysicalPosition::new(x, y));
    } else {
        let _ = w.center();
    }
}
fn corner_region_dims(inner_w: u32, inner_h: u32, radius: u8, scale: f64) -> (i32, i32, i32) {
    let width = inner_w.min(i32::MAX as u32) as i32;
    let height = inner_h.min(i32::MAX as u32) as i32;
    let effective_scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    let scaled_radius = (radius as f64 * effective_scale).round().clamp(0.0, i32::MAX as f64) as i32;
    let corner_radius = scaled_radius.clamp(0, width.min(height).max(0) / 2);
    (width, height, corner_radius)
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
        // The custom HRGN below owns the corner radius; disable DWM rounding to avoid double clipping.
        let preference: u32 = 1;
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
            let (w, h, r) = corner_region_dims(size.width, size.height, radius, scale);
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
        let radius = app.state::<AppState>().snapshot().corner_radius;
        apply_native_corners(&window, radius);
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ToggleAction {
    Show,
    Hide,
}

fn toggle_action(visible: bool, minimized: bool, focused: bool) -> ToggleAction {
    if !visible || minimized || !focused { ToggleAction::Show } else { ToggleAction::Hide }
}

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let dir = app.state::<AppState>().app_dir.clone();
        for (name, result) in [
            ("restaurar", window.unminimize().map_err(|error| error.to_string())),
            ("exibir", window.show().map_err(|error| error.to_string())),
            ("focar", window.set_focus().map_err(|error| error.to_string())),
        ] {
            if let Err(error) = result {
                crate::diagnostics::log(&dir, "WARN", format!("Não foi possível {name} a janela principal: {error}"));
            }
        }
        crate::diagnostics::set_window_visible(true);
    } else {
        crate::diagnostics::log(
            &app.state::<AppState>().app_dir,
            "ERRO",
            "A janela principal não existe; não foi possível trazê-la para frente.",
        );
    }
}

pub fn toggle_main_visibility(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        crate::diagnostics::log(&app.state::<AppState>().app_dir, "ERRO", "A janela principal não existe; o alternador de visibilidade foi ignorado.");
        return;
    };
    let visible = window.is_visible().unwrap_or(false);
    let minimized = window.is_minimized().unwrap_or(false);
    let focused = window.is_focused().unwrap_or(false);
    match toggle_action(visible, minimized, focused) {
        ToggleAction::Show => show_main(app),
        ToggleAction::Hide => {
            if let Err(error) = window.hide() {
                crate::diagnostics::log(&app.state::<AppState>().app_dir, "WARN", format!("Não foi possível ocultar a janela principal: {error}"));
            } else {
                crate::diagnostics::set_window_visible(false);
            }
        }
    }
}

pub fn apply_config(app: &AppHandle, c: &AppConfig, resize: bool) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_always_on_top(c.always_on_top);
        let _ = w.set_skip_taskbar(!c.show_in_taskbar);
        let _ = w.set_resizable(!c.locked);
        let _ = w.set_min_size(Some(LogicalSize::new(150.0, 100.0)));
        if resize {
            let _ = w.set_size(LogicalSize::new(c.width as f64, c.height as f64));
        }
        let _ = w.set_zoom(c.zoom as f64 / 100.0);
        let language =
            serde_json::to_string(&c.language).unwrap_or_else(|_| "\"pt-BR\"".to_owned());
        let script=format!("document.documentElement.style.opacity='{}';document.documentElement.style.borderRadius='{}px';if(typeof window.__widgetSetLanguage==='function')window.__widgetSetLanguage({});if(typeof window.__widgetSetLocked==='function')window.__widgetSetLocked({});",c.opacity as f64/100.0,c.corner_radius,language,c.locked);
        let _ = w.eval(&script);
        let _ = w.set_shadow(c.corner_radius > 0);
        apply_native_corners(&w, c.corner_radius);
    }
}
static SETTINGS_WINDOW_CREATING: AtomicBool = AtomicBool::new(false);

pub fn open_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        for (name, result) in [
            ("restaurar", window.unminimize().map_err(|error| error.to_string())),
            ("exibir", window.show().map_err(|error| error.to_string())),
            ("focar", window.set_focus().map_err(|error| error.to_string())),
        ] {
            if let Err(error) = result {
                crate::diagnostics::log(&app.state::<AppState>().app_dir, "WARN", format!("Não foi possível {name} a janela de configurações: {error}"));
            }
        }
        return;
    }
    if SETTINGS_WINDOW_CREATING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return;
    }
    let app = app.clone();
    thread::spawn(move || {
        let app_dir = app.state::<AppState>().app_dir.clone();
        let language = app.state::<AppState>().snapshot().language;
        let title = crate::i18n::settings_title(&language);
        let dir = app_dir.join("settings-data");
        let result = match std::fs::create_dir_all(&dir) {
            Ok(()) => WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings.html".into()))
                .title(title)
                .inner_size(760.0, 850.0)
                .min_inner_size(520.0, 550.0)
                .resizable(true)
                .data_directory(dir)
                .build()
                .map(|_| ())
                .map_err(|error| error.to_string()),
            Err(error) => Err(format!("não foi possível criar o diretório de dados das configurações: {error}")),
        };
        SETTINGS_WINDOW_CREATING.store(false, Ordering::SeqCst);
        if let Err(error) = result {
            crate::diagnostics::log(&app_dir, "ERRO", format!("Não foi possível abrir a janela de configurações: {error}"));
        }
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
fn schedule_debounced_config_save(app: &AppHandle) {
    {
        let state = app.state::<AppState>();
        state.geometry_save_revision.fetch_add(1, Ordering::SeqCst);
        if state
            .geometry_save_worker_running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }
    }

    let app = app.clone();
    thread::spawn(move || loop {
        let revision = app
            .state::<AppState>()
            .geometry_save_revision
            .load(Ordering::SeqCst);
        thread::sleep(Duration::from_millis(500));
        let state = app.state::<AppState>();
        if state.geometry_save_revision.load(Ordering::SeqCst) != revision {
            continue;
        }
        if let Err(error) = state.persist_current() {
            crate::diagnostics::log(
                &state.app_dir,
                "ERRO",
                format!("Falha ao persistir posição/tamanho da janela: {error}"),
            );
        }
        state
            .geometry_save_worker_running
            .store(false, Ordering::SeqCst);
        if state.geometry_save_revision.load(Ordering::SeqCst) != revision {
            schedule_debounced_config_save(&app);
        }
        break;
    });
}

/// Flush the most recent state before an explicit Quit action.
pub fn flush_pending_config(app: &AppHandle) {
    let state = app.state::<AppState>();
    if let Err(error) = state.persist_current() {
        crate::diagnostics::log(
            &state.app_dir,
            "ERRO",
            format!("Falha ao descarregar a configuração antes de sair: {error}"),
        );
    }
}

fn is_valid_saved_position(x: i32, y: i32) -> bool {
    x > -30_000 && y > -30_000
}

fn is_valid_saved_size(width: u32, height: u32) -> bool {
    width > 0 && height > 0
}

pub fn save_window_state(app: &AppHandle, e: &WindowEvent) {
    let state = app.state::<AppState>();
    if let Some(window) = app.get_webview_window("main") {
        if window.is_minimized().unwrap_or(false) {
            return;
        }
    }
    match e {
        WindowEvent::Moved(position) if !is_valid_saved_position(position.x, position.y) => return,
        WindowEvent::Resized(size) if !is_valid_saved_size(size.width, size.height) => return,
        _ => {}
    }
    let scale = app
        .get_webview_window("main")
        .and_then(|w| w.scale_factor().ok())
        .unwrap_or(1.0);
    let mut c = state.lock_config();
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
    schedule_debounced_config_save(app);
    let _ = app.emit("config-changed", &copy);

}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ReloadState {
    elapsed_seconds: u32,
    last_minutes: u32,
}

fn next_reload_state(mut state: ReloadState, minutes: u32, visible: bool) -> (ReloadState, bool) {
    if minutes == 0 {
        return (ReloadState::default(), false);
    }
    if minutes != state.last_minutes {
        state.last_minutes = minutes;
        state.elapsed_seconds = 0;
        return (state, false);
    }
    state.elapsed_seconds = state.elapsed_seconds.saturating_add(10).min(minutes.saturating_mul(60));
    if visible && state.elapsed_seconds >= minutes.saturating_mul(60) {
        state.elapsed_seconds = 0;
        (state, true)
    } else {
        (state, false)
    }
}

pub fn start_auto_reload(app: AppHandle) {
    thread::spawn(move || {
        let mut state = ReloadState::default();
        loop {
            thread::sleep(Duration::from_secs(10));
            let minutes = app.state::<AppState>().snapshot().auto_reload_minutes as u32;
            let window = app.get_webview_window("main");
            let visible = window.as_ref().is_some_and(|window| window.is_visible().unwrap_or(false));
            let (next, should_reload) = next_reload_state(state, minutes, visible);
            state = next;
            if should_reload {
                if let Some(window) = window {
                    if let Err(error) = window.eval("location.reload()") {
                        crate::diagnostics::log(&app.state::<AppState>().app_dir, "ERRO", format!("Falha ao recarregar automaticamente o site: {error}"));
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod toggle_action_tests {
    use super::*;

    #[test]
    fn toggle_restores_minimized_window_or_brings_unfocused_window_forward() {
        assert_eq!(toggle_action(true, true, false), ToggleAction::Show);
        assert_eq!(toggle_action(true, false, false), ToggleAction::Show);
        assert_eq!(toggle_action(false, false, false), ToggleAction::Show);
        assert_eq!(toggle_action(true, false, true), ToggleAction::Hide);
    }
}

#[cfg(test)]
mod navigation_policy_tests {
    use super::*;

    fn config(site_url: &str) -> AppConfig {
        AppConfig { site_url: site_url.to_owned(), ..AppConfig::default() }
    }

    #[test]
    fn host_matches_requires_dot_boundary() {
        assert!(host_matches("www.example.com", "example.com"));
        assert!(!host_matches("evilinvesting.com", "investing.com"));
        assert!(!host_matches("notinvesting.com", "investing.com"));
    }

    #[test]
    fn host_matches_ignores_trailing_dot() {
        assert!(host_matches("BR.INVESTING.COM.", "br.investing.com"));
        assert!(host_matches("EXAMPLE.COM", "example.com."));
    }

    #[test]
    fn host_matches_handles_ip_and_single_label_targets() {
        assert!(host_matches("127.0.0.1", "127.0.0.1"));
        assert!(!host_matches("x.127.0.0.1", "127.0.0.1"));
        assert!(host_matches("x.intranet", "intranet"));
        assert!(!host_matches("intranetx", "intranet"));
    }

    #[test]
    fn nav_policy_routes_non_provider_google_and_facebook_hosts_to_external_browser() {
        let cfg = config("https://br.investing.com/economic-calendar");
        assert_eq!(decide_main(&Url::parse("https://accounts.google.com/signin").unwrap(), &cfg), NavAction::OpenLogin);
        assert_eq!(decide_main(&Url::parse("https://www.google.com/aclk?x=1").unwrap(), &cfg), NavAction::External);
        assert_eq!(decide_main(&Url::parse("https://adservice.google.com/x").unwrap(), &cfg), NavAction::External);
        assert_eq!(decide_main(&Url::parse("https://www.facebook.com/sharer/share.php").unwrap(), &cfg), NavAction::External);
        assert_eq!(decide_main(&Url::parse("https://www.facebook.com/login/").unwrap(), &cfg), NavAction::OpenLogin);
    }

    #[test]
    fn nav_policy_does_not_hardcode_investing_when_site_changes() {
        let cfg = config("https://example.com/");
        assert_eq!(decide_main(&Url::parse("https://www.investing.com/news").unwrap(), &cfg), NavAction::External);
        assert_eq!(decide_main(&Url::parse("https://app.example.com/").unwrap(), &cfg), NavAction::Allow);
    }

    #[test]
    fn is_auth_page_matches_path_segments_not_substrings() {
        let cfg = config("https://br.investing.com/economic-calendar");
        assert!(!is_auth_page("br.investing.com", "/news/authors/joao", &cfg));
        assert!(!is_auth_page("br.investing.com", "/analysis/authority-x", &cfg));
        assert!(!is_auth_page("br.investing.com", "/accounting-news", &cfg));
        assert!(is_auth_page("br.investing.com", "/login", &cfg));
        assert!(is_auth_page("br.investing.com", "/oauth/authorize", &cfg));
        assert!(is_auth_page("br.investing.com", "/account/login", &cfg));
    }

    #[test]
    fn is_configured_page_accepts_site_root_after_login_redirect() {
        let cfg = config("https://br.investing.com/economic-calendar");
        assert!(is_configured_page(&Url::parse("https://br.investing.com/").unwrap(), &cfg));
        assert!(!is_configured_page(&Url::parse("https://br.investing.com/login").unwrap(), &cfg));
        assert!(should_close_login(true, &Url::parse("https://br.investing.com/").unwrap(), &cfg));
        assert!(!should_close_login(false, &Url::parse("https://br.investing.com/").unwrap(), &cfg));
    }

    #[test]
    fn external_open_rate_limiter_drops_burst_and_duplicates() {
        let mut state = ExternalOpenState::default();
        assert!(external_open_allowed(&mut state, "https://example.com/a", 1_000));
        assert!(!external_open_allowed(&mut state, "https://example.com/b", 2_000));
        assert!(!external_open_allowed(&mut state, "https://example.com/a", 2_500));
        assert!(external_open_allowed(&mut state, "https://example.com/a", 12_000));
    }

    #[test]
    fn reload_counter_resets_when_interval_changes() {
        let state = ReloadState { elapsed_seconds: 350, last_minutes: 60 };
        let (next, reload) = next_reload_state(state, 5, true);
        assert_eq!(next.elapsed_seconds, 0);
        assert_eq!(next.last_minutes, 5);
        assert!(!reload);
    }

    #[test]
    fn reload_is_skipped_while_hidden_and_runs_when_visible_after_expiry() {
        let state = ReloadState { elapsed_seconds: 290, last_minutes: 5 };
        let (hidden, reload_hidden) = next_reload_state(state, 5, false);
        assert_eq!(hidden.elapsed_seconds, 300);
        assert!(!reload_hidden);
        let (visible, reload_visible) = next_reload_state(hidden, 5, true);
        assert_eq!(visible.elapsed_seconds, 0);
        assert!(reload_visible);
    }
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

#[cfg(test)]
mod corner_geometry_tests {
    use super::*;

    #[test]
    fn corner_region_dims_follow_inner_size_and_clamp_radius() {
        assert_eq!(corner_region_dims(300, 200, 12, 1.0), (300, 200, 12));
        assert_eq!(corner_region_dims(0, 0, 12, 1.0), (0, 0, 0));
        assert_eq!(corner_region_dims(40, 40, 64, 1.0), (40, 40, 20));
        assert_eq!(corner_region_dims(300, 200, 12, 1.5), (300, 200, 18));
    }

    #[test]
    fn pack_lparam_preserves_negative_coordinates() {
        fn unpack_x(value: isize) -> i16 { (value as u32 as u16) as i16 }
        fn unpack_y(value: isize) -> i16 { ((value as u32 >> 16) as u16) as i16 }
        for (x, y) in [(-1920, 300), (2500, -1080), (-32768, 32767), (-1, -1)] {
            let packed = pack_lparam(x, y);
            assert_eq!(unpack_x(packed), x as i16);
            assert_eq!(unpack_y(packed), y as i16);
        }
    }
}

#[cfg(test)]
mod geometry_tests {
    use super::*;

    fn monitor(x: i64, y: i64, width: i64, height: i64, scale_factor: f64) -> MonitorBounds {
        MonitorBounds { x, y, width, height, scale_factor }
    }

    #[test]
    fn one_pixel_overlap_is_not_placeable() {
        assert!(!is_placeable(-149, 0, 150, 100, &[monitor(0, 0, 1920, 1080, 1.0)]));
    }

    #[test]
    fn fully_visible_window_is_placeable_on_negative_coordinate_monitor() {
        assert!(is_placeable(-1800, 100, 500, 700, &[monitor(-1920, 0, 1920, 1080, 1.0)]));
    }

    #[test]
    fn extreme_coordinates_do_not_overflow() {
        assert!(!is_placeable(i32::MAX, i32::MAX, 2000, 1600, &[monitor(0, 0, 1920, 1080, 1.5)]));
    }

    #[test]
    fn target_monitor_scale_factor_affects_visible_intersection() {
        let scaled = [monitor(0, 0, 1920, 1080, 1.5)];
        let standard = [monitor(0, 0, 1920, 1080, 1.0)];
        assert!(is_placeable(-70, 0, 100, 100, &scaled));
        assert!(!is_placeable(-70, 0, 100, 100, &standard));
    }

    #[test]
    fn window_state_ignores_zero_size_resizes() {
        assert!(!is_valid_saved_size(0, 0));
        assert!(!is_valid_saved_size(1000, 0));
        assert!(!is_valid_saved_size(0, 700));
        assert!(is_valid_saved_size(1000, 1400));
    }

    #[test]
    fn window_state_ignores_sentinel_positions() {
        assert!(!is_valid_saved_position(-32_000, -32_000));
        assert!(!is_valid_saved_position(100, -32_000));
        assert!(is_valid_saved_position(-1920, 100));
    }
}
