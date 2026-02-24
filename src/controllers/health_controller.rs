use actix_web::{HttpResponse, Responder, get, web};

use crate::AppState;
use crate::services::health_service;

#[get("/health")]
pub async fn health(state: web::Data<AppState>) -> impl Responder {
    let payload = health_service::health_check(&state.db);
    HttpResponse::Ok().json(payload)
}
