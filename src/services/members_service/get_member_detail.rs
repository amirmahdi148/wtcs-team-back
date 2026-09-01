use crate::utils::sanitize;
use actix_web::cookie::time::OffsetDateTime;
use actix_web::http::StatusCode;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::Row;
use sqlx::{Decode, Error, FromRow};

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

const QUOTE_QUERY: &str = r#"
SELECT
    text , author
    FROM quotes
    WHERE owner = $1

"#;
const EXPERIENCE_QUERY: &str = r#"
SELECT
      id , role , company , start_date , end_date , summary , isteammate
        FROM experiences
        WHERE member_id = $1


"#;

const PROJECTS_QUERY_OTHERS: &str = r#"
SELECT *
FROM (
  SELECT
    p.id,
    p.name,
    p.description,
    p.visibility,
    p.owner_id,
    p.is_personal,
    p.created_at,
    p.stacks
  FROM projects p
  WHERE p.owner_id = $1 AND p.visibility = 'public'

  UNION

  SELECT
    p.id,
    p.name,
    p.description,
    p.visibility,
    p.owner_id,
    p.is_personal,
    p.created_at,
    p.stacks
  FROM projects p
  JOIN project_members pm ON pm.project_id = p.id
  WHERE pm.user_id = $1 AND p.visibility = 'public'
) AS all_projects
ORDER BY name ASC;


"#;
const PROJECTS_QUERY_SELF: &str = r#"
SELECT *
FROM (
  SELECT
    p.id,
    p.name,
    p.description,
    p.visibility,
    p.owner_id,
    p.is_personal,
    p.created_at,
    p.stacks
  FROM projects p
  WHERE p.owner_id = $1 AND p.visibility = 'public'

  UNION

  SELECT
    p.id,
    p.name,
    p.description,
    p.visibility,
    p.owner_id,
    p.is_personal,
    p.created_at,
    p.stacks
  FROM projects p
  JOIN project_members pm ON pm.project_id = p.id
  WHERE pm.user_id = $1 AND p.visibility = 'public'
) AS all_projects
ORDER BY name ASC;


"#;

const KEY_ID: &str = "id";
const KEY_USERNAME: &str = "username";
const KEY_AVATAR: &str = "avatar";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeView {
    pub emoji: String,
    pub title: String,
    pub subtitle: String,
    pub accent: String,
    pub icon_bg: String,
    pub glow: String,
    pub rarity: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillView {
    pub name: String,
    pub area: String,
    pub _type: String,
    pub level: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AboutView {
    pub title: String,
    pub description: String,
    pub icon: String,
    pub order_index: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteView {
    pub text: String,
    pub from: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberView {
    #[serde(flatten)]
    pub member: Map<String, Value>,
    pub slug: Option<String>,
    pub avatar_url: Option<String>,
    pub badges: Vec<BadgeView>,
    pub quotes: Vec<QuoteView>,
    pub skills: Vec<SkillView>,
    pub about: Vec<AboutView>,
    pub experiences: Vec<ExperienceView>,
    pub projects: Vec<ProjectsView>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperienceView {
    pub role: String,
    pub company: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub summary: String,
    pub isTeammate: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsView {
    pub name: String,
    pub description: String,
    pub visibility: Visibility,
    pub is_personal: bool,
    pub created_at: DateTime<Utc>,
    pub stacks: Value,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text")] // چون ستون text است
#[sqlx(rename_all = "lowercase")] // اسم مقادیر در DB lowercase هستند
pub enum Visibility {
    Team,
    Private,
    Public,
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
            let experiences = match fetch_experience(db, member_id).await {
                Ok(items) => items,
                Err(err) => {
                    eprintln!(
                        "[members_service::get_member_detail] fetch_about failed for member_id={}: {}",
                        member_id, err
                    );
                    Vec::new()
                }
            };

            let projects = match fetch_projects(db, member_id).await {
                Ok(items) => items,
                Err(err) => {
                    eprintln!(
                        "[members_service::get_member_detail] fetch_projects failed for member_id={}: {}",
                        member_id, err
                    );
                    Vec::new()
                }
            };

            let member = build_member_payload(
                member_data,
                badges,
                quotes,
                skills,
                about,
                experiences,
                projects,
            );

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
async fn fetch_skills(db: &PgPool, member_id: i32) -> Result<Vec<SkillView>, Error> {
    let rows = sqlx::query(MEMBER_SKILLS_QUERY)
        .bind(member_id)
        .fetch_all(db)
        .await?;
    let skills = rows
        .into_iter()
        .map(|row| SkillView {
            name: row.try_get("name").unwrap_or_default(),
            area: row.try_get("area").unwrap_or_default(),
            _type: row.try_get("_type").unwrap_or_default(),
            level: row.try_get("level").unwrap_or_default(),
        })
        .collect::<Vec<SkillView>>();
    Ok(skills)
}

async fn fetch_quote(db: &PgPool, member_id: i32) -> Result<Vec<QuoteView>, Error> {
    let rows = sqlx::query(QUOTE_QUERY)
        .bind(member_id)
        .fetch_all(db)
        .await?;
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

async fn fetch_experience(db: &PgPool, member_id: i32) -> Result<Vec<ExperienceView>, sqlx::Error> {
    // استفاده از sqlx::Error برای سازگاری
    let rows = sqlx::query(EXPERIENCE_QUERY)
        .bind(member_id)
        .fetch_all(db)
        .await?;

    let mut experiences = Vec::with_capacity(rows.len());
    for row in rows {
        let role: String = row.try_get("role")?;
        let company: String = row.try_get("company")?;
        let start_date: NaiveDate = row.try_get("start_date")?;
        let ending_date_from_db: Option<NaiveDate> = row.try_get("end_date")?;
        let summary: String = row.try_get("summary")?;
        let is_teammate: bool = row.try_get("isteammate")?; // فرض می‌کنیم isTeammate در دیتابیس bool است

        let final_end_date: Option<NaiveDate> = if is_teammate {
            None
        } else {
            ending_date_from_db
        };

        experiences.push(ExperienceView {
            role,
            company,
            start_date,
            end_date: final_end_date,
            summary,
            isTeammate: is_teammate,
        });
    }
    Ok(experiences)
}

async fn fetch_projects(db: &PgPool, member_id: i32) -> Result<Vec<ProjectsView>, Error> {
    let rows = sqlx::query(PROJECTS_QUERY_OTHERS)
        .bind(member_id)
        .fetch_all(db)
        .await?;
    let mut projects = Vec::with_capacity(rows.len());
    for (_, row) in rows.into_iter().enumerate() {
        let name = row.try_get("name")?;
        let description = row.try_get("description")?;
        let visibility = row.try_get("visibility")?;
        let is_personal = row.try_get("is_personal")?;
        let created_at = row.try_get("created_at")?;
        let stacks = row.try_get("stacks")?;

        projects.push(ProjectsView {
            name,
            description,
            visibility,
            is_personal,
            created_at,
            stacks,
        })
    }
    Ok(projects)
}

fn extract_member_id(member_data: &Value) -> Option<i32> {
    member_data
        .get(KEY_ID)
        .and_then(Value::as_i64)
        .map(|v| v as i32)
}

pub fn build_member_payload(
    member_data: Value,
    badges: Vec<BadgeView>,
    quotes: Vec<QuoteView>,
    skills: Vec<SkillView>,
    about: Vec<AboutView>,
    experiences: Vec<ExperienceView>,
    projects: Vec<ProjectsView>,
) -> MemberView {
    let slug = member_data
        .get(KEY_USERNAME)
        .and_then(Value::as_str)
        .map(str::to_owned);
    let mut avatar_url = member_data
        .get(KEY_AVATAR)
        .and_then(Value::as_str)
        .map(str::to_owned);

    // Convert the member JSON object into a Map and normalize only the known
    // DB keys we expose in camelCase.
    let mut member_map = member_data.as_object().cloned().unwrap_or_default();

    // joined_at -> joinedAt
    if !member_map.contains_key("joinedAt") {
        if let Some(v) = member_map.remove("joined_at") {
            member_map.insert("joinedAt".to_string(), v);
        }
    }

    // Special-case mapping: if there is an "avatar" field (legacy), remove
    // it from the flattened member map and populate the `avatar_url` struct
    // field so the serialized output uses camelCase (via the struct field)
    // and we avoid duplicate keys.
    if member_map.contains_key("avatar") {
        if let Some(v) = member_map.remove("avatar") {
            if avatar_url.is_none() {
                if let Some(s) = v.as_str() {
                    avatar_url = Some(s.to_string());
                } else {
                    // Fallback: store the JSON representation
                    avatar_url = Some(v.to_string());
                }
            }
        }
    }

    // If the DB returned "avatar_url", move it into the dedicated struct
    // field so the final JSON has a single camelCase avatarUrl.
    if avatar_url.is_none() {
        if let Some(v) = member_map.remove("avatar_url") {
            if let Some(s) = v.as_str() {
                avatar_url = Some(s.to_string());
            } else {
                avatar_url = Some(v.to_string());
            }
        }
    }

    MemberView {
        member: member_map,
        slug,
        avatar_url,
        badges,
        quotes,
        skills,
        about,
        experiences,
        projects,
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

#[cfg(test)]
pub use build_member_payload as test_build_member_payload;
