//! Adreslerde kullanılan kısa adlar: küçük harf, rakam ve tek tire (`the-witcher-3-wild-hunt`).

/// Veritabanındaki `slug` alanının üst sınırı.
pub const MAX_LEN: usize = 120;

/// Bir başlıktan slug üretir. Sonuç boş olabilir (yalnızca simgelerden oluşan ad);
/// çağıran bunu hata sayar.
pub fn slugify(raw: &str) -> String {
    let mut slug = String::with_capacity(raw.len());
    for c in raw.chars() {
        let mapped: &str = match c {
            'a'..='z' | '0'..='9' => {
                slug.push(c);
                continue;
            }
            'A'..='Z' => {
                slug.push(c.to_ascii_lowercase());
                continue;
            }
            'ç' | 'Ç' | 'ć' | 'č' => "c",
            'ğ' | 'Ğ' => "g",
            'ı' | 'İ' | 'í' | 'ì' | 'î' | 'ï' => "i",
            'ö' | 'Ö' | 'ó' | 'ò' | 'ô' | 'õ' | 'ø' => "o",
            'ş' | 'Ş' | 'š' | 'ś' => "s",
            'ü' | 'Ü' | 'ú' | 'ù' | 'û' => "u",
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => "a",
            'é' | 'è' | 'ê' | 'ë' => "e",
            'ñ' => "n",
            'ß' => "ss",
            'æ' => "ae",
            'œ' => "oe",
            '&' => "-and-",
            // Kesme işareti kelimeyi bölmez: "Assassin's" -> "assassins".
            '\'' | '’' => "",
            _ => "-",
        };
        slug.push_str(mapped);
    }

    // Art arda gelen tireleri teke indirir, baştaki ve sondakileri atar.
    let mut clean = String::with_capacity(slug.len());
    for part in slug.split('-').filter(|part| !part.is_empty()) {
        if !clean.is_empty() {
            clean.push('-');
        }
        clean.push_str(part);
    }
    if clean.len() > MAX_LEN {
        clean.truncate(MAX_LEN);
        while clean.ends_with('-') {
            clean.pop();
        }
    }
    clean
}

/// Dışarıdan verilen slug'ın biçimi doğru mu (veritabanındaki CHECK ile aynı kural).
pub fn is_valid(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= MAX_LEN
        && slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_handles_punctuation_and_case() {
        assert_eq!(
            slugify("The Witcher 3: Wild Hunt"),
            "the-witcher-3-wild-hunt"
        );
        assert_eq!(
            slugify("Call of Duty®: Black Ops 6"),
            "call-of-duty-black-ops-6"
        );
        assert_eq!(slugify("  --Half-Life 2--  "), "half-life-2");
        assert_eq!(slugify("Assassin's Creed"), "assassins-creed");
        assert_eq!(slugify("Ratchet & Clank"), "ratchet-and-clank");
    }

    #[test]
    fn slugify_transliterates_common_letters() {
        assert_eq!(slugify("Gölgeler Vadisi"), "golgeler-vadisi");
        assert_eq!(slugify("Kırık Kılıç Efsanesi"), "kirik-kilic-efsanesi");
        assert_eq!(slugify("Pokémon Légendes"), "pokemon-legendes");
        assert_eq!(slugify("NieR:Automata™"), "nier-automata");
    }

    #[test]
    fn slugify_can_come_out_empty() {
        assert_eq!(slugify("™®"), "");
        assert_eq!(slugify("原神"), "");
    }

    #[test]
    fn slugify_respects_the_length_limit() {
        let long = "a ".repeat(200);
        let slug = slugify(&long);
        assert!(slug.len() <= MAX_LEN);
        assert!(is_valid(&slug));
    }

    #[test]
    fn is_valid_matches_the_database_rule() {
        assert!(is_valid("half-life-2"));
        assert!(is_valid("2048"));
        assert!(!is_valid(""));
        assert!(!is_valid("Half-Life"));
        assert!(!is_valid("half--life"));
        assert!(!is_valid("-half-life"));
        assert!(!is_valid("half_life"));
    }
}
