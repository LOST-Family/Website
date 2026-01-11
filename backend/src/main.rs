use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use reqwest::Client;
use std::env;

mod auth;
mod background;
mod handlers;
mod models;
mod utils;

use auth::*;
use background::spawn_background_task;
use handlers::*;
use models::AppState;

use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let upstream_url = env::var("UPSTREAM_API_URL").expect("UPSTREAM_API_URL must be set");
    let api_token = env::var("BOT_SERVER_API_TOKEN").expect("BOT_SERVER_API_TOKEN must be set");
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost/website".to_string());

    // Auth Env Vars
    let discord_client_id =
        ClientId::new(env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set"));
    let discord_client_secret = ClientSecret::new(
        env::var("DISCORD_CLIENT_SECRET").expect("DISCORD_CLIENT_SECRET must be set"),
    );
    let auth_url = AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
        .expect("Invalid token endpoint URL");
    let redirect_url = RedirectUrl::new(
        env::var("DISCORD_REDIRECT_URI").expect("DISCORD_REDIRECT_URI must be set"),
    )
    .expect("Invalid redirect URL");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    let oauth_client = BasicClient::new(
        discord_client_id,
        Some(discord_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(redirect_url);

    // Initialize Postgres Database
    let pool = sqlx::postgres::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to pool");

    // Create cache table if not exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cache (
            key TEXT PRIMARY KEY,
            body BYTEA NOT NULL,
            status INTEGER NOT NULL,
            updated_at BIGINT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to run migrations (cache)");

    // Create users table if not exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            discord_id TEXT UNIQUE NOT NULL,
            username TEXT NOT NULL,
            global_name TEXT,
            nickname TEXT,
            avatar TEXT,
            highest_role TEXT,
            is_admin BOOLEAN DEFAULT FALSE,
            linked_players TEXT,
            updated_at BIGINT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to run migrations (users)");

    let client = Client::builder()
        .timeout(Duration::from_secs(200))
        .build()
        .expect("Failed to build reqwest client");

    let app_state = AppState {
        client,
        upstream_url,
        api_token,
        db_pool: pool,
        oauth_client,
        jwt_secret,
        frontend_url,
    };

    // Spawn the background refresh task
    spawn_background_task(app_state.clone());

    println!("Starting server on port {}", port);

    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials(); // Important for cookies

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .route("/auth/discord/login", web::get().to(discord_login))
            .route("/auth/discord/callback", web::get().to(discord_callback))
            .route("/auth/me", web::get().to(get_me))
            .route("/auth/logout", web::post().to(logout))
            .route("/api/clans", web::get().to(get_clans))
            .route("/api/clans/{tag}", web::get().to(get_clan_info))
            .route("/api/clans/{tag}/members", web::get().to(get_clan_members))
            .route(
                "/api/clans/{tag}/kickpoint-reasons",
                web::get().to(get_clan_kickpoint_reasons),
            )
            .route(
                "/api/clans/{tag}/war-members",
                web::get().to(get_clan_war_members),
            )
            .route(
                "/api/clans/{tag}/raid-members",
                web::get().to(get_raid_members),
            )
            .route(
                "/api/clans/{tag}/cwl-members",
                web::get().to(get_cwl_members),
            )
            .route("/api/players/{tag}", web::get().to(get_player))
            .route("/api/users/{userId}", web::get().to(get_user))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
