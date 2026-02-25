use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use sqlx::PgPool;

mod controllers;
mod database;
mod extractors;
mod services;
pub mod structs;
pub mod utils;

struct AppState {
    db: PgPool,
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
            .service(controllers::members_controller::members_get)
            .service(controllers::members_controller::members_me)
            .service(controllers::health_controller::health)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
