extern crate core;

use crate::appconfig::ENV;
use axum::{extract::FromRef, http::StatusCode, response::IntoResponse, routing::*, Router};
use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{io, time::Duration};
use tokio::time::sleep;
use tower_http::services::ServeDir;
mod app;
mod appconfig;
mod logged_user;
mod models;
mod service;
use async_session::CookieStore;

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
        session_store: CookieStore,
}
impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> PgPool {
        app_state.db.clone()
    }
}
impl FromRef<AppState> for CookieStore {
    fn from_ref(app_state: &AppState) -> CookieStore {
        app_state.session_store.clone()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("error"));

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&ENV.database_url)
        .await
        .expect("can connect to database");
    let task_db = db.clone();

    let session_store = CookieStore::new();

    let app_state = AppState { db, session_store };

    let serve_dir = get_service(ServeDir::new(ENV.assets.clone())).handle_error(handle_error);

    let app = Router::new()
        // .nest("/api", api::router())
        .merge(app::router())
        .with_state(app_state)
        .fallback_service(serve_dir);

    tokio::spawn(async move {
        loop {
            check_videos(&task_db).await;
            sleep(Duration::from_secs(2)).await;
        }
    });

    axum::Server::bind(&ENV.addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn check_videos(db: &PgPool) {
    let videos = sqlx::query!("select * from video where state != 'ready'")
        .fetch_all(db)
        .await
        .unwrap();

    for video in videos.iter() {
        let result = reqwest::Client::new()
            .get(format!(
                "https://video.bunnycdn.com/library/{}/videos/{}",
                ENV.bunny_folder, &video.id
            ))
            .header("AccessKey", &ENV.bunny_api_key)
            .header("Content-Type", "application/json")
            .send()
            .await;

        match result {
            Ok(result) => {
                let result = &result.json::<serde_json::Value>().await.unwrap();
                let state = result.get("status").unwrap().as_i64().unwrap();
                let duration = result.get("length").unwrap().as_f64().expect("msg") as i32;
                let width = result.get("width").unwrap().as_i64().unwrap() as i32;
                let height = result.get("height").unwrap().as_i64().unwrap() as i32;
                let processing = result.get("encodeProgress").unwrap().as_i64().unwrap();

                let state = match state {
                    5 => "error".to_string(),
                    6 => "error".to_string(),
                    4 => "ready".to_string(),
                    3 => "transcoding".to_string(),
                    2 => "processing".to_string(),
                    _ => "uploading".to_string(),
                };

                println!("{} {}", state, processing);

                sqlx::query!(r#"update video
                        set state = $5, duration = $2, width = $3 , height = $4, image_link = $6, preview_link = $7, processing=$8
                        where id = $1"#,
                            video.id,
                            duration,
                            width,
                            height,
                            state,
                            video.image_link,
                            video.preview_link,
                            processing as i32
                        ).execute(db)
                    .await.unwrap();
            }
            Err(err) => {
                println!("error at check videos:\n{}", err);
            }
        };

        if (Utc::now() - video.created_at).num_days() >= 1 {
            let _ = sqlx::query!("delete from video where id = $1", video.id)
                .execute(db)
                .await;
        }
    }
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
