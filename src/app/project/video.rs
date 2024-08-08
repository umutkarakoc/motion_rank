use axum::{response::*, routing::*, Router};
use axum::extract::{Path, State};
use chrono::Utc;
use reqwest::StatusCode;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;
use crate::AppState;
use crate::appconfig::ENV;
use crate::logged_user::LoggedUser;

async fn generate_bunny_token(id: &Uuid, path: &str, auth_key: &String) -> String {
    let path = format!("/{}/{}", id, path);
    let mut hasher = Sha256::new();
    let expires = Utc::now().timestamp() + (50 * 12 * 30 * 24 * 3600);
    hasher.update(format!("{}{}{}", auth_key, path, expires).as_bytes());
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

#[derive(Deserialize)]
pub struct CreateParams {
    pub title: String,
}

async fn create_video(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((project_id, id)): Path<(Uuid, Uuid)>,
    Form(params): Form<CreateParams>
) -> impl IntoResponse {
    let image_link = generate_bunny_token(&id, "thumbnail.jpg", &ENV.bunny_auth_key).await;

    let preview_link = generate_bunny_token(&id, "preview.webp", &ENV.bunny_auth_key).await;

    sqlx::query!(r#"insert into video
            (user_id, title, id, project_id, image_link, preview_link )
            VALUES ($1, $2, $3, $4, $5, $6)"#,
                user_id,
                params.title,
                id,
                project_id,
                image_link,
                preview_link,
            ).execute(&db)
        .await.unwrap();

    StatusCode::OK
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:id", post(create_video))
}
