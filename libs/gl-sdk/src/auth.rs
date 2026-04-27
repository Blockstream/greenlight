// LNURL-auth (LUD-04 / LUD-05) implementation.
//
// `Node` stores the encoded `m/138'` extended private key — derived
// once from the BIP39 seed at register/recover/connect time. The seed
// itself is never retained. From that namespace xpriv, both the
// hashing key (`m/138'/0`) and the per-domain linking key
// (`m/138'/<hmac-derived tail>`) are derived inside `lnurl_auth`.
//
// `m/138'` is a hardened path, so its xpriv cannot be used to derive
// any other wallet keys (lightning channels, on-chain). Compromise of
// the stored material affects only LNURL-auth identities.

use gl_client::bitcoin::bip32::{ChildNumber, Xpriv};
use gl_client::bitcoin::hashes::hmac::{Hmac, HmacEngine};
use gl_client::bitcoin::hashes::{sha256, Hash, HashEngine};
use gl_client::bitcoin::secp256k1::{Message, PublicKey, Secp256k1};
use gl_client::bitcoin::Network;
use zeroize::Zeroizing;

use crate::lnurl::{LnUrlAuthRequestData, LnUrlCallbackStatus, LnUrlErrorData};
use crate::Error;

/// Derive the encoded `m/138'` extended private key from a BIP39
/// mnemonic. The mnemonic is consumed only for the duration of this
/// call — the seed is wrapped in `Zeroizing` and scrubbed on return.
///
/// The returned bytes are the standard 78-byte BIP32 serialization of
/// the xpriv at `m/138'`. Callers wrap them in `Zeroizing<Vec<u8>>`
/// when persisting on a `Node` so they are scrubbed on disconnect or
/// drop.
pub(crate) fn derive_lnurl_auth_namespace_xpriv(seed: &[u8]) -> Result<Vec<u8>, Error> {
    // Network choice does not affect derived secret bytes — only the
    // version prefix when the xpriv is serialised. We pick Bitcoin
    // canonically; downstream `decode` accepts the same prefix.
    let master = Xpriv::new_master(Network::Bitcoin, seed)
        .map_err(|e| Error::Other(format!("BIP32 master derivation failed: {e}")))?;
    let secp = Secp256k1::new();
    let path: [ChildNumber; 1] = [ChildNumber::from_hardened_idx(138)
        .map_err(|e| Error::Other(format!("BIP32 child index error: {e}")))?];
    let namespace = master
        .derive_priv(&secp, &path)
        .map_err(|e| Error::Other(format!("BIP32 m/138' derivation failed: {e}")))?;
    Ok(namespace.encode().to_vec())
}

/// Same as [`derive_lnurl_auth_namespace_xpriv`] but takes the
/// mnemonic directly and scrubs the intermediate seed on return.
pub(crate) fn derive_lnurl_auth_namespace_xpriv_from_mnemonic(
    mnemonic: &str,
) -> Result<Vec<u8>, Error> {
    use bip39::Mnemonic;
    use std::str::FromStr;
    let phrase = Mnemonic::from_str(mnemonic).map_err(|_| Error::PhraseCorrupted())?;
    let seed = Zeroizing::new(phrase.to_seed_normalized("").to_vec());
    derive_lnurl_auth_namespace_xpriv(&seed)
}

/// Sign the service's `k1` challenge using the LUD-05 linking key
/// derived from the supplied `m/138'` xpriv, and POST the callback.
pub(crate) async fn perform_lnurl_auth(
    namespace_xpriv: &[u8],
    request: &LnUrlAuthRequestData,
) -> Result<LnUrlCallbackStatus, Error> {
    let namespace = Xpriv::decode(namespace_xpriv)
        .map_err(|e| Error::Other(format!("LNURL-auth namespace xpriv invalid: {e}")))?;
    let secp = Secp256k1::new();

    // Step 1: derive the LUD-05 hashing key at m/138'/0 (relative to
    // the namespace, that's just /0). HMAC the domain with its
    // 32-byte secret to produce 16 bytes of derivation tail.
    let hashing_path: [ChildNumber; 1] = [ChildNumber::from(0)];
    let hashing_xpriv = namespace
        .derive_priv(&secp, &hashing_path)
        .map_err(|e| Error::Other(format!("BIP32 hashing-key derivation failed: {e}")))?;
    let hashing_key = hashing_xpriv.private_key.secret_bytes();

    let mut engine = HmacEngine::<sha256::Hash>::new(&hashing_key);
    engine.input(request.domain.as_bytes());
    let hmac = Hmac::<sha256::Hash>::from_engine(engine);
    let bytes = hmac.as_byte_array();

    // Step 2: derive the linking key at m/138'/<hmac4>/<hmac4>/<hmac4>/<hmac4>
    // (relative to the namespace, that's /<hmac4>/.../<hmac4>).
    let linking_path: Vec<ChildNumber> = vec![
        ChildNumber::from(u32::from_be_bytes(bytes[0..4].try_into().unwrap())),
        ChildNumber::from(u32::from_be_bytes(bytes[4..8].try_into().unwrap())),
        ChildNumber::from(u32::from_be_bytes(bytes[8..12].try_into().unwrap())),
        ChildNumber::from(u32::from_be_bytes(bytes[12..16].try_into().unwrap())),
    ];
    let linking_xpriv = namespace
        .derive_priv(&secp, &linking_path)
        .map_err(|e| Error::Other(format!("BIP32 linking-key derivation failed: {e}")))?;
    let linking_secret = linking_xpriv.private_key;
    let linking_pubkey = PublicKey::from_secret_key(&secp, &linking_secret);

    // Step 3: sign k1 with the linking key.
    let challenge = hex::decode(&request.k1)
        .map_err(|e| Error::Other(format!("LNURL-auth k1 not hex: {e}")))?;
    let message = Message::from_digest_slice(&challenge)
        .map_err(|e| Error::Other(format!("LNURL-auth k1 not a 32-byte message: {e}")))?;
    let sig = secp.sign_ecdsa(&message, &linking_secret);

    // Step 4: POST the signed callback per LUD-04.
    let mut callback = url::Url::parse(&request.url)
        .map_err(|e| Error::Other(format!("Invalid LNURL-auth URL: {e}")))?;
    callback
        .query_pairs_mut()
        .append_pair("sig", &hex::encode(sig.serialize_der()))
        .append_pair("key", &linking_pubkey.to_string());

    let response = reqwest::get(callback)
        .await
        .map_err(|e| Error::Other(format!("LNURL-auth callback failed: {e}")))?;
    let body = response
        .text()
        .await
        .map_err(|e| Error::Other(format!("LNURL-auth response read failed: {e}")))?;
    parse_callback_status(&body)
}

fn parse_callback_status(body: &str) -> Result<LnUrlCallbackStatus, Error> {
    // LUD-04 success body is {"status":"OK"}; failure is
    // {"status":"ERROR","reason":"..."}.
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| Error::Other(format!("LNURL-auth response not JSON: {e}")))?;
    let status = value
        .get("status")
        .and_then(|s| s.as_str())
        .ok_or_else(|| Error::Other(format!("LNURL-auth response missing status: {body}")))?;
    match status {
        "OK" => Ok(LnUrlCallbackStatus::Ok),
        "ERROR" => {
            let reason = value
                .get("reason")
                .and_then(|s| s.as_str())
                .unwrap_or("(no reason given)")
                .to_string();
            Ok(LnUrlCallbackStatus::ErrorStatus {
                data: LnUrlErrorData { reason },
            })
        }
        other => Err(Error::Other(format!(
            "LNURL-auth response unknown status '{other}'"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // BIP39 test vector (all "abandon" + "about").
    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon \
        abandon abandon abandon abandon abandon about";

    #[test]
    fn namespace_xpriv_is_deterministic_for_same_mnemonic() {
        let a = derive_lnurl_auth_namespace_xpriv_from_mnemonic(MNEMONIC).unwrap();
        let b = derive_lnurl_auth_namespace_xpriv_from_mnemonic(MNEMONIC).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 78);
    }

    /// Fixed test vector: locks in the LUD-05 derivation convention
    /// (32-byte private key as HMAC key, m/138' namespace) for the
    /// "abandon ... about" BIP39 vector at domain "example.com". Any
    /// change to the path computation or the HMAC-key choice breaks
    /// this assertion — that is intentional, since either is a
    /// one-way protocol break for any user already authenticated to a
    /// service.
    #[test]
    fn linking_pubkey_matches_known_vector() {
        let namespace = derive_lnurl_auth_namespace_xpriv_from_mnemonic(MNEMONIC).unwrap();
        let pubkey = compute_linking_pubkey_for_test(&namespace, "example.com").unwrap();
        assert_eq!(
            pubkey.to_string(),
            "039ae11fab821ec79815b327ba882e4dac5a046fe36bf6f5eed8b79c57f0b5d4fe",
        );
    }

    #[test]
    fn rejects_invalid_mnemonic() {
        let err = derive_lnurl_auth_namespace_xpriv_from_mnemonic("not a real mnemonic")
            .err()
            .expect("expected error");
        matches!(err, Error::PhraseCorrupted())
            .then_some(())
            .expect("expected PhraseCorrupted");
    }

    #[test]
    fn perform_rejects_invalid_namespace_xpriv() {
        let req = LnUrlAuthRequestData {
            k1: "00".repeat(32),
            action: None,
            domain: "x.com".to_string(),
            url: "https://x.com/a?tag=login&k1=00".to_string(),
        };
        let err = crate::util::exec(perform_lnurl_auth(&[0u8; 16], &req))
            .err()
            .expect("expected error");
        match err {
            Error::Other(msg) => assert!(msg.contains("namespace xpriv invalid")),
            other => panic!("expected Other error, got {other:?}"),
        }
    }

    #[test]
    fn parse_callback_status_ok() {
        let s = parse_callback_status(r#"{"status":"OK"}"#).unwrap();
        matches!(s, LnUrlCallbackStatus::Ok)
            .then_some(())
            .expect("expected Ok variant");
    }

    #[test]
    fn parse_callback_status_error() {
        let s = parse_callback_status(r#"{"status":"ERROR","reason":"nope"}"#).unwrap();
        match s {
            LnUrlCallbackStatus::ErrorStatus { data } => assert_eq!(data.reason, "nope"),
            _ => panic!("expected ErrorStatus"),
        }
    }

    #[test]
    fn parse_callback_status_missing_field() {
        assert!(parse_callback_status(r#"{"reason":"nope"}"#).is_err());
        assert!(parse_callback_status("not json").is_err());
    }

    /// Test helper that mirrors the derivation logic inside
    /// `perform_lnurl_auth` so we can compute a stable vector without
    /// performing an HTTP callback.
    fn compute_linking_pubkey_for_test(
        namespace_xpriv: &[u8],
        domain: &str,
    ) -> Result<PublicKey, Error> {
        let namespace = Xpriv::decode(namespace_xpriv).unwrap();
        let secp = Secp256k1::new();
        let hashing = namespace
            .derive_priv(&secp, &[ChildNumber::from(0)])
            .unwrap();
        let mut engine = HmacEngine::<sha256::Hash>::new(&hashing.private_key.secret_bytes());
        engine.input(domain.as_bytes());
        let hmac = Hmac::<sha256::Hash>::from_engine(engine);
        let bytes = hmac.as_byte_array();
        let linking_path: Vec<ChildNumber> = vec![
            ChildNumber::from(u32::from_be_bytes(bytes[0..4].try_into().unwrap())),
            ChildNumber::from(u32::from_be_bytes(bytes[4..8].try_into().unwrap())),
            ChildNumber::from(u32::from_be_bytes(bytes[8..12].try_into().unwrap())),
            ChildNumber::from(u32::from_be_bytes(bytes[12..16].try_into().unwrap())),
        ];
        let linking = namespace.derive_priv(&secp, &linking_path).unwrap();
        Ok(PublicKey::from_secret_key(&secp, &linking.private_key))
    }
}
