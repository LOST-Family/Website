//! CWL-Tabelle einer Liga-Gruppe.
//!
//! Die Antwort von `/clans/{tag}/currentwar/leaguegroup` nennt je Clan nur
//! Tag, Name, Level, Abzeichen und Mitglieder – **keine Sterne**, und die
//! Reihenfolge der Liste ist nicht die Platzierung (am 18.11.2026 an einer
//! echten Gruppe nachgerechnet: Listenplatz 3 war in Wahrheit Platz 5). Die
//! Platzierung ergibt sich erst aus den einzelnen Kriegen der Runden.

use serde_json::{Value, json};

/// Supercell liefert die Saison inzwischen als `2026-11-01`, früher als
/// `2026-11`. Gespeichert und verglichen wird immer der Monat.
pub fn saison_monat(saison: &str) -> String {
    let b = saison.as_bytes();
    if b.len() >= 7
        && b[4] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
    {
        saison[..7].to_string()
    } else {
        saison.to_string()
    }
}

/// Alle Kriegs-Tags der Gruppe; `#0` steht für noch nicht angesetzte Runden.
pub fn kriegs_tags(gruppe: &Value) -> Vec<String> {
    gruppe
        .get("rounds")
        .and_then(Value::as_array)
        .map(|runden| {
            runden
                .iter()
                .filter_map(|r| r.get("warTags").and_then(Value::as_array))
                .flatten()
                .filter_map(Value::as_str)
                .filter(|t| *t != "#0")
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Nur das, was die Tabelle braucht – ein ganzer Krieg ist über 100 kB groß.
pub fn krieg_kurz(krieg: &Value) -> Value {
    let seite = |name: &str| {
        let s = krieg.get(name);
        json!({
            "tag": s.and_then(|s| s.get("tag")).cloned().unwrap_or(Value::Null),
            "stars": s.and_then(|s| s.get("stars")).cloned().unwrap_or(Value::Null),
            "destructionPercentage": s
                .and_then(|s| s.get("destructionPercentage"))
                .cloned()
                .unwrap_or(Value::Null),
        })
    };
    json!({
        "state": krieg.get("state").cloned().unwrap_or(Value::Null),
        "clan": seite("clan"),
        "opponent": seite("opponent"),
    })
}

pub fn ist_beendet(krieg: &Value) -> bool {
    krieg.get("state").and_then(Value::as_str) == Some("warEnded")
}

/// Platzierung der Gruppe, bester Clan zuerst. Sterne aus laufenden und
/// beendeten Kriegen, zehn Bonussterne je gewonnenem (beendetem) Krieg, bei
/// Gleichstand entscheidet die Zerstörung. Leer, solange noch kein Krieg
/// begonnen hat – dann gibt es keine Platzierung.
pub fn tabelle(gruppe: &Value, kriege: &[Value]) -> Vec<String> {
    let mut stand: Vec<(String, i64, f64)> = gruppe
        .get("clans")
        .and_then(Value::as_array)
        .map(|clans| {
            clans
                .iter()
                .filter_map(|c| c.get("tag").and_then(Value::as_str))
                .map(|t| (t.to_string(), 0, 0.0))
                .collect()
        })
        .unwrap_or_default();

    let werte = |seite: &Value| -> Option<(String, i64, f64)> {
        Some((
            seite.get("tag")?.as_str()?.to_string(),
            seite.get("stars").and_then(Value::as_i64).unwrap_or(0),
            seite
                .get("destructionPercentage")
                .and_then(Value::as_f64)
                .unwrap_or(0.0),
        ))
    };

    let mut gezaehlt = 0;
    for krieg in kriege {
        let zustand = krieg.get("state").and_then(Value::as_str).unwrap_or("");
        if zustand != "inWar" && zustand != "warEnded" {
            continue;
        }
        let (Some(a), Some(b)) = (
            krieg.get("clan").and_then(&werte),
            krieg.get("opponent").and_then(&werte),
        ) else {
            continue;
        };
        gezaehlt += 1;

        let (mut bonus_a, mut bonus_b) = (0, 0);
        if zustand == "warEnded" {
            if a.1 > b.1 || (a.1 == b.1 && a.2 > b.2) {
                bonus_a = 10;
            } else if b.1 > a.1 || (a.1 == b.1 && b.2 > a.2) {
                bonus_b = 10;
            }
        }
        for (seite, bonus) in [(&a, bonus_a), (&b, bonus_b)] {
            if let Some(e) = stand.iter_mut().find(|e| e.0.eq_ignore_ascii_case(&seite.0)) {
                e.1 += seite.1 + bonus;
                e.2 += seite.2;
            }
        }
    }

    if gezaehlt == 0 {
        return vec![];
    }
    stand.sort_by(|x, y| {
        y.1.cmp(&x.1)
            .then_with(|| y.2.partial_cmp(&x.2).unwrap_or(std::cmp::Ordering::Equal))
    });
    stand.into_iter().map(|e| e.0).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn krieg(zustand: &str, a: (&str, i64, f64), b: (&str, i64, f64)) -> Value {
        json!({
            "state": zustand,
            "teamSize": 15,
            "clan": {"tag": a.0, "stars": a.1, "destructionPercentage": a.2, "members": []},
            "opponent": {"tag": b.0, "stars": b.1, "destructionPercentage": b.2, "members": []},
        })
    }

    fn gruppe() -> Value {
        json!({
            "state": "ended",
            "season": "2026-11-01",
            "clans": [{"tag": "#A"}, {"tag": "#B"}, {"tag": "#C"}, {"tag": "#D"}],
            "rounds": [
                {"warTags": ["#W1", "#W2"]},
                {"warTags": ["#W3", "#0"]},
                {"warTags": ["#0", "#0"]},
            ],
        })
    }

    #[test]
    fn saison_wird_auf_den_monat_gebracht() {
        assert_eq!(saison_monat("2026-11-01"), "2026-11");
        assert_eq!(saison_monat("2026-11"), "2026-11");
        assert_eq!(saison_monat("unsinn"), "unsinn");
        assert_eq!(saison_monat(""), "");
    }

    #[test]
    fn kriegs_tags_ohne_platzhalter() {
        assert_eq!(kriegs_tags(&gruppe()), vec!["#W1", "#W2", "#W3"]);
        assert!(kriegs_tags(&json!({})).is_empty());
    }

    #[test]
    fn listenreihenfolge_ist_nicht_die_platzierung() {
        // #A steht in der Liste vorn, hat aber verloren.
        let kriege = vec![
            krieg("warEnded", ("#A", 30, 80.0), ("#D", 40, 95.0)),
            krieg("warEnded", ("#B", 35, 90.0), ("#C", 35, 85.0)),
        ];
        // #D 40+10, #B 35+10 (Gleichstand bei Sternen, mehr Zerstörung), #C 35, #A 30
        assert_eq!(tabelle(&gruppe(), &kriege), vec!["#D", "#B", "#C", "#A"]);
    }

    #[test]
    fn laufender_krieg_zaehlt_sterne_aber_keinen_bonus() {
        let kriege = vec![
            krieg("warEnded", ("#A", 30, 80.0), ("#D", 31, 95.0)),
            krieg("inWar", ("#B", 38, 90.0), ("#C", 12, 40.0)),
            krieg("preparation", ("#A", 0, 0.0), ("#B", 0, 0.0)),
        ];
        // #D 41, #B 38 (kein Bonus, läuft noch), #A 30, #C 12
        assert_eq!(tabelle(&gruppe(), &kriege), vec!["#D", "#B", "#A", "#C"]);
    }

    #[test]
    fn unentschieden_gibt_keinen_bonus_und_gleichstand_entscheidet_zerstoerung() {
        let kriege = vec![
            krieg("warEnded", ("#A", 30, 80.0), ("#B", 30, 80.0)),
            krieg("warEnded", ("#C", 20, 70.0), ("#D", 19, 99.0)),
        ];
        // #A 30 / #B 30 / #C 30 (20+10) / #D 19 – Zerstörung: A=B=80 vor C=70
        let t = tabelle(&gruppe(), &kriege);
        assert_eq!(t[2], "#C");
        assert_eq!(t[3], "#D");
    }

    #[test]
    fn ohne_begonnenen_krieg_keine_platzierung() {
        let kriege = vec![krieg("preparation", ("#A", 0, 0.0), ("#B", 0, 0.0))];
        assert!(tabelle(&gruppe(), &kriege).is_empty());
        assert!(tabelle(&gruppe(), &[]).is_empty());
    }

    #[test]
    fn kurzfassung_reicht_fuer_die_tabelle() {
        let voll = vec![
            krieg("warEnded", ("#A", 30, 80.0), ("#D", 40, 95.0)),
            krieg("warEnded", ("#B", 35, 90.0), ("#C", 35, 85.0)),
        ];
        let kurz: Vec<Value> = voll.iter().map(krieg_kurz).collect();
        assert!(kurz[0].get("teamSize").is_none());
        assert!(kurz[0]["clan"].get("members").is_none());
        assert!(ist_beendet(&kurz[0]));
        assert_eq!(tabelle(&gruppe(), &kurz), tabelle(&gruppe(), &voll));
    }
}
