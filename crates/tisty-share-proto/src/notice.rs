use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

const SAID: u32 = 1;
const UNSEALED: &[u8] = b"tisty.unsealed.v1\0";

// Left in the plain root by the machine that turned a cloud back to plain, for the machines its sealed log can no longer reach.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unsealed {
    pub v: u32,
    pub cloud: String,
    pub named: String,
    pub by: String,
    pub key: String,
    pub at: i64,
    pub sig: String,
}

fn coded(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn decoded<const N: usize>(said: &str) -> Option<[u8; N]> {
    <[u8; N]>::try_from(URL_SAFE_NO_PAD.decode(said).ok()?.as_slice()).ok()
}

impl Unsealed {
    pub fn signed(cloud: &str, named: &str, by: &str, signer: &SigningKey, at: i64) -> Self {
        let mut notice = Self {
            v: SAID,
            cloud: cloud.to_string(),
            named: named.to_string(),
            by: by.to_string(),
            key: coded(signer.verifying_key().as_bytes()),
            at,
            sig: String::new(),
        };
        notice.sig = coded(&signer.sign(&notice.said()).to_bytes());
        notice
    }

    fn said(&self) -> Vec<u8> {
        let mut said = UNSEALED.to_vec();
        for part in [
            &self.v.to_be_bytes()[..],
            self.cloud.as_bytes(),
            self.named.as_bytes(),
            self.by.as_bytes(),
            self.key.as_bytes(),
            &self.at.to_be_bytes(),
        ] {
            said.extend_from_slice(&(part.len() as u64).to_be_bytes());
            said.extend_from_slice(part);
        }
        said
    }

    pub fn key(&self) -> Option<VerifyingKey> {
        VerifyingKey::from_bytes(&decoded::<32>(&self.key)?).ok()
    }

    // Signed by the key it carries; whether that key is a machine of the person's is for the log to say.
    pub fn holds(&self) -> bool {
        let sig = decoded::<64>(&self.sig).map(|bytes| Signature::from_bytes(&bytes));
        match (self.key(), sig) {
            (Some(key), Some(sig)) => self.v == SAID && key.verify(&self.said(), &sig).is_ok(),
            _ => false,
        }
    }
}

#[cfg(test)]
#[path = "notice_test.rs"]
mod tests;
