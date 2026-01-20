use crate::models::{AppState, GameType};
use crate::utils::{update_supercell_cache, update_upstream_cache};

use log::{debug, error, info};
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

    // 2. Task for CoC Cache Refresh
    let coc_cache_data = data.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(
            coc_cache_data.background_refresh_interval * 60,
        ));
        loop {
            ticker.tick().await;
            refresh_clans(&coc_cache_data, GameType::ClashOfClans).await;
        }
    });

    // 3. Task for CR Cache Refresh (offset by 15 seconds)
    let cr_cache_data = data.clone();
    tokio::spawn(async move {
        // Offset by a bit to spread load
        tokio::time::sleep(Duration::from_secs(15)).await;
        let mut ticker = interval(Duration::from_secs(
            cr_cache_data.background_refresh_interval * 60,
        ));
        loop {
            ticker.tick().await;
            refresh_clans(&cr_cache_data, GameType::ClashRoyale).await;
        }
    });

    // 4. Task for Side Clans CWL Refresh (Every 1 hour)
    let side_clans_data = data.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(3600));
        loop {
            ticker.tick().await;
            refresh_side_clans_cwl(&side_clans_data).await;
        }
    });
}

async fn measure_and_save_latency(data: &AppState) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    debug!("Background: Measuring latency...");

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

    info!("Background Refresh [{}]: Starting...", game_name);

    // 1. Fetch & Cache Guild Info (only for CoC which has the main guild)
    if game == GameType::ClashOfClans {
        let _ = update_upstream_cache(data, game, "/api/guild").await;
    }

    let clans_url = "/api/clans";
    match update_upstream_cache(data, game, clans_url).await {
        Ok(body_bytes) => {
            if let Ok(clans) = serde_json::from_slice::<Vec<Clan>>(&body_bytes) {
                info!(
                    "Background Refresh [{}]: Fetched clans list successfully. Found {} clans. Updating detailed data...",
                    game_name,
                    clans.len()
                );

                for (i, clan) in clans.iter().enumerate() {
                    // Skip non-clan entries like "warteliste" for API refreshes
                    if !clan.tag.starts_with('#') {
                        continue;
                    }

                    let encoded_tag = crate::utils::encode_tag(&clan.tag);

                    debug!(
                        "Background Refresh [{}]: Processing clan {}/{} ({})",
                        game_name,
                        i + 1,
                        clans.len(),
                        clan.tag
                    );

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
                            error!("Error refreshing {}: {}", endpoint, e);
                        }
                    }
                }
                info!(
                    "Background Refresh [{}]: All clan and member data updated.",
                    game_name
                );
            } else {
                error!(
                    "Background Refresh [{}] Error: Failed to deserialize clans list response.",
                    game_name
                );
            }
        }
        Err(e) => error!(
            "Background Refresh [{}] Error: Failed to fetch clans list: {}",
            game_name, e
        ),
    }

    info!(
        "Background Refresh [{}]: Cycle complete. Next run in {} minutes.",
        game_name, data.background_refresh_interval
    );
}

async fn refresh_side_clans_cwl(data: &AppState) {
    info!("Background Refresh [Side Clans CWL]: Starting...");

    let clans = match sqlx::query("SELECT clan_tag FROM side_clans")
        .fetch_all(&data.db_pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!("Error fetching side clans from DB: {}", e);
            return;
        }
    };

    let now = chrono::Utc::now();
    let season = now.format("%Y-%m").to_string();

    for row in clans {
        use sqlx::Row;
        let clan_tag: String = row.get("clan_tag");
        let encoded_tag = crate::utils::encode_tag(&clan_tag);
        let url = format!("https://api.clashofclans.com/v1/clans/{}", encoded_tag);

        let res = data
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", data.clash_of_clans_api_token))
            .send()
            .await;

        match res {
            Ok(resp) => {
                let status = resp.status();
                if !status.is_success() {
                    error!("Error fetching CWL stats for {}: Status {}", clan_tag, status);
                    continue;
                }
                if let Ok(bytes) = resp.bytes().await {
                    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                        info!("Fetched data for side clan {}", clan_tag);
                        if let Some(clan_name) = json.get("name").and_then(|v| v.as_str()) {
                            let _ = sqlx::query("UPDATE side_clans SET name = $1 WHERE clan_tag = $2")
                                .bind(clan_name)
                                .bind(&clan_tag)
                                .execute(&data.db_pool)
                                .await;
                        }

                        if let Some(war_league) = json.get("warLeague") {
                            let league_id: Option<i32> = war_league
                                .get("id")
                                .and_then(|v| v.as_i64())
                                .map(|v| v as i32);
                            let league_name: Option<String> = war_league
                                .get("name")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            
                            debug!("Clan {} is in league {:?}", clan_tag, league_name);

                            let mut league_badge_url: Option<String> = war_league
                                .get("iconUrls")
                                .and_then(|v| v.get("medium").or(v.get("small")))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            if league_badge_url.is_none() {
                                if let Some(id) = league_id {
                                    league_badge_url = Some(get_cwl_badge_url(id));
                                }
                            }

                            let _ = sqlx::query(
                                "INSERT INTO side_clans_cwl_stats (clan_tag, season, league_id, league_name, league_badge_url) 
                                 VALUES ($1, $2, $3, $4, $5) 
                                 ON CONFLICT (clan_tag, season) DO UPDATE 
                                 SET league_id = EXCLUDED.league_id, 
                                     league_name = EXCLUDED.league_name,
                                     league_badge_url = EXCLUDED.league_badge_url",
                            )
                            .bind(&clan_tag)
                            .bind(&season)
                            .bind(league_id)
                            .bind(league_name)
                            .bind(league_badge_url)
                            .execute(&data.db_pool)
                            .await;
                        }
                    }
                }
            }
            Err(e) => error!("Error fetching CWL stats for {}: {}", clan_tag, e),
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn get_cwl_badge_url(id: i32) -> String {
    let filename = match id {
        48000000 => "n_m6p9sTofL_is_H0l7m2t-kAnp73yI707vC-Hj-90.png",
        48000001 => "NoBqf6O968S_5R2_yAsCndW_nS0Y_wW_l0p3Yk05Xp0.png",
        48000002 => "9090u6yInKH9tC6_7J2vto_sU9zL3iS8sC3p8g0kX_E.png",
        48000003 => "F6XvA2h7tC6_7J2vto_sU9zL3iS8sC3p8g0kX_E.png",
        48000004 => "v_LPrJXvA2h7tC6_7J2vto_sU9zL3iS8sC3p8g0kX_E.png",
        48000005 => "d3u99dF6XvA2h7tC6_7J2vto_sU9zL3iS8sC3p8g0kX_E.png",
        48000006 => "8hB7vXvA2h7tC6_7J2vto_sU9zL3iS8sC3p8g0kX_E.png",
        48000007 => "6v88qT8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000008 => "309v8T8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000009 => "p9u8XT8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000010 => "jhB8XT8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000011 => "m3u8XT8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000012 => "l3u8XT8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000013 => "O_O8XT8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000014 => "N_O8XT8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000015 => "M_O8XT8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000016 => "c_O8XT8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000017 => "b_O8XT8uW_XpB8v9rJmS0Y_wW_l0p3Yk05Xp0.png",
        48000018 => "9vmAr6V5UvS6iS_I4oY8o3Xp_V0mS4G4U8Xo-G_H44A.png",
        _ => "n_m6p9sTofL_is_H0l7m2t-kAnp73yI707vC-Hj-90.png",
    };
    format!("https://api-assets.clashofclans.com/leagues/72/{}", filename)
}
