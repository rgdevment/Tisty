use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use tauri::{Manager, Wry, ipc::Invoke};

pub type Call = Box<dyn FnOnce() + Send>;

// Tauri builds the configured windows before the app's own setup, so a page can ask before the store is open.
pub struct Starting {
    open: AtomicBool,
    kept: Mutex<Option<Vec<Call>>>,
}

impl Default for Starting {
    fn default() -> Self {
        Self {
            open: AtomicBool::new(false),
            kept: Mutex::new(Some(Vec::new())),
        }
    }
}

impl Starting {
    pub fn over(&self) -> bool {
        self.open.load(Ordering::Acquire)
    }

    pub fn keep(&self, call: Call) -> Option<Call> {
        match self
            .kept
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_mut()
        {
            Some(kept) => {
                kept.push(call);
                None
            }
            None => Some(call),
        }
    }

    pub fn finish(&self) {
        loop {
            let calls = {
                let mut kept = self.kept.lock().unwrap_or_else(PoisonError::into_inner);
                match kept.as_mut() {
                    Some(calls) if !calls.is_empty() => std::mem::take(calls),
                    _ => {
                        *kept = None;
                        self.open.store(true, Ordering::Release);
                        return;
                    }
                }
            };
            for call in calls {
                call();
            }
        }
    }
}

pub fn waiting<H>(handler: H) -> impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static
where
    H: Fn(Invoke<Wry>) -> bool + Send + Sync + 'static,
{
    let handler = Arc::new(handler);
    move |invoke: Invoke<Wry>| {
        let app = invoke.message.webview().app_handle().clone();
        let starting = app.state::<Starting>();
        if starting.over() {
            return handler(invoke);
        }
        let later = Arc::clone(&handler);
        let resolver = invoke.resolver.clone();
        let command = invoke.message.command().to_string();
        let call: Call = Box::new(move || {
            if !later(invoke) {
                resolver.reject(format!("command {command} not found"));
            }
        });
        if let Some(now) = starting.keep(call) {
            now();
        }
        true
    }
}

#[cfg(test)]
#[path = "starting_test.rs"]
mod tests;
