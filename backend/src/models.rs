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
    // BS Upstream API. Optional wie der Ticket-Bot: fehlt die Konfiguration,
    // laufen die uebrigen Spiele weiter und nur der Brawl-Stars-Bereich meldet
    // 503. Ein Bot, den es (noch) nicht gibt, darf die Website nicht am Start
    // hindern.
    pub upstream_bs_url: Option<String>,
    pub bs_api_token: Option<String>,
    // Official Supercell APIs
    pub clash_of_clans_api_token: String,
    pub clash_royale_api_token: String,
    pub brawl_stars_api_token: String,
    // Ticket-Bot. Optional: fehlt die Konfiguration, laeuft die Website
    // weiter und nur das Ticket-Dashboard meldet 503. Ein Ticketsystem, das
    // nicht erreichbar ist, darf nicht die Clanseiten mitnehmen.
    pub upstream_ticket_url: Option<String>,
    pub ticket_api_token: Option<String>,
    pub db_pool: PgPool,
    // Clans, die es nicht mehr gibt. Sie stehen weiter in der Datenbank des
    // Bots — daran haengen Kickpunkte und Mitgliederverlauf —, sollen auf der
    // Website aber nirgends mehr erscheinen. Gepflegt ueber GESCHLOSSENE_CLANS.
    pub geschlossene_clans: Vec<String>,
    pub oauth_client: DiscordOAuthClient,
    pub jwt_secret: String,
    pub frontend_url: String,
    pub background_refresh_interval: u64,
    // Discord-Webhook für das Protokoll der Schreibaktionen. Optional: fehlt
    // er, laufen die Aktionen trotzdem und nur das Protokoll fällt aus. Ein
    // nicht erreichbarer Log-Kanal darf keine Verwaltungsaktion verhindern.
    pub log_webhook: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    ClashOfClans,
    ClashRoyale,
    BrawlStars,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct SideClan {
    pub clan_tag: String,
    pub name: String,
    pub belongs_to: Option<String>,
    #[serde(alias = "index")]
    pub display_index: i32,
    pub badge_url: Option<String>,
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
