use crate::error::AppError;

const RESERVED_USERNAMES: &[&str] = &[
    "admin",
    "api",
    "auth",
    "giris",
    "sign-in",
    "sign-up",
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
    "games",
    "discover",
    "genres",
    "favorites",
    "backlog",
    "library",
    "wishlist",
    "lists",
    "reviews",
    "members",
    "companies",
    "series",
    "platforms",
    "search",
    "terms",
    "privacy",
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

/// Bir addan (görünen ad, e-posta yerel kısmı) kullanıcı adı gövdesi üretir: Türkçe ve genel Latin
/// harfleri ASCII karşılığına çevrilir ("Gökhan Güler" → "gokhanguler"), boşluk ve noktalama
/// atılır, en çok 16 karakter kalır. Çevrilemeyen harfler (örn. Kiril) da atılır.
pub fn username_base(source: &str) -> String {
    let mut out = String::new();
    for c in source.chars() {
        match c {
            'a'..='z' | '0'..='9' => out.push(c),
            'A'..='Z' => out.push(c.to_ascii_lowercase()),
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'À' | 'Á' | 'Â' | 'Ã' | 'Ä'
            | 'Å' | 'Ā' | 'Ă' | 'Ą' => out.push('a'),
            'ç' | 'ć' | 'č' | 'Ç' | 'Ć' | 'Č' => out.push('c'),
            'ď' | 'đ' | 'ð' | 'Ď' | 'Đ' | 'Ð' => out.push('d'),
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' | 'ě' | 'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ė'
            | 'Ę' | 'Ě' => out.push('e'),
            'ğ' | 'ģ' | 'Ğ' | 'Ģ' => out.push('g'),
            'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' | 'ı' | 'Ì' | 'Í' | 'Î' | 'Ï' | 'Ī' | 'Į' | 'İ' => {
                out.push('i')
            }
            'ķ' | 'Ķ' => out.push('k'),
            'ł' | 'ļ' | 'ľ' | 'Ł' | 'Ļ' | 'Ľ' => out.push('l'),
            'ñ' | 'ń' | 'ň' | 'ņ' | 'Ñ' | 'Ń' | 'Ň' | 'Ņ' => out.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' | 'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø'
            | 'Ō' | 'Ő' => out.push('o'),
            'ř' | 'Ř' => out.push('r'),
            'ś' | 'š' | 'ş' | 'ș' | 'Ś' | 'Š' | 'Ş' | 'Ș' => out.push('s'),
            'ť' | 'ţ' | 'ț' | 'Ť' | 'Ţ' | 'Ț' => out.push('t'),
            'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' | 'ų' | 'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ū' | 'Ů'
            | 'Ű' | 'Ų' => out.push('u'),
            'ý' | 'ÿ' | 'Ý' | 'Ÿ' => out.push('y'),
            'ź' | 'ż' | 'ž' | 'Ź' | 'Ż' | 'Ž' => out.push('z'),
            'ß' => out.push_str("ss"),
            'æ' | 'Æ' => out.push_str("ae"),
            'œ' | 'Œ' => out.push_str("oe"),
            'þ' | 'Þ' => out.push_str("th"),
            _ => {}
        }
    }
    out.chars().take(16).collect()
}

#[cfg(test)]
mod username_base_tests {
    use super::username_base;

    #[test]
    fn transliterates_turkish_and_latin() {
        assert_eq!(username_base("Gökhan Güler"), "gokhanguler");
        assert_eq!(username_base("İŞÇİ ığ"), "isciig");
        assert_eq!(
            username_base("Zoë Müller-Straße"),
            "zoemullerstrasse"[..16].to_string()
        );
        assert_eq!(username_base("john.doe_99"), "johndoe99");
        assert_eq!(username_base("日本"), "");
    }
}
