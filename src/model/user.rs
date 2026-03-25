use serde::Deserialize;

pub type UserId = i32;

#[derive(Debug, Deserialize)]
pub struct UserFilter {
    #[serde(rename = "userId")]
    pub user_id: Option<UserId>,
}

#[derive(Debug, Deserialize)]
pub struct FullDataSelectionFilter {
    pub full: Option<bool>,
}