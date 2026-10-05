use std::collections::VecDeque;
use std::sync::Mutex;

use tauri::http::{Response, StatusCode, Uri, header};

use crate::{Answer, Refusal, Session, elsewhere, finding, weighed};

pub const LENT_AT_MOST: usize = 512 * 1024;
pub const KEPT_AT_MOST: usize = 8 * 1024 * 1024;
const OUT_AT_ONCE: usize = 256;

pub const FENCED: &str = "default-src 'none'; script-src 'unsafe-inline'; \
style-src 'unsafe-inline'; img-src data: blob:; font-src data:; media-src data: blob:; \
connect-src 'none'; frame-src 'none'; worker-src 'none'; form-action 'none'; base-uri 'none'";

#[derive(Default)]
pub struct Lent(Mutex<VecDeque<(String, String)>>);

impl Lent {
    pub fn lend(&self, body: String) -> Answer<String> {
        if body.len() > LENT_AT_MOST {
            return Err(Refusal::about("widgetTooBig", weighed(LENT_AT_MOST as u64)));
        }
        Ok(self.held(body))
    }

    pub fn lend_kept(&self, body: Vec<u8>) -> Answer<String> {
        if body.len() > KEPT_AT_MOST {
            return Err(Refusal::about("widgetTooBig", weighed(KEPT_AT_MOST as u64)));
        }
        Ok(self.held(String::from_utf8_lossy(&body).into_owned()))
    }

    fn held(&self, body: String) -> String {
        let id = ulid::Ulid::generate().to_string().to_lowercase();
        let mut out = self.0.lock().unwrap_or_else(|e| e.into_inner());
        while out.len() >= OUT_AT_ONCE {
            out.pop_front();
        }
        out.push_back((id.clone(), body));
        id
    }

    pub fn take_back(&self, id: &str) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|(one, _)| one != id);
    }

    pub fn shown(&self, uri: &Uri) -> Response<Vec<u8>> {
        let id = uri.path().trim_start_matches('/');
        let dark = uri
            .query()
            .is_some_and(|asked| asked.split('&').any(|one| one == "dark=1"));
        let body = self
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .find(|(one, _)| one == id)
            .map(|(_, body)| body.clone());
        let (status, page) = match body {
            Some(body) => (StatusCode::OK, shell(&body, dark)),
            None => (StatusCode::NOT_FOUND, String::new()),
        };
        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CONTENT_SECURITY_POLICY, FENCED)
            .header(header::CACHE_CONTROL, "no-store")
            .header("X-Content-Type-Options", "nosniff")
            .body(page.into_bytes())
            .unwrap_or_default()
    }
}

#[tauri::command]
pub fn widget_lend(lent: tauri::State<'_, Lent>, body: String) -> Answer<String> {
    lent.lend(body)
}

pub fn a_page(reference: &str) -> bool {
    let named = reference
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    reference.starts_with("attachments/") && (named.ends_with(".html") || named.ends_with(".htm"))
}

#[tauri::command]
pub async fn widget_lend_kept(
    session: tauri::State<'_, Mutex<Session>>,
    lent: tauri::State<'_, Lent>,
    reference: String,
) -> Answer<String> {
    if !a_page(&reference) {
        return Err(Refusal::of("notAllowed"));
    }
    let at = finding::where_to(&session);
    let body = elsewhere(move || crate::answers::attaching::read_out(reference, at)).await??;
    lent.lend_kept(body)
}

#[tauri::command]
pub fn widget_take_back(lent: tauri::State<'_, Lent>, id: String) {
    lent.take_back(&id);
}

pub fn shell(body: &str, dark: bool) -> String {
    let theme = if dark { " class=\"dark\"" } else { "" };
    format!(
        "<!doctype html><html{theme}><head><meta charset=\"utf-8\">\
<meta name=\"color-scheme\" content=\"light dark\"><style>{KIT}</style></head>\
<body><main class=\"w\">{body}</main><script>{BRIDGE}</script></body></html>"
    )
}

const KIT: &str = r#"
:root{--bg:#ffffff;--sheet:#ffffff;--panel:#fbfbfd;--ink:#1d1d1f;--soft:#57575c;--faint:#67676b;--line:rgb(0 0 0 / 0.16);--hair:rgb(0 0 0 / 0.09);--hover:rgb(0 0 0 / 0.035);--accent:#0060e3;--accent-soft:rgb(0 96 227 / 0.08);--on-accent:#ffffff;--red:#c62f45;--orange:#b35c00;--amber:#8a6a00;--green:#3f8a24;--teal:#0f7a68;--blue:#1f6fb2;--indigo:#4a58c4;--purple:#7a44b8;--pink:#b4408c;--ok-soft:rgb(63 138 36 / 0.12);--warn-soft:rgb(138 106 0 / 0.13);--bad-soft:rgb(198 47 69 / 0.12)}
:root.dark{--bg:#1c1c1e;--sheet:#232326;--panel:#202022;--ink:#f2f2f7;--soft:#adadb4;--faint:#9a9aa1;--line:rgb(255 255 255 / 0.2);--hair:rgb(255 255 255 / 0.12);--hover:rgb(255 255 255 / 0.045);--accent:#439bff;--accent-soft:rgb(67 155 255 / 0.11);--on-accent:#0b1220;--red:#ff7a8a;--orange:#ff9f4a;--amber:#d9b02e;--green:#6fc44a;--teal:#3ec6ae;--blue:#5aa9f0;--indigo:#8f9bff;--purple:#c08cff;--pink:#ff7ac4;--ok-soft:rgb(111 196 74 / 0.16);--warn-soft:rgb(217 176 46 / 0.16);--bad-soft:rgb(255 122 138 / 0.16)}
:root{color-scheme:light}:root.dark{color-scheme:dark}
*{box-sizing:border-box;scrollbar-width:thin;scrollbar-color:transparent transparent}
*:hover{scrollbar-color:var(--line) transparent}
html,body{margin:0;background:var(--sheet);color:var(--ink)}
body{font:13.5px/1.5 -apple-system,BlinkMacSystemFont,"Segoe UI Variable Text","Segoe UI",system-ui,sans-serif}
a{color:var(--accent)}
h1,h2,h3,h4{margin:0 0 8px;font-weight:600;line-height:1.25}
h1{font-size:20px}h2{font-size:16px}h3{font-size:14px}
p{margin:0 0 8px}
small,.label{display:block;font-size:12px;color:var(--faint);margin:0}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:12px}
.two{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px}
.stat,.card{border:1px solid var(--hair);border-radius:10px;background:var(--bg)}
.stat{padding:12px 14px;display:flex;flex-direction:column;gap:2px}
.stat b{font-size:24px;font-weight:650;letter-spacing:-0.01em;line-height:1.2}
.card{padding:14px}
.head{display:flex;align-items:baseline;justify-content:space-between;gap:8px;margin:0 0 10px}
.head h3{margin:0}
.up,.down{font-size:12px;font-style:normal;font-weight:550}
.up{color:var(--green)}.down{color:var(--red)}
table{width:100%;border-collapse:collapse;font-size:13px}
th{text-align:left;font-weight:550;color:var(--faint);font-size:12px;padding:6px 8px;border-bottom:1px solid var(--line)}
td{padding:8px;border-bottom:1px solid var(--hair)}
tr:last-child td{border-bottom:0}
.num{text-align:right;font-variant-numeric:tabular-nums}
.badge{display:inline-block;font-size:11.5px;padding:1px 8px;border-radius:999px;font-weight:550;background:var(--accent-soft);color:var(--accent)}
.badge.ok{background:var(--ok-soft);color:var(--green)}
.badge.warn{background:var(--warn-soft);color:var(--amber)}
.badge.bad{background:var(--bad-soft);color:var(--red)}
.row{display:flex;gap:8px;flex-wrap:wrap;margin-top:12px}
button{font:inherit;font-size:13px;border-radius:8px;padding:7px 14px;border:1px solid var(--line);background:var(--bg);color:var(--ink);cursor:pointer}
button:hover{background:var(--hover)}
button.primary{background:var(--accent);border-color:var(--accent);color:var(--on-accent);font-weight:550}
.list{list-style:none;margin:0;padding:0}
.list li{display:flex;align-items:center;gap:10px;padding:7px 0;border-bottom:1px solid var(--hair)}
.list li:last-child{border-bottom:0}
.box{width:16px;height:16px;border:1.5px solid var(--line);border-radius:5px;flex:0 0 auto}
.box.done{background:var(--accent);border-color:var(--accent)}
.gone{color:var(--faint);text-decoration:line-through}
svg{max-width:100%}
"#;

const BRIDGE: &str = r#"(()=>{const say=(m)=>parent.postMessage(m,"*");const box=document.querySelector("main.w");const tell=()=>say({type:"resize",height:Math.ceil(Math.max(box.scrollHeight,box.getBoundingClientRect().height))});new ResizeObserver(tell).observe(box);addEventListener("load",tell);addEventListener("message",(e)=>{if(e.source!==parent||!e.data||e.data.type!=="theme")return;document.documentElement.classList.toggle("dark",!!e.data.dark)});addEventListener("click",(e)=>{const a=e.target instanceof Element?e.target.closest("a[href]"):null;if(!a)return;e.preventDefault();say({type:"open",href:a.getAttribute("href")})})})();"#;

#[cfg(test)]
#[path = "widgets_test.rs"]
mod tests;
