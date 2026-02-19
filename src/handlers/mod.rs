use crate::AppState;
use axum::{extract::State, response::IntoResponse};

pub async fn health(State(_state): State<AppState>) -> impl IntoResponse {
    "OK"
}
