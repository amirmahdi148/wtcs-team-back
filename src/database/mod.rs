use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub fn create_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url)
        .expect("failed to create PostgreSQL pool")
}
