use crate::logged_user::LoggedUser;
use crate::AppState;
use axum::http::HeaderMap;
use axum::{extract::*, http::StatusCode, response::IntoResponse, routing::*, Router};
use serde::Deserialize;
use sqlx::postgres::PgPool;
use maud::html;
use super::layout;

pub async fn get_setting(
    State(db): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
) -> impl IntoResponse {
    let user = sqlx::query!(r#"select * from "user" where id = $1"#, user_id)
        .fetch_one(&db)
        .await
        .unwrap();

    // let page = SettingPage {
    //     email: user.email,
    //     name: user.name
    // };

    // Html(page.render_once().unwrap()).into_response()
    layout::page(html!{p {"todo"}})
}

#[derive(Deserialize)]
pub struct UpdateSettingParam {
    pub name: String,
    pub email: String,
}
pub async fn post_setting(
    State(client): State<PgPool>,
    LoggedUser(user_id): LoggedUser,
    Form(params): Form<UpdateSettingParam>,
) -> impl IntoResponse {
    sqlx::query!(
        r#"update "user" set name=$2 where id = $1"#,
        user_id,
        params.name
    )
    .execute(&client)
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("HX-Refresh", "true".parse().unwrap());
    (StatusCode::FOUND, headers).into_response()
}


pub fn router() -> Router<AppState> {
    Router::new()
        .route("/setting", get(get_setting).post(post_setting))
}
