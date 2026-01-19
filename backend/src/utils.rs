use crate::models::{AppState, ErrorResponse, GameType};
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use bytes::Bytes;
use log::error;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

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

fn get_cache_prefix(game: GameType) -> &'static str {
    match game {
        GameType::ClashOfClans => "coc",
        GameType::ClashRoyale => "cr",
    }
}

fn get_supercell_api_url(game: GameType) -> &'static str {
    match game {
        GameType::ClashOfClans => "https://api.clashofclans.com/v1",
        GameType::ClashRoyale => "https://api.clashroyale.com/v1",
    }
}

fn get_upstream_url(data: &AppState, game: GameType) -> &str {
    match game {
        GameType::ClashOfClans => &data.upstream_coc_url,
        GameType::ClashRoyale => &data.upstream_cr_url,
    }
}

fn get_upstream_token(data: &AppState, game: GameType) -> &str {
    match game {
        GameType::ClashOfClans => &data.coc_api_token,
        GameType::ClashRoyale => &data.cr_api_token,
    }
}

fn get_supercell_token(data: &AppState, game: GameType) -> &str {
    match game {
        GameType::ClashOfClans => &data.clash_of_clans_api_token,
        GameType::ClashRoyale => &data.clash_royale_api_token,
    }
}

// Function to filter out specific fields from clan data
pub fn filter_clan_data(body: Bytes, game: GameType, filter_fields: bool) -> Bytes {
    if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&body) {
        let mut modified = false;

        let fields_to_remove = [
            "maxKickpoints",
            "minSeasonWins",
            "kickpointsExpireAfterDays",
            "kickpointReasons",
        ];

        if let Some(clans) = value.as_array_mut() {
            // Filter out "Warteliste" for Clash Royale (always, for everyone)
            if game == GameType::ClashRoyale {
                let old_len = clans.len();
                clans.retain(|c| {
                    let name = c.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let name_db = c.get("nameDB").and_then(|n| n.as_str()).unwrap_or("");
                    let tag = c.get("tag").and_then(|t| t.as_str()).unwrap_or("");
                    name.to_lowercase() != "warteliste"
                        && name_db.to_lowercase() != "warteliste"
                        && tag.to_lowercase() != "warteliste"
                });
                if clans.len() != old_len {
                    modified = true;
                }
            }

            for clan in clans {
                if let Some(obj) = clan.as_object_mut() {
                    // Fix badgeUrl mismatch (singular vs plural) or badgeId (CR) - Always apply
                    let has_valid_urls = obj.get("badgeUrls")
                        .and_then(|u| u.get("large"))
                        .and_then(|l| l.as_str())
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);

                    if !has_valid_urls {
                        if let Some(url) = obj.get("badgeUrl").and_then(|u| u.as_str()) {
                            if !url.is_empty() {
                                obj.insert(
                                    "badgeUrls".to_string(),
                                    serde_json::json!({
                                        "small": url,
                                        "medium": url,
                                        "large": url
                                    }),
                                );
                                modified = true;
                            }
                        }
                    }

                    // Re-check after possible badgeUrl-to-badgeUrls migration
                    let has_valid_urls = obj.get("badgeUrls")
                        .and_then(|u| u.get("large"))
                        .and_then(|l| l.as_str())
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);

                    if !has_valid_urls {
                        if let Some(badge_id) = obj.get("badgeId") {
                            let id_str = if let Some(s) = badge_id.as_str() {
                                s.to_string()
                            } else if let Some(n) = badge_id.as_i64() {
                                n.to_string()
                            } else {
                                badge_id.to_string()
                            };

                            if !id_str.is_empty() {
                                let url = if game == GameType::ClashRoyale {
                                    format!(
                                        "https://raw.githubusercontent.com/RoyaleAPI/cr-api-assets/master/badges/{}.png",
                                        id_str
                                    )
                                } else {
                                    format!(
                                        "https://api-assets.clashofclans.com/badges/512/{}.png",
                                        id_str
                                    )
                                };
                                obj.insert(
                                    "badgeUrls".to_string(),
                                    serde_json::json!({
                                        "small": url,
                                        "medium": url,
                                        "large": url
                                    }),
                                );
                                modified = true;
                            }
                        }
                    }

                    // Ensure singular badgeUrl exists for frontend compatibility
                    let has_valid_badge_url = obj.get("badgeUrl")
                        .and_then(|v| v.as_str())
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);

                    if !has_valid_badge_url {
                        if let Some(urls) = obj.get("badgeUrls").and_then(|u| u.as_object()) {
                            if let Some(large) = urls.get("large").and_then(|l| l.as_str()) {
                                obj.insert(
                                    "badgeUrl".to_string(),
                                    serde_json::Value::String(large.to_string()),
                                );
                                modified = true;
                            }
                        }
                    }

                    if filter_fields {
                        for field in &fields_to_remove {
                            obj.remove(*field);
                        }
                        modified = true;
                    }
                }
            }
        } else if let Some(obj) = value.as_object_mut() {
            // Fix badgeUrl mismatch (singular vs plural) or badgeId (CR)
            let has_valid_urls = obj.get("badgeUrls")
                .and_then(|u| u.get("large"))
                .and_then(|l| l.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);

            if !has_valid_urls {
                if let Some(url) = obj.get("badgeUrl").and_then(|u| u.as_str()) {
                    if !url.is_empty() {
                        obj.insert(
                            "badgeUrls".to_string(),
                            serde_json::json!({
                                "small": url,
                                "medium": url,
                                "large": url
                            }),
                        );
                        modified = true;
                    }
                }
            }

            // Re-check after possible badgeUrl-to-badgeUrls migration
            let has_valid_urls = obj.get("badgeUrls")
                .and_then(|u| u.get("large"))
                .and_then(|l| l.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);

            if !has_valid_urls {
                if let Some(badge_id) = obj.get("badgeId") {
                    let id_str = if let Some(s) = badge_id.as_str() {
                        s.to_string()
                    } else if let Some(n) = badge_id.as_i64() {
                        n.to_string()
                    } else {
                        badge_id.to_string()
                    };

                    if !id_str.is_empty() {
                        let url = if game == GameType::ClashRoyale {
                            format!(
                                "https://raw.githubusercontent.com/RoyaleAPI/cr-api-assets/master/badges/{}.png",
                                id_str
                            )
                        } else {
                            format!(
                                "https://api-assets.clashofclans.com/badges/512/{}.png",
                                id_str
                            )
                        };
                        obj.insert(
                            "badgeUrls".to_string(),
                            serde_json::json!({
                                "small": url,
                                "medium": url,
                                "large": url
                            }),
                        );
                        modified = true;
                    }
                }
            }

            // Ensure singular badgeUrl exists for frontend compatibility
            let has_valid_badge_url = obj.get("badgeUrl")
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);

            if !has_valid_badge_url {
                if let Some(urls) = obj.get("badgeUrls").and_then(|u| u.as_object()) {
                    if let Some(large) = urls.get("large").and_then(|l| l.as_str()) {
                        obj.insert(
                            "badgeUrl".to_string(),
                            serde_json::Value::String(large.to_string()),
                        );
                        modified = true;
                    }
                }
            }

            if filter_fields {
                for field in &fields_to_remove {
                    obj.remove(*field);
                }
                modified = true;
            }
        }

        if modified
            && let Ok(filtered) = serde_json::to_vec(&value) {
                return Bytes::from(filtered);
            }
    }
    body
}

// Function to filter out specific fields from member data
pub fn filter_member_data(
    body: Bytes,
    game: GameType,
    exempt_tags: &[String],
    user_role: Option<&str>,
) -> Bytes {
    use crate::auth::has_required_role;

    // Coleaders and higher see everything - BUT still need badge synthesis for CR!
    // So we can't just return early if we want to fix badges for admins too.

    if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&body) {
        let mut modified = false;
        let is_member = has_required_role(user_role, "MEMBER");
        let is_coleader = has_required_role(user_role, "COLEADER");

        let fields_to_remove_not_member = [
            "totalKickpoints",
            "activeKickpoints",
            "userId",
            "discordId",
            "nickname",
            "avatar",
        ];

        let process_obj = |obj: &mut serde_json::Map<String, serde_json::Value>, tag: &str| {
            let mut inner_modified = false;

            // Synthesis for clan badge if nested in player object
            if game == GameType::ClashRoyale {
                if let Some(clan_obj) = obj.get_mut("clan").and_then(|c| c.as_object_mut()) {
                    let has_valid_urls = clan_obj.get("badgeUrls")
                        .and_then(|u| u.get("large"))
                        .and_then(|l| l.as_str())
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);

                    if !has_valid_urls {
                        if let Some(badge_id) = clan_obj.get("badgeId") {
                            let id_str = if let Some(s) = badge_id.as_str() {
                                s.to_string()
                            } else if let Some(n) = badge_id.as_i64() {
                                n.to_string()
                            } else {
                                badge_id.to_string()
                            };

                            if !id_str.is_empty() {
                                let url = if game == GameType::ClashRoyale {
                                    format!(
                                        "https://raw.githubusercontent.com/RoyaleAPI/cr-api-assets/master/badges/{}.png",
                                        id_str
                                    )
                                } else {
                                    format!(
                                        "https://api-assets.clashofclans.com/badges/512/{}.png",
                                        id_str
                                    )
                                };
                                clan_obj.insert(
                                    "badgeUrls".to_string(),
                                    serde_json::json!({
                                        "small": url,
                                        "medium": url,
                                        "large": url
                                    }),
                                );
                                clan_obj.insert(
                                    "badgeUrl".to_string(),
                                    serde_json::Value::String(url),
                                );
                                inner_modified = true;
                            }
                        }
                    }
                }
            }

            if is_coleader {
                 return inner_modified;
            }

            if exempt_tags.iter().any(|et| et == tag) {
                return inner_modified;
            }

            // Always remove internal DB fields
            obj.remove("clanDB");

            if is_member {
                // For members: Return count and sum instead of full details
                if let Some(akp) = obj.get("activeKickpoints").and_then(|v| v.as_array()) {
                    let sum: i64 = akp
                        .iter()
                        .filter_map(|kp| kp.get("amount").and_then(|a| a.as_i64()))
                        .sum();
                    obj.insert(
                        "activeKickpointsCount".to_string(),
                        serde_json::json!(akp.len()),
                    );
                    obj.insert("activeKickpointsSum".to_string(), serde_json::json!(sum));
                }
                obj.remove("activeKickpoints");

                // Hide raw IDs for members (privacy), but keep nickname/avatar/points
                if !is_coleader {
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
            let is_coleader = has_required_role(user_role, "COLEADER");

            members.retain(|m| {
                if is_coleader {
                    return true;
                }
                !m.get("isHidden").and_then(|v| v.as_bool()).unwrap_or(false)
            });

            for member in members {
                let tag = member
                    .get("tag")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                if let Some(obj) = member.as_object_mut()
                    && process_obj(obj, &tag) {
                        modified = true;
                    }
            }
        } else if let Some(obj) = value.as_object_mut() {
            let tag = obj
                .get("tag")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            if process_obj(obj, &tag) {
                modified = true;
            }
        }

        if modified
            && let Ok(filtered) = serde_json::to_vec(&value) {
                return Bytes::from(filtered);
            }
    }
    body
}

// Update upstream cache (bot server)
pub async fn update_upstream_cache(
    data: &AppState,
    game: GameType,
    url_path: &str,
) -> Result<Bytes, String> {
    let prefix = get_cache_prefix(game);
    let upstream_url = get_upstream_url(data, game);
    let token = get_upstream_token(data, game);
    let full_url = format_url(upstream_url, url_path);

    match data
        .client
        .get(&full_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status().as_u16();
            let body = res.bytes().await.map_err(|e| e.to_string())?;

            if status == 200 {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let cache_key = format!("{}:upstream:{}", prefix, url_path);

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
            } else {
                let err_msg = format!("Upstream {} returned status {}", full_url, status);
                eprintln!("Background Refresh: {}", err_msg);
                Err(err_msg)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn get_cached_or_update_supercell_cache(
    data: &AppState,
    game: GameType,
    url_path: &str,
    ttl_seconds: i64,
) -> Result<Bytes, String> {
    let prefix = get_cache_prefix(game);
    let cache_key = format!("{}:supercell:{}", prefix, url_path);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let cached_result =
        sqlx::query_as::<_, (Vec<u8>, i64)>("SELECT body, updated_at FROM cache WHERE key = $1")
            .bind(&cache_key)
            .fetch_optional(&data.db_pool)
            .await;

    if let Ok(Some((body, updated_at))) = &cached_result
        && now - updated_at < ttl_seconds {
            return Ok(Bytes::from(body.clone()));
        }

    match update_supercell_cache(data, game, url_path).await {
        Ok(body) => Ok(body),
        Err(e) => {
            // Fallback to expired cache on error
            if let Ok(Some((body, _))) = cached_result {
                return Ok(Bytes::from(body));
            }
            Err(e)
        }
    }
}

pub async fn get_cached_or_update_upstream_cache(
    data: &AppState,
    game: GameType,
    url_path: &str,
    ttl_seconds: i64,
) -> Result<Bytes, String> {
    let prefix = get_cache_prefix(game);
    let cache_key = format!("{}:upstream:{}", prefix, url_path);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let cached_result =
        sqlx::query_as::<_, (Vec<u8>, i64)>("SELECT body, updated_at FROM cache WHERE key = $1")
            .bind(&cache_key)
            .fetch_optional(&data.db_pool)
            .await;

    if let Ok(Some((body, updated_at))) = &cached_result
        && now - updated_at < ttl_seconds {
            return Ok(Bytes::from(body.clone()));
        }

    match update_upstream_cache(data, game, url_path).await {
        Ok(body) => Ok(body),
        Err(e) => {
            // Fallback to expired cache on error
            if let Ok(Some((body, _))) = cached_result {
                return Ok(Bytes::from(body));
            }
            Err(e)
        }
    }
}

pub async fn forward_request(data: &AppState, game: GameType, url_path: &str) -> HttpResponse {
    forward_request_with_filter(data, game, url_path, None, &[]).await
}

pub async fn forward_request_with_filter(
    data: &AppState,
    game: GameType,
    url_path: &str,
    user_role: Option<&str>,
    exempt_tags: &[String],
) -> HttpResponse {
    let prefix = get_cache_prefix(game);
    // Map /members-lite request to /members cache key
    let stripped_path = url_path.replace("/members-lite", "/members");
    let cache_key = format!("{}:upstream:{}", prefix, stripped_path);

    // Serve from cache ONLY
    let result =
        sqlx::query_as::<_, (Vec<u8>, i32)>("SELECT body, status FROM cache WHERE key = $1")
            .bind(&cache_key)
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
                    || parts[4] == "cwl-members"
                    || parts[4] == "members-lite"))
                || (parts.len() == 4 && parts[1] == "api" && parts[2] == "players");

            if is_clan_path {
                body = filter_clan_data(
                    body,
                    game,
                    !crate::auth::has_required_role(user_role, "MEMBER"),
                );
            } else if is_member_path {
                body = filter_member_data(body, game, exempt_tags, user_role);
            }

            let status = StatusCode::from_u16(status as u16).unwrap_or(StatusCode::OK);
            HttpResponse::build(status)
                .content_type("application/json")
                .body(body)
        }
        Ok(None) => HttpResponse::ServiceUnavailable().json(ErrorResponse {
            error: "Data not yet available in cache. Background refresh is in progress.".into(),
        }),
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Internal Database Error".into(),
            })
        }
    }
}

// Update Supercell API cache
pub async fn update_supercell_cache(
    data: &AppState,
    game: GameType,
    url_path: &str,
) -> Result<Bytes, String> {
    let prefix = get_cache_prefix(game);
    let api_url = get_supercell_api_url(game);
    let token = get_supercell_token(data, game);
    let full_url = format!("{}{}", api_url, url_path);

    match data
        .client
        .get(&full_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status().as_u16();
            let body = res.bytes().await.map_err(|e| e.to_string())?;

            if status == 200 {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let cache_key = format!("{}:supercell:{}", prefix, url_path);

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
            } else {
                let err_msg = format!("Supercell {} returned status {}", full_url, status);
                eprintln!("Background Refresh: {}", err_msg);
                Err(err_msg)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
