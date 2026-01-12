use crate::auth::{AuthenticatedUser, OptionalAuthenticatedUser, has_required_role};
use crate::models::{AppState, ErrorResponse};
use crate::utils::{
    encode_tag, forward_request, forward_request_with_filter, update_cache, update_clash_cache,
};
use actix_web::{HttpResponse, Responder, web};
use bytes::Bytes;

// 1. Get All Clans
pub async fn get_clans(
    data: web::Data<AppState>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    forward_request_with_filter(&data, "/api/clans", user_role, &[]).await
}

// 2. Get Clan Info
pub async fn get_clan_info(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    _opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let clash_url_path = format!("/clans/{}", encoded_tag);
    let upstream_url_path = format!("/api/clans/{}", encoded_tag);

    // Get from cache (Background task handles refreshes)
    let coc_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(format!("clash:{}", clash_url_path))
        .fetch_optional(&data.db_pool)
        .await;

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&upstream_url_path)
        .fetch_optional(&data.db_pool)
        .await;

    let mut clan_json = match coc_res {
        Ok(Some((body,))) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        _ => serde_json::Value::Null,
    };

    if let Ok(Some((body,))) = upstream_res {
        if let Ok(upstream_json) = serde_json::from_slice::<serde_json::Value>(&body) {
            if let Some(obj) = clan_json.as_object_mut() {
                if let Some(up_obj) = upstream_json.as_object() {
                    for (k, v) in up_obj {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            } else if clan_json.is_null() {
                clan_json = upstream_json;
            }
        }
    }

    if clan_json.is_null() {
        return HttpResponse::NotFound().json(serde_json::json!({ "error": "Clan not found" }));
    }

    HttpResponse::Ok().json(clan_json)
}

// 3. Get Clan Members
pub async fn get_clan_members(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| u.linked_players.as_slice())
        .unwrap_or(&[]);

    let clash_url_path = format!("/clans/{}", encoded_tag);
    let upstream_url_path = format!("/api/clans/{}/members", encoded_tag);

    // Get bodies from cache (Background task handles refreshes)
    let coc_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(format!("clash:{}", clash_url_path))
        .fetch_optional(&data.db_pool)
        .await;

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&upstream_url_path)
        .fetch_optional(&data.db_pool)
        .await;

    let mut coc_members = match coc_res {
        Ok(Some((body,))) => {
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
            json["memberList"].as_array().cloned().unwrap_or_default()
        }
        _ => vec![],
    };

    let upstream_body = match upstream_res {
        Ok(Some((body,))) => Bytes::from(body),
        _ => Bytes::new(),
    };

    // Filter upstream members first (privacy logic)
    let filtered_upstream_body = crate::utils::filter_member_data(upstream_body, exempt_tags, user_role);
    let upstream_members: Vec<serde_json::Value> = serde_json::from_slice(&filtered_upstream_body).unwrap_or_default();

    // Merge members
    let mut final_members = Vec::new();
    for mut u_member in upstream_members {
        if let Some(tag) = u_member.get("tag").and_then(|t| t.as_str()) {
            // Find corresponding CoC member for assets/names
            if let Some(c_member) = coc_members.iter().find(|m| m.get("tag").and_then(|t| t.as_str()) == Some(tag)) {
                if let Some(u_obj) = u_member.as_object_mut() {
                    if let Some(c_obj) = c_member.as_object() {
                        for (k, v) in c_obj {
                            if !u_obj.contains_key(k) || u_obj.get(k).map_or(true, |val| val.is_null() || val.as_str() == Some("")) {
                                u_obj.insert(k.clone(), v.clone());
                            }
                        }
                    }
                }
            }
        }
        final_members.push(u_member);
    }

    if final_members.is_empty() && !coc_members.is_empty() {
        return HttpResponse::Ok().json(coc_members);
    }

    HttpResponse::Ok().json(final_members)
}

// 4. Get Clan Kickpoint Reasons (Protected: COLEADER or higher)
pub async fn get_clan_kickpoint_reasons(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    user: AuthenticatedUser,
) -> impl Responder {
    if !has_required_role(user.claims.role.as_deref(), "COLEADER") {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires COLEADER or higher role".into(),
        });
    }

    let encoded_tag = encode_tag(&tag);
    forward_request(
        &data,
        &format!("/api/clans/{}/kickpoint-reasons", encoded_tag),
    )
    .await
}

// 5. Get Clan War Members
pub async fn get_clan_war_members(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| u.linked_players.as_slice())
        .unwrap_or(&[]);
    forward_request_with_filter(
        &data,
        &format!("/api/clans/{}/war-members", encoded_tag),
        user_role,
        exempt_tags,
    )
    .await
}

// 6. Get Raid Members
pub async fn get_raid_members(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| u.linked_players.as_slice())
        .unwrap_or(&[]);
    forward_request_with_filter(
        &data,
        &format!("/api/clans/{}/raid-members", encoded_tag),
        user_role,
        exempt_tags,
    )
    .await
}

// 7. Get CWL Members
pub async fn get_cwl_members(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| u.linked_players.as_slice())
        .unwrap_or(&[]);
    forward_request_with_filter(
        &data,
        &format!("/api/clans/{}/cwl-members", encoded_tag),
        user_role,
        exempt_tags,
    )
    .await
}

// 8. Get Player
pub async fn get_player(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let clash_url_path = format!("/players/{}", encoded_tag);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);

    // Try to update cache first
    let _ = update_clash_cache(&data, &clash_url_path).await;
    let _ = update_cache(&data, &upstream_url_path).await;

    // Get bodies from cache
    let coc_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(format!("clash:{}", clash_url_path))
        .fetch_optional(&data.db_pool)
        .await;

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&upstream_url_path)
        .fetch_optional(&data.db_pool)
        .await;

    let mut player_json = match coc_res {
        Ok(Some((body,))) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        _ => return HttpResponse::NotFound().finish(),
    };

    if let Ok(Some((u_body,))) = upstream_res {
        let u_json =
            serde_json::from_slice::<serde_json::Value>(&u_body).unwrap_or(serde_json::Value::Null);
        if let Some(obj) = player_json.as_object_mut() {
            if let Some(u_obj) = u_json.as_object() {
                for (k, v) in u_obj {
                    if !obj.contains_key(k) {
                        obj.insert(k.clone(), v.clone());
                    } else {
                        obj.insert(format!("upstream_{}", k), v.clone());
                    }
                }
            }
        }
    }

    // Now filter if needed
    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| u.linked_players.as_slice())
        .unwrap_or(&[]);

    if !has_required_role(user_role, "COLEADER") {
        let tag_is_exempt = if let Some(tag_str) = player_json.get("tag").and_then(|t| t.as_str()) {
            exempt_tags.iter().any(|et| et == tag_str)
        } else {
            false
        };

        if !tag_is_exempt {
            // filter fields
            if let Some(obj) = player_json.as_object_mut() {
                obj.remove("clanDB");
                
                if has_required_role(user_role, "MEMBER") {
                    // For members: Keep the count/amounts, but mask details (description, reason)
                    if let Some(akp) = obj.get_mut("activeKickpoints").and_then(|v| v.as_array_mut()) {
                        for kp in akp {
                            if let Some(kp_obj) = kp.as_object_mut() {
                                kp_obj.remove("description");
                                kp_obj.remove("reason");
                            }
                        }
                    }
                } else {
                    // Not a member: Hide everything sensitive
                    obj.remove("totalKickpoints");
                    obj.remove("activeKickpoints");
                    obj.remove("userId");
                    obj.remove("discordId");
                }
            }
        }
    }

    HttpResponse::Ok().json(player_json)
}

// 9. Get User
pub async fn get_user(
    data: web::Data<AppState>,
    user_id: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let is_self = opt_user
        .user
        .as_ref()
        .map(|u| u.claims.sub == *user_id)
        .unwrap_or(false);
    let is_coleader = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref())
        .map(|r| crate::auth::get_role_priority(r) >= crate::auth::get_role_priority("COLEADER"))
        .unwrap_or(false);

    if !is_self && !is_coleader {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: You can only view your own profile or require COLEADER role"
                .into(),
        });
    }

    forward_request(&data, &format!("/api/users/{}", user_id)).await
}

// 10. Get Guild Info
pub async fn get_guild_info(
    data: web::Data<AppState>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    forward_request_with_filter(&data, "/api/guild", user_role, &[]).await
}

// 11. Get My Player Accounts (Protected: User only)
pub async fn get_my_player_accounts(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let mut players_data = Vec::new();
    for tag in user.linked_players {
        let encoded_tag = encode_tag(&tag);
        let clash_url_path = format!("/players/{}", encoded_tag);
        let upstream_url_path = format!("/api/players/{}", encoded_tag);

        // Fetch CoC data
        let coc_res = update_clash_cache(&data, &clash_url_path).await;
        // Fetch Upstream data
        let upstream_res = update_cache(&data, &upstream_url_path).await;

        if let Ok(coc_body) = coc_res {
            if let Ok(mut coc_json) = serde_json::from_slice::<serde_json::Value>(&coc_body) {
                if let Some(coc_obj) = coc_json.as_object_mut() {
                    // Merge upstream data if available
                    if let Ok(u_body) = upstream_res {
                        if let Ok(u_json) = serde_json::from_slice::<serde_json::Value>(&u_body) {
                            if let Some(u_obj) = u_json.as_object() {
                                for (k, v) in u_obj {
                                    // Don't overwrite existing CoC data if there's a collision
                                    if !coc_obj.contains_key(k) {
                                        coc_obj.insert(k.clone(), v.clone());
                                    } else {
                                        coc_obj.insert(format!("upstream_{}", k), v.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                players_data.push(coc_json);
            }
        }
    }

    HttpResponse::Ok().json(players_data)
}

// 12. Get Admin Status (Protected: Admin only)
async fn get_uptime_stats(pool: &sqlx::PgPool, api_name: &str) -> (i32, i32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let one_day_ago = now - 24 * 60 * 60;

    let rows = sqlx::query_as::<_, (i32,)>(
        "SELECT latency_ms FROM latency_measurements 
         WHERE api_name = $1 AND timestamp > $2 
         ORDER BY timestamp DESC",
    )
    .bind(api_name)
    .bind(one_day_ago)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(data) if !data.is_empty() => {
            let successes = data.iter().filter(|r| r.0 != -1).count() as i32;
            let current_latency = data[0].0;
            (current_latency, successes) // Each success is 1 minute
        }
        _ => (-1, 0),
    }
}

pub async fn get_admin_status(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    if !has_required_role(user.claims.role.as_deref(), "ADMIN") {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires ADMIN role".into(),
        });
    }

    let (upstream_latency, upstream_minutes) = get_uptime_stats(&data.db_pool, "upstream").await;
    let (sc_latency, sc_minutes) = get_uptime_stats(&data.db_pool, "supercell").await;
    let (website_latency, website_uptime_minutes) =
        get_uptime_stats(&data.db_pool, "website").await;

    HttpResponse::Ok().json(serde_json::json!({
        "upstream": {
            "status": if upstream_latency != -1 { "ONLINE" } else { "OFFLINE" },
            "latency": if upstream_latency != -1 { upstream_latency } else { 0 },
            "uptime_minutes": upstream_minutes
        },
        "supercell": {
            "status": if sc_latency != -1 { "ONLINE" } else { "OFFLINE" },
            "latency": if sc_latency != -1 { sc_latency } else { 0 },
            "uptime_minutes": sc_minutes
        },
        "website": {
            "status": if website_latency != -1 { "ONLINE" } else { "OFFLINE" },
            "latency": if website_latency != -1 { website_latency } else { 0 },
            "uptime_minutes": website_uptime_minutes
        }
    }))
}

// 13. Get Latency History (Protected: Admin only)
pub async fn get_latency_history(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    if !has_required_role(user.claims.role.as_deref(), "ADMIN") {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires ADMIN role".into(),
        });
    }

    let measurements = sqlx::query_as::<_, (String, i32, i64)>(
        "SELECT api_name, latency_ms, timestamp FROM latency_measurements ORDER BY timestamp ASC",
    )
    .fetch_all(&data.db_pool)
    .await;

    match measurements {
        Ok(rows) => {
            let data: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(api_name, latency_ms, timestamp)| {
                    serde_json::json!({
                        "api": api_name,
                        "latency": latency_ms,
                        "timestamp": timestamp
                    })
                })
                .collect();
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            eprintln!("Database error fetching latency: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
