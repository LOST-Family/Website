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
    let mut clans = match update_upstream_cache(data, game, clans_url).await {
        Ok(body_bytes) => serde_json::from_slice::<Vec<Clan>>(&body_bytes).unwrap_or_default(),
        _ => vec![],
    };

    // Add side clans to the refresh list
    if game == GameType::ClashOfClans {
        if let Ok(side_clans) =
            sqlx::query_as::<_, crate::models::SideClan>("SELECT * FROM side_clans")
                .fetch_all(&data.db_pool)
                .await
        {
            for sc in side_clans {
                if !clans.iter().any(|c| c.tag == sc.clan_tag) {
                    clans.push(Clan { tag: sc.clan_tag });
                }
            }
        }
    }

    if !clans.is_empty() {
        info!(
            "Background Refresh [{}]: Found {} clans to update.",
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
                    // format!("/api/clans/{}/kickpoint-reasons", encoded_tag),
                    format!("/api/clans/{}/war-members", encoded_tag),
                    format!("/api/clans/{}/raid-members", encoded_tag),
                    format!("/api/clans/{}/cwl-members", encoded_tag),
                ]
            } else {
                vec![
                    format!("/api/clans/{}", encoded_tag),
                    format!("/api/clans/{}/members", encoded_tag),
                    // format!("/api/clans/{}/kickpoint-reasons", encoded_tag),
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
    }

    info!(
        "Background Refresh [{}]: Cycle complete. Next run in {} minutes.",
        game_name, data.background_refresh_interval
    );
}

async fn refresh_side_clans_cwl(data: &AppState) {
    info!("Background Refresh [Side Clans CWL]: Starting...");

    // 0. Update side clans from external configuration endpoint
    let sync_url = format!("{}/api/sideclans", data.upstream_coc_url);
    match data
        .client
        .get(&sync_url)
        .header("Authorization", format!("Bearer {}", data.coc_api_token))
        .send()
        .await
    {
        Ok(sync_resp) => {
            if sync_resp.status().is_success() {
                if let Ok(sync_bytes) = sync_resp.bytes().await {
                    match serde_json::from_slice::<Vec<crate::models::SideClan>>(&sync_bytes) {
                        Ok(side_clans) => {
                            info!(
                                "Background Refresh [Side Clans CWL]: Syncing {} clans from config...",
                                side_clans.len()
                            );

                            let tags_to_keep: Vec<String> =
                                side_clans.iter().map(|c| c.clan_tag.clone()).collect();
                            let _ = sqlx::query("DELETE FROM side_clans WHERE clan_tag != ALL($1)")
                                .bind(&tags_to_keep)
                                .execute(&data.db_pool)
                                .await;

                            for clan in side_clans {
                                let _ = sqlx::query(
                                    "INSERT INTO side_clans (clan_tag, name, belongs_to, display_index) 
                                     VALUES ($1, $2, $3, $4) 
                                     ON CONFLICT (clan_tag) DO UPDATE SET name = $2, belongs_to = $3, display_index = $4",
                                )
                                .bind(clan.clan_tag)
                                .bind(clan.name)
                                .bind(clan.belongs_to)
                                .bind(clan.display_index)
                                .execute(&data.db_pool)
                                .await;
                            }
                        }
                        Err(e) => {
                            error!(
                                "Background Refresh [Side Clans CWL]: Failed to deserialize side clans from sync response: {}",
                                e
                            );
                            if let Ok(body_str) = String::from_utf8(sync_bytes.to_vec()) {
                                error!(
                                    "Response snippet: {}",
                                    &body_str[..body_str.len().min(1000)]
                                );
                            }
                        }
                    }
                }
            } else {
                error!(
                    "Background Refresh [Side Clans CWL]: Failed to sync config. Status: {}",
                    sync_resp.status()
                );
            }
        }
        Err(e) => error!(
            "Background Refresh [Side Clans CWL]: Error syncing config: {}",
            e
        ),
    }

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
        let url_path = format!("/clans/{}", encoded_tag);

        match update_supercell_cache(data, GameType::ClashOfClans, &url_path).await {
            Ok(bytes) => {
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

                        // Try to get rank and accurate season from leaguegroup endpoint
                        let mut rank: Option<i32> = None;
                        let mut season_to_use = season.clone();

                        let lg_url = format!(
                            "https://api.clashofclans.com/v1/clans/{}/currentwar/leaguegroup",
                            encoded_tag
                        );
                        let lg_res = data
                            .client
                            .get(&lg_url)
                            .header(
                                "Authorization",
                                format!("Bearer {}", data.clash_of_clans_api_token),
                            )
                            .send()
                            .await;

                        if let Ok(lg_resp) = lg_res {
                            if lg_resp.status().is_success() {
                                if let Ok(lg_bytes) = lg_resp.bytes().await {
                                    if let Ok(lg_json) =
                                        serde_json::from_slice::<serde_json::Value>(&lg_bytes)
                                    {
                                        if let Some(s) = lg_json
                                            .get("season")
                                            .and_then(|v: &serde_json::Value| v.as_str())
                                        {
                                            season_to_use = s.to_string();
                                        }

                                        if let Some(clans) = lg_json
                                            .get("clans")
                                            .and_then(|v: &serde_json::Value| v.as_array())
                                        {
                                            let mut sorted_clans = clans.clone();
                                            sorted_clans.sort_by(|a, b| {
                                                let a_stars = a
                                                    .get("stars")
                                                    .and_then(|v: &serde_json::Value| v.as_i64())
                                                    .unwrap_or(0);
                                                let b_stars = b
                                                    .get("stars")
                                                    .and_then(|v: &serde_json::Value| v.as_i64())
                                                    .unwrap_or(0);
                                                let a_dest = a
                                                    .get("destructionPercentage")
                                                    .and_then(|v: &serde_json::Value| v.as_f64())
                                                    .unwrap_or(0.0);
                                                let b_dest = b
                                                    .get("destructionPercentage")
                                                    .and_then(|v: &serde_json::Value| v.as_f64())
                                                    .unwrap_or(0.0);

                                                b_stars.cmp(&a_stars).then_with(|| {
                                                    b_dest
                                                        .partial_cmp(&a_dest)
                                                        .unwrap_or(std::cmp::Ordering::Equal)
                                                })
                                            });

                                            if let Some(pos) = sorted_clans.iter().position(|c| {
                                                c.get("tag")
                                                    .and_then(|v: &serde_json::Value| v.as_str())
                                                    == Some(&clan_tag)
                                            }) {
                                                rank = Some((pos + 1) as i32);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let _ = sqlx::query(
                            "INSERT INTO side_clans_cwl_stats (clan_tag, season, league_id, league_name, league_badge_url, rank) 
                                 VALUES ($1, $2, $3, $4, $5, $6) 
                                 ON CONFLICT (clan_tag, season) DO UPDATE 
                                 SET league_id = EXCLUDED.league_id, 
                                     league_name = EXCLUDED.league_name,
                                     league_badge_url = EXCLUDED.league_badge_url,
                                     rank = EXCLUDED.rank",
                        )
                        .bind(&clan_tag)
                        .bind(&season_to_use)
                        .bind(league_id)
                        .bind(league_name)
                        .bind(league_badge_url)
                        .bind(rank)
                        .execute(&data.db_pool)
                        .await;
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
    format!(
        "https://api-assets.clashofclans.com/leagues/72/{}",
        filename
    )
}
