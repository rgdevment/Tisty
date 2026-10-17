use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use tauri::http::{Response, StatusCode, Uri, header};

use crate::{Answer, Refusal, Session, finding, weighed};

pub const LENT_AT_MOST: usize = 512 * 1024;
pub const KEPT_AT_MOST: usize = 8 * 1024 * 1024;
const OUT_AT_ONCE: usize = 256;

pub const FENCED: &str = "default-src 'none'; script-src 'unsafe-inline'; \
style-src 'unsafe-inline'; img-src data: blob:; font-src data:; media-src data: blob:; \
connect-src 'none'; frame-src 'none'; worker-src 'none'; form-action 'none'; base-uri 'none'";

// An attached page unpacks itself into blobs and frames of its own, so it may reach those, never the network.
pub const PAGED: &str = "default-src 'none'; script-src 'unsafe-inline' 'unsafe-eval' blob: data:; \
style-src 'unsafe-inline' blob:; img-src data: blob:; font-src data: blob:; media-src data: blob:; \
connect-src data: blob:; frame-src blob:; worker-src blob:; form-action 'none'; base-uri 'none'";

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
pub struct Lent {
    out: Mutex<VecDeque<(String, String, Kind)>>,
    owners: Mutex<HashMap<String, Vec<String>>>,
}

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
        let mut out = self.out.lock().unwrap_or_else(|e| e.into_inner());
        while out.len() >= OUT_AT_ONCE {
            out.pop_front();
        }
        out.push_back((id.clone(), body, kind));
        id
    }

    // A window closed from outside never runs its own clean-up, so what it borrowed is taken back by its name.
    pub fn owned_by(&self, window: &str, id: &str) {
        if !window.starts_with("view-") {
            return;
        }
        self.owners
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entry(window.to_string())
            .or_default()
            .push(id.to_string());
    }

    pub fn release(&self, window: &str) {
        let gone = self
            .owners
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(window)
            .unwrap_or_default();
        for id in gone {
            self.take_back(&id);
        }
    }

    pub fn take_back(&self, id: &str) {
        for ids in self
            .owners
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values_mut()
        {
            ids.retain(|one| one != id);
        }
        self.out
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|(one, _, _)| one != id);
    }

    pub fn shown(&self, uri: &Uri) -> Response<Vec<u8>> {
        let id = uri.path().trim_start_matches('/');
        let asked = |what: &str| {
            uri.query()
                .is_some_and(|all| all.split('&').any(|one| one == what))
        };
        let dark = asked("dark=1");
        let spanned = asked("span=1");
        let found = self
            .out
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .find(|(one, _, _)| one == id)
            .map(|(_, body, kind)| (body.clone(), *kind));
        let (status, page, policy) = match found {
            Some((body, Kind::Fenced)) => (StatusCode::OK, shell(&body, dark, spanned), FENCED),
            Some((body, Kind::Page)) => (StatusCode::OK, paged(&body, spanned), PAGED),
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
pub fn widget_lend(
    window: tauri::WebviewWindow,
    views: tauri::State<'_, crate::viewing::Views>,
    lent: tauri::State<'_, Lent>,
    body: String,
) -> Answer<String> {
    if views.stranger(window.label()) {
        return Err(Refusal::of("notAllowed"));
    }
    let id = lent.lend(body)?;
    lent.owned_by(window.label(), &id);
    Ok(id)
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
    let weighs = finding::weight_of(&reference, &at.data, at.shared.as_deref(), at.weight);
    if let Some(weighs) = weighs {
        small_enough(weighs)?;
    }
    crate::answers::attaching::read_out(reference, at)
}

pub fn a_page(reference: &str) -> bool {
    let named = reference
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    tisty_core::attach::names_an_attachment(reference)
        && (named.ends_with(".html") || named.ends_with(".htm"))
}

#[tauri::command]
pub async fn widget_lend_kept(
    window: tauri::WebviewWindow,
    views: tauri::State<'_, crate::viewing::Views>,
    session: tauri::State<'_, Mutex<Session>>,
    lent: tauri::State<'_, Lent>,
    reference: String,
) -> Answer<Borrowed> {
    if !a_page(&reference) || views.stranger(window.label()) {
        return Err(Refusal::of("notAllowed"));
    }
    let body = finding::fetched(&session, reference, kept_page).await?;
    let borrowed = lent.lend_kept(body)?;
    lent.owned_by(window.label(), &borrowed.id);
    Ok(borrowed)
}

#[tauri::command]
pub fn widget_take_back(lent: tauri::State<'_, Lent>, id: String) {
    lent.take_back(&id);
}

pub fn shell(body: &str, dark: bool, spanned: bool) -> String {
    let theme = if dark { " class=\"dark\"" } else { "" };
    let span = if spanned { SPAN } else { "" };
    format!(
        "<!doctype html><html{theme}><head><meta charset=\"utf-8\">\
<meta name=\"color-scheme\" content=\"light dark\"><style>{KIT}</style></head>\
<body><main class=\"w\">{body}</main><script>{BRIDGE}{span}</script></body></html>"
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
    declared || opens_tag(rest, "<html") || opens_tag(rest, "<head")
}

fn opens_tag(text: &str, tag: &str) -> bool {
    opens_with(text, tag)
        && text[tag.len()..]
            .chars()
            .next()
            .is_some_and(|after| after == '>' || after == '/' || after.is_whitespace())
}

// Before anything the page holds, so no script or string of its own can swallow the measurer.
pub fn paged(body: &str, spanned: bool) -> String {
    let (at, _) = prelude(body);
    let span = if spanned { SPAN } else { "" };
    format!(
        "{}<script>{MEASURER}{span}</script>{}",
        &body[..at],
        &body[at..]
    )
}

// The guide dresses its widgets from the same file, so a widget reads the same in both.
const KIT: &str = include_str!("widget.css");

const BRIDGE: &str = r##"(()=>{const say=(m)=>parent.postMessage(m,"*");const box=document.querySelector("main.w");const tell=()=>say({type:"resize",height:Math.ceil(Math.max(box.scrollHeight,box.getBoundingClientRect().height))});new ResizeObserver(tell).observe(box);addEventListener("load",tell);const themed=()=>{const h=location.hash;if(h==="#dark"||h==="#light")document.documentElement.classList.toggle("dark",h==="#dark")};themed();addEventListener("hashchange",themed);addEventListener("click",(e)=>{const a=e.target instanceof Element?e.target.closest("a[href]"):null;if(!a)return;e.preventDefault();say({type:"open",href:a.getAttribute("href")})})})();"##;

// A page may replace its whole document while it unpacks, so it is measured from the window, by what flows in its body.
const MEASURER: &str = r##"(()=>{const say=(m)=>parent.postMessage(m,"*");let queued=false;const px=(v)=>Number.parseFloat(v)||0;const span=document.createRange();let view=innerHeight;const sizes=new WeakMap();const fluid=new WeakSet();const within=(one,depth)=>{let low=0;for(const inner of one.childNodes)low=Math.max(low,edge(inner,depth));return low};const edge=(one,depth)=>{if(one.nodeType===3&&one.textContent.trim()){span.selectNodeContents(one);return span.getBoundingClientRect().bottom}if(one.nodeType!==1)return 0;const s=getComputedStyle(one);if(s.position==="fixed"||s.display==="none")return 0;if(s.display==="contents")return depth<6?within(one,depth+1):0;const r=one.getBoundingClientRect();if(r.height<=0&&r.width<=0)return 0;const was=sizes.get(one);if(was!==undefined&&innerHeight!==view&&Math.abs(r.height-was-(innerHeight-view))<1)fluid.add(one);sizes.set(one,r.height);if(fluid.has(one)&&depth<6){const low=within(one,depth+1);if(low>0)return low+px(s.paddingBottom)+px(s.borderBottomWidth)+px(s.marginBottom)}return r.top+Math.max(r.height,one.scrollHeight)+px(s.marginBottom)};const reach=()=>{const b=document.body;if(!b)return 0;const low=within(b,0);view=innerHeight;if(low<=0)return 0;const s=getComputedStyle(b);return Math.ceil(low+scrollY+px(s.paddingBottom)+px(s.borderBottomWidth)+px(s.marginBottom))};const tell=()=>{if(queued)return;queued=true;requestAnimationFrame(()=>{queued=false;const h=reach();if(h>0)say({type:"resize",height:h})})};const clicked=(e)=>{if(e.defaultPrevented)return;const a=e.target instanceof Element?e.target.closest("a[href]"):null;if(!a)return;const href=(a.getAttribute("href")||"").trim();if(href.startsWith("#")||/^javascript:/i.test(href))return;e.preventDefault();say({type:"open",href})};const last=()=>{removeEventListener("click",clicked);addEventListener("click",clicked)};const hooked=()=>{document.addEventListener("load",tell,true);addEventListener("load",tell);addEventListener("resize",tell);document.addEventListener("DOMContentLoaded",last);addEventListener("load",last);last()};let root=null;const seen=new ResizeObserver(tell);const watch=()=>{if(document.documentElement!==root){root=document.documentElement;seen.disconnect();seen.observe(root);hooked()}const b=document.body;if(b){seen.observe(b);for(const one of b.children)seen.observe(one)}tell()};new MutationObserver(watch).observe(document,{childList:true,subtree:true});watch()})();"##;

// How far the content reaches across, so a viewer can centre a widget drawn at a fixed width.
const SPAN: &str = r##"(()=>{const say=(m)=>parent.postMessage(m,"*");let at=0;const tell=()=>{cancelAnimationFrame(at);at=requestAnimationFrame(()=>{const b=document.body;if(!b)return;let l=Infinity,r=0,n=0;for(const e of b.querySelectorAll("*")){if(++n>3000)break;const q=e.getBoundingClientRect();if(!q.width||!q.height||(q.left<=0.5&&q.right>=innerWidth-0.5))continue;if(getComputedStyle(e).position==="fixed")continue;l=Math.min(l,q.left);r=Math.max(r,q.right)}if(r>0)say({type:"span",left:Math.max(0,Math.round(l)),right:Math.ceil(r),view:innerWidth})})};addEventListener("load",tell);new ResizeObserver(tell).observe(document.documentElement)})();"##;

#[cfg(test)]
#[path = "widgets_test.rs"]
mod tests;
