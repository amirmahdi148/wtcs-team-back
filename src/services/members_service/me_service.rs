use crate::services::members_service::get_member_details;
use actix_web::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

pub async fn get_me_details(id: String, pool: &PgPool) -> (StatusCode, Value) {
    if id.to_string().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            json!({
                "message" : "Failed to find member ID"
            }),
        );
    }
    let id_num: i32 = id.parse().unwrap();

    let username = match sqlx::query!("SELECT username FROM members WHERE id=$1", id_num)
        .fetch_one(pool)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "message": format!("Error while requesting database : {}" , e)
                }),
            );
        }
    };

    if username.username.to_string().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            json!({
                "message" : "Database Returned Empty username"
            }),
        );
    }

    let (status, body) = get_member_details(pool, &username.username).await;

    if status.to_string().is_empty() || body.to_string().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            json!({
                "message" : "Data is empty"
            }),
        );
    };

    (status, body)
}
