use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::AppState;
use super::auth::VerifiedWebhook;

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionCallback {
    pub id: String,
    pub status: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Handler for /callback/transaction endpoint
/// Automatically verifies X-Stellar-Signature via VerifiedWebhook extractor
pub async fn transaction_callback(
    State(_state): State<AppState>,
    VerifiedWebhook { body }: VerifiedWebhook,
) -> impl IntoResponse {
    // Parse the verified body
    let callback: TransactionCallback = match serde_json::from_slice(&body) {
        Ok(cb) => cb,
        Err(e) => {
            tracing::error!("Failed to parse callback body: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid JSON payload").into_response();
        }
    };

    tracing::info!("Received verified transaction callback: id={}, status={}", 
        callback.id, callback.status);

    // TODO: Process the transaction callback
    // This is where you would update database, trigger business logic, etc.

    (StatusCode::OK, Json(serde_json::json!({
        "status": "received"
    }))).into_response()
}
