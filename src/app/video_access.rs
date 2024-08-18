use crate::logged_user::LoggedUser;
use crate::models::User;
use crate::AppState;
use axum::extract::{Path, State};
use axum::{response::*, routing::*, Router};
use chrono::Utc;
use maud::{html, Markup};
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::mail;
pub async fn render_access(video_id: Uuid, user_id: Uuid, db: &PgPool) -> Markup {
    let result = sqlx::query!(
        r#"select v.id, v.user_id, u.name, u.email from video_access as v
        join "user" u on u.id = v.user_id
        join video on video.id = v.video_id
        where v.video_id = $1 and video.user_id = $2
        order by v.created_at"#,
        video_id,
        user_id
    )
    .fetch_all(db)
    .await
    .unwrap();

    let button = html! {
        button type="button" class="btn btn-light" data-bs-toggle="modal" data-bs-target="#sharing-modal" {
            i class="bi bi-person-fill-lock me-2"{}
            span { "Share" }
        }
    };
    let sharing = html! {
        div id="sharing_content" class="p-2" {
             form class="d-flex mb-4"
                    hx-post={"/video/"(video_id.to_string())"/access"}
                    hx-target="#sharing_content" hx-swap="outerHTML"
                    hx-select="#sharing_content"  {
                 input id="edit_access" class="form-control form-control-sm" autofocus name="email"
                     style="flex: 1" placeholder="Add email" {}
                 button type="submit" class="btn btn-light btn-sm ms-2" { "Invite" }
             }
             @if result.len() == 0 {
                 div class="d-flex align-items-center justify-content-center flex-column mt-4" {
                     p class="h6 mt-2" { "Nobody has access to this video yet" }
                 }
             } @else {
                 @for access in result.iter() {
                     div class="d-flex justify-content-between align-items-center border-bottom pb-1 mt-2" {
                         span class="bg-primary text-white d-flex justify-content-center align-items-center rounded-circle"
                             style="width: 40px; height: 40px;" {
                             (access.name.chars().next().unwrap().to_uppercase().to_string())
                         }
                         div class="d-flex flex-column ms-2" style="flex: 1;" {
                             span class="h6 m-0" { (access.name) }
                             span class="text-muted small m-0" { (access.email) }
                         }
                         button class="btn " id={"sharing_delete_"(access.id.to_string())} {
                             i class="bi bi-x-circle-fill text-danger"{}
                         }
                     }
                 }
             }
        }
    };
    html! {
        div {
            (button)
            div class="modal fade" id="sharing-modal" tabindex="-1" aria-labelledby="sharing-label" aria-hidden="true" {
                div class="modal-dialog" {
                    div class="modal-content" {
                        div class="modal-header" {
                            h1 class="modal-title fs-5" id="sharing-label" { "Share Project" }
                            button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" {}
                        }
                        div class="modal-body" {
                            (sharing)
                        }
                    }
                }
            }
        }
    }
}
pub async fn _render_access(video_id: Uuid, user_id: Uuid, db: &PgPool) -> Markup {
    let result = sqlx::query!(
        r#"select v.id, v.user_id, u.name, u.email from video_access as v
        join "user" u on u.id = v.user_id
        join video on video.id = v.video_id
        where v.video_id = $1 and video.user_id = $2
        order by v.created_at"#,
        video_id,
        user_id
    )
    .fetch_all(db)
    .await
    .unwrap();

    html! {
        #sharing_model class="dropdown is-right ml-2"  {
            div class="dropdown-trigger"{
                button class="button is-light is-light" {
                    span class="icon is-small" { i class="fa fa-link" {} }
                    span {"Share"}
                }
            }

            div class="dropdown-menu box shadow p-2 mt-2 " id="sharing_content" role="menu" style="width: 400px;"{
                form style="display:flex;"
                    hx-post={"/video/"(video_id.to_string())"/access"}
                    hx-target="#sharing_content" hx-swap="outerHTML"
                    hx-select="#sharing_content"
                    {
                    input id="edit_access" class="input is-small" autofocus name="email"
                        style="flex:1" placeholder="Add email" {}
                    button type="submit" class="button is-small ml-2 is-info is-light"  {"Invite"}
                }

                div style="min-height: 200px; max-height: 400px; overflow-y: auto " {
                    @if result.len() == 0 {
                        div style="flex: 1" class="is-flex is-align-items-center mt-6
                            is-justify-content-center is-flex-direction-column" {
                            p class="title is-size-6 mt-2" {"Nobody has access to this video yet"}
                        }
                    } @else {
                        @for access in result.iter() {
                            div class="mt-2 pb-1"
                                style="display: flex; justify-content: space-between;
                                border-bottom: 1px solid darkgrey; align-items: center" {
                                span style="width:40px; height: 40px; border-radius: 30px"
                                    class="has-background-primary is-flex is-justify-content-center is-align-items-center"
                                    { (access.name.chars().next().unwrap().to_uppercase().to_string() ) }
                                div class="is-flex is-flex-direction-column ml-2" style="flex: 1"{
                                    span class="subtitle m-0 is-size-6"{ (access.name) }
                                    span class="subtitle m-0 is-size-7"{ (access.email) }
                                }
                                button class="button is-small is-danger is-light" id={"sharing_delete_"(access.id.to_string()) } {
                                    span class="icon"{i class="fa fa-remove" {} }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn get_access(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    video: Path<Uuid>,
) -> impl IntoResponse {
    render_access(video.0, user_id, &db)
        .await
        .into_string()
        .into_response()
}

#[derive(Deserialize)]
pub struct CreateAccessParams {
    pub email: String,
}
async fn post_access(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(video_id): Path<Uuid>,
    Form(params): Form<CreateAccessParams>,
) -> impl IntoResponse {
    let video = sqlx::query!(
        "select * from video where id  = $1 and user_id = $2",
        video_id,
        user_id
    )
    .fetch_one(&db)
    .await;

    if video.is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let video = video.unwrap();

    let sender = sqlx::query_as!(User, r#"select * from "user" where id = $1"#, &user_id)
        .fetch_one(&db)
        .await
        .unwrap();

    let user = sqlx::query_as!(
        User,
        r#"select * from "user" where email = $1"#,
        &params.email
    )
    .fetch_one(&db)
    .await;

    let user = if user.is_err() {
        let user = User {
            id: Uuid::new_v4(),
            email: params.email.clone(),
            name: params.email.clone()[0..params.email.find('@').unwrap()].to_string(),
            created_at: Utc::now(),
            registered_at: None,
        };
        let _result = sqlx::query!(
            r#"INSERT INTO "user" (id, "email", name, created_at) VALUES ($1, $2, $3, $4)"#,
            user.id,
            user.email,
            user.name,
            user.created_at
        )
        .execute(&db)
        .await;
        user
    } else {
        let user = user.unwrap();
        if user.id == video.user_id {
            return render_access(video.id, user_id, &db)
                .await
                .into_string()
                .into_response();
        }
        user
    };

    sqlx::query!(
        r#"insert into video_access
            (video_id, email, user_id)
            VALUES ($1, $2, $3)
            returning *"#,
        video.id,
        params.email,
        user.id
    )
    .fetch_one(&db)
    .await
    .unwrap();

    tokio::spawn(async move {
        mail::send_share_mail(user.email, sender.name, format!("{} video", video.title)).await;
        // reqwest::Client::new()
        //     .post("https://api.sendgrid.com/v3/mail/send")
        //     .header("Authorization", ENV.sendgrid_key.clone())
        //     .header("Content-Type", "application/json")
        //     .body(
        //         json!(
        //         {
        //             "personalizations": [
        //                 {
        //                     "to": [{"email": user.email }] ,
        //                     "dynamic_template_data": {
        //                         "name": sender.name,
        //                         "share": format!("{} video", video.title),
        //                     }
        //                 },

        //             ],
        //             "from": {
        //                 "email": "info@motionrank.com"
        //             },
        //            "template_id": "d-a8e5a5bd0f5547d8ac02f84e6df33fbb"
        //         })
        //         .to_string(),
        //     )
        //     .send()
        //     .await
        //     .unwrap();
    });

    render_access(video.id, user_id, &db)
        .await
        .into_string()
        .into_response()
}

async fn delete_access(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((video_id, access_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let video_access = sqlx::query!(
        r#"select v.user_id as owner, a.user_id from video_access a
                join video v on v.id = a.video_id
                where a.id = $1"#,
        access_id
    )
    .fetch_one(&db)
    .await
    .unwrap();

    if !(video_access.owner == user_id || video_access.user_id == user_id) {
        return StatusCode::FORBIDDEN.into_response();
    }

    sqlx::query!(r#"delete from video_access where id = $1"#, access_id)
        .execute(&db)
        .await
        .unwrap();

    render_access(video_id, user_id, &db)
        .await
        .into_string()
        .into_response()
}

//
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(post_access)).route(
        "/:access",
        get(get_access).post(post_access).delete(delete_access),
    )
}
