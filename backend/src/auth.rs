use crate::models::{AppState, ErrorResponse};
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    cookie::{Cookie, time::Duration},
    web,
};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use log::error;
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse, reqwest::async_http_client};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    #[allow(dead_code)]
    state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // discord user id
    pub role: Option<String>,
    pub exp: usize,
}

#[derive(Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
    avatar: Option<String>,
    global_name: Option<String>,
    #[allow(dead_code)]
    discriminator: String,
}

#[derive(Deserialize)]
struct UserMetadata {
    #[serde(default)]
    admin: bool,
    nickname: Option<String>,
    #[serde(rename = "linkedPlayers", default)]
    linked_players: Vec<String>,
    #[serde(rename = "linkedCrPlayers", default)]
    linked_cr_players: Vec<String>,
    #[serde(rename = "highestRole")]
    highest_role: Option<String>,
}

pub fn get_role_priority(role: &str) -> i32 {
    match role.to_uppercase().as_str() {
        "ADMIN" => 1000,
        "LEADER" => 100,
        "COLEADER" => 80,
        "ELDER" => 50,
        "MEMBER" => 10,
        "NOTINCLAN" | "NOTMEMBER" => 0,
        _ => 0,
    }
}

pub fn has_required_role(user_role: Option<&str>, required_role: &str) -> bool {
    let user_priority = user_role.map(get_role_priority).unwrap_or(0);
    let required_priority = get_role_priority(required_role);
    user_priority >= required_priority
}

pub async fn discord_login(data: web::Data<AppState>) -> impl Responder {
    let (auth_url, _csrf_token) = data
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .url();

    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

pub async fn discord_callback(
    data: web::Data<AppState>,
    query: web::Query<AuthRequest>,
) -> impl Responder {
    let code = AuthorizationCode::new(query.code.clone());
    let token = match data
        .oauth_client
        .exchange_code(code)
        .request_async(async_http_client)
        .await
    {
        Ok(token) => token,
        Err(e) => {
            error!("Token exchange error: {:?}", e);
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Failed to exchange token".into(),
            });
        }
    };

    let client = reqwest::Client::new();
    let user_info: DiscordUser = match client
        .get("https://discord.com/api/users/@me")
        .header(
            "Authorization",
            format!("Bearer {}", token.access_token().secret()),
        )
        .send()
        .await
    {
        Ok(res) => match res.json().await {
            Ok(user) => user,
            Err(e) => {
                error!("User info parse error: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        },
        Err(e) => {
            error!("User info fetch error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Fetch extra metadata from internal APIs
    let mut final_is_admin = false;
    let mut final_highest_role = "NOTINCLAN".to_string();
    let mut final_nickname = None;
    let mut coc_linked = Vec::new();
    let mut cr_linked = Vec::new();

    // 1. Fetch from CoC Upstream
    match client
        .get(format!(
            "{}/api/users/{}",
            data.upstream_coc_url, user_info.id
        ))
        .header("Authorization", format!("Bearer {}", data.coc_api_token))
        .send()
        .await
    {
        Ok(res) if res.status().is_success() => {
            if let Ok(m) = res.json::<UserMetadata>().await {
                final_is_admin = m.admin;
                final_highest_role = m.highest_role.unwrap_or_else(|| "NOTINCLAN".to_string());
                final_nickname = m.nickname;
                coc_linked = m.linked_players;
                cr_linked = m.linked_cr_players;
            }
        }
        Ok(res) if res.status() == 404 => {
            // Not in CoC clan, handled by default values
        }
        Ok(res) => {
            error!("CoC metadata fetch failed: {}", res.status());
        }
        Err(e) => {
            error!("CoC metadata request error: {:?}", e);
        }
    }

    // 2. Fetch from CR Upstream
    match client
        .get(format!(
            "{}/api/users/{}",
            data.upstream_cr_url, user_info.id
        ))
        .header("Authorization", format!("Bearer {}", data.cr_api_token))
        .send()
        .await
    {
        Ok(res) if res.status().is_success() => {
            if let Ok(m) = res.json::<UserMetadata>().await {
                if m.admin {
                    final_is_admin = true;
                }

                let cr_role = m.highest_role.unwrap_or_else(|| "NOTINCLAN".to_string());
                if get_role_priority(&cr_role) > get_role_priority(&final_highest_role) {
                    final_highest_role = cr_role;
                }

                if final_nickname.is_none() {
                    final_nickname = m.nickname;
                }

                // In the CR bot, "linkedPlayers" and "linkedCrPlayers" are both CR accounts
                for tag in m.linked_players {
                    if !cr_linked.contains(&tag) {
                        cr_linked.push(tag);
                    }
                }
                for tag in m.linked_cr_players {
                    if !cr_linked.contains(&tag) {
                        cr_linked.push(tag);
                    }
                }
            }
        }
        Ok(res) if res.status() == 404 => {
            // Not in CR clan
        }
        Ok(res) => {
            error!("CR metadata fetch failed: {}", res.status());
        }
        Err(e) => {
            error!("CR metadata request error: {:?}", e);
        }
    }

    // Elevation: if user is admin, guarantee they have ADMIN role in token
    if final_is_admin {
        final_highest_role = "ADMIN".to_string();
    }

    let linked_players_json =
        serde_json::to_string(&coc_linked).unwrap_or_else(|_| "[]".to_string());
    let linked_cr_players_json =
        serde_json::to_string(&cr_linked).unwrap_or_else(|_| "[]".to_string());

    // Construct avatar URL
    let avatar_url = match &user_info.avatar {
        Some(hash) => format!(
            "https://cdn.discordapp.com/avatars/{}/{}.png",
            user_info.id, hash
        ),
        None => {
            let index = (user_info.id.parse::<u64>().unwrap_or(0) >> 22) % 6;
            format!("https://cdn.discordapp.com/embed/avatars/{}.png", index)
        }
    };

    // Save user to DB
    let db_res = sqlx::query(
        "INSERT INTO users (discord_id, username, global_name, nickname, avatar, highest_role, is_admin, linked_players, linked_cr_players, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT(discord_id) DO UPDATE SET
            username = EXCLUDED.username,
            global_name = EXCLUDED.global_name,
            nickname = EXCLUDED.nickname,
            avatar = EXCLUDED.avatar,
            highest_role = EXCLUDED.highest_role,
            is_admin = EXCLUDED.is_admin,
            linked_players = EXCLUDED.linked_players,
            linked_cr_players = EXCLUDED.linked_cr_players,
            updated_at = EXCLUDED.updated_at",
    )
    .bind(&user_info.id)
    .bind(&user_info.username)
    .bind(&user_info.global_name)
    .bind(&final_nickname)
    .bind(&avatar_url)
    .bind(&final_highest_role)
    .bind(final_is_admin)
    .bind(&linked_players_json)
    .bind(&linked_cr_players_json)
    .bind(Utc::now().timestamp())
    .execute(&data.db_pool)
    .await;

    if let Err(e) = db_res {
        error!("Database error saving user: {:?}", e);
    }

    // Create JWT
    let expiration = Utc::now()
        .checked_add_signed(ChronoDuration::days(7))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_info.id.clone(),
        role: Some(final_highest_role.clone()),
        exp: expiration as usize,
    };

    let token_str = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.jwt_secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let cookie = Cookie::build("auth_token", token_str)
        .path("/")
        //.secure(true) // Uncomment in production with HTTPS
        .http_only(true)
        .max_age(Duration::days(7))
        .finish();

    HttpResponse::Found()
        .append_header(("Location", data.frontend_url.clone()))
        .cookie(cookie)
        .finish()
}

pub async fn get_me(data: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    let user_db = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, Option<String>, bool, String, String)>(
        "SELECT discord_id, username, global_name, nickname, avatar, highest_role, is_admin, COALESCE(linked_players, '[]'), COALESCE(linked_cr_players, '[]') FROM users WHERE discord_id = $1",
    )
    .bind(&user.claims.sub)
    .fetch_one(&data.db_pool)
    .await;

    match user_db {
        Ok(u) => {
            let linked_players: Vec<String> = serde_json::from_str(&u.7).unwrap_or_default();
            let linked_cr_players: Vec<String> = serde_json::from_str(&u.8).unwrap_or_default();
            HttpResponse::Ok().json(serde_json::json!({
                "discord_id": u.0,
                "username": u.1,
                "global_name": u.2,
                "nickname": u.3,
                "avatar": u.4,
                "highest_role": u.5,
                "is_admin": u.6,
                "linked_players": linked_players,
                "linked_cr_players": linked_cr_players,
            }))
        }
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn logout() -> impl Responder {
    let cookie = Cookie::build("auth_token", "")
        .path("/")
        .max_age(Duration::seconds(0))
        .finish();

    HttpResponse::Ok().cookie(cookie).finish()
}

use actix_web::FromRequest;
use futures_util::future::LocalBoxFuture;
use std::future::ready;

pub struct AuthenticatedUser {
    pub claims: Claims,
    pub linked_players: Vec<String>,
    pub linked_cr_players: Vec<String>,
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let data = req
            .app_data::<web::Data<AppState>>()
            .expect("AppState not found")
            .clone();

        let token = match req.cookie("auth_token") {
            Some(c) => c.value().to_string(),
            None => {
                return Box::pin(ready(Err(actix_web::error::ErrorUnauthorized(
                    "No auth token",
                ))));
            }
        };

        Box::pin(async move {
            let decoding_key = jsonwebtoken::DecodingKey::from_secret(data.jwt_secret.as_bytes());
            let validation = jsonwebtoken::Validation::default();

            match jsonwebtoken::decode::<Claims>(&token, &decoding_key, &validation) {
                Ok(c) => {
                    let user_id = c.claims.sub.clone();
                    // Fetch linked players and state from DB to ensure real-time permissions
                    let user_db = sqlx::query_as::<_, (String, String, Option<String>, bool)>(
                        "SELECT COALESCE(linked_players, '[]'), COALESCE(linked_cr_players, '[]'), highest_role, is_admin FROM users WHERE discord_id = $1",
                    )
                    .bind(&user_id)
                    .fetch_one(&data.db_pool)
                    .await;

                    let (linked_players, linked_cr_players, db_role, is_admin) = match user_db {
                        Ok((lp_json, cr_json, role, admin)) => (
                            serde_json::from_str(&lp_json).unwrap_or_default(),
                            serde_json::from_str(&cr_json).unwrap_or_default(),
                            role,
                            admin,
                        ),
                        Err(_) => (vec![], vec![], None, false),
                    };

                    let mut claims = c.claims;
                    if is_admin {
                        claims.role = Some("ADMIN".to_string());
                    } else if db_role.is_some() {
                        claims.role = db_role;
                    }

                    Ok(AuthenticatedUser {
                        claims,
                        linked_players,
                        linked_cr_players,
                    })
                }
                Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
            }
        })
    }
}

pub struct OptionalAuthenticatedUser {
    pub user: Option<AuthenticatedUser>,
}

impl FromRequest for OptionalAuthenticatedUser {
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let data = req
            .app_data::<web::Data<AppState>>()
            .expect("AppState not found")
            .clone();

        let token = match req.cookie("auth_token") {
            Some(c) => c.value().to_string(),
            None => return Box::pin(ready(Ok(OptionalAuthenticatedUser { user: None }))),
        };

        Box::pin(async move {
            let decoding_key = jsonwebtoken::DecodingKey::from_secret(data.jwt_secret.as_bytes());
            let validation = jsonwebtoken::Validation::default();

            match jsonwebtoken::decode::<Claims>(&token, &decoding_key, &validation) {
                Ok(c) => {
                    let user_id = c.claims.sub.clone();
                    // Fetch linked players and state from DB to ensure real-time permissions
                    let user_db = sqlx::query_as::<_, (String, String, Option<String>, bool)>(
                        "SELECT COALESCE(linked_players, '[]'), COALESCE(linked_cr_players, '[]'), highest_role, is_admin FROM users WHERE discord_id = $1",
                    )
                    .bind(&user_id)
                    .fetch_one(&data.db_pool)
                    .await;

                    let (linked_players, linked_cr_players, db_role, is_admin) = match user_db {
                        Ok((lp_json, cr_json, role, admin)) => (
                            serde_json::from_str(&lp_json).unwrap_or_default(),
                            serde_json::from_str(&cr_json).unwrap_or_default(),
                            role,
                            admin,
                        ),
                        Err(_) => (vec![], vec![], None, false),
                    };

                    let mut claims = c.claims;
                    if is_admin {
                        claims.role = Some("ADMIN".to_string());
                    } else if db_role.is_some() {
                        claims.role = db_role;
                    }

                    Ok(OptionalAuthenticatedUser {
                        user: Some(AuthenticatedUser {
                            claims,
                            linked_players,
                            linked_cr_players,
                        }),
                    })
                }
                Err(_) => Ok(OptionalAuthenticatedUser { user: None }),
            }
        })
    }
}
