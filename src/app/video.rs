use super::{layout, video_access};
use crate::logged_user::LoggedUser;
use crate::{service::bunnycdn::generate_token, AppState};
use axum::extract::{Path, State};
use axum::{http::StatusCode, response::*, routing::*, Router};
use maud::{html, Markup};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use uuid::Uuid;

const COLORS: Lazy<Vec<&'static str>> =
    Lazy::new(|| vec!["0069d5", "bf1282", "c3001b", "00c38d", "ffffff", "000000"]);

pub fn logo() -> Markup {
    html! {

        svg width="200" class="logo css-1j8o68f" height="28.815915968844557" style="height: 30px; margin-top: 5px"
            viewBox="0 0 271.3281937893257 28.815915968844557" {
            defs id="SvgjsDefs1001" {
                linearGradient id="SvgjsLinearGradient1011" {
                    stop id="SvgjsStop1012" stop-color="#2d388a" offset="0"{}
                    stop id="SvgjsStop1013" stop-color="#00aeef" offset="1"{}
                }
            }
            g id="SvgjsG1007" featurekey="symbolFeature-0"
                transform="matrix(0.31805647300397594,0,0,0.31805647300397594,-4.675431609105436,-33.30051417946327)"
                fill="url('#SvgjsLinearGradient1011')" {
                path  xmlns="http://www.w3.org/2000/svg" d="M240,105c-8.2,0-15.9,2.2-22.5,6c-6.6-3.8-14.3-6-22.5-6s-15.9,2.2-22.5,6c-6.6-3.8-14.3-6-22.5-6s-15.9,2.2-22.5,6  c-6.6-3.8-14.3-6-22.5-6c-8.1,0-15.7,2.1-22.2,5.9c-6.7-3.9-14.5-6.2-22.8-6.2c-25,0-45.3,20.3-45.3,45.3S35,195.3,60,195.3  c8.3,0,16.1-2.3,22.8-6.2c6.6,3.7,14.1,5.9,22.2,5.9c8.2,0,15.9-2.2,22.5-6c6.6,3.8,14.3,6,22.5,6s15.9-2.2,22.5-6  c6.6,3.8,14.3,6,22.5,6s15.9-2.2,22.5-6c6.6,3.8,14.3,6,22.5,6c24.9,0,45-20.1,45-45S264.9,105,240,105z M105,106.1  c7.8,0,15,2,21.4,5.6c-11.9,7.4-20.1,20.1-21.2,34.9c-1.1-14.8-9.3-27.6-21.3-35.1C90.2,108.1,97.4,106.1,105,106.1z M60,194.7  c-24.7,0-44.7-20.1-44.7-44.7s20.1-44.7,44.7-44.7c8.1,0,15.7,2.2,22.2,5.9C68.9,119,60,133.5,60,150s8.9,31,22.2,38.8  C75.7,192.6,68.1,194.7,60,194.7z M61.1,150c0-16.3,9-30.6,22.2-38.1c12.8,7.9,21.4,22,21.4,38.1s-8.6,30.3-21.4,38.1  C70.1,180.6,61.1,166.3,61.1,150z M105,193.9c-7.7,0-14.9-2-21.1-5.4c11.9-7.4,20.2-20.3,21.3-35.1c1.1,14.8,9.3,27.5,21.2,34.9  C120.1,191.8,112.8,193.9,105,193.9z M107.3,150c0-15.8,8.6-29.6,21.3-37c12.2,7.8,20.3,21.5,20.3,37s-8.1,29.2-20.3,37  C115.8,179.6,107.3,165.8,107.3,150z M150,192.8c-7.3,0-14.3-1.9-20.3-5.1c12.2-8,20.3-21.9,20.3-37.6s-8.1-29.6-20.3-37.6  c6-3.3,13-5.1,20.3-5.1s14.3,1.9,20.3,5.1c-12.2,8-20.3,21.9-20.3,37.6s8.1,29.6,20.3,37.6C164.3,190.9,157.4,192.8,150,192.8z   M154.5,150c0-14.9,8.1-27.9,20.1-35c11,7.7,18.2,20.5,18.2,35s-7.2,27.2-18.2,35C162.6,177.9,154.5,164.9,154.5,150z M195,190.5  c-6.6,0-12.7-1.6-18.2-4.4c11-8.1,18.2-21.3,18.2-36.1s-7.2-28-18.2-36.2c5.5-2.8,11.7-4.4,18.2-4.4s12.7,1.6,18.2,4.4  c-11,8.2-18.2,21.4-18.2,36.2s7.2,28,18.2,36.2C207.8,188.9,201.6,190.5,195,190.5z"{}
            }
            g id="SvgjsG1008" featurdarkkey="nameFeature-0"
                transform="matrix(0.45685919503925093,0,0,0.45685919503925093,105.26902527704483,3.2537555592068794)"
                fill="#292929" {
                path style="stroke:black; fill:black"
                    d="M1.6 40 l0 -31.2 l8.04 0 l9 21.36 l9.04 -21.36 l8.04 0 l0 31.2 l-7.96 0 l0 -14.68 l-6.48 14.68 l-5.24 0 l-6.48 -14.68 l0 14.68 l-7.96 0 z M59.212 40.6 c-4.52 0 -8.4 -1.6 -11.64 -4.76 c-3.28 -3.16 -4.92 -7 -4.92 -11.44 s1.64 -8.28 4.92 -11.44 c3.24 -3.16 7.12 -4.76 11.64 -4.76 c4.56 0 8.48 1.6 11.72 4.76 s4.84 7 4.84 11.44 s-1.6 8.28 -4.84 11.44 s-7.16 4.76 -11.72 4.76 z M59.212 32.88 c2.4 0 4.4 -0.84 6.04 -2.48 s2.44 -3.64 2.44 -6 s-0.8 -4.4 -2.44 -6.04 s-3.64 -2.44 -6.04 -2.44 c-2.36 0 -4.36 0.84 -6 2.48 s-2.48 3.64 -2.48 6 s0.84 4.36 2.48 6 s3.64 2.48 6 2.48 z M92.424 40 l0 -23.44 l-9.72 0 l0 -7.76 l27.44 0 l0 7.76 l-9.72 0 l0 23.44 l-8 0 z M117.076 40 l0 -31.2 l8 0 l0 31.2 l-8 0 z M148.568 40.6 c-4.52 0 -8.4 -1.6 -11.64 -4.76 c-3.28 -3.16 -4.92 -7 -4.92 -11.44 s1.64 -8.28 4.92 -11.44 c3.24 -3.16 7.12 -4.76 11.64 -4.76 c4.56 0 8.48 1.6 11.72 4.76 s4.84 7 4.84 11.44 s-1.6 8.28 -4.84 11.44 s-7.16 4.76 -11.72 4.76 z M148.568 32.88 c2.4 0 4.4 -0.84 6.04 -2.48 s2.44 -3.64 2.44 -6 s-0.8 -4.4 -2.44 -6.04 s-3.64 -2.44 -6.04 -2.44 c-2.36 0 -4.36 0.84 -6 2.48 s-2.48 3.64 -2.48 6 s0.84 4.36 2.48 6 s3.64 2.48 6 2.48 z M172.06 40 l0 -31.2 l6.8 0 l13.92 17.44 l0 -17.44 l7.96 0 l0 31.2 l-6.76 0 l-13.92 -17.4 l0 17.4 l-8 0 z M225.404 40 l0 -31.2 l13.76 0 c3.92 0 6.96 1 9.08 3 c2.12 2.04 3.16 4.56 3.16 7.48 c0 3.08 -1.04 5.64 -3.16 7.64 c-0.68 0.64 -1.44 1.16 -2.28 1.6 l6.24 11.48 l-7.56 0 l-5.48 -10.04 l-5.76 0 l0 10.04 l-8 0 z M233.404 23.56 l5.32 0 c3.08 0 4.6 -1.4 4.6 -4.24 c0 -1.2 -0.36 -2.16 -1.12 -2.96 s-1.92 -1.16 -3.48 -1.16 l-5.32 0 l0 8.36 z M259.136 40 l12.64 -31.2 l7.92 0 l12.6 31.2 l-8.72 0 l-2.52 -6.96 l-10.6 0 l-2.6 6.96 l-8.72 0 z M272.936 26.560000000000002 l5.56 -0.04 l-2.76 -7.48 z M299.228 40 l0 -31.2 l6.8 0 l13.92 17.44 l0 -17.44 l7.96 0 l0 31.2 l-6.76 0 l-13.92 -17.4 l0 17.4 l-8 0 z M334.84000000000003 40 l0 -31.2 l8 0 l0 10.76 l9.68 -10.76 l10.32 0 l-12.32 13.4 l12.96 17.8 l-9.92 0 l-8.6 -11.76 l-2.12 2.36 l0 9.4 l-8 0 z"{}
            }
        }
    }
}

pub fn render_video_box(
    id: Uuid,
    title: &String,
    state: &String,
    img: &String,
    processing: i32,
    _user_id: Uuid,
) -> Markup {
    html! {
        div id={"video_"(id.to_string())} class="card bg-secondary d-flex justify-content-center flex-column me-4 mb-4"
            style="width: 200px; height: 150px; border-radius: 8px; overflow: hidden;" {

            div style="position: absolute; top: 0; height: 100%; width: 100%;" {
                @if state == "ready" || state == "playable" {
                    img alt="video title" style="height: 100%; width: 100%; object-fit: cover;" src=(img) {}
                } @else {
                    div {};
                }
            }

            @if state == "ready" {
                div style="flex:1"{}
            } @else {
                div style="flex: 1; z-index:1; width: 100%; height: 100%; background: rgba(0, 0, 0, 0.25);"
                    hx-get=[(if state != "uploading" { Some(format!("/video/{}/box", id)) } else { None } )]
                    hx-target={"#video_"(id.to_string())} hx-swap="outerHTML" hx-trigger="every 2s"
                    class="d-flex justify-content-center align-items-center flex-column p-4" {
                    h4 class="subtitle text-white fs-4 m-0 mt-2"{ (state) }
                    div class="progress mt-2" style="width: 100%;" {
                        div class="progress-bar progress-bar-striped progress-bar-animated bg-primary" role="progressbar"
                            style=(format!("width: {}%", processing))
                            aria-valuenow=(processing) aria-valuemin="0" aria-valuemax="100" {
                            (processing) "%"
                        }
                    }
                }
            }

            div style="background: rgba(0, 0, 0, 0.7); z-index:2; white-space: nowrap;"
                class="d-flex align-items-center" {
                a href={"/video/" (id.to_string()) "/" } hx-boost="true" hx-target="body"
                    class="text-white fs-6 p-2"
                    style="overflow: hidden; text-overflow: ellipsis; flex:1;"
                    { (title) }
            }
        }
    }
}

async fn get_box(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let video = sqlx::query!("select * from video where id = $1", id)
        .fetch_one(&db)
        .await
        .unwrap();
    render_video_box(
        id,
        &video.title,
        &video.state,
        &video.image_link,
        video.processing,
        user_id,
    )
    .into_string()
    .into_response()
}

pub async fn render_video_boxes(
    user_id: Uuid,
    db: &PgPool,
    project_id: Uuid,
    search: Option<String>,
) -> Markup {
    let videos = sqlx::query!(
        r#"select v.id, v.user_id, v.duration, v.title,
                    v.image_link, v.preview_link, v.state, v.processing,
                    u.name as owner_name,
                    case when v.user_id = $1 then v.project_id else v.project_id end as project_id
                    from video v
                left join video_access va on va.video_id = v.id
                left join project_access fa on fa.project_id = v.project_id
                join "user" u on u.id = v.user_id
                where (v.user_id = $1 or va.user_id = $1 or fa.user_id = $1) 
                    and v.deleted = false and v.project_id = $2
                    and (v.title like $3 or $3 is null)
                group by v.id, v.user_id, v.duration, v.title,
                    v.image_link, v.preview_link, v.state, v.processing, 
                    u.name
                order by v.created_at desc"#,
        user_id,
        project_id,
        search
            .filter(|search| search.len() > 0)
            .map(|search| format!("%{}%", search))
    )
    .fetch_all(db)
    .await
    .unwrap();

    html! {
        @if videos.len() > 0 {
            @for v in videos.iter() {
                ( render_video_box(v.id, &v.title, &v.state, &v.image_link, v.processing, user_id) )
            }
        } @else {
            div
                class="d-flex w-100 align-items-center justify-content-center flex-column" {
                i class="bi bi-collection-play h1" {}
                p class="h3" {"This project is empty"}

                button class="btn btn-dark btn-lg upload ml-2 align-items-center d-flex mt-2"
                    hx-post={"/project/"(project_id.to_string())"/upload"}
                    hx-target="#app" hx-swap="none" {
                    i class="bi bi-cloud-plus-fill me-2 text-white"{}
                    span {"Start Uploading"}
                }
            }
        }

    }
}

fn render_write_review() -> Markup {
    let form = html! {
        div id="write_review"
            style="max-width: 800px; "
            onclick="write_review_click(event)"
            class="w-100 m-2 d-flex flex-column justify-content-between p-2
                bg-light border rounded p-2" {

            textarea id="write_review_txt" class="form-control " style="flex: 1;" required rows="4"
                onclick="write_review_click(event)"
                onkeydown="if ( (window.event ? window.event.keyCode : event.which) == 13 && !(window.event ? window.event.shiftKey : event.shiftKey) ) {document.getElementById('write_review_btn').click();}"
                placeholder="Write a review..." name="text" form="write_review" { }

            div class="d-flex justify-content-between mt-2" {

                div class="d-flex flex-row p-1 pr-2 " style="border-radius: 4px; width: auto; align-self: start;" {
                    i class="bi bi-clock-fill me-2" {}
                    span id="timer" hx-preserve="true" { "00:00" }
                }

                div class="d-flex flex-row align-items-center" {
                    button id="write_review_btn" class="btn btn-danger  btn-sm  ms-4 p-0 ps-2 pe-2"
                        onclick="clean_drawings()" style="height:24px" {
                        "Clean"
                    }
                     div class="controls d-flex align-items-center justify-content-flex-end"
                        {

                        @for c in COLORS.iter() {
                            span style="position: relative" {
                                button id={"color_"(c)} onclick="select_color(event)"
                                    type="button"
                                    class="btn btn-sm color-pick btn-outline-dark"
                                    style={"width: 22px; height: 22px; border-radius: 22px;
                                    background: #" (c) "; margin-left: 5px;
                                    font-size: 15px; cursor: pointer; padding: 0; position: relative;
                                    display: flex; justify-content: center; align-items: center"} {

                                }
                            }
                        }
                    }

                    button id="write_review_btn" class="btn btn-dark   ms-4"
                        onclick="submit()" {
                        "Submit"
                    }
                }
            }
        }
    };

    html! {
        div class="d-flex align-items-center justify-content-center w-100" {
            (form)
        }
    }
}

async fn get_player(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user = sqlx::query!(r#"select * from "user" where id = $1"#, user_id)
        .fetch_one(&db)
        .await
        .unwrap();

    let video = sqlx::query!("select * from video where id = $1", id)
        .fetch_one(&db)
        .await
        .unwrap();

    if video.user_id != user_id {
        let has_sharing = sqlx::query!(
            r#"select v.* from video_access v
                where v.user_id = $1 and v.video_id = $2 "#,
            user_id,
            video.id
        )
        .fetch_one(&db)
        .await;

        let has_project_sharing = sqlx::query!(
            r#"select v.* from project_access v
            where v.user_id = $1 and v.project_id = $2 "#,
            user_id,
            video.project_id
        )
        .fetch_one(&db)
        .await;

        if has_sharing.is_err() && has_project_sharing.is_err() {
            return StatusCode::FORBIDDEN.into_response();
        }
    }

    let _video_link = generate_token(&video.id, "playlist.m3u8").await;
    let original_link = generate_token(&video.id, "original").await;
    let video_ratio = video.width as f32 / video.height as f32;

    let header = html! {
        div class="d-flex mb-2 justify-content-between align-items-center"
            style="height: 32px;" {
            a href="/" class="m-0 p-0" {
                (logo())
            }

            h1 class="h5 mb-0" {
                span class="h5 text-dark" { (video.title) }
            }

            div class="d-flex align-items-center" {
                span class="text-muted ml-2 me-4 mb-0" { "uploaded at "(video.created_at.format("%H:%M %d/%m/%Y").to_string()) }

                @if video.user_id == user_id { (video_access::render_access(video.id, user_id, &db).await) }

                button class="btn btn-light ml-2" {
                    i class="fas fa-bell" {}
                }

                div class="dropdown ml-2" {
                    button class="btn btn-light dropdown-toggle" type="button" id="dropdownMenuButton" data-bs-toggle="dropdown" aria-expanded="false" {
                        i class="fas fa-user" {}
                        span { (user.name) }
                    }
                    ul class="dropdown-menu" aria-labelledby="dropdownMenuButton" style="left: -100px;" {
                        li {
                            a href="/profile" class="dropdown-item" { "Profile" }
                        }
                        li {
                            a href="/auth/logout" class="dropdown-item text-danger" { "Logout" }
                        }
                    }
                }
            }
        }

    };

    let video_player = html! {
        div style={"position: relative; width: 100%; max-height: 100%; " (if video_ratio >= 1.0 { "width: 100%;" } else { "height: 100%;" }) " aspect-ratio:"(video_ratio)}
            class="resize_target bg-dark " {
            svg id="canvas" class="resize_target"
                viewBox={"0 0 "(video.width)" "(video.height)}
                preserveAspectRatio="none"
                onmousedown="canvas_mousedown(event)"
                onmousemove="canvas_mousemove(event)"
                onmouseup="canvas_mouseup(event)"
                style="width: 100%; height: 100%; position: absolute; z-index: 999; " {
            }
            video id="player"
                class="video-js resize_target vjs-big-play-centered rounded bg-dark "
                 style="width: 100%; height: 100%;" controls preload="auto" {
                source src=(original_link) type="video/mp4" {}
                // source src=(video_link) type="application/x-mpegURL" {}
            }
            div class="bg-dark" style="height:32px; "{
            }
        }

        div class="" style="height:40px;" id="controls"{
        }

    };

    let page = layout::page(
        html! {},
        html! {
            div class="bg-white p-2 vh-100 d-flex flex-column " id="app"  onclick="first_interact(event)" {
                (header)
                div class="d-flex p-0 flex-fill" {

                    div class="d-flex flex-column me-2 w-100 rounded overflow-hidden pt-2"  {
                        (video_player)
                        (render_write_review())
                    }
                    div id="reviews"
                        class="d-flex flex-column position-relative align-items-center"
                        style="height: 100%; width: 400px; overflow:visible"
                        hx-get="review/"
                        hx-target="#reviews"
                        hx-swap="innerHTML"
                        hx-trigger="load" {
                    }

                }
            }
            link href="/video.css" rel="stylesheet";
            script src="https://cdnjs.cloudflare.com/ajax/libs/video.js/7.10.2/video.min.js" {}
            link href="https://unpkg.com/@silvermine/videojs-quality-selector/dist/css/quality-selector.css" rel="stylesheet";
            script src="https://unpkg.com/@silvermine/videojs-quality-selector/dist/js/silvermine-videojs-quality-selector.min.js" {}
            script src="https://cdnjs.cloudflare.com/ajax/libs/svg.js/3.1.2/svg.min.js" {}
            script src="/player.js" {}
        },
    );

    page.into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:id/", get(get_player))
        .route("/:id/box", get(get_box))
        .nest("/:id/access", video_access::router())
}
