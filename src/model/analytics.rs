use serde::Serialize;
use crate::model::member::MemberId;

#[derive(Debug, Serialize)]
pub struct Analytics {
    pub member: Vec<AnalyticsMember>,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsMember {
    pub id: MemberId,
    #[serde(rename = "frontCount")]
    pub front_count: i64,
    #[serde(rename = "frontMinutes")]
    pub front_minutes: i64,
}