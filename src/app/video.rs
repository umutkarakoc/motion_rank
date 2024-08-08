use super::{layout, video_access};
use crate::appconfig::ENV;
use crate::logged_user::LoggedUser;
use crate::{service::bunnycdn::generate_token, AppState};
use axum::extract::{Path, State};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::*,
    routing::*,
    Router,
};
use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
use maud::{html, Markup};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

const COLORS: Lazy<Vec<&'static str>> =
    Lazy::new(|| vec!["0069d5", "bf1282", "c3001b", "00c38d", "ffffff", "000000"]);

pub fn render_video_box(
    id: Uuid,
    title: &String,
    state: &String,
    img: &String,
    processing: i32,
    _user_id: Uuid,
) -> Markup {
    html! {
        div id={"video_"(id.to_string())} class="card has-background-grey is-flex is-justify-content-center is-flex-direction-column mr-4 mb-4 "
           style="width: 200px; height: 150px; border-radius: 8px; overflow: hidden;" {

            div style="position: absolute; top: 0; height: 100%; width: 100%;" {
                @if state == "ready" || state == "playable" {
                    img alt="video title" style="height: 100%; width: 100%; object-fit:cover" src=(img) {}
                } @else {
                    div {};
                }
            }

            @if state == "ready" { div style="flex:1"{} }
            @else {
                div style="flex: 1; z-index:1; width: 100%; height: 100%; background: #00000040;"
                    hx-get=[(if state != "uploading" { Some(format!("/video/{}/box", id)) } else { None } )]
                    hx-target={"#video_"(id.to_string())} hx-swap="outerHTML" hx-trigger="every 2s"
                    class="is-flex is-justify-content-center is-align-items-center is-flex-direction-column p-4" {
                    h4 class="subtitle has-text-white is-size-4 m-0 mt-2"{ (state) }
                    progress class="progress is-primary mt-2" value=(processing) max="100"{ (processing) "%" }
                }
            }

            div style="background: #000000aa; z-index:2;  white-space: nowrap;" class="is-flex is-align-items-center " {
                a href={"/video/" (id.to_string()) "/" }
                    class="subtitle is-6 has-text-black p-2 has-text-white"
                    style="overflow: hidden;text-overflow: ellipsis; flex:1; color: white !important "
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
            div style="flex: 1; height: 100%"
                class="is-flex is-align-items-center is-justify-content-center is-flex-direction-column" {
                    i class="fas fa-image-film" {}
                    p class="title" {"This project is empty"}

                button class="button is-info upload ml-2" hx-post={"/project/"(project_id.to_string())"/upload"}
                    hx-target="body" hx-swap="none" {
                    span class="icon is-small" { i class="fas fa-video" {} }
                    span {"Start Uploading"}
                }
            }
        }

    }
}

async fn render_write_review() -> Markup {
    html! {
        form id="write_review" hx-swap="multi:#side,#canvas" hx-post="review"
            style="width: 100%"
            class="is-flex is-flex-direction-column is-justify-content-space-between pr-2"{

            input type="number" id="time" name="time" value="3" style="display:none"  {}
            input type="number" id="duration" name="duration" value="3" style="display:none"  {}

            textarea type="text" id="write_review" class="input mt-2" style="flex:1;" required rows="4"
                onclick="write_review_click(event)"
                onkeydown="if ( (window.event ? window.event.keyCode : event.which) == 13 && !(window.event ? window.event.shiftKey : event.shiftKey) ) {document.getElementById('write_review_btn').click()} "
                placeholder="Write a review..." name="text" form="write_review" { }

            div class="is-flex is-justify-content-space-between mt-2"{
                div class="is-flex is-flex-direction-row p-1 pr-2 has-text-dark" style="border-radius: 4px; width: auto; align-self: start"{
                    span class="icon" {i class="fas fa-clock" {} }
                    span id="timer" hx-preserve="true" { "00:00" }
                }

                button #write_review_btn class="button is-link is-light" {
                    "Submit"
                }
            }
        }
    }
}

async fn render_write_reply(reply_for: Uuid, time: i32) -> Markup {
    html! {
        form id="write_reply" hx-swap="multi:#side,#canvas" hx-post="review"
            style="width: 100%;"
            class="pr-2 is-flex is-flex-direction-column is-justify-content-space-between"{

            input type="number" name="time" value=(time) style="display:none"  {}
            input type="number" id="duration" name="duration" value="3" style="display:none"  {}
            input type="hidden" name="reply_for" value=(reply_for.to_string())  {}

            textarea type="text" id="write_reply" class="input mt-2"
                style="flex:1; resize: none; overflow:hidden; display: block"
                required
                oninput="this.style.height = ''; this.style.height = this.scrollHeight + 'px';"
                onkeydown="if ( (window.event ? window.event.keyCode : event.which) == 13 && !(window.event ? window.event.shiftKey : event.shiftKey) ) {document.getElementById('write_reply_btn').click()} "
                placeholder="Write a reply..." name="text" form="write_reply" { }

            div class="is-flex mt-1 is-justify-content-flex-end " {

                button #write_reply_btn class="button is-link is-light is-small" {
                    "Submit"
                }
            }
        }
    }
}

#[derive(Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub user_id: Uuid,
    pub video_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub text: String,
    pub time: i32,
    pub duration: i32,
    pub drawings: Option<serde_json::Value>,
    pub username: String,
}

fn render_time(time: i32) -> String {
    let time = (time as f32 / 1000f32) as i32;
    let minute = time / 60;
    let second = time % 60;

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

async fn render_reviews(video_id: Uuid, user_id: Uuid, query: PlayerQuery, db: &PgPool) -> Markup {
    let video = sqlx::query!("select * from video where id = $1", video_id)
        .fetch_one(db)
        .await
        .unwrap();

    let review = {
        if let Some(id) = query.review {
            sqlx::query!(
                r#"select r.*, u.name as "username!"
                from review r
                join "user" u on u.id = r.user_id
                where r.id = $1"#,
                id
            )
            .fetch_one(db)
            .await
            .ok()
        } else {
            None
        }
    };

    let reply_for = match review.as_ref() {
        Some(r) if r.reply_for.is_some() => sqlx::query!(
            r#"select r.*, u.name as "username!"
                from review r
                join "user" u on u.id = r.user_id
                where r.id = $1"#,
            r.reply_for
        )
        .fetch_one(db)
        .await
        .ok(),
        _ => None,
    };

    let reviews = sqlx::query!(
        r#"
        select r.id, r.time, u.id as user_id, r.created_at, 
            u.name as "username!", r.text
        from review r
        join "user" u on u.id = r.user_id
        where r.video_id = $1 and r.reply_for is null and is_deleted = false
        order by r.time asc"#,
        video_id
    )
    .fetch_all(db)
    .await
    .unwrap();

    let replies = match (review.as_ref(), query.open) {
        (Some(r), Some(true)) => sqlx::query!(
            r#"
                select r.id, r.time, u.id as user_id, r.created_at, 
                    u.name as "username!", r.text
                from review r
                join "user" u on u.id = r.user_id
                where r.reply_for = $1 and is_deleted = false
                order by r.created_at asc"#,
            reply_for.as_ref().map(|r| r.id).unwrap_or(r.id)
        )
        .fetch_all(db)
        .await
        .unwrap(),
        _ => vec![],
    };

    html! {
        div id="side" style="width: 400px; height: calc(100vh - 108px); display: flex; flex-direction: column;"
            class="mr-2" {
            div class="pb-1 mb-1 mr-2 is-flex is-justify-content-space-between is-align-items-center"
                style="border-bottom: 2px solid #00d1b2; width: 100%px"{

                a class="button is-small is-white" href="/"
                    // hx-get={"/" }
                    // hx-swap="multi:#side,#canvas" hx-push-url="true"
                    { span class="icon" { i class="fa-solid fa-chevron-left" {} } }

                span class="subtitle is-size-5 flex1"  {
                    (video.title)"'s Reviews"
                }
            }
            div class="is-flex flex1" style="height: 500px;" {
                @if reviews.len() == 0 {
                    div id="reviews" style="flex: 1" class="pr-2 is-flex is-align-items-center mt-6 is-justify-content-center is-flex-direction-column" {
                        i class="fa fa-comment-alt is-size-2 has-text-dark" {}
                        p class="title is-size-4 mt-2 has-text-black" { "There is no review yet" }
                    }
                }
                @else {
                    div id="reviews" class="pr-2" style="overflow-x: hidden; min-height: 100%; width: 100%"
                         {
                        @for r in reviews.iter() {
                            @let is_open = Some(r.id) == review.as_ref().map(|r| r.id)  || Some(r.id) == reply_for.as_ref().map(|r| r.id);
                            @let selected = Some(r.id) == review.as_ref().map(|r| r.id) ;
                            div class={"media review m-0 p-0 pb-2 is-flex has-text-black is-flex-direction-column is-align-items-stretch "
                                (if selected { "has-background-info-light selected" } else if is_open { "has-background-light" } else {""} ) }
                                data-time=(r.time) {

                                div class={" is-flex-direction-column p-1  "  }
                                    style="height: auto; width: 100%;" {
                                    div class={"p-1 is-flex is-align-items-flex-start is-align-items-center is-justify-content-center "
                                        (if selected { "is-light  is-info" } else {"is-white "} )  }
                                        style="width: 100%; height: auto" {

                                        span class="button is-light is-info is-rounded" tooltip=(r.username)
                                            style="width: 40px; height: 40px; border-radius: 50%" {
                                            (r.username.chars().next().unwrap().to_uppercase().to_string() )
                                        }

                                        span class={"ml-2 is-title is-size-6 has-text-weight-bold is-flex is-flex-direction-column flex1 "} style="text-align:left" {
                                                (r.username)
                                        }


                                        div {
                                            button class="button is-link is-small is-inverted" onclick={"navigator.clipboard.writeText('" (ENV.host) "/video/" (video_id.to_string()) "/?review="(r.id.to_string())"' )"} {
                                                span class="icon" { i class="fa-solid fa-share-from-square" {} }
                                            }

                                            @if r.user_id == user_id {
                                                button class="button is-danger is-small is-inverted ml-1"
                                                    hx-delete={"/video/"(video_id.to_string())"/review/" (r.id.to_string()) }
                                                    hx-swap="multi:#side,#canvas" hx-confirm="Confirm to delete this review"  hx-push-url="true"  {
                                                    span class="icon" { i class="fa fa-trash" {} }
                                                }
                                            }
                                        }
                                    }
                                }

                                    a class={"pt-2 pl-2 pr-2 " }
                                        href={"/video/"(video_id.to_string())"/?open="(if is_open {"true"} else {"false"})"&review=" (r.id.to_string()) }
                                        onclick={"set_player_time("(r.time)")"}
                                        hx-boost="true" hx-swap="multi:#canvas,#side" hx-push-url="true"
                                        style="width: 100%; height: auto; text-align: left; display:inline-block" {
                                        span class={"ml-1 flex1 has-text-dark "(if r.text.is_empty() {"sub-title is-size-6 has-text-grey"} else {""} )}
                                            style="text-align: left; word-wrap: break-word;width: 100%;white-space: break-spaces;" {
                                            (r.text)
                                        }
                                    }

                                div style="width: 100%" {}

                                div class="is-flex p-1 is-justify-content-space-between" style="width: 100%"{
                                    div class="is-flex" {
                                        a class="is-flex is-justify-content-flex-end is-align-items-center"
                                            href={"/video/"(video_id.to_string())"/?open="(if is_open {"true"} else {"false"})"&review=" (r.id.to_string()) }
                                            onclick={"set_player_time("(r.time)")"}
                                            hx-boost="true" hx-swap="multi:#canvas,#side" hx-push-url="true"
                                         {
                                            span class="is-size-7 icon has-text-info m-0" { i class="fas fa-clock" {} }
                                            span class="is-size-7 has-text-info  p-1" style="border-radius: 5px"{
                                                { (render_time(r.time)) }
                                            }
                                        }

                                        button class={"button ml-1 is-small " (if selected { "is-light is-info" } else if is_open { "is-light" } else {"is-white"} ) }
                                            onclick={"set_player_time("(r.time)")"}
                                            hx-get={"/video/"(video_id.to_string())"/?review=" (r.id.to_string())"&open=" (if query.open == Some(true) && is_open {"false"} else {"true"} ) }
                                            hx-push-url="true" hx-swap="multi:#canvas,#reviews"
                                            { "Comments" }
                                    }

                                    @if selected {
                                        div class="controls is-flex is-align-items-center is-justify-content-flex-end" {
                                            @for c in COLORS.iter() {
                                                span style="position: relative" {
                                                    button id={"color_"(c)} onclick="select_color(event)" class="button is-small color-pick"
                                                        style={"width: 22px; height: 22px; border-radius: 22px; background: #" (c) "; margin-left: 5px;
                                                        font-size: 15px; cursor: pointer; padding: 0; position: relative;
                                                        display: flex; justify-content: center; align-items: center"} {}

                                                    label id={"color_label_"(c)} for={"color_"(c)} class="fa fa-pen " style={"position: absolute; cursor: pointer; top: 3px; left: 10px; font-size: 13px; color: " (if c == &"ffffff" { "black" } else { "white" } ) ";"} {}
                                                }
                                            }
                                        }
                                    }
                                }


                                @if query.open == Some(true) && is_open {
                                    div class="ml-5 flex1" {
                                        @for reply in replies.iter() {
                                            @let reply_selected = Some(reply.id) == review.as_ref().map(|r| r.id);
                                            div class={"media review m-0 p-0 pb-2 is-flex has-text-black is-flex-direction-column "
                                                (if reply_selected { "has-background-info-light selected" } else {""} ) }
                                                data-time=(reply.time) {
                                                a class={" is-flex-direction-column p-0 is-light " (if reply_selected { "is-info " } else {""} ) }
                                                    href={"/video/"(video_id.to_string())"/?open=true&review=" (reply.id.to_string()) }
                                                    onclick={"set_player_time("(reply.time)")"}
                                                    hx-boost="true" hx-swap="multi:#canvas,#reviews" hx-push-url="true"
                                                    style="height: auto; width: 100%;" {

                                                    div class={"p-1 is-flex is-align-items-flex-start is-align-items-center is-justify-content-centerq " (if reply_selected{ "is-light  is-info" } else {"is-white "} )  }
                                                        style="width: 100%; height: auto" {

                                                        span class="button is-light is-info is-rounded" tooltip=(reply.username)
                                                            style="width: 40px; height: 40px; border-radius: 50%" {
                                                            (reply.username.chars().next().unwrap().to_uppercase().to_string() )
                                                        }

                                                        span class={"ml-2 is-title is-size-6 has-text-weight-bold is-flex is-flex-direction-column flex1 "} style="text-align:left" {
                                                                (reply.username)
                                                        }


                                                        @if reply.user_id == user_id {
                                                            div class="dropdown" onclick="this.classList.toggle('is-active'); event.preventDefault(); event.stopPropagation()" {
                                                                div class="dropdown-trigger"{
                                                                    button class={"button "  (if reply_selected || selected { "is-info is-light" } else {"is-light"} )}
                                                                        aria-haspopup="true" aria-controls="dropdown-menu" {
                                                                      span class="icon" { i class="fa-solid fa-ellipsis" {} }
                                                                    }
                                                                }
                                                                div class="dropdown-menu" id="dropdown-menu" role="menu" style="margin-left: -150px" {
                                                                    div class="dropdown-content"{
                                                                        button href="#" class="dropdown-item button is-danger is-inverted"
                                                                            hx-delete={"/video/"(video_id.to_string())"/review/" (reply.id.to_string()) }
                                                                            hx-swap="multi:#side,#canvas" hx-confirm="Confirm to delete this review"  hx-push-url="true"  {
                                                                            span class="icon" { i class="fa fa-trash" {} }
                                                                            span { "delete "}
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }

                                                    }

                                                    div class={"pt-2 pl-0 pr-0 " }
                                                        style="width: 100%; height: auto; text-align: left" {
                                                        span class={"ml-1 flex1 has-text-dark "(if reply.text.is_empty() {"sub-title is-size-6 has-text-grey"} else {""} )}
                                                            style="text-align: left; word-wrap: break-word;width: 100%;white-space: break-spaces;" {
                                                            (reply.text)
                                                        }

                                                    }

                                                    div class="is-flex p-1 is-justify-content-space-between" style="width: 100%"{

                                                        div class={"pt-2 pl-0 pr-0 " }
                                                            style="width: 100%; height: auto; text-align: right" {
                                                            span class="is-flex is-justify-content-flex-end is-align-items-center" {
                                                                span class="is-size-7 icon has-text-info m-0" { i class="fas fa-clock" {} }
                                                                span class="is-size-7 has-text-info  p-1" style="border-radius: 5px"{
                                                                    { (HumanTime::from(reply.created_at).to_string() ) }
                                                                }
                                                            }
                                                        }


                                                        @if reply_selected {
                                                            div class="controls is-flex is-align-items-center is-justify-content-flex-end" {
                                                                @for c in COLORS.iter() {
                                                                    span style="position: relative" {
                                                                        button id={"color_"(c)} onclick="select_color(event)" class="button is-small color-pick"
                                                                            style={"width: 22px; height: 22px; border-radius: 22px; background: #" (c) "; margin-left: 5px;
                                                                            font-size: 15px; cursor: pointer; padding: 0; position: relative;
                                                                            display: flex; justify-content: center; align-items: center"} {}

                                                                        label id={"color_label_"(c)} for={"color_"(c)} class="fa fa-pen " style={"position: absolute; cursor: pointer; top: 3px; left: 10px; font-size: 13px; color: " (if c == &"ffffff" { "black" } else { "white" } ) ";"} {}
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }

                                                                                                    }
                                            }

                                        }

                                        @match (review.as_ref(), reply_for.as_ref()) {
                                            (Some(_), Some(reply_for)) => ( render_write_reply(reply_for.id, reply_for.time).await ),
                                            (Some(review), None) => ( render_write_reply( review.id, review.time ).await ),
                                            _ => {}
                                        }
                                    }
                                }

                            }
                        }
                    }
                }

            }

            (render_write_review().await)
        }
    }
}

// div class="controls is-flex is-align-items-center is-justify-content-flex-end"{
//     @for c in COLORS.iter() {
//         span style="position: relative" {
//             button id={"color_"(c)} onclick="select_color(event)" class="button color-pick"
//                 style={"width: 30px; height: 30px; border-radius: 30px; background: #" (c) "; margin-left: 5px;
//                 font-size: 15px; cursor: pointer; padding: 0; position: relative;
//                 display: flex; justify-content: center; align-items: center"} {}

//             label id={"color_label_"(c)} for={"color_"(c)} class="fa fa-pen" style={"position: absolute; cursor: pointer; top: 6px; left: 12px; color: " (if c == &"ffffff" { "black" } else { "white" } ) ";"} {}
//         }
//     }
// }

#[derive(Deserialize)]
struct CreateReviewParams {
    pub time: i32,
    pub text: String,
    pub reply_for: Option<Uuid>,
}

async fn create_review(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(video_id): Path<Uuid>,
    Form(params): Form<CreateReviewParams>,
) -> impl IntoResponse {
    let _user = sqlx::query!(r#"select * from "user" where id = $1"#, user_id)
        .fetch_one(&db)
        .await
        .unwrap();

    let review = sqlx::query!(
        r#"insert into review
        (video_id, user_id, text, time, reply_for)
        values ($1, $2, $3, $4, $5)
        returning * "#,
        video_id,
        user_id,
        params.text,
        params.time,
        params.reply_for
    )
    .fetch_one(&db)
    .await
    .unwrap();

    let mut res = render_reviews(
        video_id,
        user_id,
        PlayerQuery {
            review: Some(review.id),
            open: Some(false),
        },
        &db,
    )
    .await
    .into_string()
    .into_response();

    let headers = res.headers_mut();

    headers.append(
        "hx-push-url",
        format!("/video/{}/?review={}&open=false", video_id, review.id)
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
    let review = sqlx::query!(r#"select * from review where id = $1 "#, review_id,)
        .fetch_one(&db)
        .await
        .unwrap();

    sqlx::query!(
        r#"update review set is_deleted = true where id = $1 and user_id = $2 "#,
        review_id,
        user_id,
    )
    .execute(&db)
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    if let Some(parent) = review.reply_for {
        headers.append(
            "hx-location",
            format!("/video/{}/?review={}&open=true", video_id, parent)
                .parse()
                .unwrap(),
        );
    } else {
        headers.append(
            "hx-location",
            format!("/video/{}/", video_id).parse().unwrap(),
        );
    }

    headers.into_response()
}

#[derive(Deserialize)]
struct CreateDrawingParams {
    pub review_id: Uuid,
    pub drawing: Vec<i32>,
    pub color: String,
}

async fn create_drawing(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(video_id): Path<Uuid>,
    Json(params): Json<CreateDrawingParams>,
) -> impl IntoResponse {
    let drawing = sqlx::query!(
        r#"insert into review_drawing
        (review_id, drawing, color)
        values ($1, $2, $3)
        returning * "#,
        params.review_id,
        &params.drawing,
        params.color
    )
    .fetch_one(&db)
    .await
    .unwrap();

    Json(json!({
        "id": drawing.id
    }))
}

#[derive(Deserialize)]
struct UpdateReviewParams {
    pub id: Uuid,
    pub text: String,
    pub time: i32,
}

async fn update_review(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(video_id): Path<Uuid>,
    Form(params): Form<UpdateReviewParams>,
) -> impl IntoResponse {
    let review = sqlx::query!(
        r#"update review set text = $1, time = $2 where id = $3 and user_id = $4 "#,
        params.text,
        params.time,
        params.id,
        user_id,
    )
    .fetch_one(&db)
    .await
    .unwrap();

    StatusCode::OK.into_response()
}

// async fn get_reviews(
//     State(db): State<PgPool>,
//     LoggedUser(user_id): LoggedUser,
//     Path(id): Path<Uuid>
// ) -> impl IntoResponse {
//     let mut res = render_reviews(id, user_id, &db)
//         .await.into_string().into_response();

//     let headers = res.headers_mut();
//     headers.append("hx-push-url", format!("/video/{}/", id).parse().unwrap() );

//     res
// }

#[derive(Deserialize, Default)]
struct PlayerQuery {
    review: Option<Uuid>,
    open: Option<bool>,
}

async fn get_player(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(id): Path<Uuid>,
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

    let drawings = sqlx::query!(
        r#"
        select d.*
        from review_drawing d
        where d.review_id = $1 "#,
        query.review
    )
    .fetch_all(&db)
    .await
    .unwrap();

    let stroke = video.width as f32;
    let stroke = stroke * 0.01f32;
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

    let video_link = generate_token(&video.id, "playlist.m3u8").await;
    let original_link = generate_token(&video.id, "original").await;
    let video_ratio = video.width as f32 / video.height as f32;

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
            div class="has-background-white p-2" style="height: 100vh" onclick="first_interact(event)" {
                div class="is-flex mb-2 is-justify-content-space-between" style="align-items: center; height:60px;"{
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
                    (render_reviews(video.id, user_id, query, &db).await)

                    div class="is-flex is-flex-direction-column" style="flex: 1;" {


                        div style="height: calc(100vh - 178px ); position: relative;" id="video_container"
                            class="is-flex is-flex-direction-column is-justify-content-center is-align-items-center has-background-white " {


                            div style={"position: absolute; max-height: 100%; " (if video_ratio >= 1.0 { "width: 100%"} else { "height: 100%" }) "; aspect-ratio:"(video_ratio)}
                                class="resize_target p-0" {
                                svg id="canvas" class="resize_target" viewBox={"0 0 "(video.width)" "(video.height)} preserveAspectRatio="none"
                                    onmousedown="canvas_mousedown(event)" onmousemove="canvas_mousemove(event)" onmouseup="canvas_mouseup(event)"
                                    style="width: 100%; height: 100%;  position: absolute; z-index: 999;" {
                                        @for drawing in drawings.iter() {
                                            polyline points=(drawing.0) stroke-linecap="round" stroke-width=(stroke) stroke=(drawing.1) fill="none" {}
                                        }

                                    }
                                video id="player" class="video-js resize_target vjs-big-play-centered" style="width: 100%; height: 100%"
                                    controls preload="auto" {
                                    // source src=(original_link) type="video/mp4" {}
                                    source src=(video_link) type="application/x-mpegURL" {}
                                }
                                div class="has-background-white pb-1" id="timeline"
                                    style="padding-top: 34px; height: 62px; width: 100%; display: none; border-bottom-style: solid; border-bottom-width: 2px;" {}
                            }

                            div class="has-background-white" #controls
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
        .route("/:id/", get(get_player))
        .route("/:id/box", get(get_box))
        .route("/:id/review", post(create_review))
        .route("/:id/review/:review", delete(delete_review))
        .route("/:id/drawing", post(create_drawing))
        .nest("/:id/access", video_access::router())
    // .nest("/:id/review/", video_review::router())
}
