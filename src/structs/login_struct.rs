use serde::{Deserialize, Serialize};
use std::panic::Location;

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignUpData {
    pub name: String,
    pub bio: String,
    pub password: String,
    pub avatar_url: String,
    pub username: String,
    pub location: String,
}
