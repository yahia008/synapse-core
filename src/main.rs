mod config;
mod db;
mod error;
mod handlers;
mod stellar;
mod metrics;
mod services;

use axum::{Router, extract::State, routing::get, middleware};
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::migrate::Migrator; // for Migrator
use std::net::SocketAddr; // for SocketAddr
use std::path::Path; // for Path
use tokio::net::TcpListener; // for TcpListener
use tracing_subscriber::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt}; // for .with() on registry
use stellar::HorizonClient;

#[derive(Clone)] // <-- Add Clone
pub struct AppState {
    db: sqlx::PgPool,
    pub horizon_client: HorizonClient,
    pub metrics_handle: PrometheusHandle,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Config::from_env()?;

    // Setup logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database pool
    let pool = db::create_pool(&config).await?;

    // Run migrations
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(&pool).await?;
    tracing::info!("Database migrations completed");

    // Initialize Stellar Horizon client
    let horizon_client = HorizonClient::new(config.stellar_horizon_url.clone());
    tracing::info!("Stellar Horizon client initialized with URL: {}", config.stellar_horizon_url);

    // Initialize metrics
    let metrics_handle = metrics::init_metrics()
        .map_err(|e| anyhow::anyhow!("Failed to initialize metrics: {}", e))?;
    tracing::info!("Metrics initialized successfully");

    // Build router with state
    let app_state = AppState { 
        db: pool.clone(),
        horizon_client,
        metrics_handle: metrics_handle.clone(),
    };
    
    // Create a state tuple for the metrics endpoint (needs both handle and pool)
    #[derive(Clone)]
    struct MetricsState {
        handle: PrometheusHandle,
        pool: sqlx::PgPool,
    }
    
    let metrics_state = MetricsState {
        handle: metrics_handle.clone(),
        pool: pool.clone(),
    };
    
    // Create metrics route with authentication middleware
    let metrics_route = Router::new()
        .route("/metrics", get(|
            axum::extract::State(state): axum::extract::State<MetricsState>
        | async move {
            metrics::metrics_handler(
                axum::extract::State(state.handle),
                axum::extract::State(state.pool),
            ).await
        }))
        .layer(middleware::from_fn_with_state(
            config.clone(),
            metrics::metrics_auth_middleware,
        ))
        .with_state(metrics_state);
    
    let app = Router::new()
        .route("/health", get(handlers::health))
        .merge(metrics_route)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

