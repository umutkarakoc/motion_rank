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

use super::super::mail;

pub async fn render_access(project_id: Uuid, user_id: Uuid, db: &PgPool) -> Markup {
    let result = sqlx::query!(
        r#"select v.id, v.user_id, u.name, u.email from project_access as v
        join "user" u on u.id = v.user_id
        join project on project.id = v.project_id
        where v.project_id = $1 and project.user_id = $2
        order by v.created_at"#,
        project_id,
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
             form class="d-flex mb-4" hx-post={"/project/"(project_id.to_string())"/access"}
                 hx-target="#sharing_content" hx-swap="outerHTML" hx-select="#sharing_content" {
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

async fn get_access(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    project: Path<Uuid>,
) -> impl IntoResponse {
    render_access(project.0, user_id, &db)
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
    Path(project_id): Path<Uuid>,
    Form(params): Form<CreateAccessParams>,
) -> impl IntoResponse {
    let project = sqlx::query!(
        "select * from project where id  = $1 and user_id = $2",
        project_id,
        user_id
    )
    .fetch_one(&db)
    .await;

    if project.is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let project = project.unwrap();

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
        if user.id == project.user_id {
            return render_access(project.id, user_id, &db)
                .await
                .into_string()
                .into_response();
        }
        user
    };

    sqlx::query!(
        r#"insert into project_access
            (project_id, email, user_id)
            VALUES ($1, $2, $3)
            returning *"#,
        project.id,
        params.email,
        user.id
    )
    .fetch_one(&db)
    .await
    .unwrap();

    tokio::spawn(async move {
        mail::send_share_mail(user.email, sender.name, format!("{} folder", project.name)).await;
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
        //                         "share": format!("{} project", project.name),
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

    render_access(project.id, user_id, &db)
        .await
        .into_string()
        .into_response()
}

async fn delete_access(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((project_id, access_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let project_access = sqlx::query!(
        r#"select v.user_id as owner, a.user_id from project_access a
                join project v on v.id = a.project_id
                where a.id = $1"#,
        access_id
    )
    .fetch_one(&db)
    .await
    .unwrap();

    if !(project_access.owner == user_id || project_access.user_id == user_id) {
        return StatusCode::FORBIDDEN.into_response();
    }

    sqlx::query!(r#"delete from project_access where id = $1"#, access_id)
        .execute(&db)
        .await
        .unwrap();

    render_access(project_id, user_id, &db)
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
