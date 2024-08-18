mod access;
mod upload;
mod video;

use crate::logged_user::LoggedUser;
use crate::models::{Project, User};
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::{response::*, routing::*, Router};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::layout;
use super::video::render_video_boxes;

#[derive(Deserialize)]
struct ProjectQuery {
    search: Option<String>,
}

fn logo() -> Markup {
    html! {

        svg width="200" class="logo css-1j8o68f" height="28.815915968844557" viewBox="0 0 271.3281937893257 28.815915968844557" {
            defs id="SvgjsDefs1001" {
                linearGradient id="SvgjsLinearGradient1011" {
                    stop id="SvgjsStop1012" stop-color="#2d388a" offset="0"{}
                    stop id="SvgjsStop1013" stop-color="#00aeef" offset="1"{}
                }
            }
            g id="SvgjsG1007" featurekey="symbolFeature-0"
                transform="matrix(0.31805647300397594,0,0,0.31805647300397594,-4.675431609105436,-33.30051417946327)"
                fill="url('#SvgjsLinearGradient1011')" {
                path xmlns="http://www.w3.org/2000/svg" d="M240,105c-8.2,0-15.9,2.2-22.5,6c-6.6-3.8-14.3-6-22.5-6s-15.9,2.2-22.5,6c-6.6-3.8-14.3-6-22.5-6s-15.9,2.2-22.5,6  c-6.6-3.8-14.3-6-22.5-6c-8.1,0-15.7,2.1-22.2,5.9c-6.7-3.9-14.5-6.2-22.8-6.2c-25,0-45.3,20.3-45.3,45.3S35,195.3,60,195.3  c8.3,0,16.1-2.3,22.8-6.2c6.6,3.7,14.1,5.9,22.2,5.9c8.2,0,15.9-2.2,22.5-6c6.6,3.8,14.3,6,22.5,6s15.9-2.2,22.5-6  c6.6,3.8,14.3,6,22.5,6s15.9-2.2,22.5-6c6.6,3.8,14.3,6,22.5,6c24.9,0,45-20.1,45-45S264.9,105,240,105z M105,106.1  c7.8,0,15,2,21.4,5.6c-11.9,7.4-20.1,20.1-21.2,34.9c-1.1-14.8-9.3-27.6-21.3-35.1C90.2,108.1,97.4,106.1,105,106.1z M60,194.7  c-24.7,0-44.7-20.1-44.7-44.7s20.1-44.7,44.7-44.7c8.1,0,15.7,2.2,22.2,5.9C68.9,119,60,133.5,60,150s8.9,31,22.2,38.8  C75.7,192.6,68.1,194.7,60,194.7z M61.1,150c0-16.3,9-30.6,22.2-38.1c12.8,7.9,21.4,22,21.4,38.1s-8.6,30.3-21.4,38.1  C70.1,180.6,61.1,166.3,61.1,150z M105,193.9c-7.7,0-14.9-2-21.1-5.4c11.9-7.4,20.2-20.3,21.3-35.1c1.1,14.8,9.3,27.5,21.2,34.9  C120.1,191.8,112.8,193.9,105,193.9z M107.3,150c0-15.8,8.6-29.6,21.3-37c12.2,7.8,20.3,21.5,20.3,37s-8.1,29.2-20.3,37  C115.8,179.6,107.3,165.8,107.3,150z M150,192.8c-7.3,0-14.3-1.9-20.3-5.1c12.2-8,20.3-21.9,20.3-37.6s-8.1-29.6-20.3-37.6  c6-3.3,13-5.1,20.3-5.1s14.3,1.9,20.3,5.1c-12.2,8-20.3,21.9-20.3,37.6s8.1,29.6,20.3,37.6C164.3,190.9,157.4,192.8,150,192.8z   M154.5,150c0-14.9,8.1-27.9,20.1-35c11,7.7,18.2,20.5,18.2,35s-7.2,27.2-18.2,35C162.6,177.9,154.5,164.9,154.5,150z M195,190.5  c-6.6,0-12.7-1.6-18.2-4.4c11-8.1,18.2-21.3,18.2-36.1s-7.2-28-18.2-36.2c5.5-2.8,11.7-4.4,18.2-4.4s12.7,1.6,18.2,4.4  c-11,8.2-18.2,21.4-18.2,36.2s7.2,28,18.2,36.2C207.8,188.9,201.6,190.5,195,190.5z"{}
            }
            g id="SvgjsG1008" featurekey="nameFeature-0"
                transform="matrix(0.45685919503925093,0,0,0.45685919503925093,105.26902527704483,3.2537555592068794)"
                fill="#292929" {
                path d="M1.6 40 l0 -31.2 l8.04 0 l9 21.36 l9.04 -21.36 l8.04 0 l0 31.2 l-7.96 0 l0 -14.68 l-6.48 14.68 l-5.24 0 l-6.48 -14.68 l0 14.68 l-7.96 0 z M59.212 40.6 c-4.52 0 -8.4 -1.6 -11.64 -4.76 c-3.28 -3.16 -4.92 -7 -4.92 -11.44 s1.64 -8.28 4.92 -11.44 c3.24 -3.16 7.12 -4.76 11.64 -4.76 c4.56 0 8.48 1.6 11.72 4.76 s4.84 7 4.84 11.44 s-1.6 8.28 -4.84 11.44 s-7.16 4.76 -11.72 4.76 z M59.212 32.88 c2.4 0 4.4 -0.84 6.04 -2.48 s2.44 -3.64 2.44 -6 s-0.8 -4.4 -2.44 -6.04 s-3.64 -2.44 -6.04 -2.44 c-2.36 0 -4.36 0.84 -6 2.48 s-2.48 3.64 -2.48 6 s0.84 4.36 2.48 6 s3.64 2.48 6 2.48 z M92.424 40 l0 -23.44 l-9.72 0 l0 -7.76 l27.44 0 l0 7.76 l-9.72 0 l0 23.44 l-8 0 z M117.076 40 l0 -31.2 l8 0 l0 31.2 l-8 0 z M148.568 40.6 c-4.52 0 -8.4 -1.6 -11.64 -4.76 c-3.28 -3.16 -4.92 -7 -4.92 -11.44 s1.64 -8.28 4.92 -11.44 c3.24 -3.16 7.12 -4.76 11.64 -4.76 c4.56 0 8.48 1.6 11.72 4.76 s4.84 7 4.84 11.44 s-1.6 8.28 -4.84 11.44 s-7.16 4.76 -11.72 4.76 z M148.568 32.88 c2.4 0 4.4 -0.84 6.04 -2.48 s2.44 -3.64 2.44 -6 s-0.8 -4.4 -2.44 -6.04 s-3.64 -2.44 -6.04 -2.44 c-2.36 0 -4.36 0.84 -6 2.48 s-2.48 3.64 -2.48 6 s0.84 4.36 2.48 6 s3.64 2.48 6 2.48 z M172.06 40 l0 -31.2 l6.8 0 l13.92 17.44 l0 -17.44 l7.96 0 l0 31.2 l-6.76 0 l-13.92 -17.4 l0 17.4 l-8 0 z M225.404 40 l0 -31.2 l13.76 0 c3.92 0 6.96 1 9.08 3 c2.12 2.04 3.16 4.56 3.16 7.48 c0 3.08 -1.04 5.64 -3.16 7.64 c-0.68 0.64 -1.44 1.16 -2.28 1.6 l6.24 11.48 l-7.56 0 l-5.48 -10.04 l-5.76 0 l0 10.04 l-8 0 z M233.404 23.56 l5.32 0 c3.08 0 4.6 -1.4 4.6 -4.24 c0 -1.2 -0.36 -2.16 -1.12 -2.96 s-1.92 -1.16 -3.48 -1.16 l-5.32 0 l0 8.36 z M259.136 40 l12.64 -31.2 l7.92 0 l12.6 31.2 l-8.72 0 l-2.52 -6.96 l-10.6 0 l-2.6 6.96 l-8.72 0 z M272.936 26.560000000000002 l5.56 -0.04 l-2.76 -7.48 z M299.228 40 l0 -31.2 l6.8 0 l13.92 17.44 l0 -17.44 l7.96 0 l0 31.2 l-6.76 0 l-13.92 -17.4 l0 17.4 l-8 0 z M334.84000000000003 40 l0 -31.2 l8 0 l0 10.76 l9.68 -10.76 l10.32 0 l-12.32 13.4 l12.96 17.8 l-9.92 0 l-8.6 -11.76 l-2.12 2.36 l0 9.4 l-8 0 z"{}
            }
        }
    }
}

async fn get_project(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ProjectQuery>,
) -> impl IntoResponse {
    let user = sqlx::query_as!(User, r#"select * from "user" where id = $1"#, user_id)
        .fetch_one(&db)
        .await
        .unwrap();

    let projects = sqlx::query_as!(
        Project,
        r#"select * from project where user_id = $1 order by id"#,
        user_id
    )
    .fetch_all(&db)
    .await
    .unwrap();

    let shared_projects = sqlx::query!(
        r#"select project.name, project.id, "user".name as owner_name, "user".id as owner_id from project_access 
        join project on project.id = project_access.project_id
        join "user" on "user".id = project.user_id
        where project_access.user_id = $1 
        order by project.name"#,
        user_id)
    .fetch_all(&db)
    .await.unwrap();

    let (title, owner) = {
        let found = projects.iter().find_map(|f| {
            if f.id == project_id {
                Some((f.name.clone(), f.user_id.clone()))
            } else {
                None
            }
        });
        if found.is_none() {
            shared_projects.iter().find_map(|f| {
                if f.id == project_id {
                    Some((f.name.clone(), f.owner_id))
                } else {
                    None
                }
            })
        } else {
            found
        }
    }
    .unwrap();

    let videos = render_video_boxes(user_id, &db, project_id, query.search).await;

    let left_menu = html! {
        div  class="dropend" hx-preserve="true" {
            button data-bs-toggle="dropdown" aria-expanded="false"
                class="btn mt-2 w-100 d-flex justify-content-between align-items-center bg-white"
                type="button" {
                "Projects"
                span class="dropdown-toggle" {}

            }
            ul class="dropdown-menu p-2" {
                button class="btn" hx-target="#app"
                    hx-post="/project" hx-push-url="true" hx-swap="outerHTML" {
                        "Create New Project"
                    }
            }
        }
        div class="d-flex flex-column mt-2" id="projects" {

            @for project in projects.iter() {
                a href={"/project/"(project.id.to_string())}
                    hx-boost="true" hx-target="#app" hx-swap="outerHTML"
                    class={"btn mb-2 btn-light " (if project_id == project.id {"btn-dark"} else {""}) }
                    { (project.name) }
            }

        }
        @if shared_projects.len() > 0 {
            div id="shared_project_title" class="d-flex is-align-items-center mt-4 justify-content-between" {
                h5 class="card-header-title p-0 text-dark" { "Shared Projects" }
            }
            div class="d-flex flex-column mt-2" id="shared_projects" {
                @for project in shared_projects.iter() {
                    a href={"/project/"(project.id.to_string())}
                    hx-boost="true" hx-target="#app" hx-swap="outerHTML"
                        class={"btn btn-light mb-2 " (if project_id == project.id {"btn-dark"} else {""}) }
                    { (project.name) }
                }
            }
        }
    };

    let top_right_menu = html! {
        div class="field d-flex" {
           input id="search" class="form-control " style="width: 280px;" type="text"
                placeholder="Search videos" hx-push-url="true" hx-get={"/project/"(project_id.to_string())}
                name="search" hx-target="#app" hx-swap="outerHTML" hx-preserve="true" {}

            @if owner == user_id {
                div class="ms-2" {
                    (access::render_access(project_id, user_id, &db).await)
                }
            }

            a class="btn btn-light upload ms-2" hx-post={"/project/"(project_id.to_string())"/upload"}
                hx-target="#app" hx-swap="none" {
                span class="icon is-small" { i class="fas fa-video" {} }
                span {"Upload"}
            }

            div class="dropdown ms-2" {
                button class="btn dropdown-toggle btn-ligh" data-bs-toggle="dropdown"
                    hx-get="/profile/menu" {
                        span class="icon" {i class="fa-solid fa-user"{} }
                        span { (user.name) }

                }
                div class="dropdown-menu" id="dropdown-menu3" role="menu" {
                    div class="dropdown-content" {
                        a href="/profile" class="dropdown-item" {"Profile" }
                        a href="/auth/logout" class="dropdown-item button btn-danger btn-outline" {"Logout"}
                    }
                }
            }
        }
    };

    layout::page(
        html! {
            script src="https://cdn.jsdelivr.net/npm/tus-js-client@latest/dist/tus.min.js" {}
            script src="/upload.js"{}
        },
        html! {
            div id="app" class="background-white" {

            div class="d-flex flex1 " {
                div class="me-2 mb-0 p-4 flex-column d-flex is-align-items-stretch background-light"
                    style="width: 240px;" {

                      (logo())
                      (left_menu)

                                     }

                div class="d-flex flex-column p-4" style="flex: 1"{
                    div class="d-flex flex-row justify-content-between" {
                        div class="d-flex"{
                            div  class="ms-2 d-flex align-items-center" {
                                div id="project_name" {
                                    p class="h3 m-0 text-black" { (title) }
                                }

                            }
                        }

                        (top_right_menu)

                    }

                    div id="page" class="d-flex flex-column" style="flex: 1; overflow-y: auto;" {
                        div class="d-flex flex-wrap is-align-content-flex-start mt-4 " {
                            (videos)
                        }
                    }

                }

            }

            }
        },
    )
    // render_projects, render_shared_projects, title, user.name, videos )
}

async fn create_project(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
) -> impl IntoResponse {
    let project = sqlx::query_as!(
        Project,
        r#"insert into project (user_id, name) VALUES ($1, 'My Project') returning *"#,
        user_id
    )
    .fetch_one(&db)
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        "hx-redirect",
        format!("/project/{}", project.id).parse().unwrap(),
    );
    headers.into_response()
}

async fn edit_project(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let project = sqlx::query!(
        "select * from project where user_id = $1 and id = $2",
        user_id,
        project_id
    )
    .fetch_one(&db)
    .await
    .unwrap();

    html! { div class="ms-2 d-flex" {
        input name="name" autofocus style="height:38px; border: none" class="form-control is-size-3 p-0 btn-primary is-shadowless
                text-grey title" hx-trigger="blur"
                hx-patch={"/project/"(project.id.to_string())} 
                hx-push-url="true" hx-swap="outerHTML" hx-target="#app" 
                value=(project.name) {}
        }
    }.into_string()
    .into_response()
}

#[derive(Deserialize)]
pub struct UpdateParams {
    pub name: String,
}

pub async fn update_project(
    LoggedUser(user_id): LoggedUser,
    State(client): State<PgPool>,
    Path(project_id): Path<Uuid>,
    Form(params): Form<UpdateParams>,
) -> impl IntoResponse {
    sqlx::query!(
        r#"update project set name = $3 where id = $1 and user_id = $2"#,
        project_id,
        user_id,
        params.name
    )
    .execute(&client)
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        "hx-location",
        format!("/project/{}", project_id).parse().unwrap(),
    );
    headers.into_response()
}

//
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:project_id", get(get_project).patch(update_project))
        .route("/:project_id/edit", get(edit_project))
        .route("/", post(create_project))
        .nest("/:project_id/access", access::router())
        .nest("/:project_id/video", video::router())
        .nest("/:project_id/upload", upload::router())
}
