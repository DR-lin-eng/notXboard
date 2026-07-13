use crate::*;
use base64::engine::general_purpose::STANDARD as BASE64;
use hmac::{Hmac, Mac};
use sha2::Sha256;

const MACHINE_BOOTSTRAP_TTL_SECONDS: i64 = 10 * 60;
type HmacSha256 = Hmac<Sha256>;

pub(crate) struct MachineBootstrapGrant {
    pub(crate) ticket: String,
    pub(crate) query: String,
    pub(crate) expires_in: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct MachineBootstrapTicket {
    kind: String,
    issuer_user_id: i64,
    subject_id: u64,
    credential: String,
    asset_id: String,
    expires_at: i64,
}

struct SignedBootstrapQuery {
    kind: String,
    expires_at: i64,
    asset_id: String,
}

pub(crate) async fn issue_machine_bootstrap_ticket(
    state: &AppState,
    kind: &str,
    issuer_user_id: i64,
    subject_id: u64,
    credential: &str,
) -> Result<MachineBootstrapGrant, String> {
    if !is_supported_kind(kind)
        || issuer_user_id <= 0
        || subject_id == 0
        || credential.trim().len() < 16
    {
        return Err("invalid machine bootstrap subject".to_string());
    }

    let ticket = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let asset_id = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let expires_at = Utc::now().timestamp() + MACHINE_BOOTSTRAP_TTL_SECONDS;
    let signature = sign_bootstrap_query(&state.app_key, kind, expires_at, &asset_id)
        .ok_or_else(|| "APP_KEY must be a base64 encoded 32-byte key".to_string())?;
    let query = format!(
        "mb_v=1&mb_kind={kind}&mb_exp={expires_at}&mb_asset={asset_id}&mb_sig={signature}"
    );
    let payload = serde_json::to_string(&MachineBootstrapTicket {
        kind: kind.to_string(),
        issuer_user_id,
        subject_id,
        credential: credential.trim().to_string(),
        asset_id: asset_id.clone(),
        expires_at,
    })
    .map_err(|err| format!("serialize machine bootstrap ticket failed: {err}"))?;
    redis_setex_string(
        state,
        &machine_bootstrap_cache_key(&ticket),
        MACHINE_BOOTSTRAP_TTL_SECONDS,
        &payload,
    )
    .await?;
    redis_setex_string(
        state,
        &machine_bootstrap_asset_cache_key(&asset_id),
        MACHINE_BOOTSTRAP_TTL_SECONDS,
        &payload,
    )
    .await?;
    Ok(MachineBootstrapGrant {
        ticket,
        query,
        expires_in: MACHINE_BOOTSTRAP_TTL_SECONDS,
    })
}

pub(crate) async fn installer_asset_authorized(state: &AppState, uri: &Uri) -> bool {
    let Some(expected_kind) = installer_kind_for_path(uri.path()) else {
        return false;
    };
    let Some(signed) = parse_and_verify_bootstrap_query(uri, &state.app_key) else {
        return false;
    };
    if signed.kind != expected_kind {
        return false;
    }
    let Ok(Some(raw)) = redis_get_string(
        state,
        &machine_bootstrap_asset_cache_key(&signed.asset_id),
    )
    .await else {
        return false;
    };
    let Ok(payload) = serde_json::from_str::<MachineBootstrapTicket>(&raw) else {
        return false;
    };
    payload_matches_query(&payload, &signed)
        && machine_bootstrap_subject_is_current(state, &payload).await
}

pub(crate) async fn exchange(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(kind): axum::extract::Path<String>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_exchange_response(&state, &kind, &headers, &uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_exchange_response(
    state: &AppState,
    kind: &str,
    headers: &HeaderMap,
    uri: &Uri,
) -> Result<Response<Body>, Response<Body>> {
    if !is_supported_kind(kind) {
        return Err(no_store_error(StatusCode::NOT_FOUND, "Not found"));
    }
    let signed = parse_and_verify_bootstrap_query(uri, &state.app_key)
        .filter(|signed| signed.kind == kind)
        .ok_or_else(|| no_store_error(StatusCode::NOT_FOUND, "Not found"))?;
    let ticket = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| valid_ticket(value))
        .ok_or_else(|| no_store_error(StatusCode::NOT_FOUND, "Not found"))?;
    let raw = redis_getdel_string(state, &machine_bootstrap_cache_key(ticket))
        .await
        .map_err(|err| {
            error!("machine bootstrap ticket exchange failed: {err}");
            no_store_error(StatusCode::SERVICE_UNAVAILABLE, "Bootstrap unavailable")
        })?
        .ok_or_else(|| no_store_error(StatusCode::NOT_FOUND, "Not found"))?;
    let payload = serde_json::from_str::<MachineBootstrapTicket>(&raw)
        .map_err(|_| no_store_error(StatusCode::NOT_FOUND, "Not found"))?;
    if !payload_matches_query(&payload, &signed)
        || !machine_bootstrap_subject_is_current(state, &payload).await
    {
        return Err(no_store_error(StatusCode::NOT_FOUND, "Not found"));
    }
    let _ = redis_del_string(
        state,
        &machine_bootstrap_asset_cache_key(&payload.asset_id),
    )
    .await;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header("Cache-Control", "no-store")
        .header("Pragma", "no-cache")
        .header("X-Content-Type-Options", "nosniff")
        .body(Body::from(payload.credential))
        .unwrap_or_else(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error")))
}

async fn machine_bootstrap_subject_is_current(
    state: &AppState,
    payload: &MachineBootstrapTicket,
) -> bool {
    let result = match payload.kind.as_str() {
        "v2bx" => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*)
                 FROM server_nodes node
                 JOIN v2_user owner ON owner.id = node.user_id
                 WHERE node.id = ?
                   AND owner.id = ?
                   AND node.v2bx_token = ?
                   AND owner.banned = 0
                   AND (
                       owner.api_key IS NULL
                       OR owner.api_key = ''
                       OR node.v2bx_token <> owner.api_key
                   )
                   AND NOT EXISTS (
                       SELECT 1 FROM server_nodes duplicate
                       WHERE duplicate.v2bx_token = node.v2bx_token
                         AND duplicate.id <> node.id
                   )",
            )
            .bind(payload.subject_id)
            .bind(payload.issuer_user_id)
            .bind(&payload.credential)
            .fetch_one(&state.db)
            .await
        }
        "tcping" => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*)
                 FROM tcping_agents agent
                 JOIN v2_user owner ON owner.id = agent.user_id
                 WHERE agent.id = ?
                   AND owner.id = ?
                   AND agent.token = ?
                   AND agent.is_enabled = 1
                   AND owner.banned = 0",
            )
            .bind(payload.subject_id)
            .bind(payload.issuer_user_id)
            .bind(&payload.credential)
            .fetch_one(&state.db)
            .await
        }
        _ => return false,
    };
    result.is_ok_and(|count| count == 1)
}

fn machine_bootstrap_cache_key(ticket: &str) -> String {
    format!("MACHINE_BOOTSTRAP_{}", sha256_hex(ticket))
}

fn machine_bootstrap_asset_cache_key(asset_id: &str) -> String {
    format!("MACHINE_BOOTSTRAP_ASSET_{}", sha256_hex(asset_id))
}

fn payload_matches_query(
    payload: &MachineBootstrapTicket,
    signed: &SignedBootstrapQuery,
) -> bool {
    payload.kind == signed.kind
        && payload.asset_id == signed.asset_id
        && payload.expires_at == signed.expires_at
        && payload.expires_at >= Utc::now().timestamp()
}

fn parse_and_verify_bootstrap_query(uri: &Uri, app_key: &str) -> Option<SignedBootstrapQuery> {
    let query = uri.query()?;
    if query.len() > 512 {
        return None;
    }
    let mut values = HashMap::<String, String>::new();
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        let key = key.into_owned();
        if !matches!(key.as_str(), "mb_v" | "mb_kind" | "mb_exp" | "mb_asset" | "mb_sig")
            || values.insert(key, value.into_owned()).is_some()
        {
            return None;
        }
    }
    if values.len() != 5 || values.get("mb_v").map(String::as_str) != Some("1") {
        return None;
    }
    let kind = values.get("mb_kind")?.to_string();
    let expires_at = values.get("mb_exp")?.parse::<i64>().ok()?;
    let asset_id = values.get("mb_asset")?.to_string();
    let signature = values.get("mb_sig")?;
    let now = Utc::now().timestamp();
    if !is_supported_kind(&kind)
        || !valid_ticket(&asset_id)
        || expires_at < now
        || expires_at > now + MACHINE_BOOTSTRAP_TTL_SECONDS + 5
    {
        return None;
    }
    verify_bootstrap_query_signature(app_key, &kind, expires_at, &asset_id, signature)?;
    Some(SignedBootstrapQuery {
        kind,
        expires_at,
        asset_id,
    })
}

fn bootstrap_signature_message(kind: &str, expires_at: i64, asset_id: &str) -> String {
    format!("notxboard-machine-bootstrap\n1\n{kind}\n{expires_at}\n{asset_id}")
}

fn sign_bootstrap_query(
    app_key: &str,
    kind: &str,
    expires_at: i64,
    asset_id: &str,
) -> Option<String> {
    let key = strict_app_key(app_key)?;
    let mut mac = HmacSha256::new_from_slice(&key).ok()?;
    mac.update(bootstrap_signature_message(kind, expires_at, asset_id).as_bytes());
    Some(
        mac.finalize()
            .into_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

fn verify_bootstrap_query_signature(
    app_key: &str,
    kind: &str,
    expires_at: i64,
    asset_id: &str,
    signature: &str,
) -> Option<()> {
    let key = strict_app_key(app_key)?;
    let signature = decode_hex_32(signature)?;
    let mut mac = HmacSha256::new_from_slice(&key).ok()?;
    mac.update(bootstrap_signature_message(kind, expires_at, asset_id).as_bytes());
    mac.verify_slice(&signature).ok()
}

fn strict_app_key(value: &str) -> Option<[u8; 32]> {
    let encoded = value.trim().strip_prefix("base64:")?;
    let decoded = BASE64.decode(encoded).ok()?;
    decoded.try_into().ok()
}

fn decode_hex_32(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut decoded = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let pair = std::str::from_utf8(pair).ok()?;
        decoded[index] = u8::from_str_radix(pair, 16).ok()?;
    }
    Some(decoded)
}

pub(crate) fn no_store_error(status: StatusCode, message: &str) -> Response<Body> {
    let mut response = json_error(status, message);
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("private, no-store"));
    response
        .headers_mut()
        .insert("pragma", HeaderValue::from_static("no-cache"));
    response
}

pub(crate) fn is_strong_machine_token(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_ticket(value: &str) -> bool {
    is_strong_machine_token(value)
}

fn is_supported_kind(value: &str) -> bool {
    matches!(value, "v2bx" | "tcping")
}

fn installer_kind_for_path(path: &str) -> Option<&'static str> {
    match path {
        "/v2bx-install.sh" => Some("v2bx"),
        "/tcping-agent-install.sh"
        | "/tcping-agent-src/go.mod"
        | "/tcping-agent-src/main.go" => Some("tcping"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installer_tickets_are_fixed_length_hex_values() {
        assert!(valid_ticket(&"a".repeat(64)));
        assert!(valid_ticket(&"A1".repeat(32)));
        assert!(!valid_ticket(&"a".repeat(63)));
        assert!(!valid_ticket(&format!("{}-", "a".repeat(63))));
    }

    #[test]
    fn installer_assets_are_bound_to_small_explicit_bundles() {
        assert_eq!(installer_kind_for_path("/v2bx-install.sh"), Some("v2bx"));
        assert_eq!(installer_kind_for_path("/tcping-agent-install.sh"), Some("tcping"));
        assert_eq!(installer_kind_for_path("/tcping-agent-src/go.mod"), Some("tcping"));
        assert_eq!(installer_kind_for_path("/tcping-agent-src/main.go"), Some("tcping"));
        assert_eq!(installer_kind_for_path("/tcping-agent-src/../secret"), None);
        assert_eq!(installer_kind_for_path("/tcping-agent-src/extra.go"), None);
    }

    #[test]
    fn cache_keys_do_not_store_plaintext_tickets() {
        let ticket = "ab".repeat(32);
        let key = machine_bootstrap_cache_key(&ticket);
        assert!(key.starts_with("MACHINE_BOOTSTRAP_"));
        assert!(!key.contains(&ticket));
    }

    #[test]
    fn signed_query_is_subject_bound_and_tamper_evident() {
        let key = "base64:MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=";
        let asset_id = "ab".repeat(32);
        let expires_at = Utc::now().timestamp() + 600;
        let signature = sign_bootstrap_query(key, "tcping", expires_at, &asset_id).unwrap();
        assert!(verify_bootstrap_query_signature(
            key,
            "tcping",
            expires_at,
            &asset_id,
            &signature,
        )
        .is_some());
        assert!(verify_bootstrap_query_signature(
            key,
            "v2bx",
            expires_at,
            &asset_id,
            &signature,
        )
        .is_none());
    }

    #[test]
    fn app_key_must_decode_to_exactly_32_bytes() {
        assert!(strict_app_key(
            "base64:MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY="
        )
        .is_some());
        assert!(strict_app_key("raw-key-is-not-accepted").is_none());
        assert!(strict_app_key("base64:c2hvcnQ=").is_none());
    }

    #[test]
    fn signed_query_parser_rejects_duplicates_unknown_fields_and_tampering() {
        let key = "base64:MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=";
        let asset_id = "cd".repeat(32);
        let expires_at = Utc::now().timestamp() + 600;
        let signature = sign_bootstrap_query(key, "v2bx", expires_at, &asset_id).unwrap();
        let query = format!(
            "mb_v=1&mb_kind=v2bx&mb_exp={expires_at}&mb_asset={asset_id}&mb_sig={signature}"
        );
        let valid = format!("/v2bx-install.sh?{query}").parse::<Uri>().unwrap();
        assert!(parse_and_verify_bootstrap_query(&valid, key).is_some());

        for invalid_query in [
            format!("{query}&mb_kind=v2bx"),
            format!("{query}&extra=1"),
            query.replace("mb_kind=v2bx", "mb_kind=tcping"),
        ] {
            let uri = format!("/v2bx-install.sh?{invalid_query}")
                .parse::<Uri>()
                .unwrap();
            assert!(parse_and_verify_bootstrap_query(&uri, key).is_none());
        }
    }

    #[test]
    fn bootstrap_errors_are_never_cacheable() {
        let response = no_store_error(StatusCode::NOT_FOUND, "Not found");
        assert_eq!(
            response.headers().get("cache-control").unwrap(),
            "private, no-store",
        );
    }
}
