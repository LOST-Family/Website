//! Dashboard fuer das Ticketsystem.
//!
//! Reicht Anfragen an den LOST Ticket-Bot durch. Der Bot entscheidet, WAS der
//! Anfragende sehen darf — diese Datei entscheidet nur, WER er ist.
//!
//! Das ist die ganze Sicherheitsannahme dieses Moduls und der Grund, warum die
//! Discord-ID hier aus dem verifizierten JWT kommt und niemals aus einem
//! Header oder Query-Parameter des Browsers: sonst koennte sich jeder
//! eingeloggte Nutzer als die Orga ausgeben und die Beschwerden ueber die
//! Anfuehrer mitlesen.
//!
//! Anders als die Clan-Endpunkte wird hier NICHT gecacht. Die Antwort haengt am
//! Fragenden; ein gemeinsamer Cache wuerde die Antwort fuer die Orga an den
//! naechsten Bewerber ausliefern.

use crate::auth::AuthenticatedUser;
use crate::models::{AppState, ErrorResponse};
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use log::error;

/// Baut die Zieladresse und holt die Antwort (nur lesend).
async fn durchreichen(
    data: &AppState,
    user: &AuthenticatedUser,
    pfad: &str,
    query: &str,
) -> HttpResponse {
    weiterleiten(data, user, pfad, query, None, false).await
}

/// Der eigentliche Weg zum Ticket-Bot, lesend (GET) oder schreibend (POST).
///
/// `admin_mitsenden` nur für die Verwaltung: dort zählt ein Website-Admin wie
/// jemand mit „Server verwalten“. Bei den Ticket-Lesepfaden geht die Kopfzeile
/// bewusst NICHT mit — wer welche Tickets lesen darf, entscheidet allein der
/// Bot anhand der Discord-Rollen (Orga-Tickets).
async fn weiterleiten(
    data: &AppState,
    user: &AuthenticatedUser,
    pfad: &str,
    query: &str,
    koerper: Option<String>,
    admin_mitsenden: bool,
) -> HttpResponse {
    let (basis, token) = match (&data.upstream_ticket_url, &data.ticket_api_token) {
        (Some(b), Some(t)) if !b.is_empty() => (b, t),
        _ => {
            return HttpResponse::ServiceUnavailable().json(ErrorResponse {
                error: "Ticket-Dashboard ist auf diesem Server nicht eingerichtet".into(),
            });
        }
    };

    let url = ziel_url(basis, pfad, query);

    let anfrage = match &koerper {
        Some(k) => data
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(k.clone()),
        None => data.client.get(&url),
    };
    let mut anfrage = anfrage
        .header("Authorization", format!("Bearer {}", token))
        // Die einzige Stelle, an der die Identitaet gesetzt wird — aus den
        // signierten Claims, nicht aus der Anfrage des Browsers.
        .header("X-Discord-User", user.claims.sub.clone());
    if admin_mitsenden && user.is_admin {
        anfrage = anfrage.header("X-Website-Admin", "1");
    }
    let antwort = anfrage.send().await;

    match antwort {
        Ok(res) => {
            let status = actix_web::http::StatusCode::from_u16(res.status().as_u16())
                .unwrap_or(actix_web::http::StatusCode::BAD_GATEWAY);
            match res.bytes().await {
                Ok(body) => HttpResponse::build(status)
                    .content_type("application/json; charset=utf-8")
                    .body(body),
                Err(e) => {
                    error!("Ticket-API: Antwort nicht lesbar: {:?}", e);
                    HttpResponse::BadGateway().json(ErrorResponse {
                        error: "Ticket-Bot antwortet nicht verwertbar".into(),
                    })
                }
            }
        }
        Err(e) => {
            error!("Ticket-API nicht erreichbar ({}): {:?}", url, e);
            HttpResponse::BadGateway().json(ErrorResponse {
                error: "Ticket-Bot ist nicht erreichbar".into(),
            })
        }
    }
}

/// Setzt die Zieladresse zusammen.
///
/// Eigene Funktion, weil sie die einzige Stelle ist, an der etwas vom Browser
/// Kommendes in die Adresse einfliesst — und damit die einzige, die sich ohne
/// laufenden Bot pruefen laesst.
fn ziel_url(basis: &str, pfad: &str, query: &str) -> String {
    let basis = basis.trim_end_matches('/');
    if query.is_empty() {
        format!("{}{}", basis, pfad)
    } else {
        format!("{}{}?{}", basis, pfad, query)
    }
}

pub async fn get_ticket_guilds(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    durchreichen(&data, &user, "/api/guilds", "").await
}

pub async fn get_ticket_stats(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
    req: HttpRequest,
) -> impl Responder {
    durchreichen(&data, &user, "/api/stats", req.query_string()).await
}

pub async fn get_ticket_panels(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
    req: HttpRequest,
) -> impl Responder {
    durchreichen(&data, &user, "/api/panels", req.query_string()).await
}

pub async fn get_ticket_list(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
    req: HttpRequest,
) -> impl Responder {
    durchreichen(&data, &user, "/api/tickets", req.query_string()).await
}

pub async fn get_ticket_detail(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<i64>,
    req: HttpRequest,
) -> impl Responder {
    durchreichen(
        &data,
        &user,
        &format!("/api/tickets/{}", id.into_inner()),
        req.query_string(),
    )
    .await
}

pub async fn get_ticket_legacy(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
    req: HttpRequest,
) -> impl Responder {
    durchreichen(&data, &user, "/api/legacy", req.query_string()).await
}

// ---------------------------------------------------------------------------
// Verwaltung: Panels, Menüs, Rollen, Fragen
// ---------------------------------------------------------------------------

/// Schreibaktionen, die über die Website erreichbar sind — eine Positivliste.
/// Was hier nicht steht, kommt gar nicht erst beim Bot an.
const VERWALTUNG_AKTIONEN: &[&str] = &[
    "panel/erstellen",
    "panel/setzen",
    "panel/rolle",
    "panel/fragen",
    "menu/erstellen",
    "menu/text",
    "menu/stil",
    "menu/kanal",
    "menu/eintrag",
    "menu/eintrag-entfernen",
    "menu/reihenfolge",
    "menu/posten",
    "menu/loeschen",
    "menus/abgleichen",
    "config/limit",
    "sperre/hinzufuegen",
    "sperre/entfernen",
];

fn verwaltung_erlaubt(aktion: &str) -> bool {
    VERWALTUNG_AKTIONEN.contains(&aktion)
}

pub async fn get_verwaltung_guilds(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    weiterleiten(&data, &user, "/api/verwaltung/guilds", "", None, true).await
}

pub async fn get_verwaltung(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
    req: HttpRequest,
) -> impl Responder {
    weiterleiten(&data, &user, "/api/verwaltung", req.query_string(), None, true).await
}

/// Eine Änderung an den Bot weiterreichen. Wer handelt, setzt der Bot aus
/// `X-Discord-User` — ein Feld im Körper hat dafür keine Bedeutung. Das
/// Protokoll in `💻┋ticket-bot-log` schreibt der Bot selbst.
pub async fn post_verwaltung(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
    aktion: web::Path<String>,
    koerper: web::Json<serde_json::Value>,
) -> impl Responder {
    let aktion = aktion.into_inner();
    if !verwaltung_erlaubt(&aktion) {
        return HttpResponse::NotFound().json(ErrorResponse {
            error: "Diese Aktion gibt es nicht".into(),
        });
    }
    if !koerper.is_object() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Der Anfragekörper muss ein JSON-Objekt sein".into(),
        });
    }
    log::info!("Ticket-Verwaltung: {} → {}", user.claims.sub, aktion);
    weiterleiten(
        &data,
        &user,
        &format!("/api/verwaltung/{}", aktion),
        "",
        Some(koerper.into_inner().to_string()),
        true,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::{verwaltung_erlaubt, ziel_url};

    #[test]
    fn verwaltung_positivliste() {
        assert!(verwaltung_erlaubt("panel/setzen"));
        assert!(verwaltung_erlaubt("menu/posten"));
        assert!(!verwaltung_erlaubt("panel/loeschen"));
        assert!(!verwaltung_erlaubt("../tickets"));
        assert!(!verwaltung_erlaubt("guilds"));
        assert!(!verwaltung_erlaubt(""));
    }

    #[test]
    fn schraegstrich_am_ende_verdoppelt_sich_nicht() {
        assert_eq!(
            ziel_url("http://127.0.0.1:7099/", "/api/stats", ""),
            "http://127.0.0.1:7099/api/stats"
        );
        assert_eq!(
            ziel_url("http://127.0.0.1:7099", "/api/stats", ""),
            "http://127.0.0.1:7099/api/stats"
        );
    }

    #[test]
    fn query_wird_durchgereicht() {
        assert_eq!(
            ziel_url("http://b", "/api/tickets", "guild=1&status=open"),
            "http://b/api/tickets?guild=1&status=open"
        );
    }

    #[test]
    fn leere_query_haengt_kein_fragezeichen_an() {
        assert!(!ziel_url("http://b", "/api/guilds", "").contains('?'));
    }
}
