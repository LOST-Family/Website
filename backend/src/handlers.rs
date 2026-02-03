use crate::auth::{AuthenticatedUser, OptionalAuthenticatedUser, has_required_role};
use crate::models::{AppState, ErrorResponse, GameType};
use crate::utils::{
    encode_tag, filter_member_data, forward_request, forward_request_with_filter,
    get_cached_or_update_supercell_cache, get_cached_or_update_upstream_cache,
    update_upstream_cache,
};
use actix_web::{HttpResponse, Responder, web};
use bytes::Bytes;
use futures_util::future::join_all;
use log::error;

// ============================================================================
// CLASH OF CLANS HANDLERS
// ============================================================================

// 1. Get All CoC Clans
pub async fn get_coc_clans(
    data: web::Data<AppState>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    forward_request_with_filter(&data, GameType::ClashOfClans, "/api/clans", user_role, &[]).await
}

// 2. Get CoC Clan Info
pub async fn get_coc_clan_info(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    _opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_clan_info_impl(&data, &tag, GameType::ClashOfClans).await
}

// 2b. Get CoC Clan Config
pub async fn get_coc_clan_config(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_clan_config_impl(&data, &tag, opt_user, GameType::ClashOfClans).await
}

// 3. Get CoC Clan Members
pub async fn get_coc_clan_members(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_clan_members_impl(&data, &tag, opt_user, GameType::ClashOfClans).await
}

// 3b. Get CoC Clan Members Lite (No Supercell API data)
pub async fn get_coc_clan_members_lite(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_clan_members_lite_impl(&data, &tag, opt_user, GameType::ClashOfClans).await
}

// 4. Get CoC Clan Kickpoint Reasons
pub async fn get_coc_clan_kickpoint_reasons(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    user: AuthenticatedUser,
) -> impl Responder {
    get_clan_kickpoint_reasons_impl(&data, &tag, user, GameType::ClashOfClans).await
}

// 5. Get CoC Clan War Members
pub async fn get_coc_clan_war_members(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| u.linked_players.as_slice())
        .unwrap_or(&[]);
    forward_request_with_filter(
        &data,
        GameType::ClashOfClans,
        &format!("/api/clans/{}/war-members", encoded_tag),
        user_role,
        exempt_tags,
    )
    .await
}

// 6. Get CoC Raid Members
pub async fn get_coc_raid_members(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| u.linked_players.as_slice())
        .unwrap_or(&[]);
    forward_request_with_filter(
        &data,
        GameType::ClashOfClans,
        &format!("/api/clans/{}/raid-members", encoded_tag),
        user_role,
        exempt_tags,
    )
    .await
}

// 7. Get CoC CWL Members
pub async fn get_coc_cwl_members(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| u.linked_players.as_slice())
        .unwrap_or(&[]);
    forward_request_with_filter(
        &data,
        GameType::ClashOfClans,
        &format!("/api/clans/{}/cwl-members", encoded_tag),
        user_role,
        exempt_tags,
    )
    .await
}

// 8. Get CoC Player
pub async fn get_coc_player(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_player_impl(&data, &tag, opt_user, GameType::ClashOfClans).await
}

// 8b. Get CoC Player Identity
pub async fn get_coc_player_identity(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_player_identity_impl(&data, &tag, opt_user, GameType::ClashOfClans).await
}

// 8c. Get CoC Player Kickpoints
pub async fn get_coc_player_kickpoints(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_player_kickpoints_impl(&data, &tag, opt_user, GameType::ClashOfClans).await
}

// 8d. Get CoC Player Kickpoints Details
pub async fn get_coc_player_kickpoints_details(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_player_kickpoints_details_impl(&data, &tag, opt_user, GameType::ClashOfClans).await
}

// ============================================================================
// CLASH ROYALE HANDLERS
// ============================================================================

// 1. Get All CR Clans
pub async fn get_cr_clans(
    data: web::Data<AppState>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    forward_request_with_filter(&data, GameType::ClashRoyale, "/api/clans", user_role, &[]).await
}

// 2. Get CR Clan Info
pub async fn get_cr_clan_info(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    _opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_clan_info_impl(&data, &tag, GameType::ClashRoyale).await
}

// 2b. Get CR Clan Config
pub async fn get_cr_clan_config(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_clan_config_impl(&data, &tag, opt_user, GameType::ClashRoyale).await
}

// 3. Get CR Clan Members
pub async fn get_cr_clan_members(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_clan_members_impl(&data, &tag, opt_user, GameType::ClashRoyale).await
}

// 3b. Get CR Clan Members Lite (No Supercell API data)
pub async fn get_cr_clan_members_lite(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_clan_members_lite_impl(&data, &tag, opt_user, GameType::ClashRoyale).await
}

// 4. Get CR Clan Kickpoint Reasons
pub async fn get_cr_clan_kickpoint_reasons(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    user: AuthenticatedUser,
) -> impl Responder {
    get_clan_kickpoint_reasons_impl(&data, &tag, user, GameType::ClashRoyale).await
}

// 5. Get CR Player
pub async fn get_cr_player(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_player_impl(&data, &tag, opt_user, GameType::ClashRoyale).await
}

// 5b. Get CR Player Identity
pub async fn get_cr_player_identity(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_player_identity_impl(&data, &tag, opt_user, GameType::ClashRoyale).await
}

// 5c. Get CR Player Kickpoints
pub async fn get_cr_player_kickpoints(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_player_kickpoints_impl(&data, &tag, opt_user, GameType::ClashRoyale).await
}

// 5d. Get CR Player Kickpoints Details
pub async fn get_cr_player_kickpoints_details(
    data: web::Data<AppState>,
    tag: web::Path<String>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    get_player_kickpoints_details_impl(&data, &tag, opt_user, GameType::ClashRoyale).await
}

// ============================================================================
// SHARED IMPLEMENTATION FUNCTIONS
// ============================================================================

fn get_cache_prefix(game: GameType) -> &'static str {
    match game {
        GameType::ClashOfClans => "coc",
        GameType::ClashRoyale => "cr",
    }
}

async fn get_clan_info_impl(data: &web::Data<AppState>, tag: &str, game: GameType) -> HttpResponse {
    let encoded_tag = encode_tag(tag);
    let supercell_url_path = format!("/clans/{}", encoded_tag);

    // 1. Get from Supercell cache (or update if missing/expired)
    // We use a relatively long TTL (1 hour) because background task should keep it fresh
    let sc_res = get_cached_or_update_supercell_cache(data, game, &supercell_url_path, 3600).await;

    let mut clan_json = match sc_res {
        Ok(body) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        Err(e) => {
            error!("Error fetching clan info: {}", e);
            return HttpResponse::NotFound().json(serde_json::json!({ "error": "Clan not found" }));
        }
    };

    // 2. Get from Upstream cache and merge
    let upstream_url_path = format!("/api/clans/{}", encoded_tag);
    let up_res = get_cached_or_update_upstream_cache(data, game, &upstream_url_path, 3600).await;

    if let Ok(up_body) = up_res
        && let Ok(up_json) = serde_json::from_slice::<serde_json::Value>(&up_body)
        && let (Some(sc_obj), Some(up_obj)) = (clan_json.as_object_mut(), up_json.as_object())
    {
        // Fields to merge from upstream (override or add)
        let fields_to_merge = [
            "nameDB",
            "index",
            "description",
            "maxKickpoints",
            "minSeasonWins",
            "kickpointsExpireAfterDays",
            "kickpointReasons",
        ];

        for field in &fields_to_merge {
            if let Some(val) = up_obj.get(*field) {
                sc_obj.insert((*field).to_string(), val.clone());
            }
        }
    }

    // Fix badgeUrl mismatch (singular vs plural)
    if let Some(obj) = clan_json.as_object_mut()
        && !obj.contains_key("badgeUrls")
        && let Some(url) = obj.get("badgeUrl").and_then(|u| u.as_str())
    {
        obj.insert(
            "badgeUrls".to_string(),
            serde_json::json!({
                "small": url,
                "medium": url,
                "large": url
            }),
        );
    }

    HttpResponse::Ok().json(clan_json)
}

async fn get_clan_config_impl(
    data: &web::Data<AppState>,
    tag: &str,
    opt_user: OptionalAuthenticatedUser,
    game: GameType,
) -> HttpResponse {
    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    if !has_required_role(user_role, "MEMBER") {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires MEMBER role".into(),
        });
    }

    let encoded_tag = encode_tag(tag);
    let prefix = get_cache_prefix(game);
    let upstream_url_path = format!("/api/clans/{}", encoded_tag);
    let cache_key = format!("{}:upstream:{}", prefix, upstream_url_path);

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&cache_key)
        .fetch_optional(&data.db_pool)
        .await;

    match upstream_res {
        Ok(Some((body,))) => {
            let json: serde_json::Value =
                serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
            if let Some(obj) = json.as_object() {
                let mut config = serde_json::Map::new();
                let fields = [
                    "maxKickpoints",
                    "minSeasonWins",
                    "kickpointsExpireAfterDays",
                    "kickpointReasons",
                ];
                for f in fields {
                    if let Some(v) = obj.get(f) {
                        config.insert(f.to_string(), v.clone());
                    }
                }
                return HttpResponse::Ok().json(config);
            }
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Clan config not found" }))
        }
        _ => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Clan config not found in cache" })),
    }
}

async fn get_clan_members_impl(
    data: &web::Data<AppState>,
    tag: &str,
    opt_user: OptionalAuthenticatedUser,
    game: GameType,
) -> HttpResponse {
    let encoded_tag = encode_tag(tag);
    let prefix = get_cache_prefix(game);
    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| {
            if game == GameType::ClashRoyale {
                u.linked_cr_players.as_slice()
            } else {
                u.linked_players.as_slice()
            }
        })
        .unwrap_or(&[]);

    let supercell_url_path = format!("/clans/{}", encoded_tag);
    let upstream_url_path = format!("/api/clans/{}/members", encoded_tag);

    // Get bodies from cache (or update if missing/expired)
    let supercell_res =
        get_cached_or_update_supercell_cache(data, game, &supercell_url_path, 3600).await;

    let upstream_res =
        get_cached_or_update_upstream_cache(data, game, &upstream_url_path, 3600).await;

    let mut supercell_members = match supercell_res {
        Ok(body) => {
            let json: serde_json::Value =
                serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
            json["memberList"].as_array().cloned().unwrap_or_default()
        }
        _ => vec![],
    };

    let upstream_body = match upstream_res {
        Ok(body) => body,
        _ => Bytes::new(),
    };

    // Filter upstream members first (privacy logic)
    let filtered_upstream_body = filter_member_data(upstream_body, exempt_tags, user_role);
    let upstream_members: Vec<serde_json::Value> =
        serde_json::from_slice(&filtered_upstream_body).unwrap_or_default();

    // Helper to normalize tags for comparison (handle casing and # prefix)
    let normalize_tag = |t: &str| t.to_uppercase().trim_start_matches('#').to_string();

    // Track tags already processed from supercell
    let supercell_tags: Vec<String> = supercell_members
        .iter()
        .filter_map(|m| m.get("tag").and_then(|t| t.as_str()).map(&normalize_tag))
        .collect();

    // Merge upstream data into supercell list
    for s_member in &mut supercell_members {
        if let Some(obj) = s_member.as_object_mut() {
            obj.insert("in_supercell".to_string(), serde_json::Value::Bool(true));
        }

        if let Some(tag_ref) = s_member.get("tag").and_then(|t| t.as_str()) {
            let member_tag = tag_ref.to_string();
            let norm_tag = normalize_tag(&member_tag);

            // Find matching upstream member
            if let Some(u_member) = upstream_members.iter().find(|m| {
                m.get("tag").and_then(|t| t.as_str()).map(&normalize_tag) == Some(norm_tag.clone())
            }) {
                if let (Some(s_obj), Some(u_obj)) = (s_member.as_object_mut(), u_member.as_object())
                {
                    s_obj.insert("in_upstream".to_string(), serde_json::Value::Bool(true));
                    let mut is_dirty = false;
                    for (k, v) in u_obj {
                        if !s_obj.contains_key(k) {
                            s_obj.insert(k.clone(), v.clone());
                        } else if k != "tag" {
                            // Check for differences in common fields
                            if k == "name" || k == "role" || k == "expLevel" {
                                let s_val = s_obj.get(k);

                                let is_truly_diff = match (k.as_str(), s_val, Some(v)) {
                                    ("name", Some(sv), Some(uv)) => sv.as_str() != uv.as_str(),
                                    ("role", Some(sv), Some(uv)) => {
                                        let s_role = sv.as_str().unwrap_or("").to_lowercase();
                                        let u_role = uv.as_str().unwrap_or("").to_lowercase();
                                        // Normalize roles: leader, coleader, admin/elder, member
                                        let norm_s = if s_role == "admin" || s_role == "elder" {
                                            "admin"
                                        } else {
                                            &s_role
                                        };
                                        let norm_u = if u_role == "admin" || u_role == "elder" {
                                            "admin"
                                        } else {
                                            &u_role
                                        };
                                        norm_s != norm_u
                                    }
                                    ("expLevel", Some(sv), Some(uv)) => {
                                        // Compare numeric values regardless of JSON type (string vs number)
                                        let s_num = sv
                                            .as_i64()
                                            .or_else(|| sv.as_str().and_then(|s| s.parse().ok()));
                                        let u_num = uv
                                            .as_i64()
                                            .or_else(|| uv.as_str().and_then(|s| s.parse().ok()));
                                        s_num != u_num
                                    }
                                    _ => false,
                                };

                                if is_truly_diff {
                                    is_dirty = true;
                                }
                            }
                            s_obj.insert(format!("upstream_{}", k), v.clone());
                        }
                    }
                    s_obj.insert("is_dirty".to_string(), serde_json::Value::Bool(is_dirty));
                    s_obj.insert("is_diff".to_string(), serde_json::Value::Bool(is_dirty));
                    s_obj.insert("is_new".to_string(), serde_json::Value::Bool(false));
                    s_obj.insert("is_left".to_string(), serde_json::Value::Bool(false));
                }
            } else if let Some(obj) = s_member.as_object_mut() {
                obj.insert("in_upstream".to_string(), serde_json::Value::Bool(false));
                obj.insert("is_dirty".to_string(), serde_json::Value::Bool(false));
                obj.insert("is_diff".to_string(), serde_json::Value::Bool(true));
                obj.insert("is_new".to_string(), serde_json::Value::Bool(true));
                obj.insert("is_left".to_string(), serde_json::Value::Bool(false));
            }

            // For CoC: Try to get additional data from player cache
            if game == GameType::ClashOfClans {
                let player_cache_key =
                    format!("{}:supercell:/players/{}", prefix, encode_tag(&member_tag));
                let player_res =
                    sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
                        .bind(&player_cache_key)
                        .fetch_optional(&data.db_pool)
                        .await;

                if let Ok(Some((p_body,))) = player_res
                    && let Ok(p_json) = serde_json::from_slice::<serde_json::Value>(&p_body)
                    && let (Some(s_obj), Some(p_obj)) =
                        (s_member.as_object_mut(), p_json.as_object())
                {
                    if let Some(stars) = p_obj.get("warStars") {
                        s_obj.insert("warStars".to_string(), stars.clone());
                    }
                    if let Some(heroes) = p_obj.get("heroes") {
                        s_obj.insert("heroes".to_string(), heroes.clone());
                    }
                    if let Some(league) = p_obj.get("league") {
                        s_obj.insert("league".to_string(), league.clone());
                    }
                }
            }
        }
    }

    // Add members that are only in upstream (Left members)
    let mut final_members = supercell_members;
    for u_member in upstream_members {
        if let Some(u_tag) = u_member.get("tag").and_then(|t| t.as_str())
            && !supercell_tags.contains(&normalize_tag(u_tag))
        {
            let mut mixed_member = u_member.clone();
            if let Some(obj) = mixed_member.as_object_mut() {
                obj.insert("in_supercell".to_string(), serde_json::Value::Bool(false));
                obj.insert("in_upstream".to_string(), serde_json::Value::Bool(true));
                obj.insert("is_dirty".to_string(), serde_json::Value::Bool(false));
                obj.insert("is_diff".to_string(), serde_json::Value::Bool(true));
                obj.insert("is_new".to_string(), serde_json::Value::Bool(false));
                obj.insert("is_left".to_string(), serde_json::Value::Bool(true));

                // Cache check for left members to get their name/TH if bot is missing it
                let player_cache_key =
                    format!("{}:supercell:/players/{}", prefix, encode_tag(u_tag));
                let player_res =
                    sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
                        .bind(&player_cache_key)
                        .fetch_optional(&data.db_pool)
                        .await;

                if let Ok(Some((p_body,))) = player_res
                    && let Ok(p_json) = serde_json::from_slice::<serde_json::Value>(&p_body)
                    && let Some(p_obj) = p_json.as_object()
                {
                    for (pk, pv) in p_obj {
                        if !obj.contains_key(pk) {
                            obj.insert(pk.clone(), pv.clone());
                        }
                    }
                }

                // Map upstream data to standard fields if still missing
                if !obj.contains_key("name") {
                    if let Some(un) = obj.get("upstream_name").or_else(|| obj.get("nickname")) {
                        obj.insert("name".to_string(), un.clone());
                    } else {
                        obj.insert(
                            "name".to_string(),
                            serde_json::Value::String(u_tag.to_string()),
                        );
                    }
                }
                if !obj.contains_key("role") {
                    if let Some(ur) = obj.get("upstream_role") {
                        obj.insert("role".to_string(), ur.clone());
                    } else {
                        obj.insert(
                            "role".to_string(),
                            serde_json::Value::String("member".to_string()),
                        );
                    }
                }
            }
            final_members.push(mixed_member);
        }
    }

    HttpResponse::Ok().json(final_members)
}

async fn get_clan_members_lite_impl(
    data: &web::Data<AppState>,
    tag: &str,
    opt_user: OptionalAuthenticatedUser,
    game: GameType,
) -> HttpResponse {
    let encoded_tag = encode_tag(tag);
    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| {
            if game == GameType::ClashRoyale {
                u.linked_cr_players.as_slice()
            } else {
                u.linked_players.as_slice()
            }
        })
        .unwrap_or(&[]);

    let upstream_url_path = format!("/api/clans/{}/members-lite", encoded_tag);
    forward_request_with_filter(data, game, &upstream_url_path, user_role, exempt_tags).await
}

async fn get_clan_kickpoint_reasons_impl(
    data: &web::Data<AppState>,
    tag: &str,
    user: AuthenticatedUser,
    game: GameType,
) -> HttpResponse {
    if !has_required_role(user.claims.role.as_deref(), "COLEADER") {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires COLEADER or higher role".into(),
        });
    }

    let encoded_tag = encode_tag(tag);
    forward_request(
        data,
        game,
        &format!("/api/clans/{}/kickpoint-reasons", encoded_tag),
    )
    .await
}

async fn get_player_impl(
    data: &web::Data<AppState>,
    tag: &str,
    opt_user: OptionalAuthenticatedUser,
    game: GameType,
) -> HttpResponse {
    let encoded_tag = encode_tag(tag);
    let supercell_url_path = format!("/players/{}", encoded_tag);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);

    // Get cached or update (5min TTL)
    let supercell_res =
        get_cached_or_update_supercell_cache(data, game, &supercell_url_path, 300).await;
    let upstream_res =
        get_cached_or_update_upstream_cache(data, game, &upstream_url_path, 300).await;

    let mut player_json = match supercell_res {
        Ok(body) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        _ => return HttpResponse::NotFound().finish(),
    };

    // Fetch upstream summary if user is authorized
    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| {
            if game == GameType::ClashRoyale {
                u.linked_cr_players.as_slice()
            } else {
                u.linked_players.as_slice()
            }
        })
        .unwrap_or(&[]);
    let tag_str = player_json
        .get("tag")
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let is_exempt = exempt_tags.iter().any(|et| et == tag_str);

    if (has_required_role(user_role, "MEMBER") || is_exempt)
        && let Ok(u_body) = upstream_res
        && let Ok(u_json) = serde_json::from_slice::<serde_json::Value>(&u_body)
        && let Some(obj) = player_json.as_object_mut()
    {
        // Only merge kickpoint summaries, NOT identity
        if let Some(akp) = u_json.get("activeKickpoints").and_then(|v| v.as_array()) {
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
        if let Some(total) = u_json.get("totalKickpoints") {
            obj.insert("totalKickpoints".to_string(), total.clone());
        }
    }

    HttpResponse::Ok().json(player_json)
}

async fn get_player_identity_impl(
    data: &web::Data<AppState>,
    tag: &str,
    opt_user: OptionalAuthenticatedUser,
    game: GameType,
) -> HttpResponse {
    let encoded_tag = encode_tag(tag);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);

    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| {
            if game == GameType::ClashRoyale {
                u.linked_cr_players.as_slice()
            } else {
                u.linked_players.as_slice()
            }
        })
        .unwrap_or(&[]);
    let tag_str = if tag.starts_with('#') {
        tag.to_string()
    } else {
        format!("#{}", tag)
    };
    let is_exempt = exempt_tags.iter().any(|et| et == &tag_str);

    if !has_required_role(user_role, "MEMBER") && !is_exempt {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires MEMBER role".into(),
        });
    }

    // Get cached or update (5min TTL)
    let upstream_res =
        get_cached_or_update_upstream_cache(data, game, &upstream_url_path, 300).await;

    match upstream_res {
        Ok(body) => {
            let u_json: serde_json::Value =
                serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
            if let Some(obj) = u_json.as_object() {
                let mut identity = serde_json::Map::new();

                // Public-ish for members
                if let Some(nick) = obj.get("nickname") {
                    identity.insert("nickname".to_string(), nick.clone());
                }
                if let Some(gn) = obj.get("global_name") {
                    identity.insert("global_name".to_string(), gn.clone());
                }
                if let Some(un) = obj.get("username") {
                    identity.insert("username".to_string(), un.clone());
                }
                if let Some(av) = obj.get("avatar") {
                    identity.insert("avatar".to_string(), av.clone());
                }

                // Sensitive - COLEADER+ or self
                if has_required_role(user_role, "COLEADER") || is_exempt {
                    if let Some(uid) = obj.get("userId") {
                        identity.insert("userId".to_string(), uid.clone());
                    }
                    if let Some(did) = obj.get("discordId") {
                        identity.insert("discordId".to_string(), did.clone());
                    }
                    if let Some(accs) = obj.get("playerAccounts") {
                        identity.insert("playerAccounts".to_string(), accs.clone());
                    }
                }

                return HttpResponse::Ok().json(identity);
            }
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Identity not found" }))
        }
        _ => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Identity not found in cache" })),
    }
}

async fn get_player_kickpoints_impl(
    data: &web::Data<AppState>,
    tag: &str,
    opt_user: OptionalAuthenticatedUser,
    game: GameType,
) -> HttpResponse {
    let encoded_tag = encode_tag(tag);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);

    // Get cached or update (5min TTL)
    let upstream_res =
        get_cached_or_update_upstream_cache(data, game, &upstream_url_path, 300).await;

    let u_json = match upstream_res {
        Ok(body) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        _ => return HttpResponse::NotFound().finish(),
    };

    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| {
            if game == GameType::ClashRoyale {
                u.linked_cr_players.as_slice()
            } else {
                u.linked_players.as_slice()
            }
        })
        .unwrap_or(&[]);

    let tag_str = u_json.get("tag").and_then(|t| t.as_str()).unwrap_or("");
    let tag_is_exempt = exempt_tags.iter().any(|et| et == tag_str);

    if !has_required_role(user_role, "MEMBER") && !tag_is_exempt {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires MEMBER role".into(),
        });
    }

    let active_count = u_json
        .get("activeKickpoints")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .unwrap_or(0);
    let active_sum: i64 = u_json
        .get("activeKickpoints")
        .and_then(|v| v.as_array())
        .map(|v| {
            v.iter()
                .filter_map(|kp| kp.get("amount").and_then(|a| a.as_i64()))
                .sum()
        })
        .unwrap_or(0);
    let total = u_json
        .get("totalKickpoints")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({
        "total": total,
        "activeCount": active_count,
        "activeSum": active_sum,
    }))
}

async fn get_player_kickpoints_details_impl(
    data: &web::Data<AppState>,
    tag: &str,
    opt_user: OptionalAuthenticatedUser,
    game: GameType,
) -> HttpResponse {
    let encoded_tag = encode_tag(tag);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);

    // Get cached or update (5min TTL)
    let upstream_res =
        get_cached_or_update_upstream_cache(data, game, &upstream_url_path, 300).await;

    let mut u_json = match upstream_res {
        Ok(body) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        _ => return HttpResponse::NotFound().finish(),
    };

    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());
    let exempt_tags = opt_user
        .user
        .as_ref()
        .map(|u| {
            if game == GameType::ClashRoyale {
                u.linked_cr_players.as_slice()
            } else {
                u.linked_players.as_slice()
            }
        })
        .unwrap_or(&[]);

    let tag_str = u_json.get("tag").and_then(|t| t.as_str()).unwrap_or("");
    let tag_is_exempt = exempt_tags.iter().any(|et| et == tag_str);

    if !has_required_role(user_role, "MEMBER") && !tag_is_exempt {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires MEMBER role".into(),
        });
    }

    let is_coleader = has_required_role(user_role, "COLEADER");

    if let Some(akp) = u_json
        .get_mut("activeKickpoints")
        .and_then(|v| v.as_array_mut())
    {
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

// ============================================================================
// USER/PROFILE HANDLERS
// ============================================================================

// Get User
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

    // Update both caches
    let _ = update_upstream_cache(&data, GameType::ClashOfClans, &url_path).await;
    let _ = update_upstream_cache(&data, GameType::ClashRoyale, &url_path).await;

    // Fetch both from cache
    let coc_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(format!("coc:upstream:{}", url_path))
        .fetch_optional(&data.db_pool)
        .await;

    let cr_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(format!("cr:upstream:{}", url_path))
        .fetch_optional(&data.db_pool)
        .await;

    let coc_data: Option<serde_json::Value> = match coc_res {
        Ok(Some((body,))) => serde_json::from_slice(&body).ok(),
        _ => None,
    };

    let cr_data: Option<serde_json::Value> = match cr_res {
        Ok(Some((body,))) => serde_json::from_slice(&body).ok(),
        _ => None,
    };

    match (coc_data, cr_data) {
        (None, None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "User not found in any upstream bot".into(),
        }),
        (Some(coc), None) => HttpResponse::Ok().json(coc),
        (None, Some(cr)) => HttpResponse::Ok().json(cr),
        (Some(mut coc), Some(cr)) => {
            // Merge CR into CoC data
            if let (Some(coc_obj), Some(cr_obj)) = (coc.as_object_mut(), cr.as_object()) {
                // Admin: true if either is true
                let coc_admin = coc_obj
                    .get("admin")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let cr_admin = cr_obj
                    .get("admin")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                coc_obj.insert(
                    "admin".to_string(),
                    serde_json::json!(coc_admin || cr_admin),
                );

                // Highest Role: max of both
                let coc_role = coc_obj
                    .get("highestRole")
                    .and_then(|v| v.as_str())
                    .unwrap_or("NOTMEMBER");
                let cr_role = cr_obj
                    .get("highestRole")
                    .and_then(|v| v.as_str())
                    .unwrap_or("NOTMEMBER");
                if crate::auth::get_role_priority(cr_role)
                    > crate::auth::get_role_priority(coc_role)
                {
                    coc_obj.insert("highestRole".to_string(), serde_json::json!(cr_role));
                }

                // Linked Players: merge and deduplicate
                let mut coc_linked = coc_obj
                    .get("linkedPlayers")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if let Some(cr_linked) = cr_obj.get("linkedPlayers").and_then(|v| v.as_array()) {
                    for tag in cr_linked {
                        if !coc_linked.contains(tag) {
                            coc_linked.push(tag.clone());
                        }
                    }
                }
                coc_obj.insert("linkedPlayers".to_string(), serde_json::json!(coc_linked));

                // Linked CR Players: merge and deduplicate
                let mut coc_cr_linked = coc_obj
                    .get("linkedCrPlayers")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if let Some(cr_cr_linked) = cr_obj.get("linkedCrPlayers").and_then(|v| v.as_array())
                {
                    for tag in cr_cr_linked {
                        if !coc_cr_linked.contains(tag) {
                            coc_cr_linked.push(tag.clone());
                        }
                    }
                }
                coc_obj.insert(
                    "linkedCrPlayers".to_string(),
                    serde_json::json!(coc_cr_linked),
                );

                // Nickname: prefer CoC but if null take CR
                if coc_obj.get("nickname").is_none_or(|v| v.is_null())
                    && let Some(cr_nick) = cr_obj.get("nickname")
                {
                    coc_obj.insert("nickname".to_string(), cr_nick.clone());
                }
            }
            HttpResponse::Ok().json(coc)
        }
    }
}

// Helper for Player Aggregation - now fetches both CoC and CR
async fn fetch_aggregated_player_accounts(
    data: &web::Data<AppState>,
    coc_linked_players: Vec<String>,
    cr_linked_players: Vec<String>,
) -> serde_json::Value {
    let mut coc_players_futures = Vec::new();
    let mut cr_players_futures = Vec::new();

    // Prepare CoC futures
    for tag in coc_linked_players {
        let data = data.clone();
        coc_players_futures.push(async move {
            let encoded_tag = encode_tag(&tag);
            let supercell_url_path = format!("/players/{}", encoded_tag);
            let upstream_url_path = format!("/api/players/{}", encoded_tag);

            let supercell_res = get_cached_or_update_supercell_cache(
                &data,
                GameType::ClashOfClans,
                &supercell_url_path,
                300,
            )
            .await;
            let upstream_res = get_cached_or_update_upstream_cache(
                &data,
                GameType::ClashOfClans,
                &upstream_url_path,
                300,
            )
            .await;

            if let Ok(supercell_body) = supercell_res
                && let Ok(mut player_json) =
                    serde_json::from_slice::<serde_json::Value>(&supercell_body)
            {
                if let Some(player_obj) = player_json.as_object_mut() {
                    player_obj.insert("gameType".to_string(), serde_json::json!("coc"));

                    if let Ok(u_body) = upstream_res
                        && let Ok(u_json) = serde_json::from_slice::<serde_json::Value>(&u_body)
                        && let Some(u_obj) = u_json.as_object()
                    {
                        for (k, v) in u_obj {
                            if !player_obj.contains_key(k) {
                                player_obj.insert(k.clone(), v.clone());
                            } else if k != "tag" {
                                player_obj.insert(format!("upstream_{}", k), v.clone());
                            }
                        }
                        if let Some(akp) = player_obj
                            .get("activeKickpoints")
                            .and_then(|v| v.as_array())
                        {
                            let sum: i64 = akp
                                .iter()
                                .filter_map(|kp| kp.get("amount").and_then(|a| a.as_i64()))
                                .sum();
                            player_obj.insert(
                                "activeKickpointsCount".to_string(),
                                serde_json::json!(akp.len()),
                            );
                            player_obj
                                .insert("activeKickpointsSum".to_string(), serde_json::json!(sum));
                        }
                    }
                    return Some(player_json);
                }
            }
            None
        });
    }

    // Prepare CR futures
    for tag in cr_linked_players {
        let data = data.clone();
        cr_players_futures.push(async move {
            let encoded_tag = encode_tag(&tag);
            let supercell_url_path = format!("/players/{}", encoded_tag);
            let upstream_url_path = format!("/api/players/{}", encoded_tag);

            let supercell_res = get_cached_or_update_supercell_cache(
                &data,
                GameType::ClashRoyale,
                &supercell_url_path,
                300,
            )
            .await;
            let upstream_res = get_cached_or_update_upstream_cache(
                &data,
                GameType::ClashRoyale,
                &upstream_url_path,
                300,
            )
            .await;

            if let Ok(supercell_body) = supercell_res
                && let Ok(mut player_json) =
                    serde_json::from_slice::<serde_json::Value>(&supercell_body)
            {
                if let Some(player_obj) = player_json.as_object_mut() {
                    player_obj.insert("gameType".to_string(), serde_json::json!("cr"));

                    if let Ok(u_body) = upstream_res
                        && let Ok(u_json) = serde_json::from_slice::<serde_json::Value>(&u_body)
                        && let Some(u_obj) = u_json.as_object()
                    {
                        for (k, v) in u_obj {
                            if !player_obj.contains_key(k) {
                                player_obj.insert(k.clone(), v.clone());
                            } else if k != "tag" {
                                player_obj.insert(format!("upstream_{}", k), v.clone());
                            }
                        }
                        if let Some(akp) = player_obj
                            .get("activeKickpoints")
                            .and_then(|v| v.as_array())
                        {
                            let sum: i64 = akp
                                .iter()
                                .filter_map(|kp| kp.get("amount").and_then(|a| a.as_i64()))
                                .sum();
                            player_obj.insert(
                                "activeKickpointsCount".to_string(),
                                serde_json::json!(akp.len()),
                            );
                            player_obj
                                .insert("activeKickpointsSum".to_string(), serde_json::json!(sum));
                        }
                    }
                    return Some(player_json);
                }
            }
            None
        });
    }

    let coc_players_data: Vec<serde_json::Value> = join_all(coc_players_futures)
        .await
        .into_iter()
        .flatten()
        .collect();
    let cr_players_data: Vec<serde_json::Value> = join_all(cr_players_futures)
        .await
        .into_iter()
        .flatten()
        .collect();

    serde_json::json!({
        "coc": coc_players_data,
        "cr": cr_players_data
    })
}

// Sync user accounts from upstreams and update DB
async fn sync_user_accounts(
    data: &AppState,
    discord_id: &str,
    current_coc: Vec<String>,
    current_cr: Vec<String>,
) -> (Vec<String>, Vec<String>) {
    let mut coc_players = current_coc;
    let mut cr_players = current_cr;
    let mut modified = false;

    let url_path = format!("/api/users/{}", discord_id);

    // 1. Fetch from Upstreams (TTL 0 to force refresh)
    let coc_bot_res =
        get_cached_or_update_upstream_cache(data, GameType::ClashOfClans, &url_path, 0).await;
    let cr_bot_res =
        get_cached_or_update_upstream_cache(data, GameType::ClashRoyale, &url_path, 0).await;

    let mut bot_a_coc = Vec::new();
    let mut bot_a_cr = Vec::new();
    let mut bot_a_success = false;

    if let Ok(body) = coc_bot_res
        && let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body)
    {
        bot_a_coc = json
            .get("linkedPlayers")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        bot_a_cr = json
            .get("linkedCrPlayers")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        bot_a_success = true;
    }

    let mut bot_b_cr = Vec::new();
    let mut bot_b_success = false;

    if let Ok(body) = cr_bot_res
        && let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body)
    {
        // CR bot: both fields are typically CR players
        let mut tags = Vec::new();
        if let Some(arr) = json.get("linkedPlayers").and_then(|v| v.as_array()) {
            tags.extend(arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())));
        }
        if let Some(arr) = json.get("linkedCrPlayers").and_then(|v| v.as_array()) {
            tags.extend(arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())));
        }
        tags.sort();
        tags.dedup();
        bot_b_cr = tags;
        bot_b_success = true;
    }

    // 2. Reconcile CoC (Authority is Bot A)
    if bot_a_success && coc_players != bot_a_coc {
        coc_players = bot_a_coc;
        modified = true;
    }

    // 3. Reconcile CR (Authority is Union of Bot A and Bot B if both present)
    if bot_a_success && bot_b_success {
        // Both bots reachable: union is the true state
        let mut final_bot_cr = bot_a_cr;
        for t in bot_b_cr {
            if !final_bot_cr.contains(&t) {
                final_bot_cr.push(t);
            }
        }
        final_bot_cr.sort();

        let mut sorted_current = cr_players.clone();
        sorted_current.sort();

        if sorted_current != final_bot_cr {
            cr_players = final_bot_cr;
            modified = true;
        }
    } else if bot_a_success {
        // Only Bot A up: ensure its contributions are present
        for tag in bot_a_cr {
            if !cr_players.contains(&tag) {
                cr_players.push(tag);
                modified = true;
            }
        }
    } else if bot_b_success {
        // Only Bot B up: ensure its contributions are present
        for tag in bot_b_cr {
            if !cr_players.contains(&tag) {
                cr_players.push(tag);
                modified = true;
            }
        }
    }

    if modified {
        let _ = sqlx::query(
            "UPDATE users SET linked_players = $1, linked_cr_players = $2 WHERE discord_id = $3",
        )
        .bind(serde_json::to_string(&coc_players).unwrap_or_else(|_| "[]".to_string()))
        .bind(serde_json::to_string(&cr_players).unwrap_or_else(|_| "[]".to_string()))
        .bind(discord_id)
        .execute(&data.db_pool)
        .await;
    }

    (coc_players, cr_players)
}

// Get My Player Accounts
pub async fn get_my_player_accounts(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let (coc_players, cr_players) = sync_user_accounts(
        &data,
        &user.claims.sub,
        user.linked_players.clone(),
        user.linked_cr_players.clone(),
    )
    .await;

    let players_data = fetch_aggregated_player_accounts(&data, coc_players, cr_players).await;
    HttpResponse::Ok().json(players_data)
}

// Get Another User's Player Accounts
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

    // Try local DB first
    let user_db = sqlx::query_as::<_, (String, String)>(
        "SELECT COALESCE(linked_players, '[]'), COALESCE(linked_cr_players, '[]') FROM users WHERE discord_id = $1",
    )
    .bind(&uid)
    .fetch_optional(&data.db_pool)
    .await;

    let (coc_linked, cr_linked): (Vec<String>, Vec<String>) = match user_db {
        Ok(Some((lp_json, cr_json))) => (
            serde_json::from_str(&lp_json).unwrap_or_default(),
            serde_json::from_str(&cr_json).unwrap_or_default(),
        ),
        _ => (Vec::new(), Vec::new()),
    };

    let (coc_linked, cr_linked) = sync_user_accounts(&data, &uid, coc_linked, cr_linked).await;

    if coc_linked.is_empty() && cr_linked.is_empty() {
        return HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "User not found in local DB or any upstream" }));
    }

    let players_data = fetch_aggregated_player_accounts(&data, coc_linked, cr_linked).await;
    HttpResponse::Ok().json(players_data)
}

// ============================================================================
// GUILD/ADMIN HANDLERS
// ============================================================================

// Get Guild Info
pub async fn get_guild_info(
    data: web::Data<AppState>,
    opt_user: OptionalAuthenticatedUser,
) -> impl Responder {
    let user_role = opt_user
        .user
        .as_ref()
        .and_then(|u| u.claims.role.as_deref());

    let result =
        sqlx::query_as::<_, (Vec<u8>, i32)>("SELECT body, status FROM cache WHERE key = $1")
            .bind("coc:upstream:/api/guild")
            .fetch_optional(&data.db_pool)
            .await;

    match result {
        Ok(Some((body, status))) => {
            let json: serde_json::Value =
                serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);

            if !has_required_role(user_role, "ADMIN")
                && let Some(obj) = json.as_object()
            {
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

            let status = actix_web::http::StatusCode::from_u16(status as u16)
                .unwrap_or(actix_web::http::StatusCode::OK);
            HttpResponse::build(status).json(json)
        }
        _ => HttpResponse::ServiceUnavailable().finish(),
    }
}

// Uptime stats helper
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
            (current_latency, successes)
        }
        _ => (-1, 0),
    }
}

// Get Admin Status
pub async fn get_admin_status(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    if !has_required_role(user.claims.role.as_deref(), "ADMIN") {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: Requires ADMIN role".into(),
        });
    }

    let (coc_upstream_latency, coc_upstream_minutes) =
        get_uptime_stats(&data.db_pool, "upstream_coc").await;
    let (cr_upstream_latency, cr_upstream_minutes) =
        get_uptime_stats(&data.db_pool, "upstream_cr").await;
    let (coc_api_latency, coc_api_minutes) = get_uptime_stats(&data.db_pool, "supercell_coc").await;
    let (cr_api_latency, cr_api_minutes) = get_uptime_stats(&data.db_pool, "supercell_cr").await;
    let (website_latency, website_uptime_minutes) =
        get_uptime_stats(&data.db_pool, "website").await;

    HttpResponse::Ok().json(serde_json::json!({
        "upstream_coc": {
            "status": if coc_upstream_latency != -1 { "ONLINE" } else { "OFFLINE" },
            "latency": if coc_upstream_latency != -1 { coc_upstream_latency } else { 0 },
            "uptime_minutes": coc_upstream_minutes
        },
        "upstream_cr": {
            "status": if cr_upstream_latency != -1 { "ONLINE" } else { "OFFLINE" },
            "latency": if cr_upstream_latency != -1 { cr_upstream_latency } else { 0 },
            "uptime_minutes": cr_upstream_minutes
        },
        "supercell_coc": {
            "status": if coc_api_latency != -1 { "ONLINE" } else { "OFFLINE" },
            "latency": if coc_api_latency != -1 { coc_api_latency } else { 0 },
            "uptime_minutes": coc_api_minutes
        },
        "supercell_cr": {
            "status": if cr_api_latency != -1 { "ONLINE" } else { "OFFLINE" },
            "latency": if cr_api_latency != -1 { cr_api_latency } else { 0 },
            "uptime_minutes": cr_api_minutes
        },
        "website": {
            "status": if website_latency != -1 { "ONLINE" } else { "OFFLINE" },
            "latency": if website_latency != -1 { website_latency } else { 0 },
            "uptime_minutes": website_uptime_minutes
        }
    }))
}

// Get Latency History
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
            error!("Database error fetching latency: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_side_clans(data: web::Data<AppState>) -> impl Responder {
    use crate::models::{SideClan, SideClanCWLStats, SideClanCwlHistory};

    let clans_query =
        sqlx::query_as::<_, SideClan>("SELECT clan_tag, name, belongs_to, display_index, badge_url FROM side_clans ORDER BY CASE WHEN display_index = 0 THEN 1 ELSE 0 END, display_index ASC, name ASC")
            .fetch_all(&data.db_pool)
            .await;

    match clans_query {
        Ok(clans) => {
            let mut results = Vec::new();
            for clan in clans {
                let history = sqlx::query_as::<_, SideClanCWLStats>(
                    "SELECT clan_tag, season, league_id, league_name, league_badge_url, rank FROM side_clans_cwl_stats WHERE clan_tag = $1 ORDER BY season DESC"
                )
                .bind(&clan.clan_tag)
                .fetch_all(&data.db_pool)
                .await
                .unwrap_or_default();

                results.push(SideClanCwlHistory { clan, history });
            }
            HttpResponse::Ok().json(results)
        }
        Err(e) => {
            error!("Database error fetching side clans: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
