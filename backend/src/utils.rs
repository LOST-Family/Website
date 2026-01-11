use crate::models::{AppState, ErrorResponse};
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use bytes::Bytes;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde_json;

pub fn format_url(base: &str, path: &str) -> String {
    format!("{}{}", base, path)
}

pub fn encode_tag(tag: &str) -> String {
    utf8_percent_encode(tag, NON_ALPHANUMERIC).to_string()
}

// Function to filter out specific fields from clan data
fn filter_clan_data(body: Bytes) -> Bytes {
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
                    for field in &fields_to_remove {
                        obj.remove(*field);
                    }
                    modified = true;
                }
            }
        } else if let Some(obj) = value.as_object_mut() {
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
fn filter_member_data(body: Bytes) -> Bytes {
    if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&body) {
        let mut modified = false;

        let fields_to_remove = ["totalKickpoints", "activeKickpoints", "clanDB"];

        if let Some(members) = value.as_array_mut() {
            for member in members {
                if let Some(obj) = member.as_object_mut() {
                    for field in &fields_to_remove {
                        obj.remove(*field);
                    }
                    modified = true;
                }
            }
        } else if let Some(obj) = value.as_object_mut() {
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
            let mut body = res.bytes().await.map_err(|e| e.to_string())?;

            // Filter out internal fields if this is a clan or member related endpoint
            if status == 200 {
                let parts: Vec<&str> = url_path.split('/').collect();
                // /api/clans OR /api/clans/{tag}
                if url_path == "/api/clans"
                    || (parts.len() == 4 && parts[1] == "api" && parts[2] == "clans")
                {
                    body = filter_clan_data(body);
                }
                // /api/clans/{tag}/members
                else if parts.len() == 5
                    && parts[1] == "api"
                    && parts[2] == "clans"
                    && parts[4] == "members"
                {
                    body = filter_member_data(body);
                }
            }

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
    // 1. serve from cache ONLY
    let result =
        sqlx::query_as::<_, (Vec<u8>, i32)>("SELECT body, status FROM cache WHERE key = $1")
            .bind(url_path)
            .fetch_optional(&data.db_pool)
            .await;

    match result {
        Ok(Some((body, status))) => {
            let status = StatusCode::from_u16(status as u16).unwrap_or(StatusCode::OK);
            HttpResponse::build(status)
                .content_type("application/json")
                .body(bytes::Bytes::from(body))
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
