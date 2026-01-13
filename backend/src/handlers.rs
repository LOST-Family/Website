use crate::auth::{AuthenticatedUser, OptionalAuthenticatedUser, has_required_role};
use crate::models::{AppState, ErrorResponse, GameType};
use crate::utils::{
    encode_tag, filter_member_data, forward_request, forward_request_with_filter,
    update_supercell_cache, update_upstream_cache,
};
use actix_web::{HttpResponse, Responder, web};
use bytes::Bytes;
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
    let prefix = get_cache_prefix(game);
    let supercell_url_path = format!("/clans/{}", encoded_tag);

    // 1. Get from Supercell cache
    let sc_cache_key = format!("{}:supercell:{}", prefix, supercell_url_path);
    let sc_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&sc_cache_key)
        .fetch_optional(&data.db_pool)
        .await;

    let mut clan_json = match sc_res {
        Ok(Some((body,))) => {
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
        }
        _ => {
            return HttpResponse::NotFound().json(serde_json::json!({ "error": "Clan not found" }));
        }
    };

    // 2. Get from Upstream cache and merge
    let upstream_url_path = format!("/api/clans/{}", encoded_tag);
    let up_cache_key = format!("{}:upstream:{}", prefix, upstream_url_path);
    let up_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&up_cache_key)
        .fetch_optional(&data.db_pool)
        .await;

    if let Ok(Some((up_body,))) = up_res {
        if let Ok(up_json) = serde_json::from_slice::<serde_json::Value>(&up_body) {
            if let (Some(sc_obj), Some(up_obj)) = (clan_json.as_object_mut(), up_json.as_object()) {
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
        }
    }

    // Fix badgeUrl mismatch (singular vs plural)
    if let Some(obj) = clan_json.as_object_mut() {
        if !obj.contains_key("badgeUrls") {
            if let Some(url) = obj.get("badgeUrl").and_then(|u| u.as_str()) {
                obj.insert(
                    "badgeUrls".to_string(),
                    serde_json::json!({
                        "small": url,
                        "medium": url,
                        "large": url
                    }),
                );
            }
        }
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

    // Get bodies from cache
    let supercell_cache_key = format!("{}:supercell:{}", prefix, supercell_url_path);
    let upstream_cache_key = format!("{}:upstream:{}", prefix, upstream_url_path);

    let supercell_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&supercell_cache_key)
        .fetch_optional(&data.db_pool)
        .await;

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&upstream_cache_key)
        .fetch_optional(&data.db_pool)
        .await;

    let mut supercell_members = match supercell_res {
        Ok(Some((body,))) => {
            let json: serde_json::Value =
                serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
            json["memberList"].as_array().cloned().unwrap_or_default()
        }
        _ => vec![],
    };

    let upstream_body = match upstream_res {
        Ok(Some((body,))) => Bytes::from(body),
        _ => Bytes::new(),
    };

    // Filter upstream members first (privacy logic)
    let filtered_upstream_body = filter_member_data(upstream_body, exempt_tags, user_role);
    let upstream_members: Vec<serde_json::Value> =
        serde_json::from_slice(&filtered_upstream_body).unwrap_or_default();

    // Merge upstream data
    for s_member in &mut supercell_members {
        if let Some(tag_ref) = s_member.get("tag").and_then(|t| t.as_str()) {
            let member_tag = tag_ref.to_string();

            // Merge Upstream (Identity/Kickpoints)
            if let Some(u_member) = upstream_members
                .iter()
                .find(|m| m.get("tag").and_then(|t| t.as_str()) == Some(&member_tag))
            {
                if let (Some(s_obj), Some(u_obj)) = (s_member.as_object_mut(), u_member.as_object())
                {
                    for (k, v) in u_obj {
                        if !s_obj.contains_key(k) {
                            s_obj.insert(k.clone(), v.clone());
                        } else if k != "tag" {
                            s_obj.insert(format!("upstream_{}", k), v.clone());
                        }
                    }
                }
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

                if let Ok(Some((p_body,))) = player_res {
                    if let Ok(p_json) = serde_json::from_slice::<serde_json::Value>(&p_body) {
                        if let (Some(s_obj), Some(p_obj)) =
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
        }
    }

    HttpResponse::Ok().json(supercell_members)
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
    let prefix = get_cache_prefix(game);
    let supercell_url_path = format!("/players/{}", encoded_tag);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);

    let supercell_cache_key = format!("{}:supercell:{}", prefix, supercell_url_path);
    let upstream_cache_key = format!("{}:upstream:{}", prefix, upstream_url_path);

    // Get from cache
    let supercell_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&supercell_cache_key)
        .fetch_optional(&data.db_pool)
        .await;

    let mut player_json = match supercell_res {
        Ok(Some((body,))) => {
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

    if has_required_role(user_role, "MEMBER") || is_exempt {
        let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
            .bind(&upstream_cache_key)
            .fetch_optional(&data.db_pool)
            .await;

        if let Ok(Some((u_body,))) = upstream_res {
            if let Ok(u_json) = serde_json::from_slice::<serde_json::Value>(&u_body) {
                if let Some(obj) = player_json.as_object_mut() {
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
            }
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
    let prefix = get_cache_prefix(game);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);
    let cache_key = format!("{}:upstream:{}", prefix, upstream_url_path);

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

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&cache_key)
        .fetch_optional(&data.db_pool)
        .await;

    match upstream_res {
        Ok(Some((body,))) => {
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
    let prefix = get_cache_prefix(game);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);
    let cache_key = format!("{}:upstream:{}", prefix, upstream_url_path);

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&cache_key)
        .fetch_optional(&data.db_pool)
        .await;

    let u_json = match upstream_res {
        Ok(Some((body,))) => {
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
    let prefix = get_cache_prefix(game);
    let upstream_url_path = format!("/api/players/{}", encoded_tag);
    let cache_key = format!("{}:upstream:{}", prefix, upstream_url_path);

    let upstream_res = sqlx::query_as::<_, (Vec<u8>,)>("SELECT body FROM cache WHERE key = $1")
        .bind(&cache_key)
        .fetch_optional(&data.db_pool)
        .await;

    let mut u_json = match upstream_res {
        Ok(Some((body,))) => {
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
    // Ensure we have fresh user data for profiles
    let _ = update_upstream_cache(&data, GameType::ClashOfClans, &url_path).await;
    forward_request(&data, GameType::ClashOfClans, &url_path).await
}

// Helper for Player Aggregation - now fetches both CoC and CR
async fn fetch_aggregated_player_accounts(
    data: &web::Data<AppState>,
    coc_linked_players: Vec<String>,
    cr_linked_players: Vec<String>,
) -> serde_json::Value {
    let mut coc_players_data = Vec::new();
    let mut cr_players_data = Vec::new();

    // Fetch CoC players
    for tag in coc_linked_players {
        let encoded_tag = encode_tag(&tag);
        let supercell_url_path = format!("/players/{}", encoded_tag);
        let upstream_url_path = format!("/api/players/{}", encoded_tag);

        let supercell_res =
            update_supercell_cache(data, GameType::ClashOfClans, &supercell_url_path).await;
        let upstream_res =
            update_upstream_cache(data, GameType::ClashOfClans, &upstream_url_path).await;

        if let Ok(supercell_body) = supercell_res {
            if let Ok(mut player_json) =
                serde_json::from_slice::<serde_json::Value>(&supercell_body)
            {
                if let Some(player_obj) = player_json.as_object_mut() {
                    player_obj.insert("gameType".to_string(), serde_json::json!("coc"));

                    if let Ok(u_body) = upstream_res {
                        if let Ok(u_json) = serde_json::from_slice::<serde_json::Value>(&u_body) {
                            if let Some(u_obj) = u_json.as_object() {
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
                                    player_obj.insert(
                                        "activeKickpointsSum".to_string(),
                                        serde_json::json!(sum),
                                    );
                                }
                            }
                        }
                    }
                }
                coc_players_data.push(player_json);
            }
        }
    }

    // Fetch CR players
    for tag in cr_linked_players {
        let encoded_tag = encode_tag(&tag);
        let supercell_url_path = format!("/players/{}", encoded_tag);
        let upstream_url_path = format!("/api/players/{}", encoded_tag);

        let supercell_res =
            update_supercell_cache(data, GameType::ClashRoyale, &supercell_url_path).await;
        let upstream_res =
            update_upstream_cache(data, GameType::ClashRoyale, &upstream_url_path).await;

        if let Ok(supercell_body) = supercell_res {
            if let Ok(mut player_json) =
                serde_json::from_slice::<serde_json::Value>(&supercell_body)
            {
                if let Some(player_obj) = player_json.as_object_mut() {
                    player_obj.insert("gameType".to_string(), serde_json::json!("cr"));

                    if let Ok(u_body) = upstream_res {
                        if let Ok(u_json) = serde_json::from_slice::<serde_json::Value>(&u_body) {
                            if let Some(u_obj) = u_json.as_object() {
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
                                    player_obj.insert(
                                        "activeKickpointsSum".to_string(),
                                        serde_json::json!(sum),
                                    );
                                }
                            }
                        }
                    }
                }
                cr_players_data.push(player_json);
            }
        }
    }

    serde_json::json!({
        "coc": coc_players_data,
        "cr": cr_players_data
    })
}

// Get My Player Accounts
pub async fn get_my_player_accounts(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let mut coc_players = user.linked_players.clone();
    let mut cr_players = user.linked_cr_players.clone();

    // If lists are empty, try a live refresh from upstreams
    if coc_players.is_empty() || cr_players.is_empty() {
        let url_path = format!("/api/users/{}", user.claims.sub);

        // Try CoC Upstream
        if let Ok(body) = update_upstream_cache(&data, GameType::ClashOfClans, &url_path).await {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
                if let Some(arr) = json.get("linkedPlayers").and_then(|v| v.as_array()) {
                    let tags: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    for tag in tags {
                        if !coc_players.contains(&tag) {
                            coc_players.push(tag);
                        }
                    }
                }
                if let Some(arr) = json.get("linkedCrPlayers").and_then(|v| v.as_array()) {
                    let tags: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    for tag in tags {
                        if !cr_players.contains(&tag) {
                            cr_players.push(tag);
                        }
                    }
                }
            }
        }

        // Try CR Upstream
        if let Ok(body) = update_upstream_cache(&data, GameType::ClashRoyale, &url_path).await {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
                if let Some(arr) = json.get("linkedPlayers").and_then(|v| v.as_array()) {
                    let tags: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    for tag in tags {
                        if !cr_players.contains(&tag) {
                            cr_players.push(tag);
                        }
                    }
                }
                if let Some(arr) = json.get("linkedCrPlayers").and_then(|v| v.as_array()) {
                    let tags: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    for tag in tags {
                        if !cr_players.contains(&tag) {
                            cr_players.push(tag);
                        }
                    }
                }
            }
        }
    }

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
        "SELECT linked_players, linked_cr_players FROM users WHERE discord_id = $1",
    )
    .bind(&uid)
    .fetch_optional(&data.db_pool)
    .await;

    let (mut coc_linked, mut cr_linked): (Vec<String>, Vec<String>) = match user_db {
        Ok(Some((lp_json, cr_json))) => (
            serde_json::from_str(&lp_json).unwrap_or_default(),
            serde_json::from_str(&cr_json).unwrap_or_default(),
        ),
        _ => (Vec::new(), Vec::new()),
    };

    // Always try refreshing from upstreams to ensure data is current
    let url_path = format!("/api/users/{}", uid);

    // 1. Try CoC Upstream
    if let Ok(body) = update_upstream_cache(&data, GameType::ClashOfClans, &url_path).await {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
            if let Some(arr) = json.get("linkedPlayers").and_then(|v| v.as_array()) {
                for tag in arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())) {
                    if !coc_linked.contains(&tag) {
                        coc_linked.push(tag);
                    }
                }
            }
            if let Some(arr) = json.get("linkedCrPlayers").and_then(|v| v.as_array()) {
                for tag in arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())) {
                    if !cr_linked.contains(&tag) {
                        cr_linked.push(tag);
                    }
                }
            }
        }
    }

    // 2. Try CR Upstream
    if let Ok(body) = update_upstream_cache(&data, GameType::ClashRoyale, &url_path).await {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
            // Combine both linkedPlayers and linkedCrPlayers from CR bot as CR accounts
            if let Some(arr) = json.get("linkedPlayers").and_then(|v| v.as_array()) {
                for tag in arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())) {
                    if !cr_linked.contains(&tag) {
                        cr_linked.push(tag);
                    }
                }
            }
            if let Some(arr) = json.get("linkedCrPlayers").and_then(|v| v.as_array()) {
                for tag in arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())) {
                    if !cr_linked.contains(&tag) {
                        cr_linked.push(tag);
                    }
                }
            }
        }
    }

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

            if !has_required_role(user_role, "ADMIN") {
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
