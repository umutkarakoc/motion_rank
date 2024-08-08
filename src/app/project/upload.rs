use axum::http::HeaderMap;
use axum::{response::*, routing::*, Router};
use serde_json::json;
use crate::{AppState, service};
use crate::appconfig::ENV;
use crate::logged_user::LoggedUser;

async fn create_upload(
    LoggedUser(_user_id): LoggedUser
) -> impl IntoResponse {
    println!("create_upload");
    let (token, expire, video_id) = service::bunnycdn::create_upload().await;

    println!("token: {}", token);
    let params = json!({
            "AuthorizationSignature": token,
            "VideoId": video_id,
            "LibraryId": ENV.bunny_folder,
            "AuthorizationExpire": expire
        });


    let mut headers = HeaderMap::new();
    headers.insert(
        "hx-trigger", format!(r##"{{"open_file_picker": {}}}"##, params).parse().unwrap(),
    );
    headers.into_response()
}



pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_upload))
}
