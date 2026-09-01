use crate::services::members_service::test_build_member_payload;
use crate::services::panel::badges_service::calculate_total_pages;
use serde_json::json;

#[test]
fn zero_pages_when_no_badges() {
    assert_eq!(calculate_total_pages(0, 10), 0);
}

#[test]
fn rounds_up_for_partial_page() {
    assert_eq!(calculate_total_pages(15, 10), 2);
    assert_eq!(calculate_total_pages(10, 10), 1);
}

#[test]
fn limit_zero_returns_zero() {
    assert_eq!(calculate_total_pages(5, 0), 0);
}

#[test]
fn test_build_member_payload_key_normalization() {
    let member = json!({
        "id": 1,
        "username": "alice",
        "joined_at": "2020-01-01T00:00:00Z",
        "avatar": "http://example.com/a.png",
        "some_field": "value"
    });

    let mv = test_build_member_payload(
        member,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );

    assert!(mv.member.get("joinedAt").is_some(), "joinedAt not found");
    assert!(
        mv.member.get("joined_at").is_none(),
        "joined_at should be removed"
    );
    assert!(
        mv.member.get("avatar").is_none(),
        "avatar should be removed from map"
    );
    assert_eq!(mv.avatar_url.as_deref(), Some("http://example.com/a.png"));
    assert_eq!(
        mv.member
            .get("some_field")
            .and_then(serde_json::Value::as_str),
        Some("value")
    );
}

#[test]
fn test_build_member_payload_avatar_url_normalization() {
    let member = json!({
        "id": 2,
        "username": "bob",
        "joined_at": "2021-01-01T00:00:00Z",
        "avatar_url": "http://example.com/b.png",
    });

    let mv = test_build_member_payload(
        member,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );

    assert!(mv.member.get("joinedAt").is_some());
    assert!(mv.member.get("avatarUrl").is_none());
    assert_eq!(mv.avatar_url.as_deref(), Some("http://example.com/b.png"));
}
