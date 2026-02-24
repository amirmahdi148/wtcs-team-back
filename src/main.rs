use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use serde_json::json;
use sqlx::PgPool;
use dotenv::dotenv;
mod controllers;
mod database;
mod services;
pub mod structs;
pub mod utils;
pub mod extractors;

struct AppState {
    db: PgPool,
}

async fn health(state: web::Data<AppState>) -> impl Responder {
    let _pool = &state.db;

    HttpResponse::Ok().json(json!({
        "status": "ok"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_pool = database::create_pool();
    let app_state = web::Data::new(AppState { db: db_pool });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(controllers::members_controller::list_members)
            .service(controllers::members_controller::add_member)

            .service(controllers::members_controller::members_me)

    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
