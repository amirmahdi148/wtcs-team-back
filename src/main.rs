use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use sqlx::PgPool;

mod controllers;
mod database;
mod extractors;
mod services;
pub mod structs;
#[cfg(test)]
pub mod tests;
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
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(app_state.clone())
            .service(controllers::members_controller::list_members)
            .service(controllers::members_controller::add_member)
            .service(controllers::members_controller::members_get)
            .service(controllers::members_controller::members_me)
            .service(controllers::health_controller::health)
            .service(controllers::badges_controller::list_badges)
            .service(controllers::badges_controller::create_badges)
            .service(controllers::auth_controller::login_handler)
            .service(controllers::badges_controller::user_badges)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
