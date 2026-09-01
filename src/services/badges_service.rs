use crate::structs::badge_struct::Badge;
use sqlx::{Error as SqlxError, PgPool};

// تابعی برای درج یک badge همراه با مدیریت Transaction و بهینه‌سازی پرفورمنس
pub async fn insert_badge_with_transaction(pool: &PgPool, data: Badge) -> Result<(), SqlxError> {
    let mut tx = pool.begin().await?;

    let badge_insert_result = sqlx::query!(
        r#"
        INSERT INTO badges (emoji, title, subtitle, rarity, accent, iconbg, glow)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        data.emoji,
        data.title,
        data.subtitle,
        data.rarity,
        data.accent,
        data.iconbg,
        data.glow
    )
    .fetch_one(tx.as_mut())
    .await?;

    let badge_id = badge_insert_result.id;

    // تلاش برای یافتن meta_item موجود، در غیر این صورت درج جدید
    let meta_item_id = sqlx::query!(
        r#"
        INSERT INTO meta_items (meta_type, source_id)
        VALUES ('badge', $1)
        ON CONFLICT (source_id, meta_type) DO UPDATE
        SET meta_type = EXCLUDED.meta_type -- یا هر فیلد دیگری که می‌خواهید به‌روز کنید
        RETURNING id
        "#,
        badge_id
    )
    .fetch_one(tx.as_mut())
    .await?
    .id;

    // به‌روزرسانی badge با meta_item_id
    // این مرحله همچنان مهم است تا اطمینان حاصل شود که badge نهایی شده است
    sqlx::query!(
        r#"UPDATE badges SET meta_item_id = $1 WHERE id = $2"#,
        meta_item_id,
        badge_id
    )
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(())
}

// تابعی که تابع بالا را فراخوانی می‌کند و خطاهای احتمالی را مدیریت می‌کند
pub async fn insert_badge_handler(pool: &PgPool, data: Badge) -> Result<String, String> {
    match insert_badge_with_transaction(pool, data).await {
        Ok(_) => Ok("Success".to_string()),
        Err(e) => Err(format!("Failed to insert badge: {}", e)),
    }
}
