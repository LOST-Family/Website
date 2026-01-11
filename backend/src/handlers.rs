use crate::auth::{AuthenticatedUser, has_required_role};
use crate::models::{AppState, ErrorResponse};
use crate::utils::{encode_tag, forward_request, update_cache};
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
    forward_request(&data, &format!("/api/players/{}", encoded_tag)).await
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
        let url_path = format!("/api/players/{}", encoded_tag);

        match update_cache(&data, &url_path).await {
            Ok(body) => {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
                    players_data.push(json);
                }
            }
            Err(e) => {
                eprintln!("Error fetching player data for {}: {}", tag, e);
            }
        }
    }

    HttpResponse::Ok().json(players_data)
}
