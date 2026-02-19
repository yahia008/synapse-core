use anyhow::Result;
use dotenvy::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server_port: u16,
    pub database_url: String,
    pub stellar_horizon_url: String,

        pub default_rate_limit: u32,
    pub whitelist_rate_limit: u32,
    pub whitelisted_ips: String,
}
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv().ok(); // Load .env file if present

        Ok(Config {
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            database_url: env::var("DATABASE_URL")?,
            stellar_horizon_url: env::var("STELLAR_HORIZON_URL")?,
            default_rate_limit: env::var("DEFAULT_RATE_LIMIT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("DEFAULT_RATE_LIMIT must be a number"),
            
            whitelist_rate_limit: env::var("WHITELIST_RATE_LIMIT")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .expect("WHITELIST_RATE_LIMIT must be a number"),
            
            whitelisted_ips: env::var("WHITELISTED_IPS")
                .unwrap_or_else(|_| "".to_string()),
        })
    }
}
