use crate::extractors::auth::create_jwt;
use crate::structs::login_struct::LoginData;
use crate::utils::hasher::hash_password;
use crate::utils::password_verifier::verify_password;
use crate::utils::sanitize;
use crate::utils::sanitize::{username, validate_and_clean_string};
use actix_web::http::StatusCode;
use serde_json::{Value, json};
use sqlx::{Error, PgPool};

pub async fn login(data: LoginData, pool: &PgPool) -> (StatusCode, Value) {
    if data.username.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            json!({
            "status": "error",
            "message": "Username cannot be empty"
            }),
        );
    };
    if data.password.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            json!({
                "status": "error",
                "message": "Password cannot be empty"
            }),
        );
    };
    let result_password = match validate_and_clean_string(&data.password) {
        Ok(data) => data,
        Err(e) => {
            return (
                StatusCode::NOT_ACCEPTABLE,
                json!({
                    "message" : e.to_string(),
                }),
            );
        }
    };
    let result_username = username(&data.username, 64);
    let result_database = match sqlx::query!(
        r#"SELECT password , id FROM members WHERE username = $1"#,
        result_username
    )
    .fetch_one(pool)
    .await
    {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "message" : "Error occurred during requesting database"
                }),
            );
        }
    };

    let ok = match verify_password(&data.password, &result_database.password) {
        Ok(()) => true,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                json!({
                    "message" : "Invalid Password"
                }),
            );
        }
    };

    let jwt = match create_jwt(result_database.id, 6048000000) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "message": "Failed to create jwt"
                }),
            );
        }
    };

    (
        StatusCode::OK,
        json!({
            "jwt": jwt,
        }),
    )
}
