use synapse_core::{create_app, AppState};
use testcontainers_modules::postgres::Postgres;
use testcontainers::runners::AsyncRunner;
use sqlx::{PgPool, migrate::Migrator};
use std::path::Path;
use tokio::net::TcpListener;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_valid_deposit_flow() {
    let container = Postgres::default().start().await.unwrap();
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", host_port);

    // Run migrations
    let pool = PgPool::connect(&database_url).await.unwrap();
    let migrator = Migrator::new(Path::join(Path::new(env!("CARGO_MANIFEST_DIR")), "migrations")).await.unwrap();
    migrator.run(&pool).await.unwrap();

    // Start App
    let app_state = AppState {
        db: pool.clone(),
        horizon_client: synapse_core::stellar::HorizonClient::new("https://horizon-testnet.stellar.org".to_string()),
    };
    let app = create_app(app_state);
    
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let base_url = format!("http://{}", addr);

    // 1. POST /callback with valid signature
    let payload = json!({
        "stellar_account": "GABC1234567890",
        "amount": "100.50",
        "asset_code": "USD",
        "callback_type": "deposit",
        "callback_status": "completed"
    });

    let res = client.post(&format!("{}/callback", base_url))
        .header("X-App-Signature", "valid-signature")
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let transaction: serde_json::Value = res.json().await.unwrap();
    let tx_id = transaction["id"].as_str().unwrap();

    // 2. GET /transactions/{id}
    let res = client.get(&format!("{}/transactions/{}", base_url, tx_id))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let fetched_tx: serde_json::Value = res.json().await.unwrap();
    assert_eq!(fetched_tx["id"], tx_id);
}

#[tokio::test]
async fn test_invalid_signature_flow() {
    let container = Postgres::default().start().await.unwrap();
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", host_port);

    // Run migrations
    let pool = PgPool::connect(&database_url).await.unwrap();
    let migrator = Migrator::new(Path::join(Path::new(env!("CARGO_MANIFEST_DIR")), "migrations")).await.unwrap();
    migrator.run(&pool).await.unwrap();

    // Start App
    let app_state = AppState {
        db: pool.clone(),
        horizon_client: synapse_core::stellar::HorizonClient::new("https://horizon-testnet.stellar.org".to_string()),
    };
    let app = create_app(app_state);
    
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let base_url = format!("http://{}", addr);

    // POST /callback with invalid signature
    let payload = json!({
        "stellar_account": "GABC1234567890",
        "amount": "100.50",
        "asset_code": "USD",
        "callback_type": "deposit",
        "callback_status": "completed"
    });

    let res = client.post(&format!("{}/callback", base_url))
        .header("X-App-Signature", "invalid-signature")
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let error_res: serde_json::Value = res.json().await.unwrap();
    assert!(error_res["error"].as_str().unwrap().contains("Invalid signature"));
}
