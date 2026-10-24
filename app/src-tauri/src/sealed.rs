use std::path::{Path, PathBuf};

use tisty_carrier::{Hitch, Secrets};

/// A Store install writes here inside its own package and the installer's uninstaller removes it, so a token leaves with the app.
pub fn home() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|local| PathBuf::from(local).join("Tisty").join("secrets"))
}

#[cfg(not(test))]
pub fn seed_home() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(|local| PathBuf::from(local).join("Tisty").join("identity"))
}

pub struct Sealed {
    at: PathBuf,
}

impl Sealed {
    pub fn under(home: &Path, scope: &str) -> Self {
        Self {
            at: home.join(plain(scope)),
        }
    }

    fn file(&self, key: &str) -> PathBuf {
        self.at.join(plain(key))
    }
}

fn plain(said: &str) -> String {
    said.chars()
        .map(
            |one| match one.is_ascii_alphanumeric() || one == '-' || one == '_' {
                true => one,
                false => '_',
            },
        )
        .collect()
}

impl Secrets for Sealed {
    fn read(&self, key: &str) -> Result<Option<String>, Hitch> {
        let sealed = match std::fs::read(self.file(key)) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let open = dpapi::opened(&sealed)?;
        String::from_utf8(open)
            .map(Some)
            .map_err(|_| Hitch::Broke("a sealed token is not text".into()))
    }

    fn write(&self, key: &str, value: &str) -> Result<(), Hitch> {
        std::fs::create_dir_all(&self.at)?;
        let sealed = dpapi::sealed(value.as_bytes())?;
        tisty_core::store::write_atomic(&self.file(key), &sealed)
            .map_err(|e| Hitch::Broke(e.to_string()))
    }

    fn forget(&self, key: &str) -> Result<(), Hitch> {
        match std::fs::remove_file(self.file(key)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

mod dpapi {
    use tisty_carrier::Hitch;
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Cryptography::{
        CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
    };

    fn blob(bytes: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: u32::try_from(bytes.len()).unwrap_or(u32::MAX),
            pbData: bytes.as_ptr().cast_mut(),
        }
    }

    // The bytes Windows hands back are its own until copied out and freed, exactly once.
    #[allow(unsafe_code)]
    fn taken(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
        let bytes = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize) }.to_vec();
        unsafe {
            let _ = LocalFree(Some(HLOCAL(out.pbData.cast())));
        }
        bytes
    }

    #[allow(unsafe_code)]
    pub fn sealed(open: &[u8]) -> Result<Vec<u8>, Hitch> {
        let mut out = CRYPT_INTEGER_BLOB::default();
        unsafe {
            CryptProtectData(
                &blob(open),
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        }
        .map_err(|e| Hitch::Broke(format!("Windows would not seal the token: {e}")))?;
        Ok(taken(out))
    }

    #[allow(unsafe_code)]
    pub fn opened(sealed: &[u8]) -> Result<Vec<u8>, Hitch> {
        let mut out = CRYPT_INTEGER_BLOB::default();
        unsafe {
            CryptUnprotectData(
                &blob(sealed),
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        }
        .map_err(|_| Hitch::Lost)?;
        Ok(taken(out))
    }
}

#[cfg(test)]
#[path = "sealed_test.rs"]
mod tests;
