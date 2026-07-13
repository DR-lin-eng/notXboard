use crate::*;

fn notice_visible_to_user(
    scope_type: &str,
    author_user_id: i64,
    author_is_trusted_global_publisher: bool,
    user_id: i64,
    target_plan_ids: &[i64],
    active_plan_ids: &[i64],
) -> bool {
    (scope_type == "global" && author_is_trusted_global_publisher)
        || author_user_id == user_id
        || (!target_plan_ids.is_empty()
            && target_plan_ids
                .iter()
                .any(|plan_id| active_plan_ids.contains(plan_id)))
}

pub async fn fetch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_fetch_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn save(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_save_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn toggle(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_toggle_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn drop_notice(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_drop_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let current = params.get("current").and_then(|v| v.parse::<i64>().ok()).filter(|v| *v > 0).unwrap_or(1);
    let page_size = params.get("page_size").and_then(|v| v.parse::<i64>().ok()).map(|v| v.clamp(5, 20)).unwrap_or(10);
    let offset = (current - 1) * page_size;

    let active_plan_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT ups.plan_id
         FROM user_plan_subscriptions ups
         WHERE ups.user_id = ?
           AND ups.status = 1
           AND (ups.expired_at IS NULL OR ups.expired_at > ?)
         ORDER BY ups.plan_id"
    )
    .bind(user.id)
    .bind(Utc::now().timestamp())
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let notices = sqlx::query(
        "SELECT notice.id, notice.sort, notice.title, notice.content, notice.show, notice.popup,
                notice.author_user_id, notice.scope_type, notice.target_plan_ids, notice.img_url,
                notice.tags, notice.created_at, notice.updated_at,
                COALESCE(author.is_admin, 0) AS author_is_admin,
                COALESCE(author.is_super_admin, 0) AS author_is_super_admin,
                COALESCE(author.banned, 1) AS author_banned
         FROM v2_notice notice
         LEFT JOIN v2_user author ON author.id = notice.author_user_id
         WHERE notice.show = 1
         ORDER BY notice.sort ASC, notice.id DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    let plan_owners = sqlx::query_as::<_, (i64, i64)>(
        "SELECT id, CAST(owner_user_id AS SIGNED) AS owner_user_id
         FROM v2_plan
         WHERE scope = 'node' AND owner_user_id IS NOT NULL",
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?
    .into_iter()
    .collect::<HashMap<_, _>>();

    let items = notices
        .into_iter()
        .filter_map(|row| {
            let scope_type = row.try_get::<Option<String>, _>("scope_type").ok().flatten().unwrap_or_else(|| "global".to_string());
            let author_user_id = row.try_get::<Option<i64>, _>("author_user_id").ok().flatten().unwrap_or_default();
            let target_plan_ids = row.try_get::<Option<String>, _>("target_plan_ids").ok().flatten().unwrap_or_default();
            let author_banned = row.try_get::<i64, _>("author_banned").unwrap_or(1) != 0;
            let target_ids = if author_banned {
                Vec::new()
            } else {
                parse_notice_target_plan_ids(&target_plan_ids)
                    .into_iter()
                    .filter(|plan_id| plan_owners.get(plan_id) == Some(&author_user_id))
                    .collect::<Vec<_>>()
            };
            let author_has_global_role = row
                .try_get::<i64, _>("author_is_admin")
                .unwrap_or_default()
                != 0
                || row
                    .try_get::<i64, _>("author_is_super_admin")
                    .unwrap_or_default()
                    != 0;
            let author_is_trusted_global_publisher = author_has_global_role && !author_banned;
            let allowed = notice_visible_to_user(
                &scope_type,
                author_user_id,
                author_is_trusted_global_publisher,
                user.id,
                &target_ids,
                &active_plan_ids,
            );
            if std::env::var("NOTICE_DEBUG").ok().as_deref() == Some("1") {
                eprintln!(
                    "NOTICE DEBUG user_id={} notice_id={} scope_type={} author_user_id={} target_ids={:?} active_plan_ids={:?} allowed={}",
                    user.id,
                    row.try_get::<i64, _>("id").unwrap_or_default(),
                    scope_type,
                    author_user_id,
                    target_ids,
                    active_plan_ids,
                    allowed
                );
            }
            if allowed {
                Some(json!({
                    "id": row.try_get::<i64, _>("id").unwrap_or_default(),
                    "sort": row.try_get::<Option<i64>, _>("sort").ok().flatten(),
                    "title": row.try_get::<String, _>("title").unwrap_or_default(),
                    "content": crate::html_safety_support::sanitize_rich_html(
                        &row.try_get::<String, _>("content").unwrap_or_default(),
                    ),
                    "show": row.try_get::<Option<i8>, _>("show").ok().flatten().unwrap_or(0) != 0,
                    "popup": row.try_get::<Option<i8>, _>("popup").ok().flatten().unwrap_or(0) != 0,
                    "author_user_id": row.try_get::<Option<i64>, _>("author_user_id").ok().flatten(),
                    "scope_type": scope_type,
                    "target_plan_ids": target_ids.into_iter().map(|v| Value::from(v)).collect::<Vec<_>>(),
                    "img_url": row.try_get::<Option<String>, _>("img_url").ok().flatten(),
                    "tags": row.try_get::<Option<String>, _>("tags").ok().flatten(),
                    "created_at": row.try_get::<Option<i64>, _>("created_at").ok().flatten(),
                    "updated_at": row.try_get::<Option<i64>, _>("updated_at").ok().flatten(),
                }))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let total = items.len() as i64;
    let data = items
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect::<Vec<_>>();

    Ok(json_value_response(success_response_payload(json!({
        "data": data,
        "total": total,
        "can_publish": false,
        "plan_options": [],
        "manageable": [],
    }))))
}

async fn build_save_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let title = payload.get("title").and_then(|v| v.as_str()).map(|v| v.trim()).unwrap_or("");
    let content = payload.get("content").and_then(|v| v.as_str()).map(|v| v.trim()).unwrap_or("");
    if title.is_empty() || content.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }

    let content = crate::html_safety_support::sanitize_rich_html(content);
    let target_plan_ids = payload.get("target_plan_ids").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if target_plan_ids.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }

    let mut plan_ids = target_plan_ids
        .into_iter()
        .filter_map(|value| parse_i64_value(&value))
        .filter(|value| *value > 0)
        .collect::<Vec<_>>();
    plan_ids.sort_unstable();
    plan_ids.dedup();
    if plan_ids.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let placeholders = vec!["?"; plan_ids.len()].join(",");
    let sql = format!(
        "SELECT id FROM v2_plan WHERE scope = 'node' AND owner_user_id = ? AND id IN ({}) FOR UPDATE",
        placeholders
    );
    let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(user.id);
    for plan_id in &plan_ids {
        query = query.bind(*plan_id);
    }
    let owned_plan_ids = query.fetch_all(&mut *tx).await.map_err(internal_error)?;
    if owned_plan_ids.len() != plan_ids.len() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "target_plan_ids contains plans you do not own"));
    }

    let serialized_targets = serde_json::to_string(&plan_ids).unwrap_or_else(|_| "[]".to_string());
    let tags = payload.get("tags").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let serialized_tags = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());

    let inserted = sqlx::query(
        "INSERT INTO v2_notice (sort, title, content, `show`, popup, author_user_id, scope_type, target_plan_ids, img_url, tags, created_at, updated_at)
         VALUES (0, ?, ?, 1, 0, ?, 'plan_subscribers', ?, NULL, ?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP())"
    )
    .bind(title)
    .bind(&content)
    .bind(user.id)
    .bind(serialized_targets)
    .bind(serialized_tags)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;
    tx.commit().await.map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "id": inserted.last_insert_id(),
        "title": title,
        "content": content,
        "show": true,
        "popup": false,
        "author_user_id": user.id,
        "scope_type": "plan_subscribers",
        "target_plan_ids": plan_ids,
    }))))
}

async fn build_toggle_response(
    state: &AppState,
    notice_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let Some(current_show) = lock_owned_notice_for_update(&mut tx, notice_id, user.id).await? else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Notice not found"));
    };
    let updated = sqlx::query(
        "UPDATE v2_notice
         SET `show` = ?, updated_at = UNIX_TIMESTAMP()
         WHERE id = ? AND author_user_id = ? AND scope_type = 'plan_subscribers'",
    )
        .bind(!current_show)
        .bind(notice_id)
        .bind(user.id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    if updated.rows_affected() != 1 {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Notice not found"));
    }
    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_drop_response(
    state: &AppState,
    notice_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let mut tx = state.db.begin().await.map_err(internal_error)?;
    if lock_owned_notice_for_update(&mut tx, notice_id, user.id)
        .await?
        .is_none()
    {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Notice not found"));
    }
    let deleted = sqlx::query("DELETE FROM v2_notice WHERE id = ? AND author_user_id = ? AND scope_type = 'plan_subscribers'")
        .bind(notice_id)
        .bind(user.id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Notice not found"));
    }
    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn lock_owned_notice_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    notice_id: u64,
    owner_user_id: i64,
) -> Result<Option<bool>, Response<Body>> {
    let row = sqlx::query(
        "SELECT `show`, target_plan_ids
         FROM v2_notice
         WHERE id = ? AND author_user_id = ? AND scope_type = 'plan_subscribers'
         LIMIT 1
         FOR UPDATE",
    )
    .bind(notice_id)
    .bind(owner_user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(internal_error)?;
    let Some(row) = row else {
        return Ok(None);
    };

    let mut plan_ids = row
        .try_get::<Option<String>, _>("target_plan_ids")
        .ok()
        .flatten()
        .map(|raw| parse_notice_target_plan_ids(&raw))
        .unwrap_or_default();
    plan_ids.sort_unstable();
    plan_ids.dedup();
    if plan_ids.is_empty() {
        return Ok(None);
    }

    let placeholders = vec!["?"; plan_ids.len()].join(",");
    let sql = format!(
        "SELECT id FROM v2_plan WHERE scope = 'node' AND owner_user_id = ? AND id IN ({}) FOR UPDATE",
        placeholders
    );
    let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(owner_user_id);
    for plan_id in &plan_ids {
        query = query.bind(*plan_id);
    }
    let owned = query.fetch_all(&mut **tx).await.map_err(internal_error)?;
    if owned.len() != plan_ids.len() {
        return Ok(None);
    }

    Ok(Some(row.try_get::<i8, _>("show").unwrap_or(0) != 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_global_notice_requires_a_current_trusted_publisher() {
        assert!(include_str!("notices.rs").contains("CAST(owner_user_id AS SIGNED) AS owner_user_id"));
        assert!(notice_visible_to_user("global", 7, true, 9, &[], &[]));
        assert!(!notice_visible_to_user("global", 7, false, 9, &[], &[]));
        assert!(notice_visible_to_user("global", 7, false, 7, &[], &[]));
    }

    #[test]
    fn plan_notice_requires_an_active_filtered_target() {
        assert!(notice_visible_to_user(
            "plan_subscribers",
            7,
            false,
            9,
            &[41],
            &[41],
        ));
        assert!(!notice_visible_to_user(
            "plan_subscribers",
            7,
            false,
            9,
            &[41],
            &[42],
        ));
    }
}
