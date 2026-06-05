use super::super::*;

#[derive(Clone, sqlx::FromRow)]
struct AdminKnowledgeRow {
    id: i64,
    language: String,
    category: String,
    title: String,
    body: String,
    sort: Option<i64>,
    show: bool,
    created_at: i64,
    updated_at: i64,
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

pub async fn get_category(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_category_response(&state, headers, uri).await {
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

pub async fn show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_show_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn drop(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_drop_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn sort(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_sort_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);

    if let Some(id) = params.get("id").and_then(|value| value.parse::<i64>().ok()).filter(|value| *value > 0) {
        let row = load_admin_knowledge_by_id(state, id).await.map_err(internal_error)?;
        let Some(row) = row else {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"知识不存在"})));
        };
        return Ok(json_value_response(success_response_payload(serialize_admin_knowledge_detail(&row))));
    }

    let rows = load_admin_knowledge_rows(state).await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Array(
        rows.iter()
            .map(serialize_admin_knowledge_list_item)
            .collect::<Vec<_>>()
    ))))
}

async fn build_get_category_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let categories = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT category
         FROM v2_knowledge
         ORDER BY category ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(json!(categories))))
}

async fn build_save_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload
        .as_object()
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let category = obj
        .get("category")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "分类不能为空"))?;
    let language = obj
        .get("language")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "语言不能为空"))?;
    let title = obj
        .get("title")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "标题不能为空"))?;
    let body_text = obj
        .get("body")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "内容不能为空"))?;
    let show = obj.get("show").and_then(|value| value.as_bool()).unwrap_or(false);
    let now = Utc::now().timestamp();

    if let Some(id) = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0) {
        let updated = sqlx::query(
            "UPDATE v2_knowledge
             SET category = ?, language = ?, title = ?, body = ?, `show` = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(&category)
        .bind(&language)
        .bind(&title)
        .bind(&body_text)
        .bind(show)
        .bind(now)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"创建失败"})))?;
        if updated.rows_affected() == 0 {
            return Ok(json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"创建失败"})));
        }
        return Ok(json_value_response(success_response_payload(Value::Bool(true))));
    }

    sqlx::query(
        "INSERT INTO v2_knowledge
            (language, category, title, body, sort, `show`, created_at, updated_at)
         VALUES (?, ?, ?, ?, 0, ?, ?, ?)"
    )
    .bind(&language)
    .bind(&category)
    .bind(&title)
    .bind(&body_text)
    .bind(show)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"创建失败"})))?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_show_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "知识库ID不能为空"))?;

    let row = load_admin_knowledge_by_id(state, id).await.map_err(internal_error)?;
    let Some(row) = row else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"知识不存在"})));
    };
    sqlx::query("UPDATE v2_knowledge SET `show` = ?, updated_at = ? WHERE id = ?")
        .bind(!row.show)
        .bind(Utc::now().timestamp())
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"保存失败"})))?;

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_drop_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "知识库ID不能为空"))?;

    let deleted = sqlx::query("DELETE FROM v2_knowledge WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"知识不存在"})));
    }
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_sort_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let ids = payload
        .get("ids")
        .and_then(|value| value.as_array())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "参数有误"))?;
    if ids.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "参数有误"));
    }

    let parsed_ids = ids
        .iter()
        .filter_map(parse_i64_value)
        .filter(|value| *value > 0)
        .collect::<Vec<_>>();
    if parsed_ids.len() != ids.len() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "参数有误"));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    for (index, knowledge_id) in parsed_ids.iter().enumerate() {
        let updated = sqlx::query("UPDATE v2_knowledge SET sort = ? WHERE id = ?")
            .bind((index + 1) as i64)
            .bind(*knowledge_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"保存失败"})))?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"保存失败"})));
        }
    }
    tx.commit().await.map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"保存失败"})))?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn load_admin_knowledge_rows(
    state: &AppState,
) -> Result<Vec<AdminKnowledgeRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminKnowledgeRow>(
        "SELECT id, language, category, title, body, sort, `show`, created_at, updated_at
         FROM v2_knowledge
         ORDER BY sort ASC, id DESC"
    )
    .fetch_all(&state.db)
    .await
}

async fn load_admin_knowledge_by_id(
    state: &AppState,
    knowledge_id: i64,
) -> Result<Option<AdminKnowledgeRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminKnowledgeRow>(
        "SELECT id, language, category, title, body, sort, `show`, created_at, updated_at
         FROM v2_knowledge
         WHERE id = ?
         LIMIT 1"
    )
    .bind(knowledge_id)
    .fetch_optional(&state.db)
    .await
}

fn serialize_admin_knowledge_list_item(row: &AdminKnowledgeRow) -> Value {
    json!({
        "id": row.id,
        "title": row.title,
        "updated_at": row.updated_at,
        "category": row.category,
        "show": row.show,
    })
}

fn serialize_admin_knowledge_detail(row: &AdminKnowledgeRow) -> Value {
    json!({
        "id": row.id,
        "language": row.language,
        "category": row.category,
        "title": row.title,
        "body": row.body,
        "sort": row.sort,
        "show": row.show,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
    })
}
