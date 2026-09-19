//! Der Schreibweg der Website — eine einzige Weiterleitung an die Bots.
//!
//! Die Website entscheidet hier **nichts**. Sie sagt dem Bot nur, *wer* gerade
//! handelt, und der Bot prüft dessen Rang in genau dem betroffenen Clan gegen
//! die echten Discord-Rollen. Das ist dieselbe Linie wie beim Ticket-Bot:
//! Sichtbarkeit und Berechtigung entscheidet der Bot, nicht die Oberfläche.
//! Die Rollenprüfung im Frontend bleibt Bequemlichkeit (Knöpfe ausgrauen) und
//! ist ausdrücklich keine Sicherheitsgrenze.
//!
//! Warum eine gemeinsame Route statt einer je Aktion: so gibt es **eine**
//! Stelle, an der der Schreibweg geprüft werden kann, statt fünfzehn. Eine
//! neue Bot-Aktion ist danach reine Frontend-Arbeit.

use actix_web::{HttpResponse, Responder, web};
use log::{error, info, warn};
use serde_json::{Value, json};

use crate::auth::AuthenticatedUser;
use crate::models::{AppState, GameType};

/// Aktionen, die über die Website ausgelöst werden dürfen — eine Positivliste,
/// keine Sperrliste. Was hier nicht steht, ist nicht erreichbar, auch wenn der
/// Bot es anbietet.
///
/// Bewusst **nicht** dabei:
/// * `restart` — ein Neustartknopf im Browser ist eine schlechte Idee, und beim
///   LOST Manager sogar eine gefährliche: er hält fällige Listening-Events im
///   Speicher. Neustarts laufen über `lostmanager-neustart.sh`.
/// * `verify` — der Spiel-API-Token, den Supercell dafür ausgibt, hat im
///   Browser nichts verloren.
const ERLAUBTE_AKTIONEN: &[&str] = &[
    "members/add",
    "members/edit",
    "members/remove",
    "members/transfer",
    "kickpoints/add",
    "kickpoints/edit",
    "kickpoints/remove",
    "kickpoint-reasons/add",
    "kickpoint-reasons/edit",
    "kickpoint-reasons/remove",
    "clanconfig",
    "links/link",
    "links/relink",
    "links/unlink",
    "copyreasons",
];

/// Felder, die im Protokoll nie auftauchen. `discordUserId` steht dort ohnehin
/// schon als „wer", und ein Token hat in einer Discord-Nachricht nichts zu
/// suchen — auch wenn heute keiner im Körper vorkommt.
const NICHT_PROTOKOLLIEREN: &[&str] = &["discordUserId", "token", "apiToken", "password"];

fn spiel_aus_pfad(kuerzel: &str) -> Option<GameType> {
    match kuerzel {
        "coc" => Some(GameType::ClashOfClans),
        "cr" => Some(GameType::ClashRoyale),
        "bs" => Some(GameType::BrawlStars),
        _ => None,
    }
}

fn spiel_name(game: GameType) -> &'static str {
    match game {
        GameType::ClashOfClans => "Clash of Clans",
        GameType::ClashRoyale => "Clash Royale",
        GameType::BrawlStars => "Brawl Stars",
    }
}

/// Brawl Stars nennt einen Clan „Club" — im Bot wie in der Supercell-API.
/// Innerhalb der Website bleibt es durchgängig bei „clan"; übersetzt wird
/// ausschließlich die Adresse, die tatsächlich hinausgeht. Dieselbe Regel wie
/// in `utils::aussen_pfad`.
fn aussen_aktion(game: GameType, aktion: &str) -> String {
    match (game, aktion) {
        (GameType::BrawlStars, "clanconfig") => "clubconfig".to_string(),
        _ => aktion.to_string(),
    }
}

pub async fn manage_weiterleitung(
    data: web::Data<AppState>,
    pfad: web::Path<(String, String)>,
    koerper: web::Json<Value>,
    user: AuthenticatedUser,
) -> impl Responder {
    let (spiel_kuerzel, aktion) = pfad.into_inner();

    let Some(game) = spiel_aus_pfad(&spiel_kuerzel) else {
        return HttpResponse::NotFound().json(json!({ "error": "Unbekanntes Spiel" }));
    };

    // Die Positivliste. Der Vergleich läuft gegen die Aktion, wie sie innerhalb
    // der Website heißt — die Übersetzung für Brawl Stars kommt erst danach,
    // sonst müsste die Liste jede Schreibweise doppelt führen.
    if !ERLAUBTE_AKTIONEN.contains(&aktion.as_str()) {
        warn!(
            "Schreibweg: {} wollte die nicht freigegebene Aktion {}/{} auslösen",
            user.claims.sub, spiel_kuerzel, aktion
        );
        return HttpResponse::NotFound().json(json!({
            "error": "Diese Aktion ist über die Website nicht verfügbar"
        }));
    }

    let (upstream, token) = match game {
        GameType::ClashOfClans => (data.upstream_coc_url.clone(), data.coc_api_token.clone()),
        GameType::ClashRoyale => (data.upstream_cr_url.clone(), data.cr_api_token.clone()),
        GameType::BrawlStars => {
            match (data.upstream_bs_url.clone(), data.bs_api_token.clone()) {
                (Some(u), Some(t)) => (u, t),
                _ => {
                    return HttpResponse::ServiceUnavailable()
                        .json(json!({ "error": "Brawl-Stars-Bot ist nicht eingerichtet" }));
                }
            }
        }
    };

    // Der Körper muss ein Objekt sein — sonst gäbe es kein Feld, in das
    // discordUserId geschrieben werden könnte.
    let mut inhalt = match koerper.into_inner() {
        Value::Object(map) => map,
        _ => {
            return HttpResponse::BadRequest()
                .json(json!({ "error": "Der Anfragekörper muss ein JSON-Objekt sein" }));
        }
    };

    // **Der entscheidende Handgriff.** Wer handelt, steht im eigenen Anmelde-
    // token und nirgends sonst. Ein mitgeschicktes Feld gleichen Namens wird
    // überschrieben — sonst könnte jeder Angemeldete im Namen eines anderen
    // handeln, und der Bot würde brav *dessen* Rechte prüfen und die Aktion
    // durchwinken.
    inhalt.insert(
        "discordUserId".to_string(),
        Value::String(user.claims.sub.clone()),
    );

    let ziel = format!(
        "{}/api/manage/{}",
        upstream.trim_end_matches('/'),
        aussen_aktion(game, &aktion)
    );

    // Der Körper geht als Zeichenkette hinaus: `data.client` ist der
    // reqwest-Client aus oauth2 und bringt kein `json`-Feature mit.
    let gesendet = Value::Object(inhalt.clone()).to_string();
    let antwort = data
        .client
        .post(&ziel)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(gesendet)
        .send()
        .await;

    let (status, rumpf) = match antwort {
        Ok(r) => {
            let code = r.status();
            let text = r.text().await.unwrap_or_default();
            (code, text)
        }
        Err(e) => {
            error!("Schreibweg: {} nicht erreichbar: {}", ziel, e);
            protokolliere(
                &data,
                &user.claims.sub,
                game,
                &aktion,
                &inhalt,
                None,
                Some(&e.to_string()),
            )
            .await;
            return HttpResponse::BadGateway()
                .json(json!({ "error": "Der Bot ist gerade nicht erreichbar" }));
        }
    };

    info!(
        "Schreibweg: {} → {}/{} ⇒ {}",
        user.claims.sub, spiel_kuerzel, aktion, status
    );
    protokolliere(
        &data,
        &user.claims.sub,
        game,
        &aktion,
        &inhalt,
        Some(status.as_u16()),
        None,
    )
    .await;

    // Antwort samt Statuscode unverändert durchreichen — auch die 403 des Bots.
    // Seine Begründung ist die verbindliche, sie soll den Nutzer erreichen.
    let mut aus = HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status.as_u16())
            .unwrap_or(actix_web::http::StatusCode::BAD_GATEWAY),
    );
    aus.content_type("application/json");
    aus.body(rumpf)
}

/// Jede Schreibaktion wandert in einen Discord-Kanal: wer, wann, was, Ergebnis.
/// Im Discord sieht man an den Befehlen, wer was getan hat; über die Website
/// wäre das sonst unsichtbar.
///
/// Geschrieben wird über einen **Webhook**, nicht mit einem Bot-Token. Ein
/// Webhook kann nur in diesen einen Kanal schreiben — ein Bot-Token im
/// Website-Backend könnte alles. Fehlt die Konfiguration, läuft die Aktion
/// trotzdem durch und nur das Protokoll fällt aus; ein nicht erreichbarer
/// Log-Kanal darf keine Verwaltungsaktion verhindern.
async fn protokolliere(
    data: &AppState,
    discord_id: &str,
    game: GameType,
    aktion: &str,
    inhalt: &serde_json::Map<String, Value>,
    status: Option<u16>,
    fehler: Option<&str>,
) {
    let Some(webhook) = data.log_webhook.as_ref() else {
        return;
    };

    let felder: Vec<String> = inhalt
        .iter()
        .filter(|(k, _)| !NICHT_PROTOKOLLIEREN.contains(&k.as_str()))
        .map(|(k, v)| format!("{k}: {}", kurz(v)))
        .collect();

    let (farbe, ergebnis) = match (status, fehler) {
        (_, Some(e)) => (0xE74C3C, format!("Bot nicht erreichbar — {e}")),
        (Some(s), _) if (200..300).contains(&s) => (0x2ECC71, format!("{s} erledigt")),
        (Some(403), _) => (0xE67E22, "403 — der Bot hat die Rechte verweigert".to_string()),
        (Some(s), _) => (0xE74C3C, format!("{s} abgelehnt")),
        (None, None) => (0x95A5A6, "unbekannt".to_string()),
    };

    let rumpf = json!({
        // Niemand wird angepingt — dieselbe Regel wie beim Ticket-Bot. Die
        // Discord-ID steht im Klartext, damit der Kanal ohne Datenbank lesbar
        // bleibt, und würde sonst bei jeder Aktion benachrichtigen.
        "allowed_mentions": { "parse": [] },
        "embeds": [{
            "title": format!("{} · {}", spiel_name(game), aktion),
            "color": farbe,
            "fields": [
                { "name": "Wer",      "value": format!("<@{discord_id}> ({discord_id})"), "inline": false },
                { "name": "Ergebnis", "value": ergebnis, "inline": false },
                { "name": "Angaben",  "value": if felder.is_empty() {
                        "—".to_string()
                    } else {
                        let t = felder.join("\n");
                        // Discord nimmt höchstens 1024 Zeichen je Feld.
                        if t.len() > 1000 { format!("{}…", &t[..1000]) } else { t }
                    }, "inline": false }
            ]
        }]
    });

    let ergebnis = data
        .client
        .post(webhook)
        .header("Content-Type", "application/json")
        .body(rumpf.to_string())
        .send()
        .await;
    if let Err(e) = ergebnis {
        // Nur vermerken, nicht scheitern lassen.
        warn!("Schreibweg: Protokoll nach Discord fehlgeschlagen: {e}");
    }
}

/// Lange Werte im Protokoll kürzen, damit ein versehentlich mitgeschickter
/// Klotz die Nachricht nicht sprengt.
fn kurz(v: &Value) -> String {
    let s = match v {
        Value::String(s) => s.clone(),
        andere => andere.to_string(),
    };
    if s.chars().count() > 120 {
        format!("{}…", s.chars().take(120).collect::<String>())
    } else {
        s
    }
}
