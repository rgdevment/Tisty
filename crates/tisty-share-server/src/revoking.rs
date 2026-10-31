use std::net::IpAddr;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use tisty_share_proto::identity::Revocation;

use crate::guard::Act;
use crate::keys::Randomness;
use crate::share::{Refused, Share};
use crate::store::Store;

/// A key as the store names it: its bytes, so two spellings of one key are never two rows.
fn named(key: &[u8; 32]) -> String {
    URL_SAFE_NO_PAD.encode(key)
}

impl<S: Store, R: Randomness> Share<S, R> {
    /// Anyone may carry a root's word taking a key back: the root's signature is the authority, not who brings it (decision 73).
    pub async fn revoke(
        &mut self,
        revocation: &Revocation,
        caller: IpAddr,
        now: i64,
    ) -> Result<(), Refused> {
        self.admit(caller, Act::Revoke, now).await?;
        let (person, share) = revocation.holds().map_err(|_| Refused::Unsigned)?;
        self.store
            .revoke(&named(share.as_bytes()), &person.shown(), revocation)
            .await?;
        Ok(())
    }

    /// What took this key back; a record the store holds that no longer reads is left out, never trusted.
    pub(crate) async fn revoked(&self, key: Option<&[u8; 32]>) -> Result<Vec<Revocation>, Refused> {
        let Some(key) = key else {
            return Ok(Vec::new());
        };
        let mut records = self.store.revocations(&named(key)).await?;
        records.retain(|one| one.holds().is_ok());
        Ok(records)
    }
}
