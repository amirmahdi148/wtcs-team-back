use actix_web::web;
use actix_web::http::StatusCode;
use serde_json::{Value, json};
use sqlx::Error;
use sqlx::Row;
use sqlx::PgPool;
use crate::structs::members_struct::AddMembers;
use crate::utils::hasher;
use crate::utils::sanitize;

pub fn show_members() -> Value {
    let members_list = vec![
        json!({ "id": 1, "name": "amir" , "position" : "Developer" }),
        json!({ "id": 2, "name": "Secret User (Yeah its Girl 'Maybe')", "position" : "UI/UX and Logic" }),
    ];

    json!({
        "members": members_list
    })
}

pub async fn add_members(db: &PgPool, payload: web::Json<AddMembers>) -> (StatusCode, Value) {
    let member = payload.into_inner();

    let username = sanitize::username(&member.username, 32);
    let name = sanitize::text(&member.name, 350);
    let bio = sanitize::text(&member.bio, 700);
    let avatar = sanitize::text(&member.avatar, usize::MAX);
    let password = sanitize::text(&member.password, 128);

    if username.is_empty() || name.is_empty() || password.is_empty() {
        return (StatusCode::BAD_REQUEST, json!({
            "ok": false,
            "error": "username, name and password are required"
        }));
    }

    if username.len() < 3 {
        return (StatusCode::BAD_REQUEST, json!({
            "ok": false,
            "error": "username must be at least 3 characters"
        }));
    }

    if password.len() < 8 {
        return (StatusCode::BAD_REQUEST, json!({
            "ok": false,
            "error": "password must be at least 8 characters"
        }));
    }

    let encrypted_pass = match hasher::hash_password(&password) {
        Ok(hash) => hash,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, json!({
                "ok": false,
                "error": err
            }));
        }
    };

    let result = sqlx::query(
        r#"
        INSERT INTO members ( name, bio, password, avatar,username)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, username, name, bio, avatar
        "#,
    )

    .bind(&name)
    .bind(&bio)
    .bind(&encrypted_pass)
    .bind(&avatar)
     .bind(&username)
    .fetch_one(db)
    .await;

    match result {
        Ok(row) => {
            let id: i32 = match row.try_get("id") {
                Ok(v) => v,
                Err(err) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, json!({
                        "ok": false,
                        "error": format!("failed to decode id: {}", err)
                    }));
                }
            };
            let out_username: String = row.get("username");
            let out_name: String = row.get("name");
            let out_bio: String = row.get("bio");
            let out_avatar: String = row.get("avatar");

            (StatusCode::CREATED, json!({
                "ok": true,
                "member": {
                    "id": id,
                    "username": out_username,
                    "name": out_name,
                    "bio": out_bio,
                    "avatar": out_avatar
                }
            }))
        }
        Err(err) => {
            let status = match &err {
                Error::Database(db_err) if db_err.is_unique_violation() => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            (status, json!({
                "ok": false,
                "error": format!("database error: {}", err)
            }))
        }
    }
}
