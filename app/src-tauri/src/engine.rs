// WebView2 refuses a window that starts differently from those open, dev builds share the installed app's folder, and nothing shown comes from an address: so one set of arguments, with only the dev port named.
const ENGINE_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --autoplay-policy=no-user-gesture-required --webrtc-ip-handling-policy=disable_non_proxied_udp --host-resolver-rules=\"MAP localhost:1420 localhost:1420, MAP * ~NOTFOUND\"";

pub fn cut_off(config: &mut tauri::Config) {
    for window in &mut config.app.windows {
        window.additional_browser_args = Some(ENGINE_ARGS.to_string());
    }
}

pub fn window<'a, M: tauri::Manager<tauri::Wry>>(
    manager: &'a M,
    label: impl Into<String>,
    page: tauri::WebviewUrl,
) -> tauri::WebviewWindowBuilder<'a, tauri::Wry, M> {
    tauri::WebviewWindowBuilder::new(manager, label, page).additional_browser_args(ENGINE_ARGS)
}

#[cfg(test)]
#[path = "engine_test.rs"]
mod tests;
