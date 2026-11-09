use std::path::Path;
use std::process::ExitCode;

use tisty_core::Paths;
use tisty_core::config::Config;
use tisty_core::guide_seal::{MAKER_ROOT, SEAL, guide_files, sealed};
use tisty_sharing::{certified_by, signer};

fn signed_here() -> Result<String, String> {
    let paths = Paths::resolve().map_err(|e| format!("no home for Tisty here: {e}"))?;
    let config = Config::load(&paths.config_file())
        .map_err(|e| format!("this Tisty's settings could not be read: {e}"))?
        .ok_or("Tisty was never opened on this machine")?;
    let state = tisty_core::cache::project(&paths.store(), paths.cache())
        .map_err(|e| format!("this Tisty's history could not be read: {e}"))?;
    let (key, certificate) =
        signer(&paths, &state, &config.device_id).ok_or("this machine answers to no root")?;
    certified_by(&certificate, MAKER_ROOT).ok_or("only the maker's root signs the guide")?;
    let author = state
        .signed
        .alias
        .clone()
        .ok_or("this Tisty signs with no alias")?;
    let guide = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../app/src-tauri/resources/guide");
    let files = guide_files(&guide).map_err(|e| format!("the guide could not be read: {e}"))?;
    let seal = sealed(&key, &certificate, &author, files).ok_or("the guide could not be signed")?;
    let mut said = serde_json::to_string_pretty(&seal).map_err(|e| e.to_string())?;
    said.push('\n');
    std::fs::write(guide.join(SEAL), said)
        .map_err(|e| format!("the seal could not be written: {e}"))?;
    Ok(author)
}

fn main() -> ExitCode {
    match signed_here() {
        Ok(author) => {
            println!("the guide is signed as {author}");
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("{why}");
            ExitCode::FAILURE
        }
    }
}
