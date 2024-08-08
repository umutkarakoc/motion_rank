use sha2::{Digest, Sha256};
use sha256::digest;
use chrono::Utc;
use uuid::Uuid;

use crate::appconfig::ENV;

pub async fn generate_token(id: &Uuid, path: &str) -> String {
    let path = format!("/{}/{}", id, path);
    let mut hasher = Sha256::new();
    let expires = Utc::now().timestamp() + (50 * 12 * 30 * 24 * 3600);
    hasher.update(format!("{}{}{}",  &ENV.bunny_auth_key, path, expires).as_bytes());
    // hasher.update("hello".as_bytes());
    let token = hasher.finalize();
    let token = base64::encode(token)
        .replace("\n", "")
        .replace("+", "-")
        .replace("/", "_")
        .replace("=", "");
    format!(
        "https://{}{}?token={}&expires={}",
        ENV.bunny_hostname,
        path,
        token,
        expires
    )
}

pub async fn create_upload() -> (String, i64, String ) {
    let id = Uuid::new_v4();
    let result = reqwest::Client::new()
        .post(format!("https://video.bunnycdn.com/library/{}/videos", ENV.bunny_folder))
        .header("AccessKey", &ENV.bunny_api_key)
        .header("Content-Type", "application/json")
        .body(format!(r#"{{"title": "{}"}}"#, id))
        .send()
        .await.unwrap();

    let result = result.json::<serde_json::Value>()
        .await.unwrap();

    let video_id = result.get("guid").unwrap()
        .as_str().unwrap();

    let expire = Utc::now().timestamp() + (24 * 3000);
    let token = digest(format!("{}{}{}{}",
                               ENV.bunny_folder,
                               ENV.bunny_api_key,
                               expire,
                               video_id));


    (token, expire, video_id.to_string())
}