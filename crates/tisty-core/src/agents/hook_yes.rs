use std::path::{Path, PathBuf};

use super::{Card, HookYes, Kind, Refused};

const SCRIPT_AT_MOST: u64 = 1024 * 1024;

pub fn hooks_folder(data: &Path) -> PathBuf {
    super::folder(data).join("hooks")
}

pub fn script_at(data: &Path, name: &str) -> PathBuf {
    hooks_folder(data).join(name)
}

pub fn yes_of(body: &str) -> Result<(HookYes, Vec<u8>), Refused> {
    let card = Card::of(body).ok_or(Refused::NoCard)?;
    let trigger = card.trigger.as_ref().ok_or(Refused::NoCard)?;
    if card.kind != Kind::Hook {
        return Err(Refused::NoCard);
    }
    let bytes = super::rendered(body)?;
    let yes = HookYes {
        print: crate::docs::print_of_text(body),
        name: card.name.clone(),
        script: crate::attach::printed(&bytes),
        event: trigger.event.word().to_string(),
        tools: trigger.tools.word().to_string(),
        timeout: trigger.timeout,
        agents: trigger.agents.clone(),
    };
    Ok((yes, bytes))
}

pub fn keep_script(data: &Path, yes: &HookYes, bytes: &[u8]) -> Result<(), Refused> {
    let folder = hooks_folder(data);
    std::fs::create_dir_all(&folder)?;
    let _ = crate::paths::ours_alone(&folder);
    crate::store::write_atomic(&script_at(data, &yes.name), bytes)?;
    Ok(())
}

pub fn drop_script(data: &Path, name: &str) {
    let _ = std::fs::remove_file(script_at(data, name));
}

pub fn script_kept(data: &Path, yes: &HookYes) -> Option<Vec<u8>> {
    let at = script_at(data, &yes.name);
    let fits = std::fs::metadata(&at).ok()?.len() <= SCRIPT_AT_MOST;
    let bytes = fits.then(|| std::fs::read(&at).ok()).flatten()?;
    (crate::attach::printed(&bytes) == yes.script).then_some(bytes)
}

#[cfg(test)]
#[path = "hook_yes_test.rs"]
mod tests;
