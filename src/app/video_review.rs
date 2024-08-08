use super::{layout, video_access};
use crate::logged_user::LoggedUser;
use crate::{service::bunnycdn::generate_token, AppState};
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::{http::StatusCode, response::*, routing::*, Router};
use maud::{html, Markup};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

const COLORS: Lazy<Vec<&'static str>> =
    Lazy::new(|| vec!["0069d5", "bf1282", "c3001b", "00c38d", "ffffff", "000000"]);

fn render_time(time: i32) -> String {
    let time = (time as f32 / 1000f32) as i32;
    let minute = (time / 60);
    let second = (time % 60);

    let second = if second < 10 {
        format!("0{}", second)
    } else {
        second.to_string()
    };
    let minute = if minute < 10 {
        format!("0{}", minute)
    } else {
        minute.to_string()
    };

    return format!("{}:{}", minute, second);
}

async fn render_comments(
    video_id: Uuid,
    review_id: Uuid,
    comment_id: Option<Uuid>,
    db: &PgPool,
) -> Markup {
    let comments = sqlx::query!(
        r#"
        select c.id, c.time, u.id as user_id, c.created_at, 
            u.name as "username!", c.text
        from review_comment c
        join "user" u on u.id = c.user_id
        join review r on r.id = c.review_id
        where c.review_id = $1
        order by c.created_at asc"#,
        review_id
    )
    .fetch_all(db)
    .await
    .unwrap();

    html! {}
}

async fn render_reviews(
    video_id: Uuid,
    user_id: Uuid,
    review_id: Uuid,
    comment_id: Option<Uuid>,
    db: &PgPool,
) -> Markup {
    let video = sqlx::query!("select * from video where id = $1", video_id)
        .fetch_one(db)
        .await
        .unwrap();

    let review = sqlx::query!(
        r#"select r.*, u.name as "username!"
        from review r
        join "user" u on u.id = r.user_id
        where r.id = $1"#,
        review_id
    )
    .fetch_one(db)
    .await
    .unwrap();

    html! {
        div id="side" style="width: 400px; height: calc(100vh - 108px); display: flex; flex-direction: column;"
            class="box mr-4 p-2" {
            div class="p-0 mb-1 is-flex is-justify-content-space-between is-align-items-center"
                style="border-bottom: 2px solid #00d1b2; width: 100%px"{

                a class="button is-small is-white"
                    hx-get={"/video/"(video_id.to_string())"/reviews" }
                    hx-swap="multi:#side,#canvas" hx-push-url="true"
                    { span class="icon" { i class="fa-solid fa-chevron-left" {} } }

                input value=(review.title) type="text" class="input" name="title"
                    hx-patch="" hx-trigger="blur" hx-swap="none" readonly[user_id != review.user_id]
                    style="border: none" {}
            }
            div class="is-flex flex1" style="height: 500px;" {
                (render_comments(video_id, review_id, comment_id, &db).await)
            }

            @if review.id != video_id {
                div class="controls is-flex is-align-items-center is-justify-content-flex-end"{
                    @for c in COLORS.iter() {
                        span style="position: relative" {
                            button id={"color_"(c)} onclick="select_color(event)" class="button color-pick"
                                style={"width: 30px; height: 30px; border-radius: 30px; background: #" (c) "; margin-left: 5px;
                                font-size: 15px; cursor: pointer; padding: 0; position: relative;
                                display: flex; justify-content: center; align-items: center"} {}

                            label id={"color_label_"(c)} for={"color_"(c)} class="fa fa-pen" style={"position: absolute; cursor: pointer; top: 6px; left: 12px; color: " (if c == &"ffffff" { "black" } else { "white" } ) ";"} {}
                        }
                    }
                }
            }

            (render_write_comment(video.id, review_id, None ).await)
        }
    }
}

async fn render_write_comment(video_id: Uuid, review_id: Uuid, text: Option<String>) -> Markup {
    html! {
        form id="write_comment" hx-swap="multi:#side,#canvas" hx-post="comment"
            class="is-flex is-flex-direction-column is-justify-content-space-between"{

            input type="number" id="time" name="time" value="3" style="display:none"  {}
            input type="number" id="duration" name="duration" value="3" style="display:none"  {}

            textarea type="text" id="write_review" class="input mt-2" style="flex:1;" required[text.is_none()] rows="4"
                onkeydown="if ( (window.event ? window.event.keyCode : event.which) == 13 && !(window.event ? window.event.shiftKey : event.shiftKey) ) {document.getElementById('write_comment_btn').click()} "
                placeholder="Write a comment..." name="text" form="write_comment" {
                    (text.unwrap_or("".to_string()))
                }

            div class="is-flex is-justify-content-space-between mt-2"{
                div class="is-flex is-flex-direction-row p-1 pr-2 has-text-dark" style="border-radius: 4px; width: auto; align-self: start"{
                    span class="icon" {i class="fas fa-clock" {} }
                    span id="timer" hx-preserve="true" { "00:00" }
                }

                button #write_comment_btn class="button is-link is-light" {
                    "Submit"
                }
            }
        }
    }
}

async fn create_review(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(video_id): Path<Uuid>,
) -> impl IntoResponse {
    let user = sqlx::query!(r#"select * from "user" where id = $1"#, user_id)
        .fetch_one(&db)
        .await
        .unwrap();

    let review = sqlx::query!(
        r#"insert into review
        (video_id, user_id, title)
        values ($1, $2, $3)
        returning * "#,
        video_id,
        user_id,
        format!("New review from  {}", user.name)
    )
    .fetch_one(&db)
    .await
    .unwrap();

    let mut res = render_reviews(video_id, user_id, review.id, None, &db)
        .await
        .into_string()
        .into_response();
    let headers = res.headers_mut();
    headers.append(
        "hx-push-url",
        format!("/video/{}/review/{}/", video_id, review.id)
            .parse()
            .unwrap(),
    );

    res
}

#[derive(Deserialize)]
struct UpdateReviewParams {
    pub title: String,
}

async fn update_review(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((video_id, review_id)): Path<(Uuid, Uuid)>,
    Form(params): Form<UpdateReviewParams>,
) -> impl IntoResponse {
    let review = sqlx::query!(
        r#"update review set title = $1 where id = $2 and user_id = $3 "#,
        params.title,
        review_id,
        user_id,
    )
    .fetch_one(&db)
    .await
    .unwrap();

    StatusCode::OK.into_response()
}

async fn delete_comment(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((video_id, review_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<PlayerQuery>,
) -> impl IntoResponse {
    if query.comment.is_none() {
        return render_reviews(video_id, user_id, review_id, None, &db)
            .await
            .into_string()
            .into_response();
    }

    sqlx::query!(
        r#"delete from review_comment where id = $1 and user_id = $2 "#,
        query.comment,
        user_id,
    )
    .execute(&db)
    .await
    .unwrap();

    let mut res = render_reviews(video_id, user_id, review_id, None, &db)
        .await
        .into_string()
        .into_response();
    let headers = res.headers_mut();
    headers.append(
        "hx-push-url",
        format!("/video/{}/review/{}/", video_id, review_id)
            .parse()
            .unwrap(),
    );

    res
}

async fn delete_review(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((video_id, review_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    sqlx::query!(
        r#"delete from review where id = $1 and user_id = $2 "#,
        review_id,
        user_id,
    )
    .execute(&db)
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.append(
        "hx-location",
        format!("/video/{}", video_id).parse().unwrap(),
    );

    (StatusCode::FOUND, headers).into_response()
}

#[derive(Deserialize)]
struct CreateTextReviewParams {
    pub time: i32,
    pub text: String,
}

async fn create_comment(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((video_id, review_id)): Path<(Uuid, Uuid)>,
    Form(params): Form<CreateTextReviewParams>,
) -> impl IntoResponse {
    let comment = sqlx::query!(
        r#"insert into review_comment
            (video_id, user_id, text, time, review_id)
            values ($1, $2, $3, $4, $5)
            returning * "#,
        video_id,
        user_id,
        params.text,
        params.time,
        review_id
    )
    .fetch_one(&db)
    .await
    .unwrap();

    let mut res = render_reviews(video_id, user_id, review_id, Some(comment.id), &db)
        .await
        .into_string()
        .into_response();
    let headers = res.headers_mut();
    headers.append(
        "hx-push-url",
        format!(
            "/video/{}/review/{}/?comment={}",
            video_id, review_id, comment.id
        )
        .parse()
        .unwrap(),
    );

    res
}

#[derive(Deserialize)]
struct CreateDrawingParams {
    pub drawing: Vec<f64>,
    pub color: String,
    pub time: i32,
}

async fn create_drawing(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((video_id, review_id)): Path<(Uuid, Uuid)>,
    Json(params): Json<CreateDrawingParams>,
) -> impl IntoResponse {
    let comment = sqlx::query!(
        r#"insert into review_comment
        (video_id, user_id , time, review_id, drawing, color)
        values ($1, $2, $3, $4, $5, $6)
        returning * "#,
        video_id,
        user_id,
        params.time,
        review_id,
        &params.drawing,
        params.color
    )
    .fetch_one(&db)
    .await
    .unwrap();

    Json(json!({
        "url":
            format!(
                "/video/{}/review/{}/?comment={}",
                video_id, review_id, comment.id
            )
    }))

    // let mut res = render_reviews(video_id, user_id, review_id, Some(comment.id), &db)
    //     .await.into_string().into_response();
    // let headers = res.headers_mut();
    // headers.append("hx-push-url", format!("/video/{}/review/{}/?comment={}", video_id, review_id, comment.id).parse().unwrap() );

    // res
}

#[derive(Deserialize)]
struct PlayerQuery {
    comment: Option<Uuid>,
}

async fn get_comments(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((id, review_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<PlayerQuery>,
) -> impl IntoResponse {
    let video = sqlx::query!("select * from video where id = $1", id)
        .fetch_one(&db)
        .await
        .unwrap();

    let mut res = render_comments(video.id, review_id, query.comment, &db)
        .await
        .into_string()
        .into_response();

    let headers = res.headers_mut();

    if let Some(comment) = query.comment {
        headers.append(
            "hx-push-url",
            format!(
                "/video/{}/review/{}/?comment={}",
                video.id, review_id, comment
            )
            .parse()
            .unwrap(),
        );
    } else {
        headers.append(
            "hx-push-url",
            format!("/video/{}/review/{}/", video.id, review_id)
                .parse()
                .unwrap(),
        );
    }

    res
}

async fn get_player(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((id, review_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<PlayerQuery>,
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

    let video_link = generate_token(&video.id, "playlist.m3u8").await;
    let original_link = generate_token(&video.id, "original").await;
    let video_ratio = video.width as f32 / video.height as f32;

    let drawings = sqlx::query!(
        r#"
        select c.drawing as "drawing!", c.color as "color"
        from review_comment c
        join "user" u on u.id = c.user_id
        join review r on r.id = c.review_id
        where c.id = $1 and drawing is not null
        order by c.time asc"#,
        query.comment
    )
    .fetch_all(&db)
    .await
    .unwrap();

    let drawings = drawings
        .iter()
        .map(|drawing| {
            let path = drawing
                .drawing
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if i % 2 == 0 {
                        p.to_string()
                    } else {
                        format!(",{} ", p)
                    }
                })
                .collect::<Vec<String>>()
                .join("");

            (path, drawing.color.to_string())
        })
        .collect::<Vec<(String, String)>>();

    let page = layout::page(
        html! {
            link href="/video.css" rel="stylesheet";
            script src="https://cdnjs.cloudflare.com/ajax/libs/video.js/7.10.2/video.min.js" {}
            link href="https://unpkg.com/@silvermine/videojs-quality-selector/dist/css/quality-selector.css" rel="stylesheet";
            script src="https://unpkg.com/@silvermine/videojs-quality-selector/dist/js/silvermine-videojs-quality-selector.min.js" {}
            script src="https://cdnjs.cloudflare.com/ajax/libs/svg.js/3.1.2/svg.min.js" {}
            script src="/player.js" {}
        },
        html! {
            div class="has-background-light p-4" style="height: 100vh" onclick="first_interact(event)" {
                div class="is-flex box mb-4 is-justify-content-space-between p-2" style="align-items: center; height:60px;"{
                    a href="/" class="m-0 p=0" {
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
                            g id="SvgjsG1008" featurekey="nameFeature-0"
                                transform="matrix(0.45685919503925093,0,0,0.45685919503925093,105.26902527704483,3.2537555592068794)"
                                fill="#292929" {
                                path style="stroke:black; fill:black"
                                    d="M1.6 40 l0 -31.2 l8.04 0 l9 21.36 l9.04 -21.36 l8.04 0 l0 31.2 l-7.96 0 l0 -14.68 l-6.48 14.68 l-5.24 0 l-6.48 -14.68 l0 14.68 l-7.96 0 z M59.212 40.6 c-4.52 0 -8.4 -1.6 -11.64 -4.76 c-3.28 -3.16 -4.92 -7 -4.92 -11.44 s1.64 -8.28 4.92 -11.44 c3.24 -3.16 7.12 -4.76 11.64 -4.76 c4.56 0 8.48 1.6 11.72 4.76 s4.84 7 4.84 11.44 s-1.6 8.28 -4.84 11.44 s-7.16 4.76 -11.72 4.76 z M59.212 32.88 c2.4 0 4.4 -0.84 6.04 -2.48 s2.44 -3.64 2.44 -6 s-0.8 -4.4 -2.44 -6.04 s-3.64 -2.44 -6.04 -2.44 c-2.36 0 -4.36 0.84 -6 2.48 s-2.48 3.64 -2.48 6 s0.84 4.36 2.48 6 s3.64 2.48 6 2.48 z M92.424 40 l0 -23.44 l-9.72 0 l0 -7.76 l27.44 0 l0 7.76 l-9.72 0 l0 23.44 l-8 0 z M117.076 40 l0 -31.2 l8 0 l0 31.2 l-8 0 z M148.568 40.6 c-4.52 0 -8.4 -1.6 -11.64 -4.76 c-3.28 -3.16 -4.92 -7 -4.92 -11.44 s1.64 -8.28 4.92 -11.44 c3.24 -3.16 7.12 -4.76 11.64 -4.76 c4.56 0 8.48 1.6 11.72 4.76 s4.84 7 4.84 11.44 s-1.6 8.28 -4.84 11.44 s-7.16 4.76 -11.72 4.76 z M148.568 32.88 c2.4 0 4.4 -0.84 6.04 -2.48 s2.44 -3.64 2.44 -6 s-0.8 -4.4 -2.44 -6.04 s-3.64 -2.44 -6.04 -2.44 c-2.36 0 -4.36 0.84 -6 2.48 s-2.48 3.64 -2.48 6 s0.84 4.36 2.48 6 s3.64 2.48 6 2.48 z M172.06 40 l0 -31.2 l6.8 0 l13.92 17.44 l0 -17.44 l7.96 0 l0 31.2 l-6.76 0 l-13.92 -17.4 l0 17.4 l-8 0 z M225.404 40 l0 -31.2 l13.76 0 c3.92 0 6.96 1 9.08 3 c2.12 2.04 3.16 4.56 3.16 7.48 c0 3.08 -1.04 5.64 -3.16 7.64 c-0.68 0.64 -1.44 1.16 -2.28 1.6 l6.24 11.48 l-7.56 0 l-5.48 -10.04 l-5.76 0 l0 10.04 l-8 0 z M233.404 23.56 l5.32 0 c3.08 0 4.6 -1.4 4.6 -4.24 c0 -1.2 -0.36 -2.16 -1.12 -2.96 s-1.92 -1.16 -3.48 -1.16 l-5.32 0 l0 8.36 z M259.136 40 l12.64 -31.2 l7.92 0 l12.6 31.2 l-8.72 0 l-2.52 -6.96 l-10.6 0 l-2.6 6.96 l-8.72 0 z M272.936 26.560000000000002 l5.56 -0.04 l-2.76 -7.48 z M299.228 40 l0 -31.2 l6.8 0 l13.92 17.44 l0 -17.44 l7.96 0 l0 31.2 l-6.76 0 l-13.92 -17.4 l0 17.4 l-8 0 z M334.84000000000003 40 l0 -31.2 l8 0 l0 10.76 l9.68 -10.76 l10.32 0 l-12.32 13.4 l12.96 17.8 l-9.92 0 l-8.6 -11.76 l-2.12 2.36 l0 9.4 l-8 0 z"{}
                            }
                        }
                    }

                    h1 class="title is-size-5 mb-0"{
                        span class="title has-text-black" { (video.title) }
                    }

                    div class="" style="align-self: flex-; display: flex; justify-content: center; align-items: center;"{
                        span class="subtitle is-size-6 ml-2 has-text-black-ter m-0 mr-4" { "uploaded at "(video.created_at.format("%H/%M %d/%m/%Y").to_string() ) }

                        @if video.user_id == user_id { (video_access::render_access(video.id, user_id, &db).await) }

                        button class="button is-light ml-2" {
                            i class="fa-solid fa-bell"{}
                        }

                        div class="dropdown is-hoverable ml-2" {
                            div class="dropdown-trigger" {
                                button class="button is-light " {
                                    span class="icon" {i class="fa-solid fa-user"{} }
                                    span { (user.name) }
                                }
                            }
                            div class="dropdown-menu" id="dropdown-menu4" role="menu" style="left: -100px" {
                                div class="dropdown-content has-background-light"  {
                                    a href="/profile" class="dropdown-item has-background-white-ter" {"Profile" }
                                    a href="/auth/logout" class="dropdown-item has-background-light button is-danger is-inverted" {"Logout"}
                                }
                            }
                        }
                    }
                }

                div class="is-flex p-0 " {
                    (render_reviews(video.id, user_id, review_id, query.comment, &db).await)

                    div class="is-flex is-flex-direction-column" style="flex: 1;" {


                        div style="height: calc(100vh - 178px ); position: relative;" id="video_container"
                            class="is-flex is-flex-direction-column is-justify-content-center is-align-items-center has-background-white box " {


                            div style={"position: absolute; max-height: 100%; " (if video_ratio >= 1.0 { "width: 100%"} else { "height: 100%" }) "; aspect-ratio:"(video_ratio)}
                                class="resize_target p-0" {
                                svg id="canvas" class="resize_target" viewBox={"0 0 "(video.width)" "(video.height)} preserveAspectRatio="none"
                                    onmousedown="canvas_mousedown(event)" onmousemove="canvas_mousemove(event)" onmouseup="canvas_mouseup(event)"
                                    style="width: 100%; height: 100%;  position: absolute; z-index: 999;" {
                                        @for drawing in drawings.iter() {
                                            polyline points=(drawing.0) stroke-linecap="round" stroke-width="10" stroke=(drawing.1) fill="none" {}
                                        }
                                    }
                                video id="player" class="videoj-s resize_target vjs-big-play-centered" style="width: 100%; height: 100%"
                                    controls preload="auto" {
                                    source src=(original_link) type="video/mp4" {}
                                    source src=(video_link) type="application/x-mpegURL" {}
                                }
                                div class="has-background-white pb-1" id="timeline"
                                    style="padding-top: 34px; height: 62px; width: 100%; display: none; border-bottom-style: solid; border-bottom-width: 2px;" {}
                            }

                            div class="box has-background-white" #controls
                                style="height: 60px; width: 100%; position: absolute; bottom: 0;  transform: translateY(70px);" {}
                        }

                    }
                }
            }
        },
    );

    page.into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        // .route("/:id", get(get_player) )
        .route("/", post(create_review))
        .route(
            "/:review/",
            get(get_player).patch(update_review).delete(delete_comment),
        )
        .route("/:review/comment", post(create_comment))
        .route("/:review/drawing", post(create_drawing))
}
