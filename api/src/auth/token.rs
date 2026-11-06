//! Oturum belirteci: `<oturum-id>.<gizli parça>`.
//! Gizli parça 256 bit rastgeledir; veritabanında yalnızca SHA-256 özeti durur
//! (OWASP Session Management Cheat Sheet).

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

pub struct NewToken {
    pub id: Uuid,
    /// İstemciye verilen tam belirteç.
    pub token: String,
    /// Veritabanında saklanacak özet.
    pub verifier_hash: Vec<u8>,
}

pub fn generate() -> anyhow::Result<NewToken> {
    let mut secret = [0u8; 32];
    getrandom::fill(&mut secret)?;
    let id = Uuid::now_v7();
    let secret = URL_SAFE_NO_PAD.encode(secret);
    Ok(NewToken {
        id,
        token: format!("{}.{secret}", id.simple()),
        verifier_hash: hash_secret(&secret),
    })
}

pub struct Parsed<'a> {
    pub id: Uuid,
    pub secret: &'a str,
}

pub fn parse(token: &str) -> Option<Parsed<'_>> {
    let (id, secret) = token.split_once('.')?;
    if secret.len() != 43 {
        return None;
    }
    Some(Parsed {
        id: Uuid::try_parse(id).ok()?,
        secret,
    })
}

pub fn hash_secret(secret: &str) -> Vec<u8> {
    Sha256::digest(secret.as_bytes()).to_vec()
}

/// Sabit sürede karşılaştırır.
pub fn verifier_matches(secret: &str, stored_hash: &[u8]) -> bool {
    hash_secret(secret).ct_eq(stored_hash).into()
}
