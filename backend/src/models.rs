use oauth2::basic::BasicClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub upstream_url: String,
    pub api_token: String,
    pub clash_api_token: String,
    pub db_pool: PgPool,
    pub oauth_client: BasicClient,
    pub jwt_secret: String,
    pub frontend_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
