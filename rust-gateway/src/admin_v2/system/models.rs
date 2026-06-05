use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, sqlx::FromRow)]
pub struct SystemLogRow {
    pub id: i64,
    pub title: String,
    pub level: Option<String>,
    pub uri: String,
    pub data: Option<String>,
    pub context: Option<String>,
    pub created_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub struct FailedJobRow {
    pub id: u64,
    pub connection: String,
    pub queue: String,
    pub failed_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HorizonJobRetryRecord {
    pub id: Option<String>,
    pub status: Option<String>,
    pub retried_at: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HorizonJobRecord {
    pub id: Option<String>,
    pub connection: Option<String>,
    pub queue: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub payload: Option<Value>,
    pub exception: Option<String>,
    pub context: Option<Value>,
    pub failed_at: Option<String>,
    pub completed_at: Option<String>,
    pub retried_by: Vec<HorizonJobRetryRecord>,
    pub reserved_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub index: Option<i64>,
}
