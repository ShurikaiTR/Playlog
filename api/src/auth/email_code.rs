//! E-posta kodlarının veritabanı tarafı: bekleme süresi, gönderme ve doğrulayıp tüketme.
//! Kodun üretimi ve özeti `code` modülündedir.

use sqlx::{FromRow, PgConnection};
use uuid::Uuid;

use crate::{auth::code, error::AppError, mail, state::AppState};

/// Kodun ne için gönderildiği; veritabanındaki `email_code_purpose` enum'unun karşılığı.
/// Kod özetine de karışır, böylece bir amaçla alınan kod başka amaçla kullanılamaz.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Purpose {
    Signup,
    PasswordReset,
    LoginCode,
    EmailChange,
}

impl Purpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Signup => "signup",
            Self::PasswordReset => "password_reset",
            Self::LoginCode => "login_code",
            Self::EmailChange => "email_change",
        }
    }
}

/// Aynı e-postaya 60 saniyede birden fazla kod gönderilmesini engeller.
pub async fn ensure_cooldown(
    state: &AppState,
    purpose: Purpose,
    email: &str,
) -> Result<(), AppError> {
    let recent: bool = sqlx::query_scalar(
        "SELECT EXISTS (
             SELECT 1 FROM email_codes
              WHERE email = $1 AND purpose = $2::email_code_purpose
                AND created_at > now() - make_interval(secs => $3))",
    )
    .bind(email)
    .bind(purpose.as_str())
    .bind(code::RESEND_COOLDOWN_SECONDS as f64)
    .fetch_one(&state.db)
    .await?;
    if recent {
        return Err(AppError::TooManyRequests("code_cooldown"));
    }
    Ok(())
}

/// Önceki kullanılmamış kodları iptal eder, yeni kodu kaydeder ve e-postayla yollar.
pub async fn issue(
    state: &AppState,
    purpose: Purpose,
    email: &str,
    pending: Option<serde_json::Value>,
) -> Result<(), AppError> {
    let plain = code::generate()?;
    let code_hash = code::hash(&state.code_secret, purpose.as_str(), email, &plain)?;

    let mut tx = state.db.begin().await?;
    sqlx::query(
        "UPDATE email_codes SET consumed_at = now()
          WHERE email = $1 AND purpose = $2::email_code_purpose AND consumed_at IS NULL",
    )
    .bind(email)
    .bind(purpose.as_str())
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO email_codes (purpose, email, code_hash, pending, expires_at)
         VALUES ($1::email_code_purpose, $2, $3, $4, now() + make_interval(mins => $5))",
    )
    .bind(purpose.as_str())
    .bind(email)
    .bind(code_hash)
    .bind(pending)
    .bind(code::TTL_MINUTES as i32)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let copy = match purpose {
        Purpose::LoginCode => &mail::LOGIN_CODE_COPY,
        Purpose::Signup | Purpose::EmailChange => &mail::SIGNUP_CODE_COPY,
        Purpose::PasswordReset => &mail::RESET_CODE_COPY,
    };
    let (subject, text, html) = mail::code_email(copy, &plain, code::TTL_MINUTES);
    state
        .mailer
        .send(email, &subject, &text, Some(&html))
        .await?;
    Ok(())
}

/// Etkin kodu kilitleyip doğrular.
/// - Kod doğruysa: kodu tüketir, `Some(pending)` döner.
/// - Yanlışsa ya da kod yoksa: deneme sayacını artırır, `None` döner (çağıran commit eder).
pub async fn consume(
    state: &AppState,
    tx: &mut PgConnection,
    purpose: Purpose,
    email: &str,
    submitted: &str,
) -> Result<Option<Option<serde_json::Value>>, AppError> {
    #[derive(FromRow)]
    struct Row {
        id: Uuid,
        code_hash: Vec<u8>,
        pending: Option<serde_json::Value>,
        attempts: i16,
    }

    let row: Option<Row> = sqlx::query_as(
        "SELECT id, code_hash, pending, attempts
           FROM email_codes
          WHERE email = $1 AND purpose = $2::email_code_purpose
            AND consumed_at IS NULL AND expires_at > now()
          ORDER BY created_at DESC
          LIMIT 1
          FOR UPDATE",
    )
    .bind(email)
    .bind(purpose.as_str())
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else { return Ok(None) };

    if row.attempts >= code::MAX_ATTEMPTS {
        return Ok(None);
    }

    let candidate = code::hash(&state.code_secret, purpose.as_str(), email, submitted)?;
    if !code::matches(&row.code_hash, &candidate) {
        sqlx::query("UPDATE email_codes SET attempts = attempts + 1 WHERE id = $1")
            .bind(row.id)
            .execute(&mut *tx)
            .await?;
        return Ok(None);
    }

    sqlx::query("UPDATE email_codes SET consumed_at = now() WHERE id = $1")
        .bind(row.id)
        .execute(&mut *tx)
        .await?;
    Ok(Some(row.pending))
}
