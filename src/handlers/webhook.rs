use crate::ApiState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::queries;
use crate::db::models::Transaction;
use crate::error::AppError;
use utoipa::ToSchema;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CallbackPayload {
    pub stellar_account: String,
    pub amount: String,
    pub asset_code: String,
    pub callback_type: Option<String>,
    pub callback_status: Option<String>,
    pub anchor_transaction_id: Option<String>,
    pub memo: Option<String>,
    pub memo_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct WebhookPayload {
    pub id: String,
    pub anchor_transaction_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookResponse {
    pub success: bool,
    pub message: String,
}

fn validate_memo_type(memo_type: &Option<String>) -> Result<(), AppError> {
    if let Some(mt) = memo_type {
        match mt.as_str() {
            "text" | "hash" | "id" => Ok(()),
            _ => Err(AppError::Validation(format!(
                "Invalid memo_type '{}'. Must be one of: text, hash, id",
                mt
            ))),
        }
    } else {
        Ok(())
    }
}

#[utoipa::path(
    post,
    path = "/callback",
    request_body = CallbackPayload,
    responses(
        (status = 201, description = "Transaction created", body = crate::schemas::TransactionSchema),
        (status = 400, description = "Invalid payload"),
        (status = 500, description = "Processing error")
    ),
    tag = "Webhooks"
)]
pub async fn callback(
    State(state): State<ApiState>,
    Json(payload): Json<CallbackPayload>,
) -> Result<impl IntoResponse, AppError> {
    validate_memo_type(&payload.memo_type)?;

    let amount = sqlx::types::BigDecimal::from_str(&payload.amount)
        .map_err(|_| AppError::Validation(format!("Invalid amount: {}", payload.amount)))?;

    let tx = Transaction::new(
        payload.stellar_account,
        amount,
        payload.asset_code,
        payload.anchor_transaction_id,
        payload.callback_type,
        payload.callback_status,
        payload.memo,
        payload.memo_type,
        payload.metadata,
    );

    let inserted = queries::insert_transaction(&state.app_state.db, &tx).await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

#[utoipa::path(
    post,
    path = "/webhook",
    request_body = WebhookPayload,
    responses(
        (status = 200, description = "Webhook processed successfully", body = WebhookResponse),
        (status = 400, description = "Invalid payload"),
        (status = 500, description = "Processing error")
    ),
    tag = "Webhooks"
)]
pub async fn handle_webhook(
    State(_state): State<ApiState>,
    Json(payload): Json<WebhookPayload>,
) -> impl IntoResponse {
    tracing::info!("Processing webhook with id: {}", payload.id);

    let response = WebhookResponse {
        success: true,
        message: format!("Webhook {} processed successfully", payload.id),
    };

    (StatusCode::OK, Json(response))
}

#[utoipa::path(
    get,
    path = "/transactions/{id}",
    params(
        ("id" = String, Path, description = "Transaction ID")
    ),
    responses(
        (status = 200, description = "Transaction found", body = crate::schemas::TransactionSchema),
        (status = 404, description = "Transaction not found"),
        (status = 500, description = "Database error")
    ),
    tag = "Transactions"
)]
pub async fn get_transaction(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let transaction = queries::get_transaction(&state.app_state.db, id).await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Transaction {} not found", id)),
            _ => AppError::DatabaseError(e.to_string()),
        })?;

    Ok(Json(transaction))
}
