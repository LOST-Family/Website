use crate::models::AppState;
use crate::utils::{update_cache, update_clash_cache};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::interval;

#[derive(Deserialize)]
struct Clan {
    tag: String,
}

pub fn spawn_background_task(data: AppState) {
    // 1. Task for Latency Measurements (Every 1 minute)
    let latency_data = data.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            let _ = measure_and_save_latency(&latency_data).await;
        }
    });

    // 2. Task for Cache Refresh (Every 10 minutes)
    let cache_data = data.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10 * 60));
        loop {
            ticker.tick().await;
            refresh_clans(&cache_data).await;
        }
    });
}

async fn measure_and_save_latency(data: &AppState) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    println!("Background: Measuring latency...");

    // 1. Measure Upstream API
    let start = std::time::Instant::now();
    let upstream_res = data
        .client
        .get(format!("{}/api/guild", data.upstream_url))
        .header("Authorization", format!("Bearer {}", data.api_token))
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    let upstream_latency = if upstream_res.is_ok() {
        start.elapsed().as_millis() as i32
    } else {
        -1 // Indicates downtime
    };

    let _ = sqlx::query(
        "INSERT INTO latency_measurements (api_name, latency_ms, timestamp) VALUES ($1, $2, $3)",
    )
    .bind("upstream")
    .bind(upstream_latency)
    .bind(now)
    .execute(&data.db_pool)
    .await;

    // 2. Measure Supercell API
    let start = std::time::Instant::now();
    let sc_res = data
        .client
        .get("https://api.clashofclans.com/v1/clans/%232PP")
        .header("Authorization", format!("Bearer {}", data.clash_api_token))
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    let sc_latency = if sc_res.is_ok() {
        start.elapsed().as_millis() as i32
    } else {
        -1
    };

    let _ = sqlx::query(
        "INSERT INTO latency_measurements (api_name, latency_ms, timestamp) VALUES ($1, $2, $3)",
    )
    .bind("supercell")
    .bind(sc_latency)
    .bind(now)
    .execute(&data.db_pool)
    .await;

    // 3. Measure Website (Frontend)
    let start = std::time::Instant::now();
    // In Docker development, we might need to hit 'website:5173' instead of 'localhost:5173'
    let url = if data.frontend_url.contains("localhost") {
        data.frontend_url.replace("localhost", "website")
    } else {
        data.frontend_url.clone()
    };

    let website_res = data
        .client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    let website_latency = if website_res.is_ok() {
        start.elapsed().as_millis() as i32
    } else {
        -1
    };

    let _ = sqlx::query(
        "INSERT INTO latency_measurements (api_name, latency_ms, timestamp) VALUES ($1, $2, $3)",
    )
    .bind("website")
    .bind(website_latency)
    .bind(now)
    .execute(&data.db_pool)
    .await;

    // Cleanup old data (older than 24 hours)
    let one_day_ago = now - 24 * 60 * 60;
    let _ = sqlx::query("DELETE FROM latency_measurements WHERE timestamp < $1")
        .bind(one_day_ago)
        .execute(&data.db_pool)
        .await;
}

async fn refresh_clans(data: &AppState) {
    // println!("Background Refresh: Fetching clans list...");

    // 1. Fetch & Cache Guild Info
    let _ = update_cache(data, "/api/guild").await;

    // 2. Fetch & Cache All Clans
    let clans_url = "/api/clans";
    match update_cache(data, clans_url).await {
        Ok(body_bytes) => {
            // 2. Parse tags to fetch detailed data for each clan
            if let Ok(clans) = serde_json::from_slice::<Vec<Clan>>(&body_bytes) {
                // println!(
                //     "Background Refresh: Found {} clans. Updating detailed data...",
                //     clans.len()
                // );
                for (i, clan) in clans.iter().enumerate() {
                    let encoded_tag = crate::utils::encode_tag(&clan.tag);

                    print!(
                        "\rBackground Refresh: Processing clan {}/{} ({})      ",
                        i + 1,
                        clans.len(),
                        clan.tag
                    );
                    use std::io::{Write, stdout};
                    let _ = stdout().flush();

                    // List of endpoints to proactively cache for each clan (concurrently)
                    let upstream_endpoints = [
                        format!("/api/clans/{}", encoded_tag),
                        format!("/api/clans/{}/members", encoded_tag),
                        format!("/api/clans/{}/kickpoint-reasons", encoded_tag),
                        format!("/api/clans/{}/war-members", encoded_tag),
                        format!("/api/clans/{}/raid-members", encoded_tag),
                        format!("/api/clans/{}/cwl-members", encoded_tag),
                    ];

                    let clash_endpoints = [
                        format!("/clans/{}", encoded_tag),
                    ];

                    let mut set = tokio::task::JoinSet::new();

                    // Spawn upstream refreshes
                    for endpoint in upstream_endpoints {
                        let data = data.clone();
                        set.spawn(async move {
                            let res = update_cache(&data, &endpoint).await;
                            (format!("upstream:{}", endpoint), res)
                        });
                    }

                    // Spawn Clash API refreshes
                    for endpoint in clash_endpoints {
                        let data = data.clone();
                        set.spawn(async move {
                            let res = update_clash_cache(&data, &endpoint).await;
                            (format!("clash:{}", endpoint), res)
                        });
                    }

                    while let Some(res) = set.join_next().await {
                        if let Ok((endpoint, Err(e))) = res {
                            eprintln!("\n    Error refreshing {}: {}", endpoint, e);
                        }
                    }
                }
                println!("\nBackground Refresh: All clan data updated.");
            } else {
                eprintln!("\nBackground Refresh Error: Failed to deserialize clans list response.");
            }
        }
        Err(e) => eprintln!(
            "Background Refresh Error: Failed to fetch clans list: {}",
            e
        ),
    }

    println!("\nBackground Refresh: Cycle complete. Next run in 1 minutes.");
}
