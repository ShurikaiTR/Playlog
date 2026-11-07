//! Kullanıcı rolleri ve yetki denetimi (OWASP Authorization Cheat Sheet: varsayılan olarak
//! reddet, her istekte sunucuda denetle). Rol her istekte veritabanından okunur, belirteçte
//! saklanmaz; rol değişikliği bir sonraki istekte geçerli olur.

use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::extractor::CurrentUser, error::AppError, state::AppState};

/// Sıra yetki düzeyidir: `User < Moderator < Admin` (veritabanındaki enum sırasıyla aynı).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Moderator,
    Admin,
}

impl Role {
    /// Bu rol en az `min` düzeyinde mi?
    pub fn at_least(self, min: Role) -> bool {
        self >= min
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Moderator => "moderator",
            Self::Admin => "admin",
        }
    }
}

async fn require(parts: &mut Parts, state: &AppState, min: Role) -> Result<CurrentUser, AppError> {
    let user = CurrentUser::from_request_parts(parts, state).await?;
    if user.role.at_least(min) {
        Ok(user)
    } else {
        Err(AppError::Forbidden("forbidden"))
    }
}

/// Moderatör ya da admin. Oturum yoksa 401, yetki yetmiyorsa 403.
pub struct RequireModerator(
    // Bugünkü moderatör uçları yalnızca yetkiyi ister; kullanıcıya ihtiyaç duyan uçlar için açık.
    #[allow(dead_code)] pub CurrentUser,
);

impl FromRequestParts<AppState> for RequireModerator {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        require(parts, state, Role::Moderator).await.map(Self)
    }
}

/// Yalnızca admin. Oturum yoksa 401, yetki yetmiyorsa 403.
pub struct RequireAdmin(pub CurrentUser);

impl FromRequestParts<AppState> for RequireAdmin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        require(parts, state, Role::Admin).await.map(Self)
    }
}

/// İlk admin: hiç admin yoksa ve bu e-postayla doğrulanmış bir hesap varsa onu admin yapar.
/// Tek ifade olduğu için atomiktir; bir admin oluştuktan sonra hiçbir etkisi kalmaz.
pub async fn bootstrap_admin(db: &PgPool, email: &str) -> anyhow::Result<()> {
    let promoted: Option<Uuid> = sqlx::query_scalar(
        "UPDATE users SET role = 'admin', updated_at = now()
          WHERE email = $1
            AND email_verified_at IS NOT NULL
            AND NOT EXISTS (SELECT 1 FROM users WHERE role = 'admin')
          RETURNING id",
    )
    .bind(email)
    .fetch_optional(db)
    .await?;
    if let Some(id) = promoted {
        tracing::info!(user = %id, "ADMIN_EMAIL hesabı ilk admin yapıldı");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Role::{self, *};

    #[test]
    fn roles_are_ordered_by_privilege() {
        assert!(User < Moderator && Moderator < Admin);
    }

    #[test]
    fn at_least_includes_higher_roles() {
        let cases = [
            (User, User, true),
            (User, Moderator, false),
            (User, Admin, false),
            (Moderator, User, true),
            (Moderator, Moderator, true),
            (Moderator, Admin, false),
            (Admin, User, true),
            (Admin, Moderator, true),
            (Admin, Admin, true),
        ];
        for (role, min, expected) in cases {
            assert_eq!(role.at_least(min), expected, "{role:?}.at_least({min:?})");
        }
    }

    #[test]
    fn serializes_as_lowercase() {
        assert_eq!(serde_json::to_string(&Moderator).unwrap(), "\"moderator\"");
        assert_eq!(serde_json::from_str::<Role>("\"admin\"").unwrap(), Admin);
        assert!(serde_json::from_str::<Role>("\"Admin\"").is_err());
        assert!(serde_json::from_str::<Role>("\"owner\"").is_err());
    }
}
