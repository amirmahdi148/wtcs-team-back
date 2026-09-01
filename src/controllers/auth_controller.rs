use actix_web::{HttpResponse, Responder, post, web};
use http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;
use sqlx::types::Json;

use crate::AppState;
use crate::services::authentication::login_service::login;
use crate::structs::login_struct::LoginData;

#[post("/login")]
pub async fn login_handler(
    state: web::Data<AppState>,
    body: web::Json<LoginData>,
) -> impl Responder {
    let (status, value) = login(body.into_inner(), &state.db).await;

    HttpResponse::build(status).json(value)
}
