use crate::appconfig::ENV;
use crate::logged_user::LoggedUser;
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::{http::StatusCode, response::*, routing::*, Router};
use maud::{html, Render};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

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

#[derive(Deserialize)]
struct ReviewQuery {
    selected: Option<Uuid>,
}

async fn render_reviews(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(video_id): Path<Uuid>,
    Query(query): Query<ReviewQuery>,
) -> impl IntoResponse {
    let user = sqlx::query!(r#"select * from "user" where id = $1"#, user_id)
        .fetch_one(&db)
        .await
        .unwrap();

    let _video = sqlx::query!("select * from video where id = $1", video_id)
        .fetch_one(&db)
        .await
        .unwrap();

    let review = {
        if let Some(id) = query.selected {
            sqlx::query!(
                r#"select r.*, u.name as "username!"
                from review r
                join "user" u on u.id = r.user_id
                where r.id = $1"#,
                id
            )
            .fetch_one(&db)
            .await
            .ok()
        } else {
            None
        }
    };

    let reviews = sqlx::query!(
        r#"
        with
        review as (
            select r.*
            from review r
            where r.video_id = $1
            and r.reply_for is null
            and is_deleted = false
        ),
        drawing as (
            select d.review_id, count(*) as count
            from review_drawing as d
            where d.review_id in (select id from review)
            group by d.review_id
        ),
        reply as (
            select d.review_id, count(*) as count
            from review_reply as d
            where d.review_id in (select id from review)
            group by d.review_id
        )
        select
            r.id, r.time, r.percent,
            u.id as user_id, r.created_at,
            u.name as "username!", r.text, r.is_resolved,
            d.count as drawing_count,
            rl.count as reply_count
        from review r
        join "user" u on u.id = r.user_id
        left outer join drawing  as d on d.review_id = r.id
        left outer join reply as rl on rl.review_id = r.id
        where r.id in (select id from review)
        order by r.time asc"#,
        video_id
    )
    .fetch_all(&db)
    .await
    .unwrap();

    let empty = html! {
        div id="reviews" style="flex: 1"
            class="pr-2 is-flex is-align-items-center mt-6 " {
            i class="fa fa-comment-alt is-size-2 " {}
            p class="title is-size-4 mt-2 has-text-black" { "There is no review yet" }
        }
    };

    let _review_timeline = html! {@for r in reviews.iter() {
        @let selected = Some(r.id) == review.as_ref().map(|review| review.id);
        div class="review position-absolute d-flex justify-content-center align-items-center"
        style={
            "width: 30px; height:30px; border-radius: 15px;"
             "left:calc("( r.percent) "% - 15px);"
        } {
        button id={"review_"(r.id) }
            type="button"
            hx-get={"/video/"(video_id.to_string())"/review/"(r.id.to_string())"/" }
            shx-push-url="true"
            data-time=(r.time)
            onclick={"
                set_player_time("(r.time)");
                $('.review .btn.btn-primary').removeClass('btn-primary').addClass('btn-light');
                $(this).removeClass('btn-light').addClass('btn-primary')"
            }

            style="cursor: pointer; width:24px; height:24px; border-radius: 50%;"
            class={
                "btn d-flex justify-content-center align-items-center "
                (if selected {"btn-primary"} else {"btn-light"})  }

            data-bs-toggle="tooltip" data-bs-placement="top"
            data-bs-custom-class="review-popover"
            data-bs-titles={(render_time(r.time)) " " (user.name)}
            data-bs-titless={"<button class='btn'>"(r.text)"</button>"}
            data-bs-title={"
                <i class='bi bi-clock-fill text-primary me-1'></i>
                <b class='text-primary'>"(render_time(r.time))"</b> 
                by <b>"(user.name)"</b>
                <p class='mt-2'>"
                    (if r.drawing_count.unwrap_or_default() > 0 {"<i class='bi bi-pen-fill text-primary me-2'></i>"} else {""})
                    (r.text)
                "</p>"}
            data-bs-html="true"

            {
                h6 class="m-0" tooltip=(r.username) {
                    (r.username.chars().next().unwrap().to_uppercase().to_string() )
                }
            }
        }

    }};

    let review_timeline_v2 = html! {@for r in reviews.iter() {
    div class="review w-100 list-group-item list-group-item-action "
        hx-get={"/video/"(video_id.to_string())"/review/"(r.id.to_string())"/" }
        hx-swap="multi:#canvas"
        shx-push-url="true"
        id={"review_"(r.id) }
        data-time=(r.time)
        style="cursor: pointer; "
        onclick={"
                    set_player_time("(r.time)");
                    $('.review').removeClass('bg-dark-subtle');
                    $(this).addClass('bg-dark-subtle')"
        }
        {
           div class="d-flex flex-row" {
                div style="width:24px; height:24px; border-radius: 50%;"
                    class="d-flex justify-content-center align-items-center bg-dark text-light"
                    {
                        h6 class="m-0" tooltip=(r.username) {
                            (r.username.chars().next().unwrap().to_uppercase().to_string() )
                        }
                    }


               h5 class="ms-2" {
                   (user.name)
               }
               div style="flex:1" {}
               @if r.user_id == user.id {
               i class="bi bi-trash"
                   hx-confirm="Confirm delete"
                   hx-delete={"/video/"(video_id.to_string())"/review/"(r.id.to_string())"/" }
                   hx-swap="none"
                   data-bs-toggle="tooltip" data-bs-placement="left"
                   data-bs-title="Delete Review"
                   "hx-on::after-request"={"$('#review_"(r.id)"').remove()" }
                   {}

               }
           }
           span{
               (r.text)
           }
           div {
               i class="bi bi-clock-fill text-primary me-1"{}
               span class="me-2 text-primary"{
                   (render_time(r.time))
               }
               @if r.drawing_count.unwrap_or_default() > 0 {
               i class="bi bi-pen-fill text-primary me-2"{}
               }
           }

       }
    }};

    let reviews = html! {
            @if reviews.len() == 0 {
                (empty)
            }
            @else {
                div class="list-group w-100 pt-2 flex-fill scroll-y" style="height:0; flex: 1 1 auto; overflow-x visible" {
                (review_timeline_v2)
                }
            }
            script {
                "init_popovers(); check_review_collapse()"
            }

    };

    reviews.render().into_string().into_response()
}

#[derive(Deserialize)]
struct CreateDrawingParams {
    pub drawing: Vec<i32>,
    pub color: String,
}

#[derive(Deserialize)]
struct CreateReviewParams {
    pub time: i32,
    pub percent: f32,
    pub text: String,
    pub reply_for: Option<Uuid>,
    drawings: Vec<CreateDrawingParams>,
}

async fn create_review(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path(video_id): Path<Uuid>,
    Json(params): Json<CreateReviewParams>,
) -> impl IntoResponse {
    let _user = sqlx::query!(r#"select * from "user" where id = $1"#, user_id)
        .fetch_one(&db)
        .await
        .unwrap();

    let review = sqlx::query!(
        r#"insert into review
        (video_id, user_id, text, time, reply_for, percent)
        values ($1, $2, $3, $4, $5, $6)
        returning * "#,
        video_id,
        user_id,
        params.text,
        params.time,
        params.reply_for,
        params.percent
    )
    .fetch_one(&db)
    .await
    .unwrap();

    for drawing in params.drawings {
        sqlx::query!(
            r#"insert into review_drawing
        (review_id, drawing, color)
        values ($1, $2, $3)
        returning * "#,
            review.id,
            &drawing.drawing,
            drawing.color
        )
        .fetch_one(&db)
        .await
        .unwrap();
    }

    Json(json!({
        "url": format!("{}/video/{}/?review={}&open=false", ENV.host, video_id, review.id),
        "video_id": video_id,
        "id": review.id
    }))
}

async fn delete_review(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((_video_id, review_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let _review = sqlx::query!(r#"select * from review where id = $1 "#, review_id,)
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

    // let mut headers = HeaderMap::new();
    // if let Some(parent) = review.reply_for {
    //     headers.append(
    //         "hx-location",
    //         format!("/video/{}/?review={}&open=true", video_id, parent)
    //             .parse()
    //             .unwrap(),
    //     );
    // } else {
    //     headers.append(
    //         "hx-location",
    //         format!("/video/{}/", video_id).parse().unwrap(),
    //     );
    // }

    // headers.into_response()
}

// #[derive(Deserialize)]
// struct _UpdateReviewParams {
//     pub id: Uuid,
//     pub text: String,
//     pub time: i32,
// }

// async fn _update_review(
//     State(db): State<PgPool>,
//     LoggedUser(user_id): LoggedUser,
//     Form(params): Form<UpdateReviewParams>,
// ) -> impl IntoResponse {
//     sqlx::query!(
//         r#"update review set text = $1, time = $2 where id = $3 and user_id = $4 "#,
//         params.text,
//         params.time,
//         params.id,
//         user_id,
//     )
//     .fetch_one(&db)
//     .await
//     .unwrap();

//     StatusCode::OK.into_response()
// }

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

async fn get_review(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Path((id, review_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
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
        review_id
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

    let canvas = html! {
        svg id="canvas" {
            @for drawing in drawings.iter() {
                polyline points=(drawing.0)
                    stroke-linecap="round"
                    stroke-width=(stroke)
                    stroke=(drawing.1)
                    fill="none" {}
            }

        }
    };

    canvas.into_string().into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:id/review/", get(render_reviews).post(create_review))
        .route(
            "/:id/review/:review/",
            get(get_review).delete(delete_review),
        )
}
