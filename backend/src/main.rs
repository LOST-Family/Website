use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
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

    // CoC Upstream API (renamed from UPSTREAM_API_URL)
    let upstream_coc_url =
        env::var("UPSTREAM_COC_API_URL").expect("UPSTREAM_COC_API_URL must be set");
    let coc_api_token = env::var("COC_BOT_API_TOKEN").expect("COC_BOT_API_TOKEN must be set");

    // CR Upstream API (new)
    let upstream_cr_url = env::var("UPSTREAM_CR_API_URL").expect("UPSTREAM_CR_API_URL must be set");
    let cr_api_token = env::var("CR_BOT_API_TOKEN").expect("CR_BOT_API_TOKEN must be set");

    // Official Supercell API tokens
    let clash_of_clans_api_token =
        env::var("CLASH_OF_CLANS_API_TOKEN").expect("CLASH_OF_CLANS_API_TOKEN must be set");
    let clash_royale_api_token =
        env::var("CLASH_ROYALE_API_TOKEN").expect("CLASH_ROYALE_API_TOKEN must be set");

    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap();
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/website".to_string());

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
    let background_refresh_interval = env::var("BACKGROUND_REFRESH_INTERVAL_MINS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u64>()
        .unwrap_or(10);

    let oauth_client = BasicClient::new(discord_client_id)
        .set_client_secret(discord_client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
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
            linked_cr_players TEXT DEFAULT '[]',
            updated_at BIGINT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to run migrations (users)");

    // Add linked_cr_players column if it doesn't exist (for existing databases)
    let _ = sqlx::query(
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS linked_cr_players TEXT DEFAULT '[]'",
    )
    .execute(&pool)
    .await;

    // Create latency_measurements table if not exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS latency_measurements (
            id SERIAL PRIMARY KEY,
            api_name TEXT NOT NULL,
            latency_ms INTEGER NOT NULL,
            timestamp BIGINT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to run migrations (latency)");

    // Reset only Website uptime on startup (User request)
    let _ = sqlx::query("DELETE FROM latency_measurements WHERE api_name = 'website'")
        .execute(&pool)
        .await;

    // Create side_clans table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS side_clans (
            clan_tag TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            belongs_to TEXT,
            display_index INTEGER DEFAULT 0
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to run migrations (side_clans)");

    // Add display_index column if it doesn't exist (for existing databases)
    let _ = sqlx::query(
        "ALTER TABLE side_clans ADD COLUMN IF NOT EXISTS display_index INTEGER DEFAULT 0",
    )
    .execute(&pool)
    .await;

    // Create side_clans_cwl_stats table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS side_clans_cwl_stats (
            clan_tag TEXT NOT NULL,
            season TEXT NOT NULL,
            league_id INTEGER,
            league_name TEXT,
            league_badge_url TEXT,
            rank INTEGER,
            PRIMARY KEY (clan_tag, season),
            FOREIGN KEY (clan_tag) REFERENCES side_clans(clan_tag) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to run migrations (side_clans_cwl_stats)");

    // Migration for existing table
    let _ = sqlx::query(
        "ALTER TABLE side_clans_cwl_stats ADD COLUMN IF NOT EXISTS league_badge_url TEXT",
    )
    .execute(&pool)
    .await;

    // Seed side clans (Including Main Clans, Excluding Independent Clans)
    let side_clans_json = r###"[
        {"clan_tag": "#2YUPV0UYC", "name": "LOST", "belongs_to": "#2YUPV0UYC", "display_index": 1},
        {"clan_tag": "#2LU2V2LPU", "name": "LOST 2", "belongs_to": "#2LU2V2LPU", "display_index": 2},
        {"clan_tag": "#2QC0QQPQ2", "name": "LOST 3", "belongs_to": "#2QC0QQPQ2", "display_index": 3},
        {"clan_tag": "#2YVPC20UY", "name": "LOST 6 EX / Vegan", "belongs_to": "#2YVPC20UY", "display_index": 4},
        {"clan_tag": "#2J8UG90R2", "name": "LOST 7", "belongs_to": "#2J8UG90R2", "display_index": 5},
        {"clan_tag": "#2RUJPG9JC", "name": "LOST 8", "belongs_to": "#2RUJPG9JC", "display_index": 6},
        {"clan_tag": "#2820UPPQC", "name": "LOST F2P", "belongs_to": "#2820UPPQC", "display_index": 7},
        {"clan_tag": "#2QJQQJ90R", "name": "BitteAufgeben", "belongs_to": "#2YUPV0UYC", "display_index": 8},
        {"clan_tag": "#2QYPGCPYV", "name": "LOST 3 CWL", "belongs_to": "#2QC0QQPQ2", "display_index": 9},
        {"clan_tag": "#2GPP0GLYQ", "name": "LOST 3 CWL 2", "belongs_to": "#2LU2V2LPU", "display_index": 10},
        {"clan_tag": "#2GP2L9L8Y", "name": "LOST 3 Event", "belongs_to": "#2QC0QQPQ2", "display_index": 11},
        {"clan_tag": "#2LJ0GV8G9", "name": "LOST 4 CWL", "belongs_to": "#2LU2V2LPU", "display_index": 12},
        {"clan_tag": "#2GPPQQCUR", "name": "LOST 4 Event", "belongs_to": "#2LU2V2LPU", "display_index": 13},
        {"clan_tag": "#2GV9QPR0U", "name": "LOST 5 CW", "belongs_to": "#2QC0QQPQ2", "display_index": 14},
        {"clan_tag": "#2RQ9QQCG2", "name": "LOST 6 EX CWL", "belongs_to": "#2YVPC20UY", "display_index": 15},
        {"clan_tag": "#2R8LG0GQY", "name": "LOST 7 CWL", "belongs_to": "#2J8UG90R2", "display_index": 16},
        {"clan_tag": "#2J8LYGUP0", "name": "LOST 8 CWL", "belongs_to": "#2RUJPG9JC", "display_index": 17},
        {"clan_tag": "#2J2J9LL8J", "name": "LOST 8 CWL 2", "belongs_to": "#2RUJPG9JC", "display_index": 18},
        {"clan_tag": "#2R2LC2UG0", "name": "LOST F2P 2 CWL", "belongs_to": "#2820UPPQC", "display_index": 19},
        {"clan_tag": "#2QPPYRRUQ", "name": "LOST F2P CWL", "belongs_to": "#2820UPPQC", "display_index": 20},
        {"clan_tag": "#2GU08UJC8", "name": "LOST F2P CWL 2", "belongs_to": "#2820UPPQC", "display_index": 21},
        {"clan_tag": "#2LGV0QQJ9", "name": "LOST Push", "belongs_to": "#2YUPV0UYC", "display_index": 22},
        {"clan_tag": "#2JP0UCC0G", "name": "Lost 5 CWL", "belongs_to": "#2QC0QQPQ2", "display_index": 23},
        {"clan_tag": "#2R8082GYC", "name": "Lost 5 CWL 2", "belongs_to": "#2QC0QQPQ2", "display_index": 24},
        {"clan_tag": "#2R8YRPJCV", "name": "Lost 7 CWL 2", "belongs_to": "#2J8UG90R2", "display_index": 25},
        {"clan_tag": "#2GGJVG9GU", "name": "Vegan 6", "belongs_to": "#2YVPC20UY", "display_index": 26}
    ]"###;

    // Clear existing side_clans to remove independent ones
    // We do this by tag to avoid FK issues if possible, or just ignore errors
    let tags_to_keep: Vec<String> = serde_json::from_str::<Vec<serde_json::Value>>(side_clans_json)
        .unwrap()
        .into_iter()
        .map(|v| v["clan_tag"].as_str().unwrap().to_string())
        .collect();

    let _ = sqlx::query("DELETE FROM side_clans WHERE clan_tag != ALL($1)")
        .bind(&tags_to_keep)
        .execute(&pool)
        .await;

    let side_clans: Vec<models::SideClan> = serde_json::from_str(side_clans_json).unwrap();
    for clan in side_clans {
        sqlx::query(
            "INSERT INTO side_clans (clan_tag, name, belongs_to, display_index) VALUES ($1, $2, $3, $4) ON CONFLICT (clan_tag) DO UPDATE SET name = $2, belongs_to = $3, display_index = $4",
        )
        .bind(clan.clan_tag)
        .bind(clan.name)
        .bind(clan.belongs_to)
        .bind(clan.display_index)
        .execute(&pool)
        .await
        .expect("Failed to seed side_clans");
    }

    let client = oauth2::reqwest::Client::builder()
        .timeout(Duration::from_secs(200))
        .build()
        .expect("Failed to build reqwest client");

    let app_state = AppState {
        client,
        upstream_coc_url,
        coc_api_token,
        upstream_cr_url,
        cr_api_token,
        clash_of_clans_api_token,
        clash_royale_api_token,
        db_pool: pool,
        oauth_client,
        jwt_secret,
        frontend_url,
        background_refresh_interval,
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
            .wrap(Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            .route("/auth/discord/login", web::get().to(discord_login))
            .route("/auth/discord/callback", web::get().to(discord_callback))
            .route("/auth/me", web::get().to(get_me))
            .route("/auth/logout", web::post().to(logout))
            .route("/api/me/accounts", web::get().to(get_my_player_accounts))
            .route("/api/users/{id}", web::get().to(get_user))
            .route(
                "/api/users/{id}/accounts",
                web::get().to(get_user_player_accounts),
            )
            // CoC Routes
            .route("/api/coc/clans", web::get().to(get_coc_clans))
            .route("/api/coc/clans/{tag}", web::get().to(get_coc_clan_info))
            .route(
                "/api/coc/clans/{tag}/config",
                web::get().to(get_coc_clan_config),
            )
            .route(
                "/api/coc/clans/{tag}/members",
                web::get().to(get_coc_clan_members),
            )
            .route(
                "/api/coc/clans/{tag}/members-lite",
                web::get().to(get_coc_clan_members_lite),
            )
            .route(
                "/api/coc/clans/{tag}/kickpoint-reasons",
                web::get().to(get_coc_clan_kickpoint_reasons),
            )
            .route(
                "/api/coc/clans/{tag}/war-members",
                web::get().to(get_coc_clan_war_members),
            )
            .route(
                "/api/coc/clans/{tag}/raid-members",
                web::get().to(get_coc_raid_members),
            )
            .route(
                "/api/coc/clans/{tag}/cwl-members",
                web::get().to(get_coc_cwl_members),
            )
            .route("/api/coc/players/{tag}", web::get().to(get_coc_player))
            .route(
                "/api/coc/players/{tag}/identity",
                web::get().to(get_coc_player_identity),
            )
            .route(
                "/api/coc/players/{tag}/kickpoints",
                web::get().to(get_coc_player_kickpoints),
            )
            .route(
                "/api/coc/players/{tag}/kickpoints/details",
                web::get().to(get_coc_player_kickpoints_details),
            )
            // CR Routes
            .route("/api/cr/clans", web::get().to(get_cr_clans))
            .route("/api/cr/clans/{tag}", web::get().to(get_cr_clan_info))
            .route(
                "/api/cr/clans/{tag}/config",
                web::get().to(get_cr_clan_config),
            )
            .route(
                "/api/cr/clans/{tag}/members",
                web::get().to(get_cr_clan_members),
            )
            .route(
                "/api/cr/clans/{tag}/members-lite",
                web::get().to(get_cr_clan_members_lite),
            )
            .route(
                "/api/cr/clans/{tag}/kickpoint-reasons",
                web::get().to(get_cr_clan_kickpoint_reasons),
            )
            .route("/api/cr/players/{tag}", web::get().to(get_cr_player))
            .route(
                "/api/cr/players/{tag}/identity",
                web::get().to(get_cr_player_identity),
            )
            .route(
                "/api/cr/players/{tag}/kickpoints",
                web::get().to(get_cr_player_kickpoints),
            )
            .route(
                "/api/cr/players/{tag}/kickpoints/details",
                web::get().to(get_cr_player_kickpoints_details),
            )
            // Common/Legacy Routes
            .route("/api/guild", web::get().to(get_guild_info))
            .route("/api/admin/status", web::get().to(get_admin_status))
            .route("/api/admin/latency", web::get().to(get_latency_history))
            .route("/api/sideclans", web::get().to(get_side_clans))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
