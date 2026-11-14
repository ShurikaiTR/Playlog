//! Yönetim uçları. Her uç rolünü kendi çıkarıcısıyla ister; korumasız uç yoktur.

mod games;
mod staff;
mod stats;
mod users;

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    staff::router()
        .merge(games::router())
        .merge(stats::router())
        .merge(users::router())
}
