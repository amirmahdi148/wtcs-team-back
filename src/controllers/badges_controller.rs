use crate::AppState;
use crate::extractors::auth::Auth;
use crate::services::badges_service;
use crate::services::panel::badges_service::{get_badges, get_user_badges};
use crate::structs::badge_struct::Badge;
use actix_web::web::Json;
use actix_web::{Error as ActixError, HttpResponse, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[get("/badges")]
pub async fn list_badges(auth: Auth, state: web::Data<AppState>) -> impl Responder {
    let id = auth.claims.sub;
    let id_i32 = id.parse::<i32>();
    let (status, badges, status_text) = get_badges(id_i32, &state.db).await;
    HttpResponse::build(status).json(json!({

        "badges" : badges,
        "statusText" : status_text
    }))
}

#[derive(Deserialize)]
pub struct Pagination {
    page: Option<usize>,
    limit: Option<usize>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct PaginationMeta {
    #[serde(rename = "currentPage")]
    pub current_page: usize,
    #[serde(rename = "totalPages")]
    pub total_pages: i64,
    pub limit: usize,
}

pub fn pagination_metadata(current_page: usize, total_pages: i64, limit: usize) -> PaginationMeta {
    PaginationMeta {
        current_page,
        total_pages,
        limit,
    }
}

#[get("/badges/{username}")]
pub async fn user_badges(
    username: web::Path<String>,
    query: web::Query<Pagination>,
    state: web::Data<AppState>,
    auth: Auth,
) -> impl Responder {
    let user_name = username.into_inner();
    let id = auth.claims.sub;
    let id_i32 = id.parse::<i32>().unwrap_or(0);

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);

    let (status, data, message, total_pages) =
        get_user_badges(page, limit, user_name, &state.db, id_i32).await;
    let pagination = pagination_metadata(page, total_pages, limit);

    HttpResponse::build(status).json(serde_json::json!({
        "message": message,
        "data": data,
        "currentPage": pagination.current_page,
        "totalPages": pagination.total_pages,
        "limit": pagination.limit,
    }))
}

#[post("/badges")]
pub async fn create_badges(
    state: web::Data<AppState>,
    data: Json<Badge>,
) -> Result<HttpResponse, ActixError> {
    let badge_data = data.into_inner();

    let result = badges_service::insert_badge_handler(&state.db, badge_data).await;

    match result {
        Ok(created_badge) => Ok(HttpResponse::Ok().json(created_badge)),
        Err(e) => {
            eprintln!("Failed to insert badge: {:?}", e);
            Ok(HttpResponse::InternalServerError().json("Failed to create badge"))
        }
    }
}
