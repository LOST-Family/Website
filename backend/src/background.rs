use crate::models::AppState;
use crate::utils::update_cache;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::interval;

#[derive(Deserialize)]
struct Clan {
    tag: String,
}

pub fn spawn_background_task(data: AppState) {
    tokio::spawn(async move {
        // Run immediately on startup
        refresh_clans(&data).await;

        // Then run every 1 minutes
        let mut ticker = interval(Duration::from_secs(1 * 60));
        // The first tick completes immediately, so we skip it since we just ran headers
        ticker.tick().await;

        loop {
            ticker.tick().await;
            refresh_clans(&data).await;
        }
    });
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
                    let endpoints = [
                        format!("/api/clans/{}", encoded_tag),
                        format!("/api/clans/{}/members", encoded_tag),
                        format!("/api/clans/{}/kickpoint-reasons", encoded_tag),
                        format!("/api/clans/{}/war-members", encoded_tag),
                        format!("/api/clans/{}/raid-members", encoded_tag),
                        format!("/api/clans/{}/cwl-members", encoded_tag),
                    ];

                    let mut set = tokio::task::JoinSet::new();
                    for endpoint in endpoints {
                        let data = data.clone();
                        set.spawn(async move {
                            let res = update_cache(&data, &endpoint).await;
                            (endpoint, res)
                        });
                    }

                    while let Some(res) = set.join_next().await {
                        if let Ok((endpoint, Err(e))) = res {
                            eprintln!("\n    Error refreshing {}: {}", endpoint, e);
                        }
                    }
                }
                // println!("\nBackground Refresh: All clan data updated.");
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
