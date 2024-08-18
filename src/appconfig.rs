use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone )]
pub struct AppConfig {
    pub host: String,
    pub addr: String,
    pub database_url: String,
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
