use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Default)]
pub struct Video {
    pub id: Uuid,
    pub user_id: Uuid,
    pub duration: i32,
    pub title: String,
    pub description: String,
    pub width: i32,
    pub height: i32,
    pub image_link: String,
    pub preview_link: String,
    pub created_at: DateTime<Utc>,
    pub ext_id: String,
    pub state: String,
    pub project_id: Option<Uuid>,
    pub processing: i32,
    pub is_share_link_active: bool,
    pub deleted: bool
}

#[derive(Deserialize, Serialize)]
pub struct VideoAccess {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub video_id: Uuid,
    pub project_id: Option<Uuid>,
    pub user_id: Uuid,
    pub email: Option<String>
}

#[derive(Deserialize, Serialize)]
pub struct ProjectAccess {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub email: Option<String>
}

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub registered_at: Option<DateTime<Utc>>
}


#[derive(Deserialize, Serialize)]
pub struct VideoPermission {
    pub id: Uuid,
    pub email: String,
    pub video_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct LoginCode {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub state: String,
    pub code: String
}

#[derive(Deserialize, Serialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
    pub video_id: Uuid
}

#[derive(Serialize, Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub user_id: Uuid,
    pub video_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub text: String,
    pub time: i32,
    pub duration: i32,
    pub is_published: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ReviewDrawing {
    pub duration: i32,
    pub drawing: Vec<f64>,
    pub color: String,
    pub review_id: Uuid
}


#[derive(Deserialize, Serialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}