//! Parola özetleme: Argon2id, OWASP Password Storage Cheat Sheet'in asgari
//! yapılandırması (19 MiB bellek, 2 tur, 1 paralellik). Salt kütüphane
//! tarafından rastgele üretilir ve PHC dizesinin içinde saklanır.

use anyhow::Context;
use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

fn hasher() -> anyhow::Result<Argon2<'static>> {
    let params = Params::new(19 * 1024, 2, 1, None).context("argon2 parametreleri geçersiz")?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Argon2 işlemcide ağırdır; HTTP işçisini bloklamamak için ayrı iş parçacığında çalışır.
pub async fn hash(password: String) -> anyhow::Result<String> {
    tokio::task::spawn_blocking(move || {
        let hash = hasher()?
            .hash_password(password.as_bytes())
            .context("parola özetlenemedi")?;
        Ok(hash.to_string())
    })
    .await?
}

/// Parolayı kayıtlı PHC özetine karşı doğrular. Kullanıcı yoksa çağıran, yanıt
/// süresi sızmasın diye `AppState::dummy_hash` ile çağırıp sonucu yok sayar.
pub async fn verify(password: String, stored: String) -> anyhow::Result<bool> {
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&stored).context("kayıtlı parola özeti okunamadı")?;
        Ok(hasher()?
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    })
    .await?
}

/// Hiçbir kullanıcıya ait olmayan, aynı parametrelerle üretilmiş özet (açılışta bir kez).
pub async fn dummy_hash() -> anyhow::Result<String> {
    let mut random = [0u8; 16];
    getrandom::fill(&mut random)?;
    hash(base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        random,
    ))
    .await
}
