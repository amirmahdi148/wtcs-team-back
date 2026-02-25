use crate::utils::sanitize;
use actix_web::http::StatusCode;
use serde::Serialize;
use serde_json::Map;
use serde_json::{json, Value};
use sqlx::Error;
use sqlx::PgPool;
use sqlx::Row;

const MEMBER_QUERY: &str = r#"
    SELECT to_jsonb(m) - 'password' AS member
    FROM members m
    WHERE m.username = $1
"#;

const MEMBER_BADGES_QUERY: &str = r#"
    SELECT
        b.emoji,
        b.title,
        b.subtitle,
        b.accent,
        b."iconbg" AS iconbg,
        b.glow,
        b.rarity
    FROM badges b
    INNER JOIN members_meta mb ON mb.meta_item_id = b.meta_item_id
    WHERE mb.owner = $1
      AND mb.meta_type = 'badge'
    ORDER BY mb.id ASC
"#;

const MEMBER_SKILLS_QUERY: &str = r#"
    SELECT
        s.name,
        s.area,
        s.type AS _type,
        ms.level
    FROM skills s
    INNER JOIN members_meta ms ON ms.meta_item_id = s.meta_item_id
    WHERE ms.owner = $1
        AND ms.meta_type = 'skill'
    ORDER BY ms.id ASC

 "#;

const MEMBER_ABOUT_QUERY: &str = r#"
    SELECT
        a.title,
        a.description,
        a.icon,
        a.order_index
    FROM about a
    INNER JOIN members_meta ma ON ma.meta_item_id = a.meta_item_id
    WHERE ma.owner = $1
      AND ma.meta_type = 'about'
    ORDER BY ma.id ASC
"#;

const QUOTE_QUERY : &str = r#"
SELECT
    text , author
    FROM quotes
    WHERE owner = $1

"#;
const KEY_ID: &str = "id";
const KEY_USERNAME: &str = "username";
const KEY_AVATAR: &str = "avatar";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BadgeView {
    emoji: String,
    title: String,
    subtitle: String,
    accent: String,
    icon_bg: String,
    glow: String,
    rarity: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SkillView {
    name : String,
    area : String,
    _type : String,
    level: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AboutView {
    title: String,
    description: String,
    icon: String,
    order_index: i32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuoteView {
    text : String,
    from : String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MemberView {
    #[serde(flatten)]
    member: Map<String, Value>,
    slug: Option<String>,
    avatar_url: Option<String>,
    badges: Vec<BadgeView>,
    quotes: Vec<QuoteView>,
    skills: Vec<SkillView>,
    about: Vec<AboutView>,
}

pub async fn get_member_detail(db: &PgPool, username: &str) -> (StatusCode, Value) {
    let user = sanitize::username(username, 32);
    if user.is_empty() {
        return bad_request("username is required");
    }

    match fetch_member(db, &user).await {
        Ok(member_data) => {
            let member_id = match extract_member_id(&member_data) {
                Some(id) => id,
                None => return internal_error("failed to decode member id"),
            };

            let badges = match fetch_badges(db, member_id).await {
                Ok(items) => items,
                Err(err) => {
                    eprintln!(
                        "[members_service::get_member_detail] fetch_badges failed for member_id={}: {}",
                        member_id, err
                    );
                    Vec::new()
                }
            };
            let quotes = match fetch_quote(db, member_id).await {
                Ok(items) => items,
                Err(err) => {
                    eprintln!(
                        "[members_service::get_member_detail] fetch_quote failed for member_id={}: {}",
                        member_id, err
                    );
                    Vec::new()
                }
            };
            let skills = match fetch_skills(db, member_id).await {
                Ok(items) => items,
                Err(err) => {
                    eprintln!(
                        "[members_service::get_member_detail] fetch_skills failed for member_id={}: {}",
                        member_id, err
                    );
                    Vec::new()
                }
            };
            let about = match fetch_about(db, member_id).await {
                Ok(items) => items,
                Err(err) => {
                    eprintln!(
                        "[members_service::get_member_detail] fetch_about failed for member_id={}: {}",
                        member_id, err
                    );
                    Vec::new()
                }
            };
            let member = build_member_payload(member_data, badges, quotes, skills, about);

            (
                StatusCode::OK,
                json!({
                    "ok": true,
                    "member": member
                }),
            )
        }
        Err(Error::RowNotFound) => not_found("member not found"),
        Err(err) => internal_error(&format!("database error: {}", err)),
    }
}

async fn fetch_member(db: &PgPool, username: &str) -> Result<Value, Error> {
    let row = sqlx::query(MEMBER_QUERY)
        .bind(username)
        .fetch_one(db)
        .await?;

    Ok(row.get("member"))
}
async fn fetch_skills (db : &PgPool , member_id : i32) -> Result<Vec<SkillView> , Error> {
    let rows = sqlx::query(MEMBER_SKILLS_QUERY).bind(member_id).fetch_all(db).await?;
    let skills = rows
        .into_iter()
        .map(|row| SkillView {
            name : row.try_get("name").unwrap_or_default(),
            area : row.try_get("area").unwrap_or_default(),
            _type : row.try_get("_type").unwrap_or_default(),
            level: row.try_get("level").unwrap_or_default(),
        }).collect::<Vec<SkillView>>();
    Ok(skills)
}

async fn fetch_quote (db : &PgPool , member_id : i32) -> Result<Vec<QuoteView>, Error> {
    let rows = sqlx::query(QUOTE_QUERY).bind(member_id).fetch_all(db).await?;
    let quotes = rows
        .into_iter()
        .map(|row| QuoteView {
            text: row.try_get("text").unwrap_or_default(),
            from: row.try_get("author").unwrap_or_default(),
        })
        .collect::<Vec<QuoteView>>();
    Ok(quotes)
}

async fn fetch_about(db: &PgPool, member_id: i32) -> Result<Vec<AboutView>, Error> {
    let rows = sqlx::query(MEMBER_ABOUT_QUERY)
        .bind(member_id)
        .fetch_all(db)
        .await?;
    let about = rows
        .into_iter()
        .map(|row| AboutView {
            title: row.try_get("title").unwrap_or_default(),
            description: row.try_get("description").unwrap_or_default(),
            icon: row.try_get("icon").unwrap_or_default(),
            order_index: row.try_get("order_index").unwrap_or_default(),
        })
        .collect::<Vec<AboutView>>();
    Ok(about)
}


async fn fetch_badges(db: &PgPool, member_id: i32) -> Result<Vec<BadgeView>, Error> {
    eprintln!(
        "[members_service::fetch_badges] start member_id={} query={}",
        member_id,
        MEMBER_BADGES_QUERY.replace('\n', " ")
    );

    let rows = sqlx::query(MEMBER_BADGES_QUERY)
        .bind(member_id)
        .fetch_all(db)
        .await?;

    eprintln!(
        "[members_service::fetch_badges] fetched_rows={} member_id={}",
        rows.len(),
        member_id
    );

    let mut badges = Vec::with_capacity(rows.len());
    for (index, row) in rows.into_iter().enumerate() {
        let emoji: String = row.try_get("emoji")?;
        let title: String = row.try_get("title")?;
        let subtitle: String = row.try_get("subtitle")?;
        let accent: String = row.try_get("accent")?;
        let icon_bg: String = row.try_get("iconbg")?;
        let glow: String = row.try_get("glow")?;
        let rarity: String = row.try_get("rarity")?;

        eprintln!(
            "[members_service::fetch_badges] row={} member_id={} title={} rarity={}",
            index, member_id, title, rarity
        );

        badges.push(BadgeView {
            emoji,
            title,
            subtitle,
            accent,
            icon_bg,
            glow,
            rarity,
        });
    }

    Ok(badges)
}

fn extract_member_id(member_data: &Value) -> Option<i32> {
    member_data.get(KEY_ID).and_then(Value::as_i64).map(|v| v as i32)
}

fn build_member_payload(
    member_data: Value,
    badges: Vec<BadgeView>,
    quotes: Vec<QuoteView>,
    skills: Vec<SkillView>,
    about: Vec<AboutView>,
) -> MemberView {
    let slug = member_data
        .get(KEY_USERNAME)
        .and_then(Value::as_str)
        .map(str::to_owned);
    let avatar_url = member_data
        .get(KEY_AVATAR)
        .and_then(Value::as_str)
        .map(str::to_owned);

    MemberView {
        member: member_data.as_object().cloned().unwrap_or_default(),
        slug,
        avatar_url,
        badges,
        quotes,
        skills,
        about,
    }
}

fn bad_request(message: &str) -> (StatusCode, Value) {
    (
        StatusCode::BAD_REQUEST,
        json!({
            "ok": false,
            "error": message
        }),
    )
}

fn not_found(message: &str) -> (StatusCode, Value) {
    (
        StatusCode::NOT_FOUND,
        json!({
            "ok": false,
            "error": message
        }),
    )
}

fn internal_error(message: &str) -> (StatusCode, Value) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({
            "ok": false,
            "error": message
        }),
    )
}

pub async fn get_member_details(db: &PgPool, username: &str) -> (StatusCode, Value) {
    get_member_detail(db, username).await
}
