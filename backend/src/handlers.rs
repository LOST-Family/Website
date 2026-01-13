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

    // Get from cache (Background task handles refreshes)
    let coc_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(format!("clash:{}", clash_url_path))
        .fetch_optional(&data.db_pool)
        .await;

    let mut clan_json = match coc_res {
        Ok(Some((body,))) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        _ => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Clan not found" })),
    };

    // Fix badgeUrl mismatch (singular vs plural) - ensured in utils but also here if needed
    if let Some(obj) = clan_json.as_object_mut() {
        if !obj.contains_key("badgeUrls") {
            if let Some(url) = obj.get("badgeUrl").and_then(|u| u.as_str()) {
                obj.insert("badgeUrls".to_string(), serde_json::json!({
                    "small": url,
                    "medium": url,
                    "large": url
                }));
            }
        }
    }

    HttpResponse::Ok().json(clan_json)
}

// 2b. Get Clan Config (Upstream settings, Protected: MEMBER or higher)
pub async fn get_clan_config(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    if !has_required_role(user_role, "MEMBER") {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires MEMBER role".into(),
        });
    }

    let encoded_tag = encode_tag(&tag);
    let upstream_url_path = format!("/api/clans/{}", encoded_tag);

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&upstream_url_path)
        .fetch_optional(&data.db_pool)
        .await;

    match upstream_res {
        Ok(Some((body,))) => {
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
            if let Some(obj) = json.as_object() {
                let mut config = serde_json::Map::new();
                let fields = ["maxKickpoints", "minSeasonWins", "kickpointsExpireAfterDays", "kickpointReasons"];
                for f in fields {
                    if let Some(v) = obj.get(f) {
                        config.insert(f.to_string(), v.clone());
                    }
                }
                return HttpResponse::Ok().json(config);
            }
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Clan config not found" }))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({ "error": "Clan config not found in cache" })),
    }
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

    // Merge upstream data and check for cached player details (for warStars, etc.)
    for c_member in &mut coc_members {
        if let Some(tag_ref) = c_member.get("tag").and_then(|t| t.as_str()) {
            let tag = tag_ref.to_string(); // Clone to release borrow on c_member
            
            // 1. Merge Upstream (Identity/Kickpoints)
            if let Some(u_member) = upstream_members.iter().find(|m| m.get("tag").and_then(|t| t.as_str()) == Some(&tag)) {
                if let (Some(c_obj), Some(u_obj)) = (c_member.as_object_mut(), u_member.as_object()) {
                    for (k, v) in u_obj {
                        if !c_obj.contains_key(k) {
                            c_obj.insert(k.clone(), v.clone());
                        } else if k != "tag" {
                            c_obj.insert(format!("upstream_{}", k), v.clone());
                        }
                    }
                }
            }

            // 2. Try to get warStars from player cache (hot-loaded by background task)
            let player_cache_key = format!("clash:/players/{}", encode_tag(&tag));
            let player_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
                .bind(&player_cache_key)
                .fetch_optional(&data.db_pool)
                .await;

            if let Ok(Some((p_body,))) = player_res {
                if let Ok(p_json) = serde_json::from_slice::<serde_json::Value>(&p_body) {
                    if let (Some(c_obj), Some(p_obj)) = (c_member.as_object_mut(), p_json.as_object()) {
                        if let Some(stars) = p_obj.get("warStars") {
                            c_obj.insert("warStars".to_string(), stars.clone());
                        }
                        // Also grab heroes and high-res league info
                        if let Some(heroes) = p_obj.get("heroes") {
                            c_obj.insert("heroes".to_string(), heroes.clone());
                        }
                        if let Some(league) = p_obj.get("league") {
                            c_obj.insert("league".to_string(), league.clone());
                        }
                    }
                }
            }
        }
    }

    HttpResponse::Ok().json(coc_members)
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

// 8. Get Player (CoC Data only + Kickpoint Summaries)
pub async fn get_player(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let clash_url_path = format!("/players/{}", encoded_tag);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);

    // Get bodies from cache
    let coc_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(format!("clash:{}", clash_url_path))
        .fetch_optional(&data.db_pool)
        .await;

    let mut player_json = match coc_res {
        Ok(Some((body,))) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        _ => return HttpResponse::NotFound().finish(),
    };

    // We still fetch upstream summary if user is authorized to see it
    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user.user.as_ref().map(|u| u.linked_players.as_slice()).unwrap_or(&[]);
    let tag_str = player_json.get("tag").and_then(|t| t.as_str()).unwrap_or("");
    let is_exempt = exempt_tags.iter().any(|et| et == tag_str);

    if has_required_role(user_role, "MEMBER") || is_exempt {
        let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
            .bind(&upstream_url_path)
            .fetch_optional(&data.db_pool)
            .await;

        if let Ok(Some((u_body,))) = upstream_res {
            if let Ok(u_json) = serde_json::from_slice::<serde_json::Value>(&u_body) {
                if let Some(obj) = player_json.as_object_mut() {
                    // Only merge kickpoint summaries, NOT identity
                    if let Some(akp) = u_json.get("activeKickpoints").and_then(|v| v.as_array()) {
                        let sum: i64 = akp.iter().filter_map(|kp| kp.get("amount").and_then(|a| a.as_i64())).sum();
                        obj.insert("activeKickpointsCount".to_string(), serde_json::json!(akp.len()));
                        obj.insert("activeKickpointsSum".to_string(), serde_json::json!(sum));
                    }
                    if let Some(total) = u_json.get("totalKickpoints") {
                        obj.insert("totalKickpoints".to_string(), total.clone());
                    }
                }
            }
        }
    }

    HttpResponse::Ok().json(player_json)
}

// 8d. Get Player Identity (Discord Info, Protected: MEMBER or higher)
pub async fn get_player_identity(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);

    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user.user.as_ref().map(|u| u.linked_players.as_slice()).unwrap_or(&[]);
    let tag_str = if tag.starts_with('#') { tag.to_string() } else { format!("#{}", tag) };
    let is_exempt = exempt_tags.iter().any(|et| et == &tag_str);

    if !has_required_role(user_role, "MEMBER") && !is_exempt {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires MEMBER role".into(),
        });
    }

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&upstream_url_path)
        .fetch_optional(&data.db_pool)
        .await;

    match upstream_res {
        Ok(Some((body,))) => {
            let u_json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
            if let Some(obj) = u_json.as_object() {
                let mut identity = serde_json::Map::new();
                
                // Public-ish for members
                if let Some(nick) = obj.get("nickname") { identity.insert("nickname".to_string(), nick.clone()); }
                if let Some(gn) = obj.get("global_name") { identity.insert("global_name".to_string(), gn.clone()); }
                if let Some(un) = obj.get("username") { identity.insert("username".to_string(), un.clone()); }
                if let Some(av) = obj.get("avatar") { identity.insert("avatar".to_string(), av.clone()); }

                // Sensitive - COLEADER+ or self
                if has_required_role(user_role, "COLEADER") || is_exempt {
                    if let Some(uid) = obj.get("userId") { identity.insert("userId".to_string(), uid.clone()); }
                    if let Some(did) = obj.get("discordId") { identity.insert("discordId".to_string(), did.clone()); }
                    if let Some(accs) = obj.get("playerAccounts") { identity.insert("playerAccounts".to_string(), accs.clone()); }
                }

                return HttpResponse::Ok().json(identity);
            }
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Identity not found" }))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({ "error": "Identity not found in cache" })),
    }
}

// 8b. Get Player Kickpoints Summary
pub async fn get_player_kickpoints(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&upstream_url_path)
        .fetch_optional(&data.db_pool)
        .await;

    let u_json = match upstream_res {
        Ok(Some((body,))) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        _ => return HttpResponse::NotFound().finish(),
    };

    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user.user.as_ref().map(|u| u.linked_players.as_slice()).unwrap_or(&[]);

    let tag_str = u_json.get("tag").and_then(|t| t.as_str()).unwrap_or("");
    let tag_is_exempt = exempt_tags.iter().any(|et| et == tag_str);

    if !has_required_role(user_role, "MEMBER") && !tag_is_exempt {
         return HttpResponse::Forbidden().json(ErrorResponse { error: "Access denied: Requires MEMBER role".into() });
    }

    let active_count = u_json.get("activeKickpoints").and_then(|v| v.as_array()).map(|v| v.len()).unwrap_or(0);
    let active_sum: i64 = u_json.get("activeKickpoints").and_then(|v| v.as_array())
        .map(|v| v.iter().filter_map(|kp| kp.get("amount").and_then(|a| a.as_i64())).sum())
        .unwrap_or(0);
    let total = u_json.get("totalKickpoints").and_then(|v| v.as_i64()).unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({
        "total": total,
        "activeCount": active_count,
        "activeSum": active_sum,
    }))
}

// 8c. Get Player Kickpoints Details
pub async fn get_player_kickpoints_details(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&upstream_url_path)
        .fetch_optional(&data.db_pool)
        .await;

    let mut u_json = match upstream_res {
        Ok(Some((body,))) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        _ => return HttpResponse::NotFound().finish(),
    };

    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user.user.as_ref().map(|u| u.linked_players.as_slice()).unwrap_or(&[]);

    let tag_str = u_json.get("tag").and_then(|t| t.as_str()).unwrap_or("");
    let tag_is_exempt = exempt_tags.iter().any(|et| et == tag_str);

    if !has_required_role(user_role, "MEMBER") && !tag_is_exempt {
         return HttpResponse::Forbidden().json(ErrorResponse { error: "Access denied: Requires MEMBER role".into() });
    }

    let is_coleader = has_required_role(user_role, "COLEADER");

    if let Some(akp) = u_json.get_mut("activeKickpoints").and_then(|v| v.as_array_mut()) {
        if !is_coleader && !tag_is_exempt {
            for kp in akp.iter_mut() {
                if let Some(kp_obj) = kp.as_object_mut() {
                    kp_obj.remove("description");
                    kp_obj.remove("reason");
                }
            }
        }
        return HttpResponse::Ok().json(akp);
    }

    HttpResponse::Ok().json(serde_json::Value::Array(vec![]))
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

    let url_path = format!("/api/users/{}", user_id);
    // Ensure we have fresh user data for profiles (not pre-cached)
    let _ = update_cache(&data, &url_path).await;
    forward_request(&data, &url_path).await
}

// 10. Get Guild Info (Summarized for non-admins)
pub async fn get_guild_info(
    data: web::Data<AppState>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let user_role = opt_user.user.as_ref().and_then(|u| u.claims.role.as_deref());

    // 1. serve from cache ONLY
    let result =
        sqlx::query_as::<_, (Vec<u8>, i32)>("SELECT body, status FROM cache WHERE key = $1")
            .bind("/api/guild")
            .fetch_optional(&data.db_pool)
            .await;

    match result {
        Ok(Some((body, status))) => {
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);

            if !has_required_role(user_role, "ADMIN") {
                // Public summary
                if let Some(obj) = json.as_object() {
                    let mut summary = serde_json::Map::new();
                    if let Some(count) = obj.get("membercount") {
                        summary.insert("membercount".to_string(), count.clone());
                    }
                    if let Some(name) = obj.get("name") {
                        summary.insert("name".to_string(), name.clone());
                    }
                    if let Some(icon) = obj.get("icon") {
                        summary.insert("icon".to_string(), icon.clone());
                    }
                    return HttpResponse::Ok().json(summary);
                }
            }

            let status = actix_web::http::StatusCode::from_u16(status as u16)
                .unwrap_or(actix_web::http::StatusCode::OK);
            HttpResponse::build(status).json(json)
        }
        _ => HttpResponse::ServiceUnavailable().finish(),
    }
}

// 11. Helper for Player Aggregation
async fn fetch_aggregated_player_accounts(
    data: &web::Data<AppState>,
    linked_players: Vec<String>,
) -> Vec<serde_json::Value> {
    let mut players_data = Vec::new();
    for tag in linked_players {
        let encoded_tag = encode_tag(&tag);
        let clash_url_path = format!("/players/{}", encoded_tag);
        let upstream_url_path = format!("/api/players/{}", encoded_tag);

        // Fetch CoC data
        let coc_res = update_clash_cache(data, &clash_url_path).await;
        // Fetch Upstream data
        let upstream_res = update_cache(data, &upstream_url_path).await;

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
                                    } else if k != "tag" {
                                        coc_obj.insert(format!("upstream_{}", k), v.clone());
                                    }
                                }

                                // Clean implementation: only return counts in main list
                                if let Some(akp) = coc_obj.get("activeKickpoints").and_then(|v| v.as_array()) {
                                    let sum: i64 = akp.iter().filter_map(|kp| kp.get("amount").and_then(|a| a.as_i64())).sum();
                                    coc_obj.insert("activeKickpointsCount".to_string(), serde_json::json!(akp.len()));
                                    coc_obj.insert("activeKickpointsSum".to_string(), serde_json::json!(sum));
                                }
                                coc_obj.remove("activeKickpoints");
                            }
                        }
                    }
                }
                players_data.push(coc_json);
            }
        }
    }
    players_data
}

// 11b. Get My Player Accounts (Protected: User only)
pub async fn get_my_player_accounts(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let players_data = fetch_aggregated_player_accounts(&data, user.linked_players).await;
    HttpResponse::Ok().json(players_data)
}

// 11c. Get Another User's Player Accounts (Protected: Admin only)
pub async fn get_user_player_accounts(
    data: web::Data<AppState>,
    user_id: web::Path<String>,
    auth_user: AuthenticatedUser,
) -> impl Responder {
    if !has_required_role(auth_user.claims.role.as_deref(), "ADMIN") {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires ADMIN role".into(),
        });
    }

    let uid = user_id.into_inner();

    // 1. Try local DB
    let user_db = sqlx::query_as::<_, (String,)>(
        "SELECT linked_players FROM users WHERE discord_id = $1",
    )
    .bind(&uid)
    .fetch_optional(&data.db_pool)
    .await;

    let linked_players: Vec<String> = match user_db {
        Ok(Some((lp_json,))) => serde_json::from_str(&lp_json).unwrap_or_default(),
        _ => {
            // 2. Try Upstream
            let url_path = format!("/api/users/{}", uid);
            let upstream_res = update_cache(&data, &url_path).await;
            match upstream_res {
                Ok(body) => {
                    let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
                    json.get("linkedPlayers")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|p| p.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default()
                }
                _ => {
                    return HttpResponse::NotFound()
                        .json(serde_json::json!({ "error": "User not found" }));
                }
            }
        }
    };

    let players_data = fetch_aggregated_player_accounts(&data, linked_players).await;
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
