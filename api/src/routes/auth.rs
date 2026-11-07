//! Kimlik doğrulama uçları. Tarayıcı bunlara doğrudan gitmez; SvelteKit sunucusu çağırır.

use std::time::Duration;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgConnection};
use uuid::Uuid;

use crate::{
    auth::{code, extractor::CurrentUser, password, role::Role, token, validate},
    error::AppError,
    mail,
    state::AppState,
};

const SIGNUP: &str = "signup";
pub(super) const TEN_MINUTES: Duration = Duration::from_secs(600);
const FIFTEEN_MINUTES: Duration = Duration::from_secs(900);
const PASSWORD_RESET: &str = "password_reset";
const LOGIN_CODE: &str = "login_code";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/signup/start", post(signup_start))
        .route("/signup/verify", post(signup_verify))
        .route("/login", post(login))
        .route("/login-code/start", post(login_code_start))
        .route("/login-code/verify", post(login_code_verify))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/password-reset/start", post(password_reset_start))
        .route("/password-reset/confirm", post(password_reset_confirm))
}

// ---------- İstek / yanıt tipleri ----------

#[derive(Deserialize)]
struct SignupStart {
    /// Turnstile yanıt belirteci (cf-turnstile-response).
    #[serde(default)]
    turnstile_token: Option<String>,
    email: String,
    username: String,
    password: String,
    display_name: Option<String>,
    #[serde(default)]
    marketing_opt_in: bool,
}

#[derive(Deserialize)]
struct VerifyCode {
    email: String,
    code: String,
}

#[derive(Deserialize)]
struct Login {
    /// Turnstile yanıt belirteci (cf-turnstile-response).
    #[serde(default)]
    turnstile_token: Option<String>,
    /// E-posta ya da kullanıcı adı.
    identifier: String,
    password: String,
}

#[derive(Deserialize)]
struct PasswordResetStart {
    /// Turnstile yanıt belirteci (cf-turnstile-response).
    #[serde(default)]
    turnstile_token: Option<String>,
    email: String,
}

#[derive(Deserialize)]
struct PasswordResetConfirm {
    email: String,
    code: String,
    new_password: String,
}

#[derive(Serialize)]
struct CodeSent {
    status: &'static str,
    expires_in: i64,
}

impl CodeSent {
    fn new() -> Json<Self> {
        Json(Self {
            status: "code_sent",
            expires_in: code::TTL_MINUTES * 60,
        })
    }
}

#[derive(Serialize)]
pub(super) struct UserView {
    pub(super) id: Uuid,
    pub(super) username: String,
    pub(super) display_name: Option<String>,
    pub(super) email: String,
    pub(super) role: Role,
}

#[derive(Serialize)]
pub(super) struct SessionResponse {
    /// SvelteKit bunu `__Host-` önekli, HttpOnly çereze yazar.
    pub(super) token: String,
    pub(super) user: UserView,
}

// ---------- Kayıt ----------

async fn signup_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SignupStart>,
) -> Result<(StatusCode, Json<CodeSent>), AppError> {
    rate_limit(
        &state,
        &format!("signup:ip:{}", client_ip(&headers)),
        10,
        TEN_MINUTES,
    )?;
    verify_captcha(&state, &headers, body.turnstile_token.as_deref()).await?;
    let email = validate::email(&body.email)?;
    let username = validate::username(&body.username)?;
    let display_name = validate::display_name(body.display_name.as_deref())?;
    validate::password(&body.password)?;

    let username_taken: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE username = $1)")
            .bind(&username)
            .fetch_one(&state.db)
            .await?;
    if username_taken {
        return Err(AppError::Conflict("username_taken"));
    }

    ensure_cooldown(&state, SIGNUP, &email).await?;

    let email_taken: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)")
            .bind(&email)
            .fetch_one(&state.db)
            .await?;
    if email_taken {
        // Hesap sayımını sızdırmamak için aynı yanıt dönülür; sahibine bilgi e-postası gider.
        state
            .mailer
            .send(
                &email,
                "Playthru: bu e-posta zaten kayıtlı",
                "Bu e-posta adresiyle zaten bir Playthru hesabı var. Giriş yapabilir ya da \
                 şifreni sıfırlayabilirsin. Bu isteği sen yapmadıysan bu e-postayı yok sayabilirsin.",
                None,
            )
            .await?;
        return Ok((StatusCode::ACCEPTED, CodeSent::new()));
    }

    let password_hash = password::hash(body.password).await?;
    let pending = serde_json::json!({
        "username": username,
        "display_name": display_name,
        "marketing_opt_in": body.marketing_opt_in,
        "password_hash": password_hash,
    });
    issue_code(&state, SIGNUP, &email, Some(pending)).await?;
    Ok((StatusCode::ACCEPTED, CodeSent::new()))
}

async fn signup_verify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<VerifyCode>,
) -> Result<Json<SessionResponse>, AppError> {
    rate_limit(
        &state,
        &format!("verify:ip:{}", client_ip(&headers)),
        30,
        TEN_MINUTES,
    )?;
    let email = validate::email(&body.email)?;
    let code_input = validate::code(&body.code)?;

    let mut tx = state.db.begin().await?;
    let Some(pending) = consume_code(&state, &mut tx, SIGNUP, &email, code_input).await? else {
        // Yanlış kod: sayaç artırıldı, değişiklik kalıcı olmalı.
        tx.commit().await?;
        return Err(AppError::Unauthorized("invalid_code"));
    };

    let pending: serde_json::Value = pending.ok_or(AppError::Unauthorized("invalid_code"))?;
    let username = pending["username"].as_str().ok_or_else(corrupt)?.to_owned();
    let password_hash = pending["password_hash"].as_str().ok_or_else(corrupt)?;
    let display_name = pending["display_name"].as_str().map(str::to_owned);
    let marketing_opt_in = pending["marketing_opt_in"].as_bool().unwrap_or(false);

    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (username, email, email_verified_at, display_name, marketing_opt_in_at)
         VALUES ($1, $2, now(), $3, CASE WHEN $4 THEN now() END)
         RETURNING id",
    )
    .bind(&username)
    .bind(&email)
    .bind(&display_name)
    .bind(marketing_opt_in)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_unique_violation)?;

    sqlx::query("INSERT INTO credentials (user_id, password_hash) VALUES ($1, $2)")
        .bind(user_id)
        .bind(password_hash)
        .execute(&mut *tx)
        .await?;

    let token = create_session(&mut tx, user_id, &headers).await?;
    tx.commit().await?;

    Ok(Json(SessionResponse {
        token,
        user: UserView {
            id: user_id,
            username,
            display_name,
            email,
            // Yeni hesap her zaman sütunun varsayılanı olan 'user' rolüyle açılır.
            role: Role::User,
        },
    }))
}

// ---------- Giriş / çıkış ----------

#[derive(FromRow)]
struct LoginRow {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    email: String,
    password_hash: Option<String>,
    role: Role,
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Login>,
) -> Result<Json<SessionResponse>, AppError> {
    let identifier = body.identifier.trim().to_lowercase();
    // Hem IP hem hesap başına: tek IP'den çok hesap ve çok IP'den tek hesap denemeleri sınırlanır.
    let ip_key = format!("login:ip:{}", client_ip(&headers));
    let id_key = format!("login:id:{identifier}");
    rate_limit(&state, &ip_key, 20, FIFTEEN_MINUTES)?;
    rate_limit(&state, &id_key, 8, FIFTEEN_MINUTES)?;
    verify_captcha(&state, &headers, body.turnstile_token.as_deref()).await?;
    // Sabit sorgular: dinamik SQL kullanılmaz, her ikisi de benzersiz indeksten çalışır.
    let query = if identifier.contains('@') {
        "SELECT u.id, u.username, u.display_name, u.email, u.role, c.password_hash
           FROM users u LEFT JOIN credentials c ON c.user_id = u.id
          WHERE u.email = $1"
    } else {
        "SELECT u.id, u.username, u.display_name, u.email, u.role, c.password_hash
           FROM users u LEFT JOIN credentials c ON c.user_id = u.id
          WHERE u.username = $1"
    };

    let row: Option<LoginRow> = sqlx::query_as(query)
        .bind(&identifier)
        .fetch_optional(&state.db)
        .await?;

    // Kullanıcı yoksa ya da parolası yoksa (yalnızca Google/Discord) da Argon2 çalışır.
    let stored = row
        .as_ref()
        .and_then(|r| r.password_hash.clone())
        .unwrap_or_else(|| state.dummy_hash.to_string());
    let password_ok = password::verify(body.password, stored).await?;

    let Some(row) = row.filter(|r| password_ok && r.password_hash.is_some()) else {
        return Err(AppError::Unauthorized("invalid_credentials"));
    };

    state.limiter.reset(&id_key);
    let mut conn = state.db.acquire().await?;
    let token = create_session(&mut conn, row.id, &headers).await?;
    Ok(Json(SessionResponse {
        token,
        user: UserView {
            id: row.id,
            username: row.username,
            display_name: row.display_name,
            email: row.email,
            role: row.role,
        },
    }))
}

// ---------- E-posta koduyla giriş ----------

#[derive(Deserialize)]
struct LoginCodeStart {
    /// Turnstile yanıt belirteci (cf-turnstile-response).
    #[serde(default)]
    turnstile_token: Option<String>,
    email: String,
}

/// Giriş kodunu yollar. Hesap olsun ya da olmasın yanıt (ve hız sınırı davranışı) aynıdır;
/// sınırlar veritabanına değil bellekteki sayaca bakar, böylece hesap sayımı sızmaz.
async fn login_code_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LoginCodeStart>,
) -> Result<(StatusCode, Json<CodeSent>), AppError> {
    rate_limit(
        &state,
        &format!("logincode:ip:{}", client_ip(&headers)),
        10,
        TEN_MINUTES,
    )?;
    verify_captcha(&state, &headers, body.turnstile_token.as_deref()).await?;
    let email = validate::email(&body.email)?;
    // Aynı adrese dakikada bir kod: istemcinin geri sayımıyla aynı süre.
    state
        .limiter
        .hit(
            &format!("logincode:email:{email}"),
            1,
            Duration::from_secs(code::RESEND_COOLDOWN_SECONDS as u64),
        )
        .map_err(|_| AppError::TooManyRequests("code_cooldown"))?;

    let eligible: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM users WHERE email = $1 AND email_verified_at IS NOT NULL)",
    )
    .bind(&email)
    .fetch_one(&state.db)
    .await?;
    if eligible {
        issue_code(&state, LOGIN_CODE, &email, None).await?;
    }
    Ok((StatusCode::ACCEPTED, CodeSent::new()))
}

#[derive(FromRow)]
struct CodeLoginRow {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    email: String,
    role: Role,
}

/// Kodu doğrular (en fazla `MAX_ATTEMPTS` deneme, tek kullanımlık) ve parolalı girişle
/// aynı biçimde oturum açar.
async fn login_code_verify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<VerifyCode>,
) -> Result<Json<SessionResponse>, AppError> {
    rate_limit(
        &state,
        &format!("verify:ip:{}", client_ip(&headers)),
        30,
        TEN_MINUTES,
    )?;
    let email = validate::email(&body.email)?;
    let code_input = validate::code(&body.code)?;
    rate_limit(
        &state,
        &format!("logincode:verify:{email}"),
        10,
        FIFTEEN_MINUTES,
    )?;

    let mut tx = state.db.begin().await?;
    if consume_code(&state, &mut tx, LOGIN_CODE, &email, code_input)
        .await?
        .is_none()
    {
        // Yanlış kod: sayaç artırıldı, değişiklik kalıcı olmalı.
        tx.commit().await?;
        return Err(AppError::Unauthorized("invalid_code"));
    }

    let row: CodeLoginRow = sqlx::query_as(
        "SELECT id, username, display_name, email, role
           FROM users WHERE email = $1 AND email_verified_at IS NOT NULL",
    )
    .bind(&email)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::Unauthorized("invalid_code"))?;

    let token = create_session(&mut tx, row.id, &headers).await?;
    tx.commit().await?;
    state.limiter.reset(&format!("logincode:verify:{email}"));

    Ok(Json(SessionResponse {
        token,
        user: UserView {
            id: row.id,
            username: row.username,
            display_name: row.display_name,
            email: row.email,
            role: row.role,
        },
    }))
}

async fn logout(State(state): State<AppState>, user: CurrentUser) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(user.session_id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn me(user: CurrentUser) -> Json<UserView> {
    Json(UserView {
        id: user.user_id,
        username: user.username,
        display_name: user.display_name,
        email: user.email,
        role: user.role,
    })
}

// ---------- Şifre sıfırlama ----------

async fn password_reset_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PasswordResetStart>,
) -> Result<(StatusCode, Json<CodeSent>), AppError> {
    rate_limit(
        &state,
        &format!("reset:ip:{}", client_ip(&headers)),
        10,
        TEN_MINUTES,
    )?;
    verify_captcha(&state, &headers, body.turnstile_token.as_deref()).await?;
    let email = validate::email(&body.email)?;
    ensure_cooldown(&state, PASSWORD_RESET, &email).await?;

    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)")
        .bind(&email)
        .fetch_one(&state.db)
        .await?;
    // Hesap yoksa da aynı yanıt dönülür (hesap sayımı sızmasın).
    if exists {
        issue_code(&state, PASSWORD_RESET, &email, None).await?;
    }
    Ok((StatusCode::ACCEPTED, CodeSent::new()))
}

async fn password_reset_confirm(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PasswordResetConfirm>,
) -> Result<StatusCode, AppError> {
    rate_limit(
        &state,
        &format!("verify:ip:{}", client_ip(&headers)),
        30,
        TEN_MINUTES,
    )?;
    let email = validate::email(&body.email)?;
    let code_input = validate::code(&body.code)?;
    validate::password(&body.new_password)?;

    let new_hash = password::hash(body.new_password).await?;

    let mut tx = state.db.begin().await?;
    if consume_code(&state, &mut tx, PASSWORD_RESET, &email, code_input)
        .await?
        .is_none()
    {
        tx.commit().await?;
        return Err(AppError::Unauthorized("invalid_code"));
    }

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::Unauthorized("invalid_code"))?;

    sqlx::query(
        "INSERT INTO credentials (user_id, password_hash) VALUES ($1, $2)
         ON CONFLICT (user_id)
         DO UPDATE SET password_hash = EXCLUDED.password_hash, updated_at = now()",
    )
    .bind(user_id)
    .bind(new_hash)
    .execute(&mut *tx)
    .await?;

    // Parola değişince tüm oturumlar kapanır.
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------- Yardımcılar ----------

/// SvelteKit'in ilettiği gerçek istemci IP'si (API yalnızca iç ağdadır, başlığa güvenilir).
/// Başlık yoksa ortak bir "unknown" kovasına düşer; bu daha sıkı sınırlanır, güvenli taraf.
pub(super) fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-client-ip")
        .and_then(|v| v.to_str().ok())
        .filter(|ip| ip.parse::<std::net::IpAddr>().is_ok())
        .unwrap_or("unknown")
        .to_owned()
}

/// Turnstile açıksa belirteci doğrular; kapalıysa (yerel geliştirme) atlar.
async fn verify_captcha(
    state: &AppState,
    headers: &HeaderMap,
    token: Option<&str>,
) -> Result<(), AppError> {
    match &state.turnstile {
        Some(turnstile) => turnstile.verify(token, &client_ip(headers)).await,
        None => Ok(()),
    }
}

pub(super) fn rate_limit(
    state: &AppState,
    key: &str,
    max: u32,
    window: Duration,
) -> Result<(), AppError> {
    state
        .limiter
        .hit(key, max, window)
        .map_err(|_| AppError::TooManyRequests("rate_limited"))
}

fn corrupt() -> AppError {
    AppError::Internal(anyhow::anyhow!("bekleyen kayıt verisi bozuk"))
}

/// Kullanıcı adı/e-posta benzersizlik ihlalini (yarış durumu) 409'a çevirir.
pub(super) fn map_unique_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &err
        && db.is_unique_violation()
    {
        return match db.constraint() {
            Some("users_username_key") => AppError::Conflict("username_taken"),
            Some("users_email_key") => AppError::Conflict("email_taken"),
            _ => AppError::Internal(err.into()),
        };
    }
    AppError::Internal(err.into())
}

/// Aynı e-postaya 60 saniyede birden fazla kod gönderilmesini engeller.
async fn ensure_cooldown(
    state: &AppState,
    purpose: &'static str,
    email: &str,
) -> Result<(), AppError> {
    let recent: bool = sqlx::query_scalar(
        "SELECT EXISTS (
             SELECT 1 FROM email_codes
              WHERE email = $1 AND purpose = $2::email_code_purpose
                AND created_at > now() - make_interval(secs => $3))",
    )
    .bind(email)
    .bind(purpose)
    .bind(code::RESEND_COOLDOWN_SECONDS as f64)
    .fetch_one(&state.db)
    .await?;
    if recent {
        return Err(AppError::TooManyRequests("code_cooldown"));
    }
    Ok(())
}

/// Önceki kullanılmamış kodları iptal eder, yeni kodu kaydeder ve e-postayla yollar.
async fn issue_code(
    state: &AppState,
    purpose: &'static str,
    email: &str,
    pending: Option<serde_json::Value>,
) -> Result<(), AppError> {
    let plain = code::generate()?;
    let code_hash = code::hash(&state.code_secret, purpose, email, &plain)?;

    let mut tx = state.db.begin().await?;
    sqlx::query(
        "UPDATE email_codes SET consumed_at = now()
          WHERE email = $1 AND purpose = $2::email_code_purpose AND consumed_at IS NULL",
    )
    .bind(email)
    .bind(purpose)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO email_codes (purpose, email, code_hash, pending, expires_at)
         VALUES ($1::email_code_purpose, $2, $3, $4, now() + make_interval(mins => $5))",
    )
    .bind(purpose)
    .bind(email)
    .bind(code_hash)
    .bind(pending)
    .bind(code::TTL_MINUTES as i32)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let copy = match purpose {
        LOGIN_CODE => &mail::LOGIN_CODE_COPY,
        SIGNUP => &mail::SIGNUP_CODE_COPY,
        _ => &mail::RESET_CODE_COPY,
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
async fn consume_code(
    state: &AppState,
    tx: &mut PgConnection,
    purpose: &'static str,
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
    .bind(purpose)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else { return Ok(None) };

    if row.attempts >= code::MAX_ATTEMPTS {
        return Ok(None);
    }

    let candidate = code::hash(&state.code_secret, purpose, email, submitted)?;
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

/// Yeni oturum açar, istemciye verilecek belirteci döner.
/// IP ve tarayıcı bilgisi SvelteKit'in ilettiği başlıklardan gelir (API yalnızca iç ağdadır).
pub(super) async fn create_session(
    conn: &mut PgConnection,
    user_id: Uuid,
    headers: &HeaderMap,
) -> Result<String, AppError> {
    let new = token::generate()?;
    let header = |name: &str, max: usize| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.chars().take(max).collect::<String>())
    };

    sqlx::query(
        "INSERT INTO sessions (id, user_id, verifier_hash, user_agent, ip, expires_at)
         VALUES ($1, $2, $3, $4, $5::text::inet, now() + interval '30 days')",
    )
    .bind(new.id)
    .bind(user_id)
    .bind(new.verifier_hash)
    .bind(header("x-client-user-agent", 300))
    .bind(header("x-client-ip", 45).filter(|ip| ip.parse::<std::net::IpAddr>().is_ok()))
    .execute(&mut *conn)
    .await?;
    Ok(new.token)
}
