use axum::{async_trait, extract::{FromRequestParts, FromRef}, http::request::Parts};
use once_cell::sync::Lazy;

use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Debug, Default, Deserialize, Clone )]
pub struct AppConfig {
    pub secret: String,
    pub host: String,
    pub addr: String,
    pub database_url: String,
    pub sendgrid_key: String,
    pub assets: String,
    pub bunny_folder: String,
    pub bunny_hostname: String,
    pub bunny_api_key : String,
    pub bunny_auth_key: String,
    pub mailer: String,
    pub mailer_password: String
}

pub static ENV: Lazy<AppConfig> = Lazy::new(|| {
    dotenv::dotenv().ok();
    let config_ = config::Config::builder()
        .add_source(config::Environment::default())
        .build()
        .unwrap();
    let config: AppConfig = config_.try_deserialize().unwrap();

    config
});

pub struct DatabaseConnection(sqlx::pool::PoolConnection<sqlx::Postgres>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool.acquire().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
