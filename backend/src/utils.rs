use crate::models::{AppState, ErrorResponse};
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use bytes::Bytes;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

pub fn format_url(base: &str, path: &str) -> String {
    format!("{}{}", base, path)
}

pub fn encode_tag(tag: &str) -> String {
    utf8_percent_encode(tag, NON_ALPHANUMERIC).to_string()
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

            // We use INSERT OR REPLACE to update existing keys
            let _ = sqlx::query(
                "INSERT OR REPLACE INTO cache (key, body, status, updated_at) VALUES (?, ?, ?, ?)",
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
        sqlx::query_as::<_, (Vec<u8>, i32)>("SELECT body, status FROM cache WHERE key = ?")
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
