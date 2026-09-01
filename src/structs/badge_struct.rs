use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Badge {
    pub emoji: String,
    pub title: String,
    pub subtitle: String,
    pub rarity: String,
    pub accent: String,
    #[serde(rename = "iconBg")]
    pub iconbg: String,
    pub glow: String,
}
