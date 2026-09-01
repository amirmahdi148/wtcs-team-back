use http::StatusCode;
use sqlx::PgPool;
use std::num::ParseIntError;

use crate::structs::badge_struct::Badge;

pub async fn get_badges(
    id: Result<i32, ParseIntError>,
    pool: &PgPool,
) -> (StatusCode, Vec<Badge>, String) {
    let real_id = match id {
        Ok(v) if v != 0 => v,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                vec![],
                "id is not valid".to_string(),
            );
        }
    };
    let role = sqlx::query!(r#"SELECT user_role FROM members WHERE id = $1"#, real_id)
        .fetch_one(pool)
        .await
        .map(|r| r.user_role)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

    let role = match role {
        Ok(r) => r,
        Err(code) => {
            return (
                StatusCode::BAD_REQUEST,
                vec![],
                "Error occurred".to_string(),
            );
        }
    };
    if role != "admin" && role != "owner" {
        return (
            StatusCode::BAD_REQUEST,
            vec![],
            "Your role is not high enough.".to_string(),
        );
    }
    let badges = sqlx::query_as!(
        Badge,
        r#"SELECT emoji , title , subtitle , rarity , accent , iconbg , glow FROM badges"#
    )
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    let all_badges = match badges {
        Ok(b) => b,
        Err(code) => {
            return (
                StatusCode::BAD_REQUEST,
                vec![],
                "Error occurred".to_string(),
            );
        }
    };
    let badges_json: Vec<Badge> = all_badges;

    (
        StatusCode::OK,
        badges_json,
        "All Badges returned".to_string(),
    )
}

pub async fn get_user_badges(
    page: usize,
    limit: usize,
    username: String,
    pool: &PgPool,
    id: i32,
) -> (StatusCode, Vec<Badge>, String, i64) {
    if page < 1 || limit == 0 || limit > 3000 {
        return (
            StatusCode::BAD_REQUEST,
            vec![],
            "Page and limit of pagination is not valid".to_string(),
            0,
        );
    }

    let offset = (page - 1) * limit;

    let user_additional =
        match sqlx::query!(r#"SELECT id  FROM members WHERE username = $1"#, username)
            .fetch_optional(pool)
            .await
        {
            Ok(Some(r)) => r,
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    vec![],
                    "No user found".to_string(),
                    0,
                );
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    vec![],
                    "Error occurred".to_string(),
                    0,
                );
            }
        };
    let current_user = match sqlx::query!(r#"SELECT user_role  FROM members WHERE id = $1"#, id)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                vec![],
                "No user found".to_string(),
                0,
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![],
                "Error occurred".to_string(),
                0,
            );
        }
    };

    if current_user.user_role != "owner" && current_user.user_role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            vec![],
            "You can't access here".to_string(),
            0,
        );
    }

    let total_badges = match sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM members_meta WHERE owner = $1 AND meta_type = 'badge'"#,
        user_additional.id
    )
    .fetch_one(pool)
    .await
    {
        Ok(count) => count.unwrap_or(0),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![],
                "Error occurred".to_string(),
                0,
            );
        }
    };

    let badges = match sqlx::query_as!(
        Badge,
        r#"SELECT
        b.emoji,
        b.title,
        b.subtitle,
        b.accent,
        b.iconbg,
        b.glow,
        b.rarity
    FROM badges b
    INNER JOIN members_meta mb ON mb.meta_item_id = b.meta_item_id
    WHERE mb.owner = $1
      AND mb.meta_type = 'badge'
    ORDER BY mb.id ASC
    LIMIT $2 OFFSET $3"#,
        user_additional.id,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool)
    .await
    {
        Ok(list) => list,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![],
                "Error occurred".to_string(),
                0,
            );
        }
    };

    let total_pages = calculate_total_pages(total_badges, limit);

    (
        StatusCode::OK,
        badges,
        format!(
            "Found {} badges for user {}",
            total_badges, user_additional.id
        ),
        total_pages,
    )
}

pub fn calculate_total_pages(total_badges: i64, limit: usize) -> i64 {
    if limit == 0 || total_badges == 0 {
        return 0;
    }

    (total_badges + limit as i64 - 1) / limit as i64
}
