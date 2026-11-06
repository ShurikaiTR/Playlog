use crate::error::AppError;

const RESERVED_USERNAMES: &[&str] = &[
    "admin",
    "api",
    "auth",
    "giris",
    "kayit-ol",
    "kayit_ol",
    "kesfet",
    "oyun",
    "oyunlar",
    "liste",
    "favoriler",
    "planlananlar",
    "ayarlar",
    "profil",
    "hesap",
    "destek",
    "yardim",
    "playthru",
    "root",
    "support",
    "static",
    "assets",
    "login",
    "register",
    "settings",
];

pub fn username(raw: &str) -> Result<String, AppError> {
    let name = raw.trim().to_ascii_lowercase();
    let valid_chars = name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if !(3..=20).contains(&name.len()) || !valid_chars {
        return Err(AppError::Validation("invalid_username"));
    }
    if RESERVED_USERNAMES.contains(&name.as_str()) {
        return Err(AppError::Validation("reserved_username"));
    }
    Ok(name)
}

/// Yalnızca açık hataları yakalar; adresin gerçekten var olduğunu kanıtlayan tek
/// şey e-postaya gönderilen koddur.
pub fn email(raw: &str) -> Result<String, AppError> {
    let email = raw.trim().to_lowercase();
    let ok = email.len() <= 254
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && local.len() <= 64
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !domain.contains('@')
        })
        && !email.chars().any(|c| c.is_whitespace() || c.is_control());
    if ok {
        Ok(email)
    } else {
        Err(AppError::Validation("invalid_email"))
    }
}

/// Uzunluk karakter sayısıyla ölçülür; boşluk dahil her karaktere izin verilir.
pub fn password(raw: &str) -> Result<(), AppError> {
    let len = raw.chars().count();
    if (8..=128).contains(&len) {
        Ok(())
    } else {
        Err(AppError::Validation("invalid_password"))
    }
}

pub fn code(raw: &str) -> Result<&str, AppError> {
    if raw.len() == 6 && raw.bytes().all(|b| b.is_ascii_digit()) {
        Ok(raw)
    } else {
        Err(AppError::Validation("invalid_code"))
    }
}

/// Görünen ad isteğe bağlıdır: boşsa `None`, en fazla 40 karakter, kontrol karakteri yok.
pub fn display_name(raw: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(raw) = raw.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    if raw.chars().count() > 40 || raw.chars().any(char::is_control) {
        return Err(AppError::Validation("invalid_display_name"));
    }
    Ok(Some(raw.to_owned()))
}
