use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AddMembers {
    pub username: String,
    pub name: String,
    pub bio: String,
    pub password: String,
    pub avatar: String,
}
