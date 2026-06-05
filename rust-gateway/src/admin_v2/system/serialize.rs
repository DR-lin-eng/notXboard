use crate::*;

use super::models::{FailedJobRow, SystemLogRow};

pub fn serialize_system_log_row(row: &SystemLogRow) -> Value {
    json!({
        "id": row.id,
        "level": row.level,
        "title": row.title,
        "uri": row.uri,
        "data": row.data,
        "context": row.context,
        "created_at": row.created_at,
    })
}

pub fn serialize_failed_job_row(row: &FailedJobRow) -> Value {
    json!({
        "id": row.id,
        "connection": row.connection,
        "queue": row.queue,
        "failed_at": row.failed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}
