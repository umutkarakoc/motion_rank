use super::{layout, mail};
use crate::AppState;
use crate::{models::User, ENV};
use async_session::{CookieStore, Session, SessionStore};
use axum::http::HeaderMap;
use axum::response::Html;
use axum::{extract::*, http::StatusCode, response::IntoResponse, routing::*, Router};
use chrono::Utc;
use maud::html;
use reqwest::header::{LOCATION, SET_COOKIE};
use serde::Deserialize;
use serde_json::json;
use sqlx::postgres::PgPool;
use std::ops::Add;
use uuid::Uuid;

pub async fn get_login() -> Html<String> {
    layout::page(
        html! {},
        html! {
            div style="width: 100vw; height: 100vh; display: flex; justify-content: center;
                align-items: stretch; flex-direction: row"{
            div style="max-width: 600px; padding: 40px; z-index: 100; display: flex; flex-direction: row;
                justify-content: center; align-items: center;" {
                div class="card p-4 d-flex flex-column justify-content-center" style="max-width: 650px; min-width: 500px; width: 100%;" {
                    img style="width: 100%; height: 30px; object-fit:contain" src="/logofull.png"  { }

                    h3 class="card-title h4 mt-3" {"Login to MotionRank"}
                    form hx-post="/auth/send_magic_link" hx-target="this" hx-swap="outerHTML"
                        class="d-flex flex-column mb-3" {
                        div class="mb-3"{
                            label for="email"{}
                            div class="input-group" {
                                span class="input-group-text" style="width: 40px;" { 
                                    svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" class="bi bi-envelope" viewBox="0 0 17 16"{
                                        path d="M0 4a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2zm2-1a1 1 0 0 0-1 1v.217l7 4.2 7-4.2V4a1 1 0 0 0-1-1zm13 2.383-4.708 2.825L15 11.105zm-.034 6.876-5.64-3.471L8 9.583l-1.326-.795-5.64 3.47A1 1 0 0 0 2 13h12a1 1 0 0 0 .966-.741M1 11.105l4.708-2.897L1 5.383z" {}
                                    }
                                }
                                input id="email" name="email" type="email" required placeholder="email"
                                    value="" class="form-control"{}
                            }
                        }

                        button class="btn btn-dark" {"Login"}
                    }
                }
            }}
        },
    )
}

#[derive(Deserialize)]
pub struct SendCodeParams {
    pub email: String,
}

pub async fn send_magic_link(
    State(client): State<PgPool>,
    Form(params): Form<SendCodeParams>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let email = params.email.to_string();
    let token = Uuid::new_v4();


    let link = format!("{}/auth/confirm/{}", ENV.host, token);

    {
        let email = email.clone();
        tokio::spawn(async move {
            mail::send_login_mail(email, link).await;
        });
    }

    sqlx::query!(
        "INSERT INTO magic_link (id, email, token, state) VALUES ($1, $2, $3, 'sent')",
        id,
        email,
        token,
    )
    .execute(&client)
    .await
    .unwrap();

    html!{
        form hx-post={"/auth/check/"(id.to_string())} hx-trigger="every 1s" hx-swap="none" class="d-flex flex-column" {
            div class="mb-3  d-flex flex-column"{
                div class="mb-3"{
                    label for="email" {}
                    div class="input-group" {
                        span class="input-group-text" style="width: 40px;" { 
                            svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" class="bi bi-envelope" viewBox="0 0 17 16"{
                                path d="M0 4a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2zm2-1a1 1 0 0 0-1 1v.217l7 4.2 7-4.2V4a1 1 0 0 0-1-1zm13 2.383-4.708 2.825L15 11.105zm-.034 6.876-5.64-3.471L8 9.583l-1.326-.795-5.64 3.47A1 1 0 0 0 2 13h12a1 1 0 0 0 .966-.741M1 11.105l4.708-2.897L1 5.383z" {}
                            }
                        }
                        input id="email" name="email" type="email" hx-preserve="true" disabled required placeholder="email" 
                            value=(email)  class="form-control" {}
                    }
                }
                
                div class="text-dark"{
                    "We just sent a magic login link to your email address. Check your email for confirmation."
                }

                div class=" placeholder-glow  mt-3 flex-fill d-flex "  {
                    div class="placeholder flex-fill bg-primary rounded" role="" {
                        
                    }
                }
            }
        }
    }.into_string().into_response()
}

pub async fn confirm(
    State(client): State<PgPool>,
    Path(token): Path<Uuid>,
) -> impl IntoResponse {
    let login_code = sqlx::query!(
        r#"SELECT * FROM magic_link
        WHERE token = $1 and state = 'sent' and created_at > $2
        ORDER BY created_at DESC LIMIT 1"#,
        token,
        Utc::now().add(chrono::Duration::minutes(-30)),
    )
    .fetch_one(&client)
    .await;

    match login_code {
        Ok(login_code) => {
            let exist = sqlx::query_as!(
                User,
                r#"SELECT * FROM "user" WHERE "email" = $1"#,
                login_code.email,
            )
            .fetch_one(&client)
            .await;

            if exist.is_err() {
                sqlx::query!(
                        r#"INSERT INTO "user" (id, "email", name, created_at, registered_at) VALUES ($1, $2, $3, $4, $5)"#,
                        Uuid::new_v4(),
                        login_code.email.clone(),
                        login_code.email.clone()[0..login_code.email.find('@').unwrap()].to_string(),
                        Utc::now(),
                        Some(Utc::now())
                    )
                    .execute(&client)
                    .await.unwrap();
            }

            sqlx::query!(
                "UPDATE magic_link SET state = 'verified' WHERE id = $1",
                login_code.id
            )
            .execute(&client)
            .await
            .unwrap();

            layout::page(
                html! {},
                html! {
                    app style="width: 100vw; height: 100vh; display: flex; flex-direction:column;
                        justify-content: center; align-items: center" {
                        h5 class="h4" { "Your login was successful"}
                        p class="h5" {"You can close this window"}
                    }
                }
            )
            .into_response()
        }
        Err(_err) => layout::page(
            html! {},
            html! {
                app style="width: 100vw; height: 100vh; display: flex; flex-direction:column;
                    justify-content: center; align-items: center" {
                    h5 class="h4" { "Your magic link has timed out"}
                    p class="h5" {"You can close this window"}
                }
            }
        )
        .into_response(),
    }
}

pub async fn check(
    State(client): State<PgPool>,
    State(store): State<CookieStore>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let magic_link = sqlx::query!(
        "SELECT * FROM magic_link
            WHERE id = $1
            ORDER BY created_at DESC LIMIT 1",
        id
    )
    .fetch_one(&client)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "unknown error".to_string(),
        )
            .into_response()
    })
    .unwrap();

    if magic_link.created_at < Utc::now().add(chrono::Duration::minutes(-30)) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "time_out"})),
        )
            .into_response();
    }

    if magic_link.state != "verified" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "code_not_verified"})),
        )
            .into_response();
    }

    sqlx::query!(
        "UPDATE magic_link SET state = 'used' WHERE id = $1",
        magic_link.id
    )
    .execute(&client)
    .await
    .unwrap();

    let user = sqlx::query_as!(
        User,
        r#"SELECT * FROM "user" WHERE "email" = $1"#,
        magic_link.email,
    )
    .fetch_one(&client)
    .await
    .unwrap();

    let mut session = Session::new();
    session.insert("user_id", user.id).unwrap();
    let token = store.store_session(session).await.unwrap().unwrap();

    let cookie = format!(
        "motionrank_token={}; HttpOnly; Max-Age={}; SameSite=None; Secure; Path=/",
        token,
        60 * 60 * 24 * 30 * 5
    );

    let mut headers = HeaderMap::new();
    headers.insert("hx-redirect", "/".parse().unwrap());
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (StatusCode::OK, headers).into_response()
}

async fn logout() -> impl IntoResponse {
    let cookie = format!("motionrank_token=deleted; HttpOnly; SameSite=None; Secure; Path=/");

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    headers.insert(LOCATION, "/auth/login".parse().unwrap());
    (headers, StatusCode::FOUND).into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(get_login))
        .route("/logout", post(logout).get(logout))
        .route("/send_magic_link", post(send_magic_link))
        .route("/confirm/:token", get(confirm))
        .route("/check/:id", post(check))
}
