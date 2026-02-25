use serde_json::{json, Value};

pub fn show_members() -> Value {
    let members_list = vec![
        json!({ "id": 1, "name": "Amir", "position": "Backend Developer" }),
        json!({ "id": 2, "name": "Sample User", "position": "Product Designer" }),
    ];

    json!({
        "members": members_list
    })
}
