use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub type DiscordOAuthClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::basic::BasicTokenResponse,
    oauth2::basic::BasicTokenIntrospectionResponse,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
>;

#[derive(Clone)]
pub struct AppState {
    pub client: oauth2::reqwest::Client,
    // CoC Upstream API (formerly just "upstream")
    pub upstream_coc_url: String,
    pub coc_api_token: String,
    // CR Upstream API (new)
    pub upstream_cr_url: String,
    pub cr_api_token: String,
    // Official Supercell APIs
    pub clash_of_clans_api_token: String,
    pub clash_royale_api_token: String,
    pub db_pool: PgPool,
    pub oauth_client: DiscordOAuthClient,
    pub jwt_secret: String,
    pub frontend_url: String,
    pub background_refresh_interval: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    ClashOfClans,
    ClashRoyale,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct SideClan {
    pub clan_tag: String,
    pub name: String,
    pub belongs_to: Option<String>,
    #[serde(alias = "index")]
    pub display_index: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct SideClanCWLStats {
    pub clan_tag: String,
    pub season: String, // YYYY-MM
    pub league_id: Option<i32>,
    pub league_name: Option<String>,
    pub league_badge_url: Option<String>,
    pub rank: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SideClanCwlHistory {
    pub clan: SideClan,
    pub history: Vec<SideClanCWLStats>,
}
