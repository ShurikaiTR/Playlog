use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// API hataları. İstemciye yalnızca kısa bir kod döner; ayrıntı günlüğe yazılır.
#[derive(Debug)]
pub enum AppError {
    /// İstek biçimi ya da alan değerleri geçersiz.
    Validation(&'static str),
    /// Kimlik doğrulama gerekli ya da bilgiler hatalı.
    Unauthorized(&'static str),
    /// Oturum var ama rol bu işlem için yetmiyor.
    Forbidden(&'static str),
    /// İstenen kayıt yok.
    NotFound(&'static str),
    /// Çakışma (örn. kullanıcı adı alınmış).
    Conflict(&'static str),
    /// Çok fazla deneme.
    TooManyRequests(&'static str),
    /// Beklenmeyen sunucu hatası; ayrıntı istemciye gitmez.
    Internal(anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::Validation(code) => (StatusCode::UNPROCESSABLE_ENTITY, code),
            Self::Unauthorized(code) => (StatusCode::UNAUTHORIZED, code),
            Self::Forbidden(code) => (StatusCode::FORBIDDEN, code),
            Self::NotFound(code) => (StatusCode::NOT_FOUND, code),
            Self::Conflict(code) => (StatusCode::CONFLICT, code),
            Self::TooManyRequests(code) => (StatusCode::TOO_MANY_REQUESTS, code),
            Self::Internal(err) => {
                tracing::error!(error = ?err, "beklenmeyen hata");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal")
            }
        };
        (status, Json(ErrorBody { error: code })).into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self::Internal(err.into())
    }
}

/// Kullanıcı adı/e-posta benzersizlik ihlalini (yarış durumu) 409'a çevirir.
pub fn map_unique_violation(err: sqlx::Error) -> AppError {
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
