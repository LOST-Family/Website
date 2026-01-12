use crate::auth::{AuthenticatedUser, has_required_role};
use crate::models::{AppState, ErrorResponse};
use crate::utils::{encode_tag, forward_request, update_cache, update_clash_cache};
use actix_web::{HttpResponse, Responder, web};

// 1. Get All Clans
pub async fn get_clans(data: web::Data<AppState>) -> impl Responder {
    // Pass relative path logic is handled in utils now (mostly),
    // but utils::update_cache takes the path.
    // We update forward_request to expect just the path, not the full URL.
    forward_request(&data, "/api/clans").await
}

// 2. Get Clan Info
pub async fn get_clan_info(data: web::Data<AppState>, tag: web::Path<String>) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    forward_request(&data, &format!("/api/clans/{}", encoded_tag)).await
}

// 3. Get Clan Members
pub async fn get_clan_members(data: web::Data<AppState>, tag: web::Path<String>) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    forward_request(&data, &format!("/api/clans/{}/members", encoded_tag)).await
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
) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    forward_request(&data, &format!("/api/clans/{}/war-members", encoded_tag)).await
}

// 6. Get Raid Members
pub async fn get_raid_members(data: web::Data<AppState>, tag: web::Path<String>) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    forward_request(&data, &format!("/api/clans/{}/raid-members", encoded_tag)).await
}

// 7. Get CWL Members
pub async fn get_cwl_members(data: web::Data<AppState>, tag: web::Path<String>) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    forward_request(&data, &format!("/api/clans/{}/cwl-members", encoded_tag)).await
}

// 8. Get Player
pub async fn get_player(data: web::Data<AppState>, tag: web::Path<String>) -> impl Responder {
    let encoded_tag = encode_tag(&tag);
    let url_path = format!("/players/{}", encoded_tag);
    let cache_key = format!("clash:{}", url_path);

    // Try to update cache first (or we could just rely on forward_request if we have a background task)
    // But for players, we probably want to fetch on demand if not in cache.
    let _ = update_clash_cache(&data, &url_path).await;

    forward_request(&data, &cache_key).await
}

// 9. Get User
pub async fn get_user(data: web::Data<AppState>, user_id: web::Path<String>) -> impl Responder {
    forward_request(&data, &format!("/api/users/{}", user_id)).await
}

// 10. Get Guild Info
pub async fn get_guild_info(data: web::Data<AppState>) -> impl Responder {
    forward_request(&data, "/api/guild").await
}

// 11. Get My Player Accounts (Protected: User only)
pub async fn get_my_player_accounts(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    // 1. Get user's linked players from DB
    let user_db =
        sqlx::query_as::<_, (String,)>("SELECT linked_players FROM users WHERE discord_id = $1")
            .bind(&user.claims.sub)
            .fetch_one(&data.db_pool)
            .await;

    let linked_players: Vec<String> = match user_db {
        Ok((lp_json,)) => serde_json::from_str(&lp_json).unwrap_or_default(),
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    let mut players_data = Vec::new();
    for tag in linked_players {
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
                                        // If it's something like 'tag' or 'name', they should match anyway,
                                        // but if we want specific upstream fields that might overlap,
                                        // we could handle them differently.
                                        // For now, let's just insert non-colliding ones or keep CoC as primary.
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
