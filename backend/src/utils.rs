use crate::models::{AppState, ErrorResponse};
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use bytes::Bytes;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde_json;

pub fn format_url(base: &str, path: &str) -> String {
    format!("{}{}", base, path)
}

pub fn encode_tag(tag: &str) -> String {
    let tag = if tag.starts_with('#') {
        tag.to_string()
    } else {
        format!("#{}", tag)
    };
    utf8_percent_encode(&tag, NON_ALPHANUMERIC).to_string()
}

// Function to filter out specific fields from clan data
pub fn filter_clan_data(body: Bytes) -> Bytes {
    if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&body) {
        let mut modified = false;

        let fields_to_remove = [
            "maxKickpoints",
            "minSeasonWins",
            "kickpointsExpireAfterDays",
            "kickpointReasons",
        ];

        if let Some(clans) = value.as_array_mut() {
            for clan in clans {
                if let Some(obj) = clan.as_object_mut() {
                    // Fix badgeUrl mismatch (singular vs plural)
                    if !obj.contains_key("badgeUrls") {
                        if let Some(url) = obj.get("badgeUrl").and_then(|u| u.as_str()) {
                            obj.insert("badgeUrls".to_string(), serde_json::json!({
                                "small": url,
                                "medium": url,
                                "large": url
                            }));
                        }
                    }

                    for field in &fields_to_remove {
                        obj.remove(*field);
                    }
                    modified = true;
                }
            }
        } else if let Some(obj) = value.as_object_mut() {
            // Fix badgeUrl mismatch (singular vs plural)
            if !obj.contains_key("badgeUrls") {
                if let Some(url) = obj.get("badgeUrl").and_then(|u| u.as_str()) {
                    obj.insert("badgeUrls".to_string(), serde_json::json!({
                        "small": url,
                        "medium": url,
                        "large": url
                    }));
                }
            }

            for field in &fields_to_remove {
                obj.remove(*field);
            }
            modified = true;
        }

        if modified {
            if let Ok(filtered) = serde_json::to_vec(&value) {
                return Bytes::from(filtered);
            }
        }
    }
    body
}

// Function to filter out specific fields from member data
pub fn filter_member_data(body: Bytes, exempt_tags: &[String], user_role: Option<&str>) -> Bytes {
    use crate::auth::has_required_role;

    // Coleaders and higher see everything
    if has_required_role(user_role, "COLEADER") {
        return body;
    }

    if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&body) {
        let mut modified = false;
        let is_member = has_required_role(user_role, "MEMBER");

        let fields_to_remove_not_member = [
            "totalKickpoints",
            "activeKickpoints",
            "userId",
            "discordId",
            "nickname",
            "avatar",
        ];

        let process_obj = |obj: &mut serde_json::Map<String, serde_json::Value>, tag: &str| {
            if exempt_tags.iter().any(|et| et == tag) {
                return false;
            }

            // Always remove internal DB fields
            obj.remove("clanDB");

            if is_member {
                // For members: Return count and sum instead of full details
                if let Some(akp) = obj.get("activeKickpoints").and_then(|v| v.as_array()) {
                    let sum: i64 = akp.iter().filter_map(|kp| kp.get("amount").and_then(|a| a.as_i64())).sum();
                    obj.insert("activeKickpointsCount".to_string(), serde_json::json!(akp.len()));
                    obj.insert("activeKickpointsSum".to_string(), serde_json::json!(sum));
                }
                obj.remove("activeKickpoints");
                
                // Hide raw IDs for members (privacy), but keep nickname/avatar/points
                let is_coleader = has_required_role(user_role, "COLEADER");
                if !is_coleader {
                    // We keep a flag so the frontend knows they're linked
                    if obj.contains_key("userId") {
                        obj.insert("isLinked".to_string(), serde_json::json!(true));
                    }
                    obj.remove("userId");
                    obj.remove("discordId");
                }
            } else {
                // Not a member: Remove counts and identity links
                for field in &fields_to_remove_not_member {
                    obj.remove(*field);
                }
            }
            true
        };

        if let Some(members) = value.as_array_mut() {
            for member in members {
                let tag = member.get("tag").and_then(|t| t.as_str()).unwrap_or("").to_string();
                if let Some(obj) = member.as_object_mut() {
                    if process_obj(obj, &tag) {
                        modified = true;
                    }
                }
            }
        } else if let Some(obj) = value.as_object_mut() {
            let tag = obj.get("tag").and_then(|t| t.as_str()).unwrap_or("").to_string();
            if process_obj(obj, &tag) {
                modified = true;
            }
        }

        if modified {
            if let Ok(filtered) = serde_json::to_vec(&value) {
                return Bytes::from(filtered);
            }
        }
    }
    body
}

// Public function used by both handlers and background task
pub async fn update_cache(data: &AppState, url_path: &str) -> Result<Bytes, String> {
    let full_url = format_url(&data.upstream_url, url_path);

    match data
        .client
        .get(&full_url)
        .header("Authorization", format!("Bearer {}", data.api_token))
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status().as_u16();
            let body = res.bytes().await.map_err(|e| e.to_string())?;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            // We use ON CONFLICT to update existing keys
            let _ = sqlx::query(
                "INSERT INTO cache (key, body, status, updated_at) 
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (key) DO UPDATE SET 
                    body = EXCLUDED.body, 
                    status = EXCLUDED.status, 
                    updated_at = EXCLUDED.updated_at",
            )
            .bind(url_path)
            .bind(body.to_vec())
            .bind(status as i32)
            .bind(timestamp)
            .execute(&data.db_pool)
            .await;

            Ok(body)
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn forward_request(data: &AppState, url_path: &str) -> HttpResponse {
    forward_request_with_filter(data, url_path, None, &[]).await
}

pub async fn forward_request_with_filter(
    data: &AppState,
    url_path: &str,
    user_role: Option<&str>,
    exempt_tags: &[String],
) -> HttpResponse {
    // 1. serve from cache ONLY
    let result =
        sqlx::query_as::<_, (Vec<u8>, i32)>("SELECT body, status FROM cache WHERE key = $1")
            .bind(url_path)
            .fetch_optional(&data.db_pool)
            .await;

    match result {
        Ok(Some((body, status))) => {
            let mut body = Bytes::from(body);

            let parts: Vec<&str> = url_path.split('/').collect();
            let is_clan_path = url_path == "/api/clans"
                || (parts.len() == 4 && parts[1] == "api" && parts[2] == "clans");
            let is_member_path = (parts.len() == 5
                && parts[1] == "api"
                && parts[2] == "clans"
                && (parts[4] == "members"
                    || parts[4] == "war-members"
                    || parts[4] == "raid-members"
                    || parts[4] == "cwl-members"))
                || (parts.len() == 4 && parts[1] == "api" && parts[2] == "players")
                || (parts.len() == 3 && parts[1] == "players")
                || (url_path.starts_with("clash:/players/"));

            if is_clan_path && !crate::auth::has_required_role(user_role, "MEMBER") {
                body = filter_clan_data(body);
            } else if is_member_path {
                body = filter_member_data(body, exempt_tags, user_role);
            }

            let status = StatusCode::from_u16(status as u16).unwrap_or(StatusCode::OK);
            HttpResponse::build(status)
                .content_type("application/json")
                .body(body)
        }
        Ok(None) => {
            // Data is not in cache (either not yet refreshed or not supported for proactive caching)
            HttpResponse::ServiceUnavailable().json(ErrorResponse {
                error: "Data not yet available in cache. Background refresh is in progress.".into(),
            })
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Internal Database Error".into(),
            })
        }
    }
}

pub async fn update_clash_cache(data: &AppState, url_path: &str) -> Result<Bytes, String> {
    let full_url = format!("https://api.clashofclans.com/v1{}", url_path);

    match data
        .client
        .get(&full_url)
        .header("Authorization", format!("Bearer {}", data.clash_api_token))
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status().as_u16();
            let body = res.bytes().await.map_err(|e| e.to_string())?;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            let cache_key = format!("clash:{}", url_path);

            // We use ON CONFLICT to update existing keys
            let _ = sqlx::query(
                "INSERT INTO cache (key, body, status, updated_at) 
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (key) DO UPDATE SET 
                    body = EXCLUDED.body, 
                    status = EXCLUDED.status, 
                    updated_at = EXCLUDED.updated_at",
            )
            .bind(&cache_key)
            .bind(body.to_vec())
            .bind(status as i32)
            .bind(timestamp)
            .execute(&data.db_pool)
            .await;

            Ok(body)
        }
        Err(e) => Err(e.to_string()),
    }
}
