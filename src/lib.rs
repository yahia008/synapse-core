pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod services;
pub mod stellar;

use axum::{Router, routing::get};
use crate::stellar::HorizonClient;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub horizon_client: HorizonClient,
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/settlements", get(handlers::settlements::list_settlements))
        .route("/settlements/:id", get(handlers::settlements::get_settlement))
        .route("/callback", axum::routing::post(handlers::webhook::callback))
        .route("/transactions/:id", get(handlers::webhook::get_transaction))
        .with_state(state)
}
