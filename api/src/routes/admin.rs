//! Yönetim uçları. Her uç rolünü kendi çıkarıcısıyla ister; korumasız uç yoktur.

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, patch},
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::role::{RequireAdmin, RequireModerator, Role},
    error::AppError,
    state::AppState,
};

/// Yetkili listesi bu sayıyla sınırlıdır; yetkili sayısı küçük kalacağı için sayfalama yok.
const STAFF_LIMIT: i64 = 200;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/staff", get(staff))
        .route("/users/{id}/role", patch(change_role))
}

#[derive(Serialize, FromRow)]
struct StaffMember {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    role: Role,
}

/// Admin ve moderatörler; önce admin, sonra kullanıcı adına göre.
async fn staff(
    State(state): State<AppState>,
    _: RequireModerator,
) -> Result<Json<Vec<StaffMember>>, AppError> {
    // `role <> 'user'` koşulu kısmi indeks `users_staff_role_idx` ile eşleşir.
    let members = sqlx::query_as(
        "SELECT id, username, display_name, role
           FROM users
          WHERE role <> 'user'
          ORDER BY role DESC, username
          LIMIT $1",
    )
    .bind(STAFF_LIMIT)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(members))
}

#[derive(Deserialize)]
struct RoleChange {
    role: String,
}

async fn change_role(
    State(state): State<AppState>,
    RequireAdmin(actor): RequireAdmin,
    Path(target_id): Path<Uuid>,
    Json(body): Json<RoleChange>,
) -> Result<Json<StaffMember>, AppError> {
    let new_role = parse_role(&body.role)?;
    // Kendi rolünü değiştiren admin kendini kilitleyebilir ya da yetkisini geri alamaz.
    if target_id == actor.user_id {
        return Err(AppError::Forbidden("own_role"));
    }

    let mut tx = state.db.begin().await?;

    // Admin satırlarını kilitlemek eşzamanlı rol değişikliklerini sıraya sokar: iki admin aynı anda
    // birbirini düşüremez ve "son admin" denetimi yarış durumuna açık kalmaz.
    let admins: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE role = 'admin' FOR UPDATE")
            .fetch_all(&mut *tx)
            .await?;
    // İstek başında admindi; kilidi beklerken başka bir admin onu düşürmüş olabilir.
    if !admins.contains(&actor.user_id) {
        return Err(AppError::Forbidden("forbidden"));
    }

    let target: Option<StaffMember> = sqlx::query_as(
        "SELECT id, username, display_name, role FROM users WHERE id = $1 FOR UPDATE",
    )
    .bind(target_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(mut target) = target else {
        return Err(AppError::NotFound("user_not_found"));
    };

    if target.role == new_role {
        return Ok(Json(target));
    }
    if target.role == Role::Admin && admins.len() <= 1 {
        return Err(AppError::Conflict("last_admin"));
    }

    sqlx::query("UPDATE users SET role = $2, updated_at = now() WHERE id = $1")
        .bind(target_id)
        .bind(new_role)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    tracing::info!(
        actor = %actor.user_id,
        target = %target_id,
        old = target.role.as_str(),
        new = new_role.as_str(),
        "rol değişti"
    );
    target.role = new_role;
    Ok(Json(target))
}

fn parse_role(raw: &str) -> Result<Role, AppError> {
    match raw {
        "user" => Ok(Role::User),
        "moderator" => Ok(Role::Moderator),
        "admin" => Ok(Role::Admin),
        _ => Err(AppError::Validation("invalid_role")),
    }
}
