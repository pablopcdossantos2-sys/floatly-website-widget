import { invoke } from "@tauri-apps/api/core";
type Cfg = {site_url:string;[key:string]:unknown};
const status=document.querySelector<HTMLParagraphElement>("#status")!;
const retry=document.querySelector<HTMLButtonElement>("#retry")!;
let timer:number|undefined;let target="https://br.investing.com/economic-calendar";let navigating=false;
function failed(){status.textContent="Não foi possível acessar o site. Verifique a conexão e tente novamente.";retry.hidden=false;if(timer===undefined)timer=setInterval(()=>void attempt(true),30000);}
async function attempt(quiet=false){if(navigating)return;if(!quiet){status.textContent="Verificando a conexão com o site…";retry.hidden=true;}try{const c=new AbortController();const t=setTimeout(()=>c.abort(),8000);try{await fetch(new URL("/favicon.ico",target).href,{mode:"no-cors",cache:"no-store",signal:c.signal});}finally{clearTimeout(t);}navigating=true;status.textContent="Abrindo o calendário…";location.replace(target);}catch{navigating=false;failed();}}
retry.addEventListener("click",()=>void attempt());
void(async()=>{try{const c=await invoke<Cfg>("get_config");if(c.site_url)target=c.site_url;}catch{}void attempt();})();