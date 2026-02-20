use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use crate::db::{models::Transaction, queries};
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CallbackPayload {
    pub stellar_account: String,
    pub amount: bigdecimal::BigDecimal,
    pub asset_code: String,
    pub anchor_transaction_id: Option<String>,
    pub callback_type: Option<String>,
    pub callback_status: Option<String>,
}

use crate::ApiState;

pub async fn callback(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<CallbackPayload>,
) -> Result<impl IntoResponse, AppError> {
    // Simple mock signature check for integration tests
    let sig = headers.get("X-App-Signature")
        .and_then(|h| h.to_str().ok());
    
    if sig != Some("valid-signature") {
        return Err(AppError::BadRequest("Invalid signature".to_string()));
    }

    let tx = Transaction::new(
        payload.stellar_account,
        payload.amount,
        payload.asset_code,
        payload.anchor_transaction_id,
        payload.callback_type,
        payload.callback_status,
    );

    let inserted = queries::insert_transaction(&state.app_state.db, &tx).await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

pub async fn get_transaction(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let tx = queries::get_transaction(&state.app_state.db, id).await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Transaction {} not found", id)),
            _ => AppError::DatabaseError(e.to_string()),
        })?;

    Ok(Json(tx))
}
