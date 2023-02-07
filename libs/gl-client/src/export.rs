//! Utilities to work with export/backup files.
use anyhow::{anyhow, Context, Error};
use bytes::{Buf, Bytes, BytesMut};
use chacha20poly1305::{AeadInPlace, ChaCha20Poly1305, KeyInit};
use secp256k1::{ecdh::SharedSecret, PublicKey, Secp256k1, SecretKey};
use std::io::Read;

const VERSION: u8 = 0x01;
/// Version byte, node ID, nonce, tag
const HEADER_LEN: usize = 1 + 33 + 12 + 16;

pub fn decrypt(mut enc: BytesMut, privkey: &SecretKey) -> Result<Bytes, Error> {
    let mut r = enc.clone().reader();
    // Start by reading the header
    let mut version = [0u8; 1];
    r.read_exact(&mut version)?;

    if VERSION != version[0] {
        return Err(anyhow!(
            "Backup version {} is not supported by this client version {}",
            version[0],
            VERSION
        ));
    }

    let mut ephkey = [0u8; 33];
    r.read_exact(&mut ephkey)?;

    let mut nonce = [0u8; 12];
    r.read_exact(&mut nonce)?;

    let mut tag = [0u8; 16];
    r.read_exact(&mut tag)?;

    let secp = Secp256k1::default();
    let ephkey = PublicKey::from_slice(&ephkey).context("loading ephemeral key")?;
    let node_id = privkey.public_key(&secp);

    let shared_secret = SharedSecret::new(&ephkey, &privkey);
    enc.advance(HEADER_LEN);

    let cipher = ChaCha20Poly1305::new(&shared_secret.secret_bytes().into());

    cipher
        .decrypt_in_place_detached(&nonce.into(), &node_id.serialize(), &mut enc, &tag.into())
        .map_err(|e| anyhow!("Error decrypting: {}", e))?;

    Ok(enc.clone().into())
}
