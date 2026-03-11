use crate::models::{AppState, ErrorResponse, GameType};
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use bytes::Bytes;
use log::error;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

pub fn format_url(base: &str, path: &str) -> String {
    format!("{}{}", base, path)
}

pub fn encode_tag(tag: &str) -> String {
    let tag = if tag.starts_with('#') {
        tag.to_string()
    } else {
        format!("#{}", tag)
    };
    utf8_percent_encode(&tag, NON_ALPHANUMERIC).to_string()
}

fn get_cache_prefix(game: GameType) -> &'static str {
    match game {
        GameType::ClashOfClans => "coc",
        GameType::ClashRoyale => "cr",
    }
}

fn get_supercell_api_url(game: GameType) -> &'static str {
    match game {
        GameType::ClashOfClans => "https://api.clashofclans.com/v1",
        GameType::ClashRoyale => "https://api.clashroyale.com/v1",
    }
}

fn get_upstream_url(data: &AppState, game: GameType) -> &str {
    match game {
        GameType::ClashOfClans => &data.upstream_coc_url,
        GameType::ClashRoyale => &data.upstream_cr_url,
    }
}

fn get_upstream_token(data: &AppState, game: GameType) -> &str {
    match game {
        GameType::ClashOfClans => &data.coc_api_token,
        GameType::ClashRoyale => &data.cr_api_token,
    }
}

fn get_supercell_token(data: &AppState, game: GameType) -> &str {
    match game {
        GameType::ClashOfClans => &data.clash_of_clans_api_token,
        GameType::ClashRoyale => &data.clash_royale_api_token,
    }
}

/// Given a Clash Royale badgeId (e.g. 16000000), return a fallback badge URL
/// from the RoyaleAPI GitHub assets repository.
pub fn badge_url_from_id(badge_id: i64) -> Option<String> {
    let name = match badge_id {
        16000000 => "Flame_01",
        16000001 => "Flame_02",
        16000002 => "Flame_03",
        16000003 => "Flame_04",
        16000004 => "Sword_01",
        16000005 => "Sword_02",
        16000006 => "Sword_03",
        16000007 => "Sword_04",
        16000008 => "Bolt_01",
        16000009 => "Bolt_02",
        16000010 => "Bolt_03",
        16000011 => "Bolt_04",
        16000012 => "Crown_01",
        16000013 => "Crown_02",
        16000014 => "Crown_03",
        16000015 => "Crown_04",
        16000016 => "Arrow_01",
        16000017 => "Arrow_02",
        16000018 => "Arrow_03",
        16000019 => "Arrow_04",
        16000020 => "Diamond_Star_01",
        16000021 => "Diamond_Star_02",
        16000022 => "Diamond_Star_03",
        16000023 => "Diamond_Star_04",
        16000024 => "Skull_01",
        16000025 => "Skull_02",
        16000026 => "Skull_03",
        16000027 => "Skull_04",
        16000028 => "Skull_05",
        16000029 => "Skull_06",
        16000030 => "Moon_01",
        16000031 => "Moon_02",
        16000032 => "Moon_03",
        16000033 => "Pine_01",
        16000034 => "Pine_02",
        16000035 => "Pine_03",
        16000036 => "Traditional_Star_01",
        16000037 => "Traditional_Star_02",
        16000038 => "Traditional_Star_03",
        16000039 => "Traditional_Star_04",
        16000040 => "Traditional_Star_05",
        16000041 => "Traditional_Star_06",
        16000042 => "Star_Shine_01",
        16000043 => "Star_Shine_02",
        16000044 => "Star_Shine_03",
        16000045 => "Diamond_01",
        16000046 => "Diamond_02",
        16000047 => "Diamond_03",
        16000048 => "flag_a_01",
        16000049 => "flag_a_02",
        16000050 => "flag_a_03",
        16000051 => "flag_b_01",
        16000052 => "flag_b_02",
        16000053 => "flag_b_03",
        16000054 => "flag_c_03",
        16000055 => "flag_c_04",
        16000056 => "flag_c_05",
        16000057 => "flag_c_06",
        16000058 => "flag_c_07",
        16000059 => "flag_c_08",
        16000060 => "flag_d_01",
        16000061 => "flag_d_02",
        16000062 => "flag_d_03",
        16000063 => "flag_d_04",
        16000064 => "flag_d_05",
        16000065 => "flag_d_06",
        16000066 => "flag_f_01",
        16000067 => "flag_f_02",
        16000068 => "flag_g_01",
        16000069 => "flag_g_02",
        16000070 => "flag_i_01",
        16000071 => "flag_i_02",
        16000072 => "flag_h_01",
        16000073 => "flag_h_02",
        16000074 => "flag_h_03",
        16000075 => "flag_j_01",
        16000076 => "flag_j_02",
        16000077 => "flag_j_03",
        16000078 => "flag_k_01",
        16000079 => "flag_k_02",
        16000080 => "flag_k_03",
        16000081 => "flag_k_04",
        16000082 => "flag_k_05",
        16000083 => "flag_k_06",
        16000084 => "flag_l_01",
        16000085 => "flag_l_02",
        16000086 => "flag_l_03",
        16000087 => "flag_m_01",
        16000088 => "flag_m_02",
        16000089 => "flag_m_03",
        16000090 => "flag_n_01",
        16000091 => "flag_n_02",
        16000092 => "flag_n_03",
        16000093 => "flag_n_04",
        16000094 => "flag_n_05",
        16000095 => "flag_n_06",
        16000096 => "Twin_Peaks_01",
        16000097 => "Twin_Peaks_02",
        16000098 => "Gem_01",
        16000099 => "Gem_02",
        16000100 => "Gem_03",
        16000101 => "Gem_04",
        16000102 => "Coin_01",
        16000103 => "Coin_02",
        16000104 => "Coin_03",
        16000105 => "Coin_04",
        16000106 => "Elixir_01",
        16000107 => "Elixir_02",
        16000108 => "Heart_01",
        16000109 => "Heart_02",
        16000110 => "Heart_04",
        16000111 => "Heart_03",
        16000112 => "Tower_01",
        16000113 => "Tower_02",
        16000114 => "Tower_03",
        16000115 => "Tower_04",
        16000116 => "Fan_01",
        16000117 => "Fan_02",
        16000118 => "Fan_03",
        16000119 => "Fan_04",
        16000120 => "Fugi_01",
        16000121 => "Fugi_02",
        16000122 => "Fugi_03",
        16000123 => "Fugi_04",
        16000124 => "YingYang_01",
        16000125 => "YingYang_02",
        16000126 => "flag_c_01",
        16000127 => "flag_c_02",
        16000128 => "Cherry_Blossom_01",
        16000129 => "Cherry_Blossom_02",
        16000130 => "Cherry_Blossom_03",
        16000131 => "Cherry_Blossom_04",
        16000132 => "Cherry_Blossom_06",
        16000133 => "Cherry_Blossom_05",
        16000134 => "Cherry_Blossom_07",
        16000135 => "Cherry_Blossom_08",
        16000136 => "Bamboo_01",
        16000137 => "Bamboo_02",
        16000138 => "Bamboo_03",
        16000139 => "Bamboo_04",
        16000140 => "Orange_01",
        16000141 => "Orange_02",
        16000142 => "Lotus_01",
        16000143 => "Lotus_02",
        16000144 => "A_Char_King_01",
        16000145 => "A_Char_King_02",
        16000146 => "A_Char_King_03",
        16000147 => "A_Char_King_04",
        16000148 => "A_Char_Barbarian_01",
        16000149 => "A_Char_Barbarian_02",
        16000150 => "A_Char_Prince_01",
        16000151 => "A_Char_Prince_02",
        16000152 => "A_Char_Knight_01",
        16000153 => "A_Char_Knight_02",
        16000154 => "A_Char_Goblin_01",
        16000155 => "A_Char_Goblin_02",
        16000156 => "A_Char_DarkPrince_01",
        16000157 => "A_Char_DarkPrince_02",
        16000158 => "A_Char_DarkPrince_03",
        16000159 => "A_Char_DarkPrince_04",
        16000160 => "A_Char_MiniPekka_01",
        16000161 => "A_Char_MiniPekka_02",
        16000162 => "A_Char_Pekka_01",
        16000163 => "A_Char_Pekka_02",
        16000164 => "A_Char_Hammer_01",
        16000165 => "A_Char_Hammer_02",
        16000166 => "A_Char_Rocket_01",
        16000167 => "A_Char_Rocket_02",
        16000168 => "Freeze_01",
        16000169 => "Freeze_02",
        16000170 => "Clover_01",
        16000171 => "Clover_02",
        16000172 => "flag_h_04",
        16000173 => "flag_e_02",
        16000174 => "flag_i_03",
        16000175 => "flag_e_01",
        16000176 => "A_Char_Barbarian_03",
        16000177 => "A_Char_Prince_03",
        16000178 => "A_Char_Bomb_01",
        16000179 => "A_Char_Bomb_02",
        _ => return None,
    };
    Some(format!(
        "https://raw.githubusercontent.com/RoyaleAPI/cr-api-assets/master/badges/{}.png",
        name
    ))
}

// Function to filter out specific fields from clan data
pub fn filter_clan_data(body: Bytes, game: GameType, filter_fields: bool) -> Bytes {
    if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&body) {
        let mut modified = false;

        let fields_to_remove = [
            "maxKickpoints",
            "minSeasonWins",
            "kickpointsExpireAfterDays",
            "kickpointReasons",
        ];

        if let Some(clans) = value.as_array_mut() {
            // Filter out "Warteliste" for Clash Royale (always, for everyone)
            if game == GameType::ClashRoyale {
                let old_len = clans.len();
                clans.retain(|c| {
                    let name = c.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let name_db = c.get("nameDB").and_then(|n| n.as_str()).unwrap_or("");
                    let tag = c.get("tag").and_then(|t| t.as_str()).unwrap_or("");
                    name.to_lowercase() != "warteliste"
                        && name_db.to_lowercase() != "warteliste"
                        && tag.to_lowercase() != "warteliste"
                });
                if clans.len() != old_len {
                    modified = true;
                }
            }

            for clan in clans {
                if let Some(obj) = clan.as_object_mut() {
                    // Fix badgeUrl mismatch (singular vs plural) - Always apply
                    if !obj.contains_key("badgeUrls")
                        && let Some(url) = obj.get("badgeUrl").and_then(|u| u.as_str())
                    {
                        obj.insert(
                            "badgeUrls".to_string(),
                            serde_json::json!({
                                "small": url,
                                "medium": url,
                                "large": url
                            }),
                        );
                        modified = true;
                    }

                    // Fallback: construct badgeUrls from badgeId if still missing
                    if !obj.contains_key("badgeUrls") {
                        if let Some(badge_id) = obj.get("badgeId").and_then(|v| v.as_i64()) {
                            if let Some(url) = badge_url_from_id(badge_id) {
                                obj.insert(
                                    "badgeUrls".to_string(),
                                    serde_json::json!({
                                        "small": &url,
                                        "medium": &url,
                                        "large": &url
                                    }),
                                );
                                modified = true;
                            }
                        }
                    }

                    if filter_fields {
                        for field in &fields_to_remove {
                            obj.remove(*field);
                        }
                        modified = true;
                    }
                }
            }
        } else if let Some(obj) = value.as_object_mut() {
            // Fix badgeUrl mismatch (singular vs plural)
            if !obj.contains_key("badgeUrls")
                && let Some(url) = obj.get("badgeUrl").and_then(|u| u.as_str())
            {
                obj.insert(
                    "badgeUrls".to_string(),
                    serde_json::json!({
                        "small": url,
                        "medium": url,
                        "large": url
                    }),
                );
            }

            // Fallback: construct badgeUrls from badgeId if still missing
            if !obj.contains_key("badgeUrls") {
                if let Some(badge_id) = obj.get("badgeId").and_then(|v| v.as_i64()) {
                    if let Some(url) = badge_url_from_id(badge_id) {
                        obj.insert(
                            "badgeUrls".to_string(),
                            serde_json::json!({
                                "small": &url,
                                "medium": &url,
                                "large": &url
                            }),
                        );
                    }
                }
            }

            if filter_fields {
                for field in &fields_to_remove {
                    obj.remove(*field);
                }
                modified = true;
            }
        }

        if modified && let Ok(filtered) = serde_json::to_vec(&value) {
            return Bytes::from(filtered);
        }
    }
    body
}

/// Enrich clan list entries that are missing badge data by looking up
/// each clan's Supercell API cache to extract badgeId and resolve it.
async fn enrich_clan_badges(body: Bytes, pool: &sqlx::PgPool, game: GameType) -> Bytes {
    let mut value: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return body,
    };

    let clans = match value.as_array_mut() {
        Some(arr) => arr,
        None => return body,
    };

    let prefix = get_cache_prefix(game);
    let mut modified = false;

    for clan in clans.iter_mut() {
        let obj = match clan.as_object_mut() {
            Some(o) => o,
            None => continue,
        };

        // Skip if already has badge data
        if obj.contains_key("badgeUrls") {
            continue;
        }

        // Get clan tag to look up Supercell cache
        let tag = match obj.get("tag").and_then(|t| t.as_str()) {
            Some(t) => t.to_string(),
            None => continue,
        };

        let encoded = encode_tag(&tag);
        let cache_key = format!("{}:supercell:/clans/{}", prefix, encoded);

        if let Ok(Some((sc_body, _))) =
            sqlx::query_as::<_, (Vec<u8>, i32)>("SELECT body, status FROM cache WHERE key = $1")
                .bind(&cache_key)
                .fetch_optional(pool)
                .await
        {
            if let Ok(sc_json) = serde_json::from_slice::<serde_json::Value>(&sc_body) {
                // Prefer badgeId -> GitHub URL (clean square images)
                if let Some(badge_id) = sc_json.get("badgeId").and_then(|v| v.as_i64()) {
                    if let Some(url) = badge_url_from_id(badge_id) {
                        obj.insert(
                            "badgeUrls".to_string(),
                            serde_json::json!({
                                "small": &url,
                                "medium": &url,
                                "large": &url
                            }),
                        );
                        modified = true;
                    }
                }
                // Fallback to badgeUrls from Supercell API
                else if let Some(badge_urls) = sc_json.get("badgeUrls") {
                    obj.insert("badgeUrls".to_string(), badge_urls.clone());
                    modified = true;
                }
            }
        }
    }

    if modified {
        if let Ok(enriched) = serde_json::to_vec(&value) {
            return Bytes::from(enriched);
        }
    }
    body
}

// Function to filter out specific fields from member data
pub fn filter_member_data(body: Bytes, exempt_tags: &[String], user_role: Option<&str>) -> Bytes {
    use crate::auth::has_required_role;

    // Coleaders and higher see everything
    if has_required_role(user_role, "COLEADER") {
        return body;
    }

    if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&body) {
        let mut modified = false;
        let is_member = has_required_role(user_role, "MEMBER");

        let fields_to_remove_not_member = [
            "totalKickpoints",
            "activeKickpoints",
            "userId",
            "discordId",
            "nickname",
            "avatar",
        ];

        let process_obj = |obj: &mut serde_json::Map<String, serde_json::Value>, tag: &str| {
            if exempt_tags.iter().any(|et| et == tag) {
                return false;
            }

            // Always remove internal DB fields
            obj.remove("clanDB");

            if is_member {
                // For members: Return count and sum instead of full details
                if let Some(akp) = obj.get("activeKickpoints").and_then(|v| v.as_array()) {
                    let sum: i64 = akp
                        .iter()
                        .filter_map(|kp| kp.get("amount").and_then(|a| a.as_i64()))
                        .sum();
                    obj.insert(
                        "activeKickpointsCount".to_string(),
                        serde_json::json!(akp.len()),
                    );
                    obj.insert("activeKickpointsSum".to_string(), serde_json::json!(sum));
                }
                obj.remove("activeKickpoints");

                // Hide raw IDs for members (privacy), but keep nickname/avatar/points
                let is_coleader = has_required_role(user_role, "COLEADER");
                if !is_coleader {
                    if obj.contains_key("userId") {
                        obj.insert("isLinked".to_string(), serde_json::json!(true));
                    }
                    obj.remove("userId");
                    obj.remove("discordId");
                }
            } else {
                // Not a member: Remove counts and identity links
                for field in &fields_to_remove_not_member {
                    obj.remove(*field);
                }
            }
            true
        };

        let is_coleader = has_required_role(user_role, "COLEADER");

        if let Some(arr) = value.as_array_mut() {
            arr.retain(|m| {
                if is_coleader {
                    return true;
                }
                !m.get("isHidden").and_then(|v| v.as_bool()).unwrap_or(false)
            });
            for member in arr {
                if let Some(obj) = member.as_object_mut() {
                    let tag = obj
                        .get("tag")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string();
                    if process_obj(obj, &tag) {
                        modified = true;
                    }
                }
            }
        } else if let Some(obj) = value.as_object_mut() {
            // Handle wrapper objects like { "members": [...] } or { "clans": [ { "members": [...] } ] }
            if let Some(m_arr) = obj.get_mut("members").and_then(|v| v.as_array_mut()) {
                m_arr.retain(|m| {
                    if is_coleader {
                        return true;
                    }
                    !m.get("isHidden").and_then(|v| v.as_bool()).unwrap_or(false)
                });
                for member in m_arr {
                    if let Some(m_obj) = member.as_object_mut() {
                        let tag = m_obj
                            .get("tag")
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        if process_obj(m_obj, &tag) {
                            modified = true;
                        }
                    }
                }
            }

            // Important: we re-borrow obj here AFTER m_arr is dropped
            if let Some(c_arr) = obj.get_mut("clans").and_then(|v| v.as_array_mut()) {
                for clan in c_arr {
                    if let Some(m_arr) = clan.get_mut("members").and_then(|v| v.as_array_mut()) {
                        m_arr.retain(|m| {
                            if is_coleader {
                                return true;
                            }
                            !m.get("isHidden").and_then(|v| v.as_bool()).unwrap_or(false)
                        });
                        for member in m_arr {
                            if let Some(m_obj) = member.as_object_mut() {
                                let tag = m_obj
                                    .get("tag")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                if process_obj(m_obj, &tag) {
                                    modified = true;
                                }
                            }
                        }
                    }
                }
            }

            // Also check if the top-level object itself is a player
            if obj.contains_key("tag")
                && (obj.contains_key("role") || obj.contains_key("townHallLevel"))
            {
                let tag = obj
                    .get("tag")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                if process_obj(obj, &tag) {
                    modified = true;
                }
            }
        }

        if modified && let Ok(filtered) = serde_json::to_vec(&value) {
            return Bytes::from(filtered);
        }
    }
    body
}

// Update upstream cache (bot server)
pub async fn update_upstream_cache(
    data: &AppState,
    game: GameType,
    url_path: &str,
) -> Result<Bytes, String> {
    let prefix = get_cache_prefix(game);
    let upstream_url = get_upstream_url(data, game);
    let token = get_upstream_token(data, game);
    let full_url = format_url(upstream_url, url_path);

    match data
        .client
        .get(&full_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status().as_u16();
            let body = res.bytes().await.map_err(|e| e.to_string())?;

            if status == 200 {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let cache_key = format!("{}:upstream:{}", prefix, url_path);

                let _ = sqlx::query(
                    "INSERT INTO cache (key, body, status, updated_at) 
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (key) DO UPDATE SET 
                    body = EXCLUDED.body, 
                    status = EXCLUDED.status, 
                    updated_at = EXCLUDED.updated_at",
                )
                .bind(&cache_key)
                .bind(body.to_vec())
                .bind(status as i32)
                .bind(timestamp)
                .execute(&data.db_pool)
                .await;

                Ok(body)
            } else {
                let err_msg = format!("Upstream {} returned status {}", full_url, status);
                eprintln!("Background Refresh: {}", err_msg);
                Err(err_msg)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn get_cached_or_update_supercell_cache(
    data: &AppState,
    game: GameType,
    url_path: &str,
    ttl_seconds: i64,
) -> Result<Bytes, String> {
    let prefix = get_cache_prefix(game);
    let cache_key = format!("{}:supercell:{}", prefix, url_path);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let cached_result =
        sqlx::query_as::<_, (Vec<u8>, i64)>("SELECT body, updated_at FROM cache WHERE key = $1")
            .bind(&cache_key)
            .fetch_optional(&data.db_pool)
            .await;

    if let Ok(Some((body, updated_at))) = &cached_result
        && now - updated_at < ttl_seconds
    {
        return Ok(Bytes::from(body.clone()));
    }

    match update_supercell_cache(data, game, url_path).await {
        Ok(body) => Ok(body),
        Err(e) => {
            // Fallback to expired cache on error
            if let Ok(Some((body, _))) = cached_result {
                return Ok(Bytes::from(body));
            }
            Err(e)
        }
    }
}

pub async fn get_cached_or_update_upstream_cache(
    data: &AppState,
    game: GameType,
    url_path: &str,
    ttl_seconds: i64,
) -> Result<Bytes, String> {
    let prefix = get_cache_prefix(game);
    let cache_key = format!("{}:upstream:{}", prefix, url_path);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let cached_result =
        sqlx::query_as::<_, (Vec<u8>, i64)>("SELECT body, updated_at FROM cache WHERE key = $1")
            .bind(&cache_key)
            .fetch_optional(&data.db_pool)
            .await;

    if let Ok(Some((body, updated_at))) = &cached_result
        && now - updated_at < ttl_seconds
    {
        return Ok(Bytes::from(body.clone()));
    }

    match update_upstream_cache(data, game, url_path).await {
        Ok(body) => Ok(body),
        Err(e) => {
            // Fallback to expired cache on error
            if let Ok(Some((body, _))) = cached_result {
                return Ok(Bytes::from(body));
            }
            Err(e)
        }
    }
}

pub async fn forward_request(data: &AppState, game: GameType, url_path: &str) -> HttpResponse {
    forward_request_with_filter(data, game, url_path, None, &[]).await
}

pub async fn forward_request_with_filter(
    data: &AppState,
    game: GameType,
    url_path: &str,
    user_role: Option<&str>,
    exempt_tags: &[String],
) -> HttpResponse {
    let prefix = get_cache_prefix(game);
    // Map /members-lite request to /members cache key
    let stripped_path = url_path.replace("/members-lite", "/members");
    let cache_key = format!("{}:upstream:{}", prefix, stripped_path);

    // Serve from cache ONLY
    let result =
        sqlx::query_as::<_, (Vec<u8>, i32)>("SELECT body, status FROM cache WHERE key = $1")
            .bind(&cache_key)
            .fetch_optional(&data.db_pool)
            .await;

    match result {
        Ok(Some((body, status))) => {
            let mut body = Bytes::from(body);

            let parts: Vec<&str> = url_path.split('/').collect();
            let is_clan_path = url_path == "/api/clans"
                || (parts.len() == 4 && parts[1] == "api" && parts[2] == "clans");
            let is_member_path = (parts.len() == 5
                && parts[1] == "api"
                && parts[2] == "clans"
                && (parts[4] == "members"
                    || parts[4] == "war-members"
                    || parts[4] == "raid-members"
                    || parts[4] == "cwl-members"
                    || parts[4] == "members-lite"))
                || (parts.len() == 4 && parts[1] == "api" && parts[2] == "players");

            if is_clan_path {
                body = filter_clan_data(
                    body,
                    game,
                    !crate::auth::has_required_role(user_role, "MEMBER"),
                );
                // Enrich clan list with badge data from Supercell API cache
                body = enrich_clan_badges(body, &data.db_pool, game).await;
            } else if is_member_path {
                body = filter_member_data(body, exempt_tags, user_role);
            }

            let status = StatusCode::from_u16(status as u16).unwrap_or(StatusCode::OK);
            HttpResponse::build(status)
                .content_type("application/json")
                .body(body)
        }
        Ok(None) => HttpResponse::ServiceUnavailable().json(ErrorResponse {
            error: "Data not yet available in cache. Background refresh is in progress.".into(),
        }),
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Internal Database Error".into(),
            })
        }
    }
}

// Update Supercell API cache
pub async fn update_supercell_cache(
    data: &AppState,
    game: GameType,
    url_path: &str,
) -> Result<Bytes, String> {
    let prefix = get_cache_prefix(game);
    let api_url = get_supercell_api_url(game);
    let token = get_supercell_token(data, game);
    let full_url = format!("{}{}", api_url, url_path);

    match data
        .client
        .get(&full_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status().as_u16();
            let body = res.bytes().await.map_err(|e| e.to_string())?;

            if status == 200 {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let cache_key = format!("{}:supercell:{}", prefix, url_path);

                let _ = sqlx::query(
                    "INSERT INTO cache (key, body, status, updated_at) 
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (key) DO UPDATE SET 
                    body = EXCLUDED.body, 
                    status = EXCLUDED.status, 
                    updated_at = EXCLUDED.updated_at",
                )
                .bind(&cache_key)
                .bind(body.to_vec())
                .bind(status as i32)
                .bind(timestamp)
                .execute(&data.db_pool)
                .await;

                Ok(body)
            } else {
                let err_msg = format!("Supercell {} returned status {}", full_url, status);
                eprintln!("Background Refresh: {}", err_msg);
                Err(err_msg)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
