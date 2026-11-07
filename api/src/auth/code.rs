//! 6 haneli e-posta kodları. Kod veritabanında HMAC-SHA256 ile saklanır
//! (sunucu sırrıyla anahtarlı); veritabanı sızsa bile 10^6 olasılık
//! çevrimdışı denenemez.

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

pub const TTL_MINUTES: i64 = 10;
pub const MAX_ATTEMPTS: i16 = 5;
pub const RESEND_COOLDOWN_SECONDS: i64 = 60;

/// Önyargısız 6 haneli kod (reddetme örneklemesi).
pub fn generate() -> anyhow::Result<String> {
    loop {
        let mut buf = [0u8; 4];
        getrandom::fill(&mut buf)?;
        let n = u32::from_le_bytes(buf);
        // 4_294_000_000, 1_000_000'un katıdır: mod alma yan etkisi yok.
        if n < 4_294_000_000 {
            return Ok(format!("{:06}", n % 1_000_000));
        }
    }
}

pub fn hash(secret: &[u8], purpose: &str, email: &str, code: &str) -> anyhow::Result<Vec<u8>> {
    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(secret)?;
    mac.update(purpose.as_bytes());
    mac.update(b"\0");
    mac.update(email.as_bytes());
    mac.update(b"\0");
    mac.update(code.as_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

pub fn matches(expected: &[u8], actual: &[u8]) -> bool {
    expected.ct_eq(actual).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_codes_are_six_digits() {
        for _ in 0..200 {
            let code = generate().unwrap();
            assert_eq!(code.len(), 6);
            assert!(code.bytes().all(|b| b.is_ascii_digit()));
        }
    }

    #[test]
    fn hash_is_bound_to_purpose_email_and_code() {
        let secret = b"test-secret";
        let base = hash(secret, "login_code", "a@b.co", "123456").unwrap();
        assert!(matches(
            &base,
            &hash(secret, "login_code", "a@b.co", "123456").unwrap()
        ));
        // A login code can't be replayed as a reset or signup code, for another address, or with another digit.
        assert!(!matches(
            &base,
            &hash(secret, "password_reset", "a@b.co", "123456").unwrap()
        ));
        assert!(!matches(
            &base,
            &hash(secret, "login_code", "c@d.co", "123456").unwrap()
        ));
        assert!(!matches(
            &base,
            &hash(secret, "login_code", "a@b.co", "123457").unwrap()
        ));
    }

    #[test]
    fn hash_depends_on_server_secret() {
        let a = hash(b"one", "login_code", "a@b.co", "123456").unwrap();
        let b = hash(b"two", "login_code", "a@b.co", "123456").unwrap();
        assert!(!matches(&a, &b));
    }
}
