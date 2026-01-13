use crate::models::{AppState, GameType};
use crate::utils::{update_supercell_cache, update_upstream_cache};
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

    // 2. Task for CoC Cache Refresh (Every 10 minutes)
    let coc_cache_data = data.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10 * 60));
        loop {
            ticker.tick().await;
            refresh_clans(&coc_cache_data, GameType::ClashOfClans).await;
        }
    });

    // 3. Task for CR Cache Refresh (Every 10 minutes, offset by 5 minutes)
    let cr_cache_data = data.clone();
    tokio::spawn(async move {
        // Offset by 5 minutes to spread load
        tokio::time::sleep(Duration::from_secs(5 * 60)).await;
        let mut ticker = interval(Duration::from_secs(10 * 60));
        loop {
            ticker.tick().await;
            refresh_clans(&cr_cache_data, GameType::ClashRoyale).await;
        }
    });
}

async fn measure_and_save_latency(data: &AppState) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    println!("Background: Measuring latency...");

    // 1. Measure CoC Upstream API
    let start = std::time::Instant::now();
    let coc_upstream_res = data
        .client
        .get(format!("{}/api/guild", data.upstream_coc_url))
        .header("Authorization", format!("Bearer {}", data.coc_api_token))
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    let coc_upstream_latency = if coc_upstream_res.is_ok() {
        start.elapsed().as_millis() as i32
    } else {
        -1
    };

    let _ = sqlx::query(
        "INSERT INTO latency_measurements (api_name, latency_ms, timestamp) VALUES ($1, $2, $3)",
    )
    .bind("upstream_coc")
    .bind(coc_upstream_latency)
    .bind(now)
    .execute(&data.db_pool)
    .await;

    // 2. Measure CR Upstream API
    let start = std::time::Instant::now();
    let cr_upstream_res = data
        .client
        .get(format!("{}/api/guild", data.upstream_cr_url))
        .header("Authorization", format!("Bearer {}", data.cr_api_token))
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    let cr_upstream_latency = if cr_upstream_res.is_ok() {
        start.elapsed().as_millis() as i32
    } else {
        -1
    };

    let _ = sqlx::query(
        "INSERT INTO latency_measurements (api_name, latency_ms, timestamp) VALUES ($1, $2, $3)",
    )
    .bind("upstream_cr")
    .bind(cr_upstream_latency)
    .bind(now)
    .execute(&data.db_pool)
    .await;

    // 3. Measure CoC Supercell API
    let start = std::time::Instant::now();
    let coc_sc_res = data
        .client
        .get("https://api.clashofclans.com/v1/clans/%232PP")
        .header(
            "Authorization",
            format!("Bearer {}", data.clash_of_clans_api_token),
        )
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    let coc_sc_latency = if coc_sc_res.is_ok() {
        start.elapsed().as_millis() as i32
    } else {
        -1
    };

    let _ = sqlx::query(
        "INSERT INTO latency_measurements (api_name, latency_ms, timestamp) VALUES ($1, $2, $3)",
    )
    .bind("supercell_coc")
    .bind(coc_sc_latency)
    .bind(now)
    .execute(&data.db_pool)
    .await;

    // 4. Measure CR Supercell API
    let start = std::time::Instant::now();
    let cr_sc_res = data
        .client
        .get("https://api.clashroyale.com/v1/clans/%232PP")
        .header(
            "Authorization",
            format!("Bearer {}", data.clash_royale_api_token),
        )
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    let cr_sc_latency = if cr_sc_res.is_ok() {
        start.elapsed().as_millis() as i32
    } else {
        -1
    };

    let _ = sqlx::query(
        "INSERT INTO latency_measurements (api_name, latency_ms, timestamp) VALUES ($1, $2, $3)",
    )
    .bind("supercell_cr")
    .bind(cr_sc_latency)
    .bind(now)
    .execute(&data.db_pool)
    .await;

    // 5. Measure Website (Frontend)
    let start = std::time::Instant::now();
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

async fn refresh_clans(data: &AppState, game: GameType) {
    let game_name = match game {
        GameType::ClashOfClans => "CoC",
        GameType::ClashRoyale => "CR",
    };

    println!("Background Refresh [{}]: Starting...", game_name);

    // 1. Fetch & Cache Guild Info (only for CoC which has the main guild)
    if game == GameType::ClashOfClans {
        let _ = update_upstream_cache(data, game, "/api/guild").await;
    }

    // 2. Fetch & Cache All Clans
    let clans_url = "/api/clans";
    match update_upstream_cache(data, game, clans_url).await {
        Ok(body_bytes) => {
            if let Ok(clans) = serde_json::from_slice::<Vec<Clan>>(&body_bytes) {
                println!(
                    "Background Refresh [{}]: Found {} clans. Updating detailed data...",
                    game_name,
                    clans.len()
                );

                for (i, clan) in clans.iter().enumerate() {
                    // Skip non-clan entries like "warteliste" for API refreshes
                    if !clan.tag.starts_with('#') {
                        continue;
                    }

                    let encoded_tag = crate::utils::encode_tag(&clan.tag);

                    print!(
                        "\rBackground Refresh [{}]: Processing clan {}/{} ({})      ",
                        game_name,
                        i + 1,
                        clans.len(),
                        clan.tag
                    );
                    use std::io::{Write, stdout};
                    let _ = stdout().flush();

                    // Upstream endpoints
                    let upstream_endpoints = if game == GameType::ClashOfClans {
                        vec![
                            format!("/api/clans/{}", encoded_tag),
                            format!("/api/clans/{}/members", encoded_tag),
                            format!("/api/clans/{}/kickpoint-reasons", encoded_tag),
                            format!("/api/clans/{}/war-members", encoded_tag),
                            format!("/api/clans/{}/raid-members", encoded_tag),
                            format!("/api/clans/{}/cwl-members", encoded_tag),
                        ]
                    } else {
                        vec![
                            format!("/api/clans/{}", encoded_tag),
                            format!("/api/clans/{}/members", encoded_tag),
                            format!("/api/clans/{}/kickpoint-reasons", encoded_tag),
                        ]
                    };

                    let supercell_endpoints = vec![format!("/clans/{}", encoded_tag)];

                    let mut set = tokio::task::JoinSet::new();

                    // Spawn upstream refreshes
                    for endpoint in upstream_endpoints {
                        let data = data.clone();
                        set.spawn(async move {
                            let res = update_upstream_cache(&data, game, &endpoint).await;
                            (format!("upstream:{}", endpoint), res)
                        });
                    }

                    // Spawn Supercell API refreshes
                    for endpoint in supercell_endpoints {
                        let data = data.clone();
                        set.spawn(async move {
                            let res = update_supercell_cache(&data, game, &endpoint).await;
                            (format!("supercell:{}", endpoint), res)
                        });
                    }

                    while let Some(res) = set.join_next().await {
                        if let Ok((endpoint, Err(e))) = res {
                            eprintln!("\n    Error refreshing {}: {}", endpoint, e);
                        }
                    }

                    // For CoC: Refresh members to ensure profile pages are fast
                    if game == GameType::ClashOfClans {
                        let prefix = "coc";
                        let members_cache_key =
                            format!("{}:upstream:/api/clans/{}/members", prefix, encoded_tag);
                        let members_res = sqlx::query_as::<_, (Vec<u8>,)>(
                            "SELECT body FROM cache WHERE key = $1",
                        )
                        .bind(&members_cache_key)
                        .fetch_optional(&data.db_pool)
                        .await;

                        if let Ok(Some((body,))) = members_res {
                            if let Ok(members) = serde_json::from_slice::<serde_json::Value>(&body)
                            {
                                if let Some(member_list) = members.as_array() {
                                    let mut player_set = tokio::task::JoinSet::new();
                                    for member in member_list.iter().take(50) {
                                        if let Some(player_tag) =
                                            member.get("tag").and_then(|t| t.as_str())
                                        {
                                            let enc_player_tag =
                                                crate::utils::encode_tag(player_tag);
                                            let data_sc = data.clone();
                                            let path_sc = format!("/players/{}", enc_player_tag);
                                            player_set.spawn(async move {
                                                let _ = update_supercell_cache(
                                                    &data_sc,
                                                    GameType::ClashOfClans,
                                                    &path_sc,
                                                )
                                                .await;
                                            });

                                            let data_up = data.clone();
                                            let path_up =
                                                format!("/api/players/{}", enc_player_tag);
                                            player_set.spawn(async move {
                                                let _ = update_upstream_cache(
                                                    &data_up,
                                                    GameType::ClashOfClans,
                                                    &path_up,
                                                )
                                                .await;
                                            });
                                        }
                                    }
                                    while let Some(_) = player_set.join_next().await {}
                                }
                            }
                        }
                    }

                    // For CR: Also refresh player data
                    if game == GameType::ClashRoyale {
                        let prefix = "cr";
                        let members_cache_key =
                            format!("{}:upstream:/api/clans/{}/members", prefix, encoded_tag);
                        let members_res = sqlx::query_as::<_, (Vec<u8>,)>(
                            "SELECT body FROM cache WHERE key = $1",
                        )
                        .bind(&members_cache_key)
                        .fetch_optional(&data.db_pool)
                        .await;

                        if let Ok(Some((body,))) = members_res {
                            if let Ok(members) = serde_json::from_slice::<serde_json::Value>(&body)
                            {
                                if let Some(member_list) = members.as_array() {
                                    let mut player_set = tokio::task::JoinSet::new();
                                    for member in member_list.iter().take(50) {
                                        if let Some(player_tag) =
                                            member.get("tag").and_then(|t| t.as_str())
                                        {
                                            let enc_player_tag =
                                                crate::utils::encode_tag(player_tag);
                                            let data_sc = data.clone();
                                            let path_sc = format!("/players/{}", enc_player_tag);
                                            player_set.spawn(async move {
                                                let _ = update_supercell_cache(
                                                    &data_sc,
                                                    GameType::ClashRoyale,
                                                    &path_sc,
                                                )
                                                .await;
                                            });

                                            let data_up = data.clone();
                                            let path_up =
                                                format!("/api/players/{}", enc_player_tag);
                                            player_set.spawn(async move {
                                                let _ = update_upstream_cache(
                                                    &data_up,
                                                    GameType::ClashRoyale,
                                                    &path_up,
                                                )
                                                .await;
                                            });
                                        }
                                    }
                                    while let Some(_) = player_set.join_next().await {}
                                }
                            }
                        }
                    }
                }
                println!(
                    "\nBackground Refresh [{}]: All clan and member data updated.",
                    game_name
                );
            } else {
                eprintln!(
                    "\nBackground Refresh [{}] Error: Failed to deserialize clans list response.",
                    game_name
                );
            }
        }
        Err(e) => eprintln!(
            "Background Refresh [{}] Error: Failed to fetch clans list: {}",
            game_name, e
        ),
    }

    println!(
        "\nBackground Refresh [{}]: Cycle complete. Next run in 10 minutes.",
        game_name
    );
}
