// mod user;
mod auth;
mod layout;
mod mail;
mod project;
mod video;
mod video_access;
mod video_review;
// mod video_review;

use crate::{logged_user::LoggedUser, AppState};
use axum::{extract::State, http::HeaderMap, response::IntoResponse, routing::get, Router};
use reqwest::StatusCode;
use sqlx::PgPool;

async fn get_home(State(db): State<PgPool>, LoggedUser(user_id): LoggedUser) -> impl IntoResponse {
    let project = sqlx::query!(
        r#"select * from project where user_id = $1 order by created_at"#,
        user_id
    )
    .fetch_one(&db)
    .await;

    match project {
        Ok(project) => {
            let mut headers = HeaderMap::new();
            headers.append(
                "location",
                format!("/project/{}", project.id).parse().unwrap(),
            );
            (StatusCode::TEMPORARY_REDIRECT, headers).into_response()
        }
        Err(_) => {
            let user = sqlx::query!(r#"select * from "user" where id = $1 "#, user_id)
                .fetch_one(&db)
                .await
                .unwrap();
            let project = sqlx::query!(
                r#"insert into project
                (user_id, name)
                VALUES ($1, $2) returning *"#,
                user.id,
                format!("{}'s Project", user.name)
            )
            .fetch_one(&db)
            .await
            .unwrap();

            let mut headers = HeaderMap::new();
            headers.append(
                "location",
                format!("/project/{}", project.id).parse().unwrap(),
            );
            (StatusCode::TEMPORARY_REDIRECT, headers).into_response()
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_home))
        .nest("/project", project::router())
        .nest("/video", video::router())
        .nest("/video", video_review::router())
        .nest("/auth", auth::router())
}
