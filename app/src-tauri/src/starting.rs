use std::sync::{Arc, Mutex, PoisonError};
use tauri::{Manager, Wry, ipc::Invoke};

pub type Call = Box<dyn FnOnce() + Send>;

// Tauri builds the configured windows before the app's own setup, so a page can ask before the store is open.
pub struct Starting(Mutex<Option<Vec<Call>>>);

impl Default for Starting {
    fn default() -> Self {
        Self(Mutex::new(Some(Vec::new())))
    }
}

impl Starting {
    pub fn over(&self) -> bool {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_none()
    }

    pub fn keep(&self, call: Call) -> Option<Call> {
        match self
            .0
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

    pub fn finished(&self) -> Vec<Call> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
            .unwrap_or_default()
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
        if let Some(now) = starting.keep(Box::new(move || {
            later(invoke);
        })) {
            now();
        }
        true
    }
}

#[cfg(test)]
#[path = "starting_test.rs"]
mod tests;
