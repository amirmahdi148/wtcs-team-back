use crate::AppState;
use crate::extractors::auth::{make_jwt, verify_jwt, Auth};
use crate::services::members_service::{add_members, show_members};
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



#[get("/members/me")]
pub async fn members_me(auth: Auth) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "ok": true,
        "subject": auth.claims.sub,
        "exp": auth.claims.exp
    }))
}
