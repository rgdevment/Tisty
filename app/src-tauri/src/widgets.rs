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

// An attached page unpacks itself into blobs and frames of its own, so it may reach those, never the network.
pub const PAGED: &str = "default-src 'none'; script-src 'unsafe-inline' 'unsafe-eval' blob:; \
style-src 'unsafe-inline' blob:; img-src data: blob:; font-src data: blob:; media-src data: blob:; \
connect-src data: blob:; frame-src blob: data:; worker-src blob:; form-action 'none'; base-uri 'none'";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Fenced,
    Page,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct Borrowed {
    pub id: String,
    pub whole: bool,
}

#[derive(Default)]
pub struct Lent(Mutex<VecDeque<(String, String, Kind)>>);

impl Lent {
    pub fn lend(&self, body: String) -> Answer<String> {
        if body.len() > LENT_AT_MOST {
            return Err(Refusal::about("widgetTooBig", weighed(LENT_AT_MOST as u64)));
        }
        Ok(self.held(body, Kind::Fenced))
    }

    pub fn lend_kept(&self, body: Vec<u8>) -> Answer<Borrowed> {
        if body.len() > KEPT_AT_MOST {
            return Err(too_big_a_page());
        }
        let body = String::from_utf8_lossy(&body).into_owned();
        let whole = whole(&body);
        let kind = if whole { Kind::Page } else { Kind::Fenced };
        Ok(Borrowed {
            id: self.held(body, kind),
            whole,
        })
    }

    fn held(&self, body: String, kind: Kind) -> String {
        let id = ulid::Ulid::generate().to_string().to_lowercase();
        let mut out = self.0.lock().unwrap_or_else(|e| e.into_inner());
        while out.len() >= OUT_AT_ONCE {
            out.pop_front();
        }
        out.push_back((id.clone(), body, kind));
        id
    }

    pub fn take_back(&self, id: &str) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|(one, _, _)| one != id);
    }

    pub fn shown(&self, uri: &Uri) -> Response<Vec<u8>> {
        let id = uri.path().trim_start_matches('/');
        let dark = uri
            .query()
            .is_some_and(|asked| asked.split('&').any(|one| one == "dark=1"));
        let found = self
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .find(|(one, _, _)| one == id)
            .map(|(_, body, kind)| (body.clone(), *kind));
        let (status, page, policy) = match found {
            Some((body, Kind::Fenced)) => (StatusCode::OK, shell(&body, dark), FENCED),
            Some((body, Kind::Page)) => (StatusCode::OK, paged(&body), PAGED),
            None => (StatusCode::NOT_FOUND, String::new(), FENCED),
        };
        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CONTENT_SECURITY_POLICY, policy)
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

fn too_big_a_page() -> Refusal {
    Refusal::about("pageTooBig", weighed(KEPT_AT_MOST as u64))
}

pub fn small_enough(weighs: u64) -> Answer<()> {
    match weighs > KEPT_AT_MOST as u64 {
        true => Err(too_big_a_page()),
        false => Ok(()),
    }
}

fn kept_page(reference: String, at: finding::Where) -> Answer<Vec<u8>> {
    let found = finding::handed_over(&reference, &at)?;
    if let Ok(told) = std::fs::metadata(&found) {
        small_enough(told.len())?;
    }
    crate::answers::attaching::read_out(reference, at)
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
) -> Answer<Borrowed> {
    if !a_page(&reference) {
        return Err(Refusal::of("notAllowed"));
    }
    let at = finding::where_to(&session);
    let body = elsewhere(move || kept_page(reference, at)).await??;
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

fn opens_with(text: &str, word: &str) -> bool {
    text.get(..word.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(word))
}

fn prelude(body: &str) -> (usize, bool) {
    let mut at = 0;
    let mut declared = false;
    loop {
        let rest = &body[at..];
        let bare = rest.trim_start_matches(|c: char| c == '\u{feff}' || c.is_whitespace());
        at += rest.len() - bare.len();
        let closing = if bare.starts_with("<!--") {
            "-->"
        } else if opens_with(bare, "<!doctype") {
            declared = true;
            ">"
        } else {
            return (at, declared);
        };
        match bare.find(closing) {
            Some(end) => at += end + closing.len(),
            None => return (body.len(), declared),
        }
    }
}

pub fn whole(body: &str) -> bool {
    let (at, declared) = prelude(body);
    let rest = &body[at..];
    declared || opens_with(rest, "<html") || opens_with(rest, "<head")
}

// Before anything the page holds, so no script or string of its own can swallow the measurer.
pub fn paged(body: &str) -> String {
    let (at, _) = prelude(body);
    format!("{}<script>{MEASURER}</script>{}", &body[..at], &body[at..])
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

const BRIDGE: &str = r##"(()=>{const say=(m)=>parent.postMessage(m,"*");const box=document.querySelector("main.w");const tell=()=>say({type:"resize",height:Math.ceil(Math.max(box.scrollHeight,box.getBoundingClientRect().height))});new ResizeObserver(tell).observe(box);addEventListener("load",tell);const themed=()=>{const h=location.hash;if(h==="#dark"||h==="#light")document.documentElement.classList.toggle("dark",h==="#dark")};themed();addEventListener("hashchange",themed);addEventListener("click",(e)=>{const a=e.target instanceof Element?e.target.closest("a[href]"):null;if(!a)return;e.preventDefault();say({type:"open",href:a.getAttribute("href")})})})();"##;

// A page may replace its whole document while it unpacks, so it is measured from the window, by what flows in its body.
const MEASURER: &str = r##"(()=>{const say=(m)=>parent.postMessage(m,"*");let queued=false;const px=(v)=>Number.parseFloat(v)||0;const span=document.createRange();const reach=()=>{const b=document.body;if(!b)return 0;let low=0;for(const one of b.childNodes){let bottom=0;if(one.nodeType===1){const s=getComputedStyle(one);if(s.position==="fixed"||s.display==="none")continue;const r=one.getBoundingClientRect();if(r.height<=0&&r.width<=0)continue;bottom=r.bottom+px(s.marginBottom)}else if(one.nodeType===3&&one.textContent.trim()){span.selectNodeContents(one);bottom=span.getBoundingClientRect().bottom}low=Math.max(low,bottom)}if(low<=0)return 0;const s=getComputedStyle(b);return Math.ceil(low+scrollY+px(s.paddingBottom)+px(s.borderBottomWidth)+px(s.marginBottom))};const tell=()=>{if(queued)return;queued=true;requestAnimationFrame(()=>{queued=false;const h=reach();if(h>0)say({type:"resize",height:h})})};let root=null;const seen=new ResizeObserver(tell);const watch=()=>{if(document.documentElement!==root){root=document.documentElement;seen.disconnect();seen.observe(root)}tell()};new MutationObserver(watch).observe(document,{childList:true,subtree:true});addEventListener("load",tell,true);addEventListener("resize",tell);watch();addEventListener("click",(e)=>{const a=e.target instanceof Element?e.target.closest("a[href]"):null;if(!a)return;const href=a.getAttribute("href")||"";if(href.startsWith("#"))return;e.preventDefault();say({type:"open",href})},true)})();"##;

#[cfg(test)]
#[path = "widgets_test.rs"]
mod tests;
