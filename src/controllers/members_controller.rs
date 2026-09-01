use crate::AppState;
use crate::extractors::auth::Auth;
use crate::services::members_service::me_service::get_me_details;
use crate::services::members_service::{add_members, get_member_details, show_members};
use crate::structs::members_struct::AddMembers;
use actix_web::{HttpResponse, Responder, get, post, web};
use serde_json::json;

#[get("/members")]
pub async fn list_members() -> impl Responder {
    HttpResponse::Ok().json(show_members())
}

#[post("/members")]
pub async fn add_member(
    state: web::Data<AppState>,
    payload: web::Json<AddMembers>,
) -> impl Responder {
    let (status, body) = add_members(&state.db, payload).await;
    HttpResponse::build(status).json(body)
}

#[get("/me")]
pub async fn members_me(auth: Auth, state: web::Data<AppState>) -> impl Responder {
    let (status, body) = get_me_details(auth.claims.sub, &state.db).await;
    HttpResponse::build(status).json(body)
}

#[get("/members/{username}")]
pub async fn members_get(
    state: web::Data<AppState>,
    username: web::Path<String>,
) -> impl Responder {
    let (status, body) = get_member_details(&state.db, &username).await;
    HttpResponse::build(status).json(body)
}
