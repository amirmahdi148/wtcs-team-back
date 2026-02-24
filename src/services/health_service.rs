use serde_json::{Value, json};
use sqlx::PgPool;

pub fn health_check(_db: &PgPool) -> Value {
    json!({
        "status": "ok"
    })
}
