use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use crate::db::pool_manager::PoolManager;

pub async fn search_transactions(
    State(_pool_manager): State<PoolManager>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}
