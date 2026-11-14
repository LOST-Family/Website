<script lang="ts">
    /**
     * Verwaltung der Ticket-Panels, Menüs und der Sperrliste eines Servers.
     *
     * Jede Änderung geht als einzelne Aktion an den Bot; der prüft Rechte und
     * Werte selbst und formuliert Fehler für Menschen — die werden hier
     * unverändert angezeigt. Nach jeder erfolgreichen Aktion wird die Übersicht
     * neu geladen, damit nie ein lokal geratener Stand stehen bleibt.
     *
     * Menüs ändern sich in Discord erst durch „Posten/Abgleichen“. Welche
     * gepostete Nachricht seit dem letzten Abgleich verändert wurde, merkt sich
     * diese Komponente im localStorage — der Bot weiß das nicht.
     */
    import {
        ladeVerwaltung,
        verwaltungAktion,
        farbeHex,
        zeit,
        type VerwaltungUebersicht,
        type VwPanel,
        type VwMenu,
        type VwMenuEintrag,
        type VwKanal,
        type VwRolle,
    } from './tickets';

    export let apiBaseUrl: string;
    export let guildId: string;
    export let theme: 'dark' | 'light' = 'dark';

    // -----------------------------------------------------------------------
    // Typen der Entwürfe
    // -----------------------------------------------------------------------

    interface PanelEntwurf {
        AKTIV: boolean;
        NAME_OFFEN: string;
        NAME_GESCHLOSSEN: string;
        ZAEHLER: number | null;
        STELLEN: number | null;
        MAX_OFFEN: number | null;
        COOLDOWN: number | null;
        KATEGORIE_OFFEN: string;
        KATEGORIE_GESCHLOSSEN: string;
        LOG_KANAL: string;
        WILLKOMMENSTEXT: string;
        EMBED_TITEL: string;
        EMBED_TEXT: string;
        SCHLIESSEN_WENN_WEG: boolean;
        WEGGEGANGEN_TEXT: string;
        DM_BEIM_OEFFNEN: boolean;
        DM_BEIM_SCHLIESSEN: boolean;
    }
    type Feld = keyof PanelEntwurf;
    type Abschnitt = 'allgemein' | 'kanaele' | 'begruessung' | 'verhalten';

    const ABSCHNITTE: Record<Abschnitt, Feld[]> = {
        allgemein: [
            'AKTIV',
            'NAME_OFFEN',
            'NAME_GESCHLOSSEN',
            'ZAEHLER',
            'STELLEN',
            'MAX_OFFEN',
            'COOLDOWN',
        ],
        kanaele: ['KATEGORIE_OFFEN', 'KATEGORIE_GESCHLOSSEN', 'LOG_KANAL'],
        begruessung: ['WILLKOMMENSTEXT', 'EMBED_TITEL', 'EMBED_TEXT'],
        verhalten: [
            'SCHLIESSEN_WENN_WEG',
            'WEGGEGANGEN_TEXT',
            'DM_BEIM_OEFFNEN',
            'DM_BEIM_SCHLIESSEN',
        ],
    };
    const ZAHLFELDER: Feld[] = ['ZAEHLER', 'STELLEN', 'MAX_OFFEN', 'COOLDOWN'];

    interface FrageEntwurf {
        label: string;
        placeholder: string;
        lang: boolean;
        pflicht: boolean;
        max: number | null;
    }

    interface MenuTextEntwurf {
        titel: string;
        text: string;
        platzhalter: string;
        eigeneFarbe: boolean;
        farbe: string;
    }

    interface EintragEntwurf {
        label: string;
        emoji: string;
        beschreibung: string;
        knopfstil: number;
    }

    const KNOPFSTILE: { wert: number; name: string }[] = [
        { wert: 1, name: 'Blau' },
        { wert: 2, name: 'Grau' },
        { wert: 3, name: 'Grün' },
        { wert: 4, name: 'Rot' },
    ];
    const KNOPFFARBEN: Record<number, string> = {
        1: '#5865f2',
        2: '#4e5058',
        3: '#248046',
        4: '#da373c',
    };

    // -----------------------------------------------------------------------
    // Zustand
    // -----------------------------------------------------------------------

    let daten: VerwaltungUebersicht | null = null;
    let laedt = true;
    let ladeFehler: string | null = null;

    let beschaeftigt = false;
    let meldung: string | null = null;
    let aktionsFehler: string | null = null;
    let meldungZeitgeber: ReturnType<typeof setTimeout> | null = null;

    let bereich: 'panels' | 'menues' | 'server' = 'panels';

    // Panel-Bearbeitung
    let panelId: number | null = null;
    let entwurf: PanelEntwurf | null = null;
    let original: PanelEntwurf | null = null;
    let fragen: FrageEntwurf[] = [];
    let fragenOriginal = '[]';
    let neueSupportRolle = '';
    let neuePingRolle = '';
    let neuesPanelName = '';
    let neuesPanelOffen = false;

    // Menü-Bearbeitung
    let menuId: number | null = null;
    let menuText: MenuTextEntwurf | null = null;
    let menuTextOriginal: MenuTextEntwurf | null = null;
    let eintraege: Record<number, EintragEntwurf> = {};
    let eintraegeOriginal: Record<number, EintragEntwurf> = {};
    let menuKanalNeu = '';
    let neuerEintragPanel = '';
    let neuesMenuOffen = false;
    let neuesMenuKanal = '';
    let neuesMenuStil: 'knoepfe' | 'dropdown' = 'knoepfe';

    // Server
    let limitEntwurf: number | null = null;
    let limitAlt: number | null = null;
    let sperreWer = '';
    let sperreRolle = false;
    let sperreGrund = '';

    // Gepostete Menüs, die seit dem letzten Posten geändert wurden.
    let ungeglichen: number[] = [];

    // -----------------------------------------------------------------------
    // Laden
    // -----------------------------------------------------------------------

    let geladenFuer: string | null = null;
    $: if (guildId && guildId !== geladenFuer) {
        geladenFuer = guildId;
        zuruecksetzen();
        laden(false);
    }

    function zuruecksetzen() {
        daten = null;
        bereich = 'panels';
        panelId = null;
        entwurf = null;
        original = null;
        menuId = null;
        menuText = null;
        menuTextOriginal = null;
        eintraege = {};
        eintraegeOriginal = {};
        limitEntwurf = null;
        meldung = null;
        aktionsFehler = null;
        neuesPanelOffen = false;
        neuesMenuOffen = false;
        ungeglichenLaden();
    }

    async function laden(still: boolean) {
        const fuer = guildId;
        if (!still) laedt = true;
        ladeFehler = null;
        try {
            const neu = await ladeVerwaltung(apiBaseUrl, fuer);
            if (fuer !== guildId) return;
            daten = neu;
            nachLaden();
        } catch (e) {
            if (fuer !== guildId) return;
            ladeFehler = e instanceof Error ? e.message : 'Unbekannter Fehler';
        } finally {
            if (fuer === guildId) laedt = false;
        }
    }

    /**
     * Entwürfe an den neuen Stand anpassen: Was der Nutzer nicht angefasst
     * hat, kommt frisch vom Server; was er gerade bearbeitet, bleibt stehen.
     */
    function uebernehmen<T extends object>(
        aktuell: T | null,
        alt: T | null,
        neu: T,
    ): T {
        if (!aktuell || !alt) return { ...neu };
        const a = aktuell as Record<string, unknown>;
        const o = alt as Record<string, unknown>;
        const n = neu as Record<string, unknown>;
        const aus: Record<string, unknown> = {};
        for (const k of Object.keys(n)) {
            aus[k] = a[k] === o[k] ? n[k] : a[k];
        }
        return aus as T;
    }

    function nachLaden() {
        if (!daten) return;

        // Merker für Menüs, die es nicht mehr gibt, wegwerfen.
        const ids = new Set(daten.menus.map((m) => m.id));
        ungeglichenSetzen(ungeglichen.filter((id) => ids.has(id)));

        if (limitEntwurf === null || limitAlt === limitEntwurf) {
            limitEntwurf = daten.config.limit;
        }
        limitAlt = daten.config.limit;

        if (panelId !== null) {
            const p = daten.panels.find((x) => x.id === panelId);
            if (!p) {
                panelId = null;
                entwurf = null;
                original = null;
            } else {
                const neu = panelWerte(p);
                entwurf = uebernehmen(entwurf, original, neu);
                original = neu;
                const neueFragen = fragenAus(p);
                if (fragenText(fragen) === fragenOriginal) {
                    fragen = neueFragen;
                }
                fragenOriginal = fragenText(neueFragen);
            }
        }

        if (menuId !== null) {
            const m = daten.menus.find((x) => x.id === menuId);
            if (!m) {
                menuId = null;
                menuText = null;
                menuTextOriginal = null;
            } else {
                const neu = menuTextAus(m);
                menuText = uebernehmen(menuText, menuTextOriginal, neu);
                menuTextOriginal = neu;
                const neueEintraege: Record<number, EintragEntwurf> = {};
                for (const e of m.entries) {
                    const n = eintragAus(e);
                    neueEintraege[e.panel_id] = uebernehmen(
                        eintraege[e.panel_id] ?? null,
                        eintraegeOriginal[e.panel_id] ?? null,
                        n,
                    );
                }
                eintraege = neueEintraege;
                eintraegeOriginal = Object.fromEntries(
                    m.entries.map((e) => [e.panel_id, eintragAus(e)]),
                );
                menuKanalNeu = menuKanalNeu || m.channel_id;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Aktionen
    // -----------------------------------------------------------------------

    async function ausfuehren(
        aktion: string,
        koerper: Record<string, unknown>,
    ): Promise<boolean> {
        if (beschaeftigt) return false;
        beschaeftigt = true;
        aktionsFehler = null;
        meldung = null;
        try {
            const antwort = await verwaltungAktion(apiBaseUrl, aktion, {
                guild: guildId,
                ...koerper,
            });
            meldungZeigen(antwort.meldung);
            await laden(true);
            return true;
        } catch (e) {
            aktionsFehler =
                e instanceof Error ? e.message : 'Unbekannter Fehler';
            return false;
        } finally {
            beschaeftigt = false;
        }
    }

    function meldungZeigen(text: string) {
        meldung = text;
        if (meldungZeitgeber) clearTimeout(meldungZeitgeber);
        meldungZeitgeber = setTimeout(() => (meldung = null), 8000);
    }

    function lokalerFehler(text: string) {
        aktionsFehler = text;
        meldung = null;
    }

    // --- Merker für nicht abgeglichene Menüs --------------------------------

    function merkerSchluessel(): string {
        return `ticket-verwaltung-ungeglichen-${guildId}`;
    }

    function ungeglichenLaden() {
        try {
            const roh = localStorage.getItem(merkerSchluessel());
            const liste = roh ? JSON.parse(roh) : [];
            ungeglichen = Array.isArray(liste)
                ? liste.filter((x) => typeof x === 'number')
                : [];
        } catch {
            ungeglichen = [];
        }
    }

    function ungeglichenSetzen(ids: number[]) {
        ungeglichen = [...new Set(ids)];
        try {
            localStorage.setItem(merkerSchluessel(), JSON.stringify(ungeglichen));
        } catch {
            // Privater Modus o. ä. — dann gilt der Merker nur bis zum Neuladen.
        }
    }

    function menuMarkieren(id: number) {
        const m = daten?.menus.find((x) => x.id === id);
        if (m?.posted) ungeglichenSetzen([...ungeglichen, id]);
    }

    function menuesMitPanelMarkieren(pid: number) {
        const betroffen = (daten?.menus ?? [])
            .filter((m) => m.posted && m.entries.some((e) => e.panel_id === pid))
            .map((m) => m.id);
        ungeglichenSetzen([...ungeglichen, ...betroffen]);
    }

    function istUngeglichen(m: VwMenu, liste: number[]): boolean {
        return m.posted && liste.includes(m.id);
    }

    // -----------------------------------------------------------------------
    // Panels
    // -----------------------------------------------------------------------

    function panelWerte(p: VwPanel): PanelEntwurf {
        return {
            AKTIV: p.active,
            NAME_OFFEN: p.name_pattern_open ?? '',
            NAME_GESCHLOSSEN: p.name_pattern_closed ?? '',
            ZAEHLER: p.counter,
            STELLEN: p.counter_padding,
            MAX_OFFEN: p.max_open_per_user,
            COOLDOWN: p.cooldown_seconds,
            KATEGORIE_OFFEN: p.category_opened ?? '',
            KATEGORIE_GESCHLOSSEN: p.category_closed ?? '',
            LOG_KANAL: p.log_channel_id ?? '',
            WILLKOMMENSTEXT: p.welcome_text ?? '',
            EMBED_TITEL: p.welcome_embed_title ?? '',
            EMBED_TEXT: p.welcome_embed_text ?? '',
            SCHLIESSEN_WENN_WEG: p.close_on_owner_leave,
            WEGGEGANGEN_TEXT: p.owner_leave_message ?? '',
            DM_BEIM_OEFFNEN: p.dm_on_open,
            DM_BEIM_SCHLIESSEN: p.dm_on_close,
        };
    }

    /** Vergleichsform: ein geleertes Zahlenfeld liefert `undefined`, gemeint ist `null`. */
    function fragenText(liste: FrageEntwurf[]): string {
        return JSON.stringify(liste.map((f) => ({ ...f, max: f.max ?? null })));
    }

    function fragenAus(p: VwPanel): FrageEntwurf[] {
        return (p.fragen ?? []).map((f) => ({
            label: f.label ?? '',
            placeholder: f.placeholder ?? '',
            lang: !!f.lang,
            pflicht: f.pflicht !== false,
            max: f.max ?? null,
        }));
    }

    function panelOeffnen(id: number) {
        const p = daten?.panels.find((x) => x.id === id);
        if (!p) return;
        panelId = id;
        entwurf = panelWerte(p);
        original = panelWerte(p);
        fragen = fragenAus(p);
        fragenOriginal = fragenText(fragen);
        neueSupportRolle = '';
        neuePingRolle = '';
        aktionsFehler = null;
    }

    function panelSchliessen() {
        if (panelUngespeichert && !confirm('Ungespeicherte Änderungen verwerfen?'))
            return;
        panelId = null;
        entwurf = null;
        original = null;
    }

    function geaenderteFelder(
        e: PanelEntwurf | null,
        o: PanelEntwurf | null,
        a: Abschnitt,
    ): Feld[] {
        if (!e || !o) return [];
        return ABSCHNITTE[a].filter((f) => e[f] !== o[f]);
    }

    async function abschnittSpeichern(a: Abschnitt) {
        if (!entwurf || !original || panelId === null) return;
        const felder = geaenderteFelder(entwurf, original, a);
        if (felder.length === 0) return;
        const werte: Record<string, string | number | boolean> = {};
        for (const f of felder) {
            const w = entwurf[f];
            if (ZAHLFELDER.includes(f)) {
                if (typeof w !== 'number' || !Number.isInteger(w) || w < 0) {
                    lokalerFehler(`${feldName(f)}: bitte eine ganze Zahl ab 0 eingeben.`);
                    return;
                }
            }
            werte[f] = w as string | number | boolean;
        }
        const pid = panelId;
        const ok = await ausfuehren('panel/setzen', { panel: pid, felder: werte });
        if (!ok) return;
        // Gespeichertes auf den Serverstand setzen (falls der Bot normalisiert).
        if (entwurf && original) {
            const e = { ...entwurf } as Record<Feld, unknown>;
            for (const f of felder) e[f] = original[f];
            entwurf = e as unknown as PanelEntwurf;
        }
        if (felder.includes('AKTIV')) menuesMitPanelMarkieren(pid);
    }

    function feldName(f: Feld): string {
        const namen: Partial<Record<Feld, string>> = {
            ZAEHLER: 'Zähler',
            STELLEN: 'Stellen',
            MAX_OFFEN: 'Limit je Person',
            COOLDOWN: 'Cooldown',
        };
        return namen[f] ?? f;
    }

    async function neuesPanel() {
        const name = neuesPanelName.trim();
        if (!name) {
            lokalerFehler('Bitte einen Namen eingeben.');
            return;
        }
        const vorher = new Set((daten?.panels ?? []).map((p) => p.id));
        if (await ausfuehren('panel/erstellen', { name })) {
            neuesPanelName = '';
            neuesPanelOffen = false;
            const neu = daten?.panels.find((p) => !vorher.has(p.id));
            if (neu) panelOeffnen(neu.id);
        }
    }

    async function rolleAendern(
        art: 'support' | 'ping',
        rolle: string,
        entfernen: boolean,
    ) {
        if (panelId === null || !rolle) return;
        const ok = await ausfuehren('panel/rolle', {
            panel: panelId,
            rolle,
            art,
            entfernen,
        });
        if (ok && !entfernen) {
            if (art === 'support') neueSupportRolle = '';
            else neuePingRolle = '';
        }
    }

    // --- Formular ------------------------------------------------------------

    function frageHinzufuegen() {
        if (fragen.length >= 5) return;
        fragen = [
            ...fragen,
            { label: '', placeholder: '', lang: false, pflicht: true, max: null },
        ];
    }

    function frageEntfernen(i: number) {
        fragen = fragen.filter((_, j) => j !== i);
    }

    function frageVerschieben(i: number, richtung: number) {
        const j = i + richtung;
        if (j < 0 || j >= fragen.length) return;
        const neu = [...fragen];
        [neu[i], neu[j]] = [neu[j], neu[i]];
        fragen = neu;
    }

    async function fragenSpeichern() {
        if (panelId === null) return;
        for (let i = 0; i < fragen.length; i++) {
            const f = fragen[i];
            if (!f.label.trim()) {
                lokalerFehler(`Frage ${i + 1} hat keinen Text.`);
                return;
            }
            if (f.label.trim().length > 45) {
                lokalerFehler(`Frage ${i + 1}: Text höchstens 45 Zeichen.`);
                return;
            }
            if (f.placeholder.length > 100) {
                lokalerFehler(`Frage ${i + 1}: Platzhalter höchstens 100 Zeichen.`);
                return;
            }
            if (
                f.max != null &&
                (!Number.isInteger(f.max) || f.max < 1 || f.max > 4000)
            ) {
                lokalerFehler(`Frage ${i + 1}: Höchstlänge 1 bis 4000 oder leer.`);
                return;
            }
        }
        const ok = await ausfuehren('panel/fragen', {
            panel: panelId,
            fragen: fragen.map((f) => ({
                label: f.label.trim(),
                placeholder: f.placeholder.trim() || null,
                lang: f.lang,
                pflicht: f.pflicht,
                max: f.max ?? null,
            })),
        });
        const p = daten?.panels.find((x) => x.id === panelId);
        if (ok && p) fragen = fragenAus(p);
    }

    // -----------------------------------------------------------------------
    // Menüs
    // -----------------------------------------------------------------------

    function menuTextAus(m: VwMenu): MenuTextEntwurf {
        const hex = farbeHex(m.embed_color);
        return {
            titel: m.embed_title ?? '',
            text: m.embed_text ?? '',
            platzhalter: m.placeholder ?? '',
            eigeneFarbe: hex !== null,
            farbe: hex ?? farbeHex(daten?.config.panel_color) ?? '#00cfff',
        };
    }

    function eintragAus(e: VwMenuEintrag): EintragEntwurf {
        return {
            label: e.label ?? '',
            emoji: e.emoji ?? '',
            beschreibung: e.description ?? '',
            knopfstil: e.button_style || 2,
        };
    }

    function menuOeffnen(id: number) {
        const m = daten?.menus.find((x) => x.id === id);
        if (!m) return;
        menuId = id;
        menuText = menuTextAus(m);
        menuTextOriginal = menuTextAus(m);
        eintraege = Object.fromEntries(
            m.entries.map((e) => [e.panel_id, eintragAus(e)]),
        );
        eintraegeOriginal = Object.fromEntries(
            m.entries.map((e) => [e.panel_id, eintragAus(e)]),
        );
        menuKanalNeu = m.channel_id;
        neuerEintragPanel = '';
        aktionsFehler = null;
    }

    function menuSchliessen() {
        if (menuUngespeichert && !confirm('Ungespeicherte Änderungen verwerfen?'))
            return;
        menuId = null;
        menuText = null;
        menuTextOriginal = null;
    }

    function gleich<T>(a: T, b: T): boolean {
        return JSON.stringify(a) === JSON.stringify(b);
    }

    async function menuTextSpeichern() {
        if (!menu || !menuText) return;
        const id = menu.id;
        const ok = await ausfuehren('menu/text', {
            menu: id,
            titel: menuText.titel,
            text: menuText.text,
            platzhalter: menuText.platzhalter,
            farbe: menuText.eigeneFarbe ? menuText.farbe : '',
        });
        if (ok) {
            if (menuTextOriginal) menuText = { ...menuTextOriginal };
            menuMarkieren(id);
        }
    }

    async function menuStil(stil: 'knoepfe' | 'dropdown') {
        if (!menu || menu.style === stil) return;
        const id = menu.id;
        if (await ausfuehren('menu/stil', { menu: id, stil })) menuMarkieren(id);
    }

    async function menuKanalAendern() {
        if (!menu || !menuKanalNeu || menuKanalNeu === menu.channel_id) return;
        if (
            menu.posted &&
            !confirm(
                `Die bisherige Nachricht in #${menu.channel_name ?? menu.channel_id} wird gelöscht. ` +
                    'Im neuen Kanal erscheint das Menü erst nach erneutem Posten. Fortfahren?',
            )
        )
            return;
        await ausfuehren('menu/kanal', { menu: menu.id, kanal: menuKanalNeu });
    }

    async function eintragSpeichern(pid: number) {
        if (!menu) return;
        const e = eintraege[pid];
        if (!e) return;
        const id = menu.id;
        const ok = await ausfuehren('menu/eintrag', {
            menu: id,
            panel: pid,
            label: e.label.trim(),
            emoji: e.emoji.trim(),
            beschreibung: e.beschreibung.trim(),
            knopfstil: e.knopfstil,
        });
        if (ok) {
            const o = eintraegeOriginal[pid];
            if (o) eintraege = { ...eintraege, [pid]: { ...o } };
            menuMarkieren(id);
        }
    }

    async function eintragHinzufuegen() {
        if (!menu || !neuerEintragPanel) return;
        const pid = Number(neuerEintragPanel);
        const p = daten?.panels.find((x) => x.id === pid);
        const id = menu.id;
        const ok = await ausfuehren('menu/eintrag', {
            menu: id,
            panel: pid,
            label: p?.name ?? '',
            emoji: '',
            beschreibung: '',
            knopfstil: 2,
        });
        if (ok) {
            neuerEintragPanel = '';
            menuMarkieren(id);
        }
    }

    async function eintragEntfernen(pid: number) {
        if (!menu) return;
        const id = menu.id;
        if (await ausfuehren('menu/eintrag-entfernen', { menu: id, panel: pid }))
            menuMarkieren(id);
    }

    async function eintragVerschieben(i: number, richtung: number) {
        if (!menu) return;
        const liste = sortiert(menu.entries).map((e) => e.panel_id);
        const j = i + richtung;
        if (j < 0 || j >= liste.length) return;
        [liste[i], liste[j]] = [liste[j], liste[i]];
        const id = menu.id;
        if (await ausfuehren('menu/reihenfolge', { menu: id, panels: liste }))
            menuMarkieren(id);
    }

    async function menuPosten(m: VwMenu) {
        const kanal = `#${m.channel_name ?? m.channel_id}`;
        const frage = m.posted
            ? `Das Menü wird in ${kanal} aktualisiert. Das ist auf dem Server sofort sichtbar.`
            : `Das Menü wird in ${kanal} gepostet. Das ist auf dem Server sofort sichtbar.`;
        if (!confirm(frage)) return;
        if (await ausfuehren('menu/posten', { menu: m.id })) {
            ungeglichenSetzen(ungeglichen.filter((x) => x !== m.id));
        }
    }

    async function menuLoeschen(m: VwMenu) {
        const frage = m.posted
            ? `Menü und Nachricht in #${m.channel_name ?? m.channel_id} löschen?`
            : 'Menü löschen? Es ist noch nicht gepostet.';
        if (!confirm(frage)) return;
        if (await ausfuehren('menu/loeschen', { menu: m.id })) {
            menuId = null;
            menuText = null;
            menuTextOriginal = null;
        }
    }

    async function alleAbgleichen() {
        if (
            !confirm(
                'Alle geposteten Menüs werden in Discord mit der Konfiguration abgeglichen. Das ist auf dem Server sofort sichtbar.',
            )
        )
            return;
        if (await ausfuehren('menus/abgleichen', {})) ungeglichenSetzen([]);
    }

    async function neuesMenu() {
        if (!neuesMenuKanal) {
            lokalerFehler('Bitte einen Kanal wählen.');
            return;
        }
        const vorher = new Set((daten?.menus ?? []).map((m) => m.id));
        if (
            await ausfuehren('menu/erstellen', {
                kanal: neuesMenuKanal,
                stil: neuesMenuStil,
            })
        ) {
            neuesMenuOffen = false;
            neuesMenuKanal = '';
            const neu = daten?.menus.find((m) => !vorher.has(m.id));
            if (neu) menuOeffnen(neu.id);
        }
    }

    function sortiert(liste: VwMenuEintrag[]): VwMenuEintrag[] {
        return [...liste].sort((a, b) => a.position - b.position);
    }

    // -----------------------------------------------------------------------
    // Server
    // -----------------------------------------------------------------------

    async function limitSpeichern() {
        const n = limitEntwurf;
        if (n == null || !Number.isInteger(n) || n < 0 || n > 100) {
            lokalerFehler('Limit: ganze Zahl von 0 bis 100 (0 = unbegrenzt).');
            return;
        }
        await ausfuehren('config/limit', { anzahl: n });
    }

    async function sperren() {
        const wer = sperreWer.trim();
        if (!/^\d{17,20}$/.test(wer)) {
            lokalerFehler(
                sperreRolle
                    ? 'Bitte eine Rolle wählen.'
                    : 'Bitte eine Discord-ID (17–20 Ziffern) eingeben.',
            );
            return;
        }
        if (
            await ausfuehren('sperre/hinzufuegen', {
                wer,
                rolle: sperreRolle,
                grund: sperreGrund.trim(),
            })
        ) {
            sperreWer = '';
            sperreGrund = '';
        }
    }

    async function entsperren(wer: string) {
        await ausfuehren('sperre/entfernen', { wer });
    }

    // -----------------------------------------------------------------------
    // Darstellung
    // -----------------------------------------------------------------------

    function rolle(id: string, liste: VwRolle[]): VwRolle | null {
        return liste.find((r) => r.id === id) ?? null;
    }

    function rollenFarbe(r: VwRolle | null): string {
        return farbeHex(r?.color) ?? (theme === 'light' ? '#94a3b8' : '#72767d');
    }

    function kanalGruppen(kanaele: VwKanal[]): { name: string; kanaele: VwKanal[] }[] {
        const gruppen: { name: string; kanaele: VwKanal[] }[] = [];
        const index = new Map<string, number>();
        for (const k of kanaele) {
            const name = k.kategorie ?? 'Ohne Kategorie';
            const schluessel = k.kategorie_id ?? '';
            if (!index.has(schluessel)) {
                index.set(schluessel, gruppen.length);
                gruppen.push({ name, kanaele: [] });
            }
            gruppen[index.get(schluessel)!].kanaele.push(k);
        }
        return gruppen;
    }

    function panelName(id: number, panels: VwPanel[]): string {
        return panels.find((p) => p.id === id)?.name ?? `Panel ${id}`;
    }

    function panelAktiv(id: number, panels: VwPanel[]): boolean {
        return panels.find((p) => p.id === id)?.active ?? false;
    }

    /** Nächster Kanalname nach dem Entwurf — zeigt, was das Muster bewirkt. */
    function namensVorschau(muster: string, zaehler: number | null, stellen: number | null): string {
        if (!muster) return '—';
        const nr = String((zaehler ?? 0) + 1).padStart(Math.max(0, stellen ?? 0), '0');
        return muster.replace(/\{count\}/g, nr);
    }

    function platzhalterErsetzen(text: string, panel: string): string {
        return text.replace(/\{user\}/g, '@Nutzer').replace(/\{panel_name\}/g, panel);
    }

    /**
     * Sehr kleiner Teil von Discords Markdown für die Vorschau. Erst alles
     * maskieren, dann gezielt Fett/Kursiv/Unterstrichen einsetzen — so kann
     * kein eingegebenes HTML durchrutschen.
     */
    function discordText(text: string): string {
        return text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
            .replace(/__(.+?)__/g, '<u>$1</u>')
            .replace(/\*(.+?)\*/g, '<em>$1</em>');
    }

    /** Eigene Emojis (`<:name:id>`) als `:name:` zeigen, Unicode bleibt. */
    function emojiText(emoji: string): string {
        const m = emoji.match(/^<a?:(\w+):\d+>$/);
        return m ? `:${m[1]}:` : emoji;
    }

    function inReihen<T>(liste: T[], groesse: number): T[][] {
        const reihen: T[][] = [];
        for (let i = 0; i < liste.length; i += groesse) {
            reihen.push(liste.slice(i, i + groesse));
        }
        return reihen;
    }

    // --- Abgeleitete Werte ---------------------------------------------------

    $: panel = panelId !== null ? (daten?.panels.find((p) => p.id === panelId) ?? null) : null;
    $: menu = menuId !== null ? (daten?.menus.find((m) => m.id === menuId) ?? null) : null;
    $: rollen = daten?.rollen ?? [];
    $: waehlbareRollen = rollen.filter((r) => !r.managed);
    $: gruppen = kanalGruppen(daten?.kanaele ?? []);
    $: kategorien = daten?.kategorien ?? [];

    $: geaendert = {
        allgemein: geaenderteFelder(entwurf, original, 'allgemein').length > 0,
        kanaele: geaenderteFelder(entwurf, original, 'kanaele').length > 0,
        begruessung: geaenderteFelder(entwurf, original, 'begruessung').length > 0,
        verhalten: geaenderteFelder(entwurf, original, 'verhalten').length > 0,
    };
    $: fragenGeaendert = fragenText(fragen) !== fragenOriginal;
    $: panelUngespeichert =
        geaendert.allgemein ||
        geaendert.kanaele ||
        geaendert.begruessung ||
        geaendert.verhalten ||
        fragenGeaendert;

    $: menuTextGeaendert = !!menuText && !!menuTextOriginal && !gleich(menuText, menuTextOriginal);
    $: eintragGeaendert = Object.fromEntries(
        Object.keys(eintraege).map((k) => [
            k,
            !gleich(eintraege[Number(k)], eintraegeOriginal[Number(k)]),
        ]),
    ) as Record<string, boolean>;
    $: menuUngespeichert = menuTextGeaendert || Object.values(eintragGeaendert).some((x) => x);

    $: menuEintraege = menu ? sortiert(menu.entries) : [];
    $: fehlendePanels = menu
        ? (daten?.panels ?? []).filter((p) => !menu!.entries.some((e) => e.panel_id === p.id))
        : [];
    $: vorschauAktiv = menuEintraege.filter((e) => panelAktiv(e.panel_id, daten?.panels ?? []));
    $: vorschauInaktiv = menuEintraege.filter((e) => !panelAktiv(e.panel_id, daten?.panels ?? []));
    $: vorschauFarbe = menuText
        ? menuText.eigeneFarbe
            ? menuText.farbe
            : (farbeHex(daten?.config.panel_color) ?? '#1e1f22')
        : '#1e1f22';
    $: anzahlUngeglichen = (daten?.menus ?? []).filter((m) => istUngeglichen(m, ungeglichen)).length;
    $: gepostete = (daten?.menus ?? []).filter((m) => m.posted).length;
</script>

<div class="verwaltung" class:light={theme === 'light'}>
    {#if laedt}
        <div class="mitte"><div class="spinner" aria-label="Lädt"></div></div>
    {:else if ladeFehler}
        <div class="fehler" role="alert">
            {ladeFehler}
            <button class="knopf umriss klein" on:click={() => laden(false)}>Erneut versuchen</button>
        </div>
    {:else if daten}
        <div class="meldungen" aria-live="polite">
            {#if aktionsFehler}
                <div class="fehler" role="alert">
                    {aktionsFehler}
                    <button class="knopf text klein" on:click={() => (aktionsFehler = null)}>Schließen</button>
                </div>
            {/if}
            {#if meldung}
                <div class="bestaetigung">{meldung}</div>
            {/if}
        </div>

        <fieldset class="huelle" disabled={beschaeftigt} aria-busy={beschaeftigt}>
            <nav class="unterreiter" aria-label="Verwaltungsbereich">
                <button
                    class:aktiv={bereich === 'panels'}
                    aria-pressed={bereich === 'panels'}
                    on:click={() => (bereich = 'panels')}>Panels</button
                >
                <button
                    class:aktiv={bereich === 'menues'}
                    aria-pressed={bereich === 'menues'}
                    on:click={() => (bereich = 'menues')}
                    >Menüs{#if anzahlUngeglichen > 0}<span class="zahl-etikett warn">{anzahlUngeglichen} offen</span>{/if}</button
                >
                <button
                    class:aktiv={bereich === 'server'}
                    aria-pressed={bereich === 'server'}
                    on:click={() => (bereich = 'server')}>Server</button
                >
            </nav>

            <!-- ============================================================ -->
            <!-- Panels                                                       -->
            <!-- ============================================================ -->
            {#if bereich === 'panels'}
                {#if panel && entwurf}
                    <button class="knopf text zurueck" on:click={panelSchliessen}>← Alle Panels</button>

                    <div class="titelzeile">
                        <h2>{panel.name}</h2>
                        <span class="etikett">ID {panel.id} · {panel.open} offen · {panel.total} gesamt</span>
                    </div>
                    <p class="leise">
                        Einstellungen wirken beim nächsten Ticket. Nur „Aktiv“ ändert auch
                        die Menüs in Discord — dort erst nach dem Abgleichen.
                    </p>

                    <!-- Allgemein -->
                    <section class="abschnitt">
                        <h3>Allgemein</h3>
                        <label class="schalter">
                            <input type="checkbox" bind:checked={entwurf.AKTIV} />
                            <span>Aktiv</span>
                        </label>
                        <p class="hilfe">
                            Panels werden nicht gelöscht, nur deaktiviert. Ein inaktives Panel
                            verschwindet aus den Menüs; Alt-Tickets bleiben erhalten.
                        </p>
                        <div class="raster">
                            <label class="feld">
                                <span class="etikett">Name offen</span>
                                <input type="text" maxlength="90" bind:value={entwurf.NAME_OFFEN} />
                                <span class="hilfe">
                                    <code>{'{count}'}</code> = Ticketnummer. Nächster Kanal:
                                    <b class="mono">{namensVorschau(entwurf.NAME_OFFEN, entwurf.ZAEHLER, entwurf.STELLEN)}</b>
                                </span>
                            </label>
                            <label class="feld">
                                <span class="etikett">Name geschlossen</span>
                                <input type="text" maxlength="90" bind:value={entwurf.NAME_GESCHLOSSEN} />
                                <span class="hilfe">Wird beim Schließen gesetzt, sonst wie oben.</span>
                            </label>
                            <label class="feld">
                                <span class="etikett">Zähler</span>
                                <input type="number" min="0" max="1000000" step="1" bind:value={entwurf.ZAEHLER} />
                                <span class="hilfe">Nummer des letzten Tickets; das nächste bekommt +1.</span>
                            </label>
                            <label class="feld">
                                <span class="etikett">Stellen</span>
                                <input type="number" min="0" max="8" step="1" bind:value={entwurf.STELLEN} />
                                <span class="hilfe">Mit Nullen auffüllen, z. B. 4 → 0042.</span>
                            </label>
                            <label class="feld">
                                <span class="etikett">Limit je Person</span>
                                <input type="number" min="0" max="100" step="1" bind:value={entwurf.MAX_OFFEN} />
                                <span class="hilfe">Offene Tickets je Person in diesem Panel, 0 = unbegrenzt.</span>
                            </label>
                            <label class="feld">
                                <span class="etikett">Cooldown (Sekunden)</span>
                                <input type="number" min="0" max="2592000" step="1" bind:value={entwurf.COOLDOWN} />
                                <span class="hilfe">Wartezeit bis zum nächsten Ticket derselben Person, 0 = keine.</span>
                            </label>
                        </div>
                        <div class="aktionen">
                            <button
                                class="knopf"
                                class:umriss={!geaendert.allgemein}
                                disabled={!geaendert.allgemein}
                                on:click={() => abschnittSpeichern('allgemein')}>Speichern</button
                            >
                        </div>
                    </section>

                    <!-- Kanäle -->
                    <section class="abschnitt">
                        <h3>Kanäle</h3>
                        <div class="raster">
                            <label class="feld">
                                <span class="etikett">Kategorie offen</span>
                                <select bind:value={entwurf.KATEGORIE_OFFEN}>
                                    <option value="">— keine —</option>
                                    {#each kategorien as k (k.id)}
                                        <option value={k.id}>{k.name}</option>
                                    {/each}
                                    {#if entwurf.KATEGORIE_OFFEN && !kategorien.some((k) => k.id === entwurf?.KATEGORIE_OFFEN)}
                                        <option value={entwurf.KATEGORIE_OFFEN}>unbekannt ({entwurf.KATEGORIE_OFFEN})</option>
                                    {/if}
                                </select>
                                <span class="hilfe">Ist sie voll (50 Kanäle), nimmt der Bot „Name 1“, „Name 2“ …</span>
                            </label>
                            <label class="feld">
                                <span class="etikett">Kategorie geschlossen</span>
                                <select bind:value={entwurf.KATEGORIE_GESCHLOSSEN}>
                                    <option value="">— keine —</option>
                                    {#each kategorien as k (k.id)}
                                        <option value={k.id}>{k.name}</option>
                                    {/each}
                                    {#if entwurf.KATEGORIE_GESCHLOSSEN && !kategorien.some((k) => k.id === entwurf?.KATEGORIE_GESCHLOSSEN)}
                                        <option value={entwurf.KATEGORIE_GESCHLOSSEN}>unbekannt ({entwurf.KATEGORIE_GESCHLOSSEN})</option>
                                    {/if}
                                </select>
                                <span class="hilfe">Leer = Ticket bleibt beim Schließen, wo es ist.</span>
                            </label>
                            <label class="feld">
                                <span class="etikett">Log-Kanal</span>
                                <select bind:value={entwurf.LOG_KANAL}>
                                    <option value="">— keine —</option>
                                    {#each gruppen as g (g.name)}
                                        <optgroup label={g.name}>
                                            {#each g.kanaele as k (k.id)}
                                                <option value={k.id}>#{k.name}</option>
                                            {/each}
                                        </optgroup>
                                    {/each}
                                    {#if entwurf.LOG_KANAL && !(daten.kanaele ?? []).some((k) => k.id === entwurf?.LOG_KANAL)}
                                        <option value={entwurf.LOG_KANAL}>unbekannt ({entwurf.LOG_KANAL})</option>
                                    {/if}
                                </select>
                            </label>
                        </div>
                        <div class="aktionen">
                            <button
                                class="knopf"
                                class:umriss={!geaendert.kanaele}
                                disabled={!geaendert.kanaele}
                                on:click={() => abschnittSpeichern('kanaele')}>Speichern</button
                            >
                        </div>
                    </section>

                    <!-- Rollen -->
                    <section class="abschnitt">
                        <h3>Rollen</h3>
                        {#each [{ art: 'support', titel: 'Support-Rollen', hilfe: 'Wer Tickets dieses Panels sieht und bearbeitet.', ids: panel.support }, { art: 'ping', titel: 'Ping-Rollen', hilfe: 'Wer beim Öffnen eines Tickets gepingt wird.', ids: panel.ping }] as gruppe (gruppe.art)}
                            <div class="rollengruppe">
                                <span class="etikett">{gruppe.titel}</span>
                                <p class="hilfe">{gruppe.hilfe}</p>
                                <ul class="chips">
                                    {#each gruppe.ids as id (id)}
                                        {@const r = rolle(id, rollen)}
                                        <li class="chip" class:geloescht={!r}>
                                            <span class="punkt" style="background: {rollenFarbe(r)}"></span>
                                            <span>{r ? r.name : `gelöschte Rolle (${id})`}</span>
                                            <button
                                                class="chip-weg"
                                                aria-label="{r ? r.name : id} entfernen"
                                                title="Entfernen"
                                                on:click={() => rolleAendern(gruppe.art === 'ping' ? 'ping' : 'support', id, true)}
                                                >×</button
                                            >
                                        </li>
                                    {:else}
                                        <li class="leise">keine</li>
                                    {/each}
                                </ul>
                                <div class="zeile">
                                    {#if gruppe.art === 'support'}
                                        <select bind:value={neueSupportRolle} aria-label="Support-Rolle hinzufügen">
                                            <option value="">Rolle wählen …</option>
                                            {#each waehlbareRollen.filter((r) => !panel?.support.includes(r.id)) as r (r.id)}
                                                <option value={r.id}>{r.name}</option>
                                            {/each}
                                        </select>
                                        <button
                                            class="knopf umriss"
                                            disabled={!neueSupportRolle}
                                            on:click={() => rolleAendern('support', neueSupportRolle, false)}>Hinzufügen</button
                                        >
                                    {:else}
                                        <select bind:value={neuePingRolle} aria-label="Ping-Rolle hinzufügen">
                                            <option value="">Rolle wählen …</option>
                                            {#each waehlbareRollen.filter((r) => !panel?.ping.includes(r.id)) as r (r.id)}
                                                <option value={r.id}>{r.name}</option>
                                            {/each}
                                        </select>
                                        <button
                                            class="knopf umriss"
                                            disabled={!neuePingRolle}
                                            on:click={() => rolleAendern('ping', neuePingRolle, false)}>Hinzufügen</button
                                        >
                                    {/if}
                                </div>
                            </div>
                        {/each}
                    </section>

                    <!-- Begrüßung -->
                    <section class="abschnitt">
                        <h3>Begrüßung</h3>
                        <p class="hilfe">
                            Platzhalter: <code>{'{user}'}</code> erwähnt den Eröffner,
                            <code>{'{panel_name}'}</code> setzt den Panelnamen ein.
                        </p>
                        <div class="raster zwei">
                            <div class="spalte">
                                <label class="feld">
                                    <span class="etikett">Willkommenstext <span class="zaehler">{entwurf.WILLKOMMENSTEXT.length}/1800</span></span>
                                    <textarea rows="4" maxlength="1800" bind:value={entwurf.WILLKOMMENSTEXT}></textarea>
                                </label>
                                <label class="feld">
                                    <span class="etikett">Embed-Titel <span class="zaehler">{entwurf.EMBED_TITEL.length}/256</span></span>
                                    <input type="text" maxlength="256" bind:value={entwurf.EMBED_TITEL} />
                                </label>
                                <label class="feld">
                                    <span class="etikett">Embed-Text <span class="zaehler">{entwurf.EMBED_TEXT.length}/4000</span></span>
                                    <textarea rows="5" maxlength="4000" bind:value={entwurf.EMBED_TEXT}></textarea>
                                </label>
                            </div>
                            <div class="spalte">
                                <span class="etikett">Vorschau</span>
                                <div class="discord-vorschau">
                                    {#if entwurf.WILLKOMMENSTEXT}
                                        <p class="dc-text">{@html discordText(platzhalterErsetzen(entwurf.WILLKOMMENSTEXT, panel.name))}</p>
                                    {/if}
                                    {#if entwurf.EMBED_TITEL || entwurf.EMBED_TEXT}
                                        <div class="dc-embed" style="border-left-color: {farbeHex(daten.config.ticket_color) ?? '#1e1f22'}">
                                            {#if entwurf.EMBED_TITEL}
                                                <p class="dc-titel">{platzhalterErsetzen(entwurf.EMBED_TITEL, panel.name)}</p>
                                            {/if}
                                            {#if entwurf.EMBED_TEXT}
                                                <p class="dc-text">{@html discordText(platzhalterErsetzen(entwurf.EMBED_TEXT, panel.name))}</p>
                                            {/if}
                                        </div>
                                    {/if}
                                    {#if !entwurf.WILLKOMMENSTEXT && !entwurf.EMBED_TITEL && !entwurf.EMBED_TEXT}
                                        <p class="leise">Keine Begrüßung festgelegt.</p>
                                    {/if}
                                </div>
                            </div>
                        </div>
                        <div class="aktionen">
                            <button
                                class="knopf"
                                class:umriss={!geaendert.begruessung}
                                disabled={!geaendert.begruessung}
                                on:click={() => abschnittSpeichern('begruessung')}>Speichern</button
                            >
                        </div>
                    </section>

                    <!-- Formular -->
                    <section class="abschnitt">
                        <h3>Formular</h3>
                        <p class="hilfe">
                            Bis zu fünf Fragen, die beim Öffnen abgefragt werden. Ohne Fragen
                            öffnet das Ticket sofort.
                        </p>
                        <ol class="fragen">
                            {#each fragen as f, i (i)}
                                <li class="frage">
                                    <div class="frage-kopf">
                                        <span class="etikett">Frage {i + 1}</span>
                                        <div class="frage-knoepfe">
                                            <button class="knopf text klein" disabled={i === 0} aria-label="Frage {i + 1} nach oben" on:click={() => frageVerschieben(i, -1)}>↑</button>
                                            <button class="knopf text klein" disabled={i === fragen.length - 1} aria-label="Frage {i + 1} nach unten" on:click={() => frageVerschieben(i, 1)}>↓</button>
                                            <button class="knopf text klein gefahr" on:click={() => frageEntfernen(i)}>Entfernen</button>
                                        </div>
                                    </div>
                                    <div class="raster">
                                        <label class="feld">
                                            <span class="etikett">Text <span class="zaehler" class:ueber={f.label.length > 45}>{f.label.length}/45</span></span>
                                            <input type="text" maxlength="45" bind:value={f.label} />
                                        </label>
                                        <label class="feld">
                                            <span class="etikett">Platzhalter <span class="zaehler" class:ueber={f.placeholder.length > 100}>{f.placeholder.length}/100</span></span>
                                            <input type="text" maxlength="100" bind:value={f.placeholder} />
                                        </label>
                                        <label class="feld">
                                            <span class="etikett">Antwort</span>
                                            <select bind:value={f.lang}>
                                                <option value={false}>kurz (eine Zeile)</option>
                                                <option value={true}>lang (Absatz)</option>
                                            </select>
                                        </label>
                                        <label class="feld">
                                            <span class="etikett">Höchstlänge</span>
                                            <input type="number" min="1" max="4000" step="1" placeholder="Discord-Standard" bind:value={f.max} />
                                        </label>
                                    </div>
                                    <label class="schalter">
                                        <input type="checkbox" bind:checked={f.pflicht} />
                                        <span>Pflichtfeld</span>
                                    </label>
                                </li>
                            {:else}
                                <li class="leise">Kein Formular.</li>
                            {/each}
                        </ol>
                        <div class="aktionen">
                            <button class="knopf umriss" disabled={fragen.length >= 5} on:click={frageHinzufuegen}>Frage hinzufügen</button>
                            <button
                                class="knopf"
                                class:umriss={!fragenGeaendert}
                                disabled={!fragenGeaendert}
                                on:click={fragenSpeichern}>Formular speichern</button
                            >
                        </div>
                    </section>

                    <!-- Verhalten -->
                    <section class="abschnitt">
                        <h3>Verhalten</h3>
                        <label class="schalter">
                            <input type="checkbox" bind:checked={entwurf.SCHLIESSEN_WENN_WEG} />
                            <span>Schließen, wenn der Eröffner den Server verlässt</span>
                        </label>
                        <label class="feld">
                            <span class="etikett">Text dazu <span class="zaehler">{entwurf.WEGGEGANGEN_TEXT.length}/1800</span></span>
                            <textarea rows="2" maxlength="1800" bind:value={entwurf.WEGGEGANGEN_TEXT}></textarea>
                            <span class="hilfe">Wird ins Ticket geschrieben, wenn der Eröffner geht.</span>
                        </label>
                        <label class="schalter">
                            <input type="checkbox" bind:checked={entwurf.DM_BEIM_OEFFNEN} />
                            <span>Eröffner beim Öffnen per DM benachrichtigen</span>
                        </label>
                        <label class="schalter">
                            <input type="checkbox" bind:checked={entwurf.DM_BEIM_SCHLIESSEN} />
                            <span>Eröffner beim Schließen per DM benachrichtigen</span>
                        </label>
                        <div class="aktionen">
                            <button
                                class="knopf"
                                class:umriss={!geaendert.verhalten}
                                disabled={!geaendert.verhalten}
                                on:click={() => abschnittSpeichern('verhalten')}>Speichern</button
                            >
                        </div>
                    </section>
                {:else}
                    <div class="titelzeile">
                        <h2>Panels</h2>
                        <button class="knopf" on:click={() => (neuesPanelOffen = !neuesPanelOffen)}>Neues Panel</button>
                    </div>
                    {#if neuesPanelOffen}
                        <form class="zeile neu" on:submit|preventDefault={neuesPanel}>
                            <label class="feld wachsen">
                                <span class="etikett">Name des Panels</span>
                                <input type="text" maxlength="80" bind:value={neuesPanelName} />
                            </label>
                            <button class="knopf" type="submit" disabled={!neuesPanelName.trim()}>Anlegen</button>
                            <button class="knopf text" type="button" on:click={() => (neuesPanelOffen = false)}>Abbrechen</button>
                        </form>
                    {/if}

                    {#if daten.panels.length === 0}
                        <p class="leise">Auf diesem Server gibt es noch keine Panels.</p>
                    {:else}
                        <ul class="liste">
                            {#each daten.panels as p (p.id)}
                                <li>
                                    <button class="zeilenknopf" class:still={!p.active} on:click={() => panelOeffnen(p.id)}>
                                        <span class="haupt">
                                            <span class="name">{p.name}</span>
                                            <span class="etikett">
                                                {p.active ? 'aktiv' : 'inaktiv'} · nächster Kanal {p.next_name ?? '—'}
                                                · {p.fragen.length} {p.fragen.length === 1 ? 'Frage' : 'Fragen'}
                                            </span>
                                        </span>
                                        <span class="zahlen mono"><b>{p.open}</b> / {p.total}</span>
                                    </button>
                                </li>
                            {/each}
                        </ul>
                        <p class="hilfe">Zahlen: offene / alle Tickets des Panels.</p>
                    {/if}
                {/if}

            <!-- ============================================================ -->
            <!-- Menüs                                                        -->
            <!-- ============================================================ -->
            {:else if bereich === 'menues'}
                {#if menu && menuText}
                    <button class="knopf text zurueck" on:click={menuSchliessen}>← Alle Menüs</button>

                    <div class="titelzeile">
                        <h2>Menü in #{menu.channel_name ?? menu.channel_id}</h2>
                        <span class="etikett">ID {menu.id} · {menu.posted ? 'gepostet' : 'nicht gepostet'}</span>
                    </div>

                    {#if istUngeglichen(menu, ungeglichen)}
                        <div class="hinweis-band">
                            <span>Noch nicht in Discord übernommen.</span>
                            <button class="knopf umriss" on:click={() => menu && menuPosten(menu)}>Abgleichen</button>
                        </div>
                    {/if}

                    <div class="raster zwei">
                        <div class="spalte">
                            <!-- Kanal und Stil -->
                            <section class="abschnitt">
                                <h3>Kanal und Stil</h3>
                                <div class="zeile">
                                    <label class="feld wachsen">
                                        <span class="etikett">Kanal</span>
                                        <select bind:value={menuKanalNeu}>
                                            {#each gruppen as g (g.name)}
                                                <optgroup label={g.name}>
                                                    {#each g.kanaele as k (k.id)}
                                                        <option value={k.id}>#{k.name}</option>
                                                    {/each}
                                                </optgroup>
                                            {/each}
                                            {#if !(daten.kanaele ?? []).some((k) => k.id === menu?.channel_id)}
                                                <option value={menu.channel_id}>#{menu.channel_name ?? menu.channel_id}</option>
                                            {/if}
                                        </select>
                                    </label>
                                    <button class="knopf umriss" disabled={menuKanalNeu === menu.channel_id} on:click={menuKanalAendern}>Umziehen</button>
                                </div>
                                {#if menu.posted}
                                    <p class="hilfe">Umziehen löscht die gepostete Nachricht im alten Kanal.</p>
                                {/if}
                                <div class="umschalter" role="group" aria-label="Stil">
                                    <button class:aktiv={menu.style === 'knoepfe'} aria-pressed={menu.style === 'knoepfe'} on:click={() => menuStil('knoepfe')}>Knöpfe</button>
                                    <button class:aktiv={menu.style === 'dropdown'} aria-pressed={menu.style === 'dropdown'} on:click={() => menuStil('dropdown')}>Dropdown</button>
                                </div>
                            </section>

                            <!-- Text -->
                            <section class="abschnitt">
                                <h3>Text</h3>
                                <label class="feld">
                                    <span class="etikett">Titel <span class="zaehler">{menuText.titel.length}/256</span></span>
                                    <input type="text" maxlength="256" bind:value={menuText.titel} />
                                </label>
                                <label class="feld">
                                    <span class="etikett">Text <span class="zaehler">{menuText.text.length}/4000</span></span>
                                    <textarea rows="6" maxlength="4000" bind:value={menuText.text}></textarea>
                                </label>
                                {#if menu.style === 'dropdown'}
                                    <label class="feld">
                                        <span class="etikett">Platzhalter im Dropdown <span class="zaehler">{menuText.platzhalter.length}/150</span></span>
                                        <input type="text" maxlength="150" bind:value={menuText.platzhalter} />
                                    </label>
                                {/if}
                                <div class="zeile">
                                    <label class="schalter">
                                        <input type="checkbox" bind:checked={menuText.eigeneFarbe} />
                                        <span>Eigene Farbe</span>
                                    </label>
                                    {#if menuText.eigeneFarbe}
                                        <input class="farbwahl" type="color" bind:value={menuText.farbe} aria-label="Farbe der Leiste" />
                                        <span class="etikett">{menuText.farbe}</span>
                                    {:else}
                                        <span class="hilfe">Serverfarbe {farbeHex(daten.config.panel_color) ?? ''}</span>
                                    {/if}
                                </div>
                                <div class="aktionen">
                                    <button
                                        class="knopf"
                                        class:umriss={!menuTextGeaendert}
                                        disabled={!menuTextGeaendert}
                                        on:click={menuTextSpeichern}>Text speichern</button
                                    >
                                </div>
                            </section>

                            <!-- Einträge -->
                            <section class="abschnitt">
                                <h3>Einträge</h3>
                                {#if menuEintraege.length === 0}
                                    <p class="leise">Noch keine Einträge.</p>
                                {/if}
                                <ol class="eintraege">
                                    {#each menuEintraege as e, i (e.panel_id)}
                                        {#if eintraege[e.panel_id]}
                                            <li class="eintrag" class:still={!panelAktiv(e.panel_id, daten.panels)}>
                                                <div class="frage-kopf">
                                                    <span class="etikett">
                                                        {panelName(e.panel_id, daten.panels)}{#if !panelAktiv(e.panel_id, daten.panels)}{' · Panel inaktiv'}{/if}
                                                    </span>
                                                    <div class="frage-knoepfe">
                                                        <button class="knopf text klein" disabled={i === 0} aria-label="Nach oben" on:click={() => eintragVerschieben(i, -1)}>↑</button>
                                                        <button class="knopf text klein" disabled={i === menuEintraege.length - 1} aria-label="Nach unten" on:click={() => eintragVerschieben(i, 1)}>↓</button>
                                                        <button class="knopf text klein gefahr" on:click={() => eintragEntfernen(e.panel_id)}>Entfernen</button>
                                                    </div>
                                                </div>
                                                <div class="raster">
                                                    <label class="feld">
                                                        <span class="etikett">Beschriftung <span class="zaehler">{eintraege[e.panel_id].label.length}/80</span></span>
                                                        <input type="text" maxlength="80" bind:value={eintraege[e.panel_id].label} />
                                                    </label>
                                                    <label class="feld">
                                                        <span class="etikett">Emoji</span>
                                                        <input type="text" placeholder="z. B. 📩 oder <:name:id>" bind:value={eintraege[e.panel_id].emoji} />
                                                    </label>
                                                    {#if menu.style === 'dropdown'}
                                                        <label class="feld">
                                                            <span class="etikett">Beschreibung <span class="zaehler">{eintraege[e.panel_id].beschreibung.length}/100</span></span>
                                                            <input type="text" maxlength="100" bind:value={eintraege[e.panel_id].beschreibung} />
                                                        </label>
                                                    {:else}
                                                        <label class="feld">
                                                            <span class="etikett">Knopffarbe</span>
                                                            <select bind:value={eintraege[e.panel_id].knopfstil}>
                                                                {#each KNOPFSTILE as s (s.wert)}
                                                                    <option value={s.wert}>{s.name}</option>
                                                                {/each}
                                                            </select>
                                                        </label>
                                                    {/if}
                                                </div>
                                                <div class="aktionen">
                                                    <button
                                                        class="knopf klein-pille"
                                                        class:umriss={!eintragGeaendert[e.panel_id]}
                                                        disabled={!eintragGeaendert[e.panel_id]}
                                                        on:click={() => eintragSpeichern(e.panel_id)}>Eintrag speichern</button
                                                    >
                                                </div>
                                            </li>
                                        {/if}
                                    {/each}
                                </ol>
                                {#if fehlendePanels.length > 0}
                                    <div class="zeile">
                                        <select bind:value={neuerEintragPanel} aria-label="Panel hinzufügen">
                                            <option value="">Panel hinzufügen …</option>
                                            {#each fehlendePanels as p (p.id)}
                                                <option value={String(p.id)}>{p.name}{p.active ? '' : ' (inaktiv)'}</option>
                                            {/each}
                                        </select>
                                        <button class="knopf umriss" disabled={!neuerEintragPanel} on:click={eintragHinzufuegen}>Hinzufügen</button>
                                    </div>
                                {/if}
                            </section>
                        </div>

                        <!-- Vorschau -->
                        <div class="spalte">
                            <section class="abschnitt klebend">
                                <h3>Vorschau</h3>
                                <div class="discord-vorschau">
                                    <div class="dc-embed" style="border-left-color: {vorschauFarbe}">
                                        {#if menuText.titel}
                                            <p class="dc-titel">{menuText.titel}</p>
                                        {/if}
                                        {#if menuText.text}
                                            <p class="dc-text">{@html discordText(menuText.text)}</p>
                                        {/if}
                                        {#if !menuText.titel && !menuText.text}
                                            <p class="leise">Ein Menü braucht einen Titel oder einen Text.</p>
                                        {/if}
                                    </div>
                                    {#if menu.style === 'knoepfe'}
                                        {#each inReihen(vorschauAktiv, 5) as reihe, ri (ri)}
                                            <div class="dc-reihe">
                                                {#each reihe as e (e.panel_id)}
                                                    {@const ent = eintraege[e.panel_id]}
                                                    <span class="dc-knopf" style="background: {KNOPFFARBEN[ent?.knopfstil ?? 2]}">
                                                        {#if ent?.emoji}<span class="dc-emoji">{emojiText(ent.emoji)}</span>{/if}
                                                        {ent?.label || panelName(e.panel_id, daten.panels)}
                                                    </span>
                                                {/each}
                                            </div>
                                        {/each}
                                    {:else if vorschauAktiv.length > 0}
                                        <div class="dc-dropdown">
                                            <span>{menuText.platzhalter || 'Bitte wählen'}</span>
                                            <span aria-hidden="true">⌄</span>
                                        </div>
                                        <ul class="dc-optionen">
                                            {#each vorschauAktiv as e (e.panel_id)}
                                                {@const ent = eintraege[e.panel_id]}
                                                <li>
                                                    {#if ent?.emoji}<span class="dc-emoji">{emojiText(ent.emoji)}</span>{/if}
                                                    <span>
                                                        <span class="dc-opt-name">{ent?.label || panelName(e.panel_id, daten.panels)}</span>
                                                        {#if ent?.beschreibung}<span class="dc-opt-beschr">{ent.beschreibung}</span>{/if}
                                                    </span>
                                                </li>
                                            {/each}
                                        </ul>
                                    {/if}
                                    {#if vorschauAktiv.length === 0}
                                        <p class="leise">Keine sichtbaren Einträge.</p>
                                    {/if}
                                    {#each vorschauInaktiv as e (e.panel_id)}
                                        <p class="dc-inaktiv">
                                            {eintraege[e.panel_id]?.label || panelName(e.panel_id, daten.panels)} —
                                            wird nicht angezeigt (Panel inaktiv)
                                        </p>
                                    {/each}
                                </div>
                                <p class="hilfe">Vorschau nach dem Entwurf; in Discord erst nach dem Posten/Abgleichen.</p>
                                <div class="aktionen">
                                    <button class="knopf" on:click={() => menu && menuPosten(menu)}>
                                        {menu.posted ? 'In Discord abgleichen' : 'In Discord posten'}
                                    </button>
                                    <button class="knopf text gefahr" on:click={() => menu && menuLoeschen(menu)}>Löschen</button>
                                </div>
                            </section>
                        </div>
                    </div>
                {:else}
                    <div class="titelzeile">
                        <h2>Menüs</h2>
                        <div class="aktionen eng">
                            {#if gepostete > 0}
                                <button class="knopf umriss" on:click={alleAbgleichen}>Alle geposteten Menüs abgleichen</button>
                            {/if}
                            <button class="knopf" on:click={() => (neuesMenuOffen = !neuesMenuOffen)}>Neues Menü</button>
                        </div>
                    </div>
                    {#if neuesMenuOffen}
                        <form class="zeile neu" on:submit|preventDefault={neuesMenu}>
                            <label class="feld wachsen">
                                <span class="etikett">Kanal</span>
                                <select bind:value={neuesMenuKanal}>
                                    <option value="">Kanal wählen …</option>
                                    {#each gruppen as g (g.name)}
                                        <optgroup label={g.name}>
                                            {#each g.kanaele as k (k.id)}
                                                <option value={k.id}>#{k.name}</option>
                                            {/each}
                                        </optgroup>
                                    {/each}
                                </select>
                            </label>
                            <label class="feld">
                                <span class="etikett">Stil</span>
                                <select bind:value={neuesMenuStil}>
                                    <option value="knoepfe">Knöpfe</option>
                                    <option value="dropdown">Dropdown</option>
                                </select>
                            </label>
                            <button class="knopf" type="submit" disabled={!neuesMenuKanal}>Anlegen</button>
                            <button class="knopf text" type="button" on:click={() => (neuesMenuOffen = false)}>Abbrechen</button>
                        </form>
                        <p class="hilfe">Ein neues Menü wird nur angelegt, nicht gepostet.</p>
                    {/if}

                    {#if daten.menus.length === 0}
                        <p class="leise">Auf diesem Server gibt es noch keine Menüs.</p>
                    {:else}
                        <ul class="liste">
                            {#each daten.menus as m (m.id)}
                                <li>
                                    <button class="zeilenknopf" on:click={() => menuOeffnen(m.id)}>
                                        <span class="haupt">
                                            <span class="name">#{m.channel_name ?? m.channel_id}</span>
                                            <span class="etikett">
                                                {m.style === 'dropdown' ? 'Dropdown' : 'Knöpfe'} ·
                                                {m.posted ? 'gepostet' : 'nicht gepostet'} ·
                                                {m.entries.length} {m.entries.length === 1 ? 'Eintrag' : 'Einträge'}
                                            </span>
                                            {#if m.embed_title || m.embed_text}
                                                <span class="auszug">{m.embed_title || m.embed_text}</span>
                                            {/if}
                                        </span>
                                        {#if istUngeglichen(m, ungeglichen)}
                                            <span class="zahl-etikett warn">nicht abgeglichen</span>
                                        {/if}
                                    </button>
                                </li>
                            {/each}
                        </ul>
                    {/if}
                {/if}

            <!-- ============================================================ -->
            <!-- Server                                                       -->
            <!-- ============================================================ -->
            {:else}
                <div class="titelzeile"><h2>Server</h2></div>

                <section class="abschnitt">
                    <h3>Limit offener Tickets</h3>
                    <form class="zeile" on:submit|preventDefault={limitSpeichern}>
                        <label class="feld">
                            <span class="etikett">Je Person, alle Panels zusammen</span>
                            <input type="number" min="0" max="100" step="1" bind:value={limitEntwurf} />
                        </label>
                        <button
                            class="knopf"
                            class:umriss={limitEntwurf === daten.config.limit}
                            type="submit"
                            disabled={limitEntwurf === daten.config.limit}>Speichern</button
                        >
                    </form>
                    <p class="hilfe">0 = unbegrenzt, höchstens 100.</p>
                </section>

                <section class="abschnitt">
                    <h3>Sperrliste</h3>
                    <p class="hilfe">Gesperrte Personen oder Rollen können keine Tickets öffnen.</p>
                    {#if daten.sperrliste.length === 0}
                        <p class="leise">Niemand gesperrt.</p>
                    {:else}
                        <ul class="liste">
                            {#each daten.sperrliste as s (s.subject_id)}
                                <li class="sperre">
                                    <span class="haupt">
                                        <span class="name">{s.name ?? s.subject_id}</span>
                                        <span class="etikett">
                                            {s.is_role ? 'Rolle' : 'Person'} · {s.subject_id}{#if s.added_at}{` · seit ${zeit(s.added_at)}`}{/if}
                                        </span>
                                        {#if s.reason}<span class="auszug">{s.reason}</span>{/if}
                                    </span>
                                    <button class="knopf text" on:click={() => entsperren(s.subject_id)}>Entsperren</button>
                                </li>
                            {/each}
                        </ul>
                    {/if}

                    <form class="sperren" on:submit|preventDefault={sperren}>
                        <label class="feld">
                            <span class="etikett">Art</span>
                            <select bind:value={sperreRolle} on:change={() => (sperreWer = '')}>
                                <option value={false}>Person</option>
                                <option value={true}>Rolle</option>
                            </select>
                        </label>
                        {#if sperreRolle}
                            <label class="feld wachsen">
                                <span class="etikett">Rolle</span>
                                <select bind:value={sperreWer}>
                                    <option value="">Rolle wählen …</option>
                                    {#each waehlbareRollen as r (r.id)}
                                        <option value={r.id}>{r.name}</option>
                                    {/each}
                                </select>
                            </label>
                        {:else}
                            <label class="feld wachsen">
                                <span class="etikett">Discord-ID</span>
                                <input type="text" inputmode="numeric" placeholder="17–20 Ziffern" bind:value={sperreWer} />
                            </label>
                        {/if}
                        <label class="feld wachsen">
                            <span class="etikett">Grund</span>
                            <input type="text" bind:value={sperreGrund} />
                        </label>
                        <button class="knopf umriss" type="submit" disabled={!sperreWer.trim()}>Sperren</button>
                    </form>
                </section>
            {/if}
        </fieldset>
    {/if}
</div>

<style>
    .verwaltung {
        --akzent: #00cfff;
        --akzent-hover: #33d9ff;
        --eisblau: #8be8ff;
        --text-2: #9aa0a6;
        --text-3: #8a9099;
        --linie: rgba(255, 255, 255, 0.08);
        --umriss: rgba(255, 255, 255, 0.38);
        --feld-bg: rgba(0, 0, 0, 0.25);
        --feld-rand: rgba(255, 255, 255, 0.18);
        --warn: #faa61a;
        --gefahr: #ff8a8c;
        --mono: 'IBM Plex Mono', ui-monospace, SFMono-Regular, Menlo, monospace;
        color: inherit;
    }
    .verwaltung.light {
        --akzent: #007c99;
        --akzent-hover: #006680;
        --eisblau: #006680;
        --text-2: #5b6470;
        --text-3: #6b7280;
        --linie: #e2e8f0;
        --umriss: #94a3b8;
        --feld-bg: #fff;
        --feld-rand: #cbd5e1;
        --warn: #b45309;
        --gefahr: #c0262b;
    }

    .huelle {
        border: 0;
        margin: 0;
        padding: 0;
        min-width: 0;
    }

    .mitte {
        display: flex;
        justify-content: center;
        align-items: center;
        min-height: 200px;
    }
    .spinner {
        width: 32px;
        height: 32px;
        border: 3px solid rgba(0, 207, 255, 0.15);
        border-left-color: var(--akzent);
        border-radius: 50%;
        animation: dreh 1s linear infinite;
    }
    @keyframes dreh {
        to {
            transform: rotate(360deg);
        }
    }

    /* --- Meldungen --------------------------------------------------- */
    .meldungen {
        display: flex;
        flex-direction: column;
        gap: 8px;
        margin-bottom: 16px;
    }
    .meldungen:empty {
        display: none;
    }
    .fehler,
    .bestaetigung {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        padding: 12px 16px;
        border-radius: 10px;
        line-height: 1.5;
    }
    .fehler {
        background: rgba(237, 66, 69, 0.12);
        border: 1px solid rgba(237, 66, 69, 0.4);
        color: var(--gefahr);
    }
    .bestaetigung {
        border: 1px solid var(--linie);
        border-left: 3px solid var(--eisblau);
    }
    .hinweis-band {
        display: flex;
        align-items: center;
        justify-content: space-between;
        flex-wrap: wrap;
        gap: 12px;
        padding: 12px 16px;
        margin-bottom: 24px;
        border: 1px solid var(--warn);
        border-radius: 10px;
        color: var(--warn);
        font-weight: 600;
    }

    /* --- Unterreiter ------------------------------------------------- */
    .unterreiter {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
        border-bottom: 1px solid var(--linie);
        margin-bottom: 24px;
    }
    .unterreiter button {
        min-height: 44px;
        padding: 0 16px;
        border: 0;
        border-bottom: 2px solid transparent;
        background: none;
        color: var(--text-2);
        font: inherit;
        font-size: 0.95rem;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        gap: 8px;
        margin-bottom: -1px;
    }
    .unterreiter button.aktiv {
        color: inherit;
        border-bottom-color: var(--akzent);
        font-weight: 600;
    }
    .unterreiter button:hover {
        color: inherit;
    }

    /* --- Überschriften ----------------------------------------------- */
    .titelzeile {
        display: flex;
        align-items: center;
        justify-content: space-between;
        flex-wrap: wrap;
        gap: 12px;
        margin-bottom: 16px;
    }
    .titelzeile h2 {
        margin: 0;
        font-size: 1.4rem;
        font-weight: 600;
        letter-spacing: -0.02em;
        line-height: 1.05;
    }
    .abschnitt {
        padding: 24px 0;
        border-top: 1px solid var(--linie);
    }
    .abschnitt h3 {
        margin: 0 0 12px;
        font-size: 1.05rem;
        font-weight: 600;
    }
    .klebend {
        position: sticky;
        top: 16px;
    }

    .etikett,
    .mono,
    .zahl-etikett,
    .zaehler,
    code {
        font-family: var(--mono);
    }
    .etikett {
        font-size: 0.75rem;
        color: var(--text-3);
        letter-spacing: 0.02em;
    }
    .zaehler {
        margin-left: 8px;
        font-size: 0.7rem;
        color: var(--text-3);
    }
    .zaehler.ueber {
        color: var(--gefahr);
    }
    .zahl-etikett {
        font-size: 0.72rem;
        font-weight: 500;
    }
    .zahl-etikett.warn {
        color: var(--warn);
    }
    code {
        font-size: 0.85em;
        color: var(--eisblau);
    }
    .leise,
    .hilfe {
        color: var(--text-2);
        line-height: 1.5;
    }
    .leise {
        margin: 8px 0;
        font-size: 0.9rem;
    }
    .hilfe {
        font-size: 0.82rem;
        margin: 4px 0 8px;
    }
    .feld .hilfe {
        margin: 0;
    }

    /* --- Knöpfe ------------------------------------------------------ */
    .knopf {
        min-height: 44px;
        padding: 0 20px;
        border-radius: 999px;
        border: 1px solid var(--akzent);
        background: var(--akzent);
        color: #04131a;
        font: inherit;
        font-size: 0.95rem;
        font-weight: 600;
        cursor: pointer;
        white-space: nowrap;
    }
    .knopf:hover:not(:disabled) {
        background: var(--akzent-hover);
        border-color: var(--akzent-hover);
    }
    .verwaltung.light .knopf {
        color: #fff;
    }
    .knopf.umriss {
        background: transparent;
        border-color: var(--umriss);
        color: inherit;
    }
    .knopf.umriss:hover:not(:disabled) {
        background: transparent;
        border-color: var(--eisblau);
    }
    .knopf.text {
        background: transparent;
        border-color: transparent;
        color: var(--akzent);
        padding: 0 12px;
    }
    .knopf.text:hover:not(:disabled) {
        background: transparent;
        border-color: transparent;
        text-decoration: underline;
    }
    .knopf.gefahr {
        color: var(--gefahr);
    }
    .knopf.klein {
        font-size: 0.85rem;
        min-width: 44px;
    }
    .knopf.klein-pille {
        font-size: 0.85rem;
    }
    .knopf:disabled {
        opacity: 0.45;
        cursor: default;
    }
    .zurueck {
        margin: 0 0 16px -12px;
    }

    button:focus-visible,
    input:focus-visible,
    select:focus-visible,
    textarea:focus-visible {
        outline: 2px solid var(--akzent);
        outline-offset: 2px;
    }

    .aktionen {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 8px;
        margin-top: 16px;
    }
    .aktionen.eng {
        margin-top: 0;
    }

    .umschalter {
        display: inline-flex;
        border: 1px solid var(--umriss);
        border-radius: 999px;
        padding: 2px;
        margin-top: 16px;
    }
    .umschalter button {
        min-height: 40px;
        padding: 0 16px;
        border: 0;
        border-radius: 999px;
        background: none;
        color: inherit;
        font: inherit;
        cursor: pointer;
    }
    .umschalter button.aktiv {
        background: var(--akzent);
        color: #04131a;
        font-weight: 600;
    }
    .verwaltung.light .umschalter button.aktiv {
        color: #fff;
    }

    /* --- Formularfelder ---------------------------------------------- */
    .raster {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
        gap: 16px;
    }
    .raster.zwei {
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 32px;
    }
    .spalte {
        display: flex;
        flex-direction: column;
        gap: 12px;
        min-width: 0;
    }
    .feld {
        display: flex;
        flex-direction: column;
        gap: 4px;
        min-width: 0;
    }
    .wachsen {
        flex: 1;
        min-width: 180px;
    }
    .zeile {
        display: flex;
        flex-wrap: wrap;
        align-items: flex-end;
        gap: 8px;
        margin-top: 8px;
    }
    .zeile.neu {
        margin-bottom: 24px;
    }
    input[type='text'],
    input[type='number'],
    select,
    textarea {
        min-height: 44px;
        padding: 8px 12px;
        border-radius: 10px;
        border: 1px solid var(--feld-rand);
        background: var(--feld-bg);
        color: inherit;
        font: inherit;
        font-size: 0.95rem;
        box-sizing: border-box;
        width: 100%;
    }
    .zeile > select {
        flex: 1;
        min-width: 180px;
        width: auto;
    }
    select option,
    select optgroup {
        background: #11161f;
        color: #dcddde;
    }
    .verwaltung.light select option,
    .verwaltung.light select optgroup {
        background: #fff;
        color: #2e3338;
    }
    textarea {
        resize: vertical;
        line-height: 1.5;
    }
    .schalter {
        display: flex;
        align-items: center;
        gap: 12px;
        min-height: 44px;
        cursor: pointer;
    }
    .schalter input {
        width: 20px;
        height: 20px;
        accent-color: var(--akzent);
    }
    .farbwahl {
        width: 56px;
        height: 44px;
        padding: 4px;
        border-radius: 10px;
        border: 1px solid var(--feld-rand);
        background: var(--feld-bg);
        cursor: pointer;
    }

    /* --- Listen mit Haarlinien --------------------------------------- */
    .liste {
        list-style: none;
        margin: 0;
        padding: 0;
        border-top: 1px solid var(--linie);
    }
    .liste > li {
        border-bottom: 1px solid var(--linie);
    }
    .zeilenknopf,
    .sperre {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        width: 100%;
        min-height: 56px;
        padding: 12px 8px;
        box-sizing: border-box;
    }
    .zeilenknopf {
        border: 0;
        background: none;
        color: inherit;
        font: inherit;
        text-align: left;
        cursor: pointer;
    }
    .zeilenknopf:hover {
        background: rgba(139, 232, 255, 0.05);
    }
    .verwaltung.light .zeilenknopf:hover {
        background: #f1f5f9;
    }
    .zeilenknopf.still {
        opacity: 0.55;
    }
    .haupt {
        display: flex;
        flex-direction: column;
        gap: 4px;
        min-width: 0;
    }
    .name {
        font-weight: 600;
    }
    .auszug {
        color: var(--text-2);
        font-size: 0.85rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 60ch;
    }
    .zahlen {
        font-size: 0.9rem;
        color: var(--text-2);
        white-space: nowrap;
    }
    .zahlen b {
        color: var(--eisblau);
    }

    /* --- Rollen-Chips ------------------------------------------------ */
    .rollengruppe + .rollengruppe {
        margin-top: 24px;
    }
    .chips {
        list-style: none;
        margin: 8px 0;
        padding: 0;
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
    }
    .chip {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        padding-left: 12px;
        border: 1px solid var(--linie);
        border-radius: 999px;
        font-size: 0.9rem;
    }
    .chip.geloescht {
        color: var(--text-2);
        border-style: dashed;
    }
    .punkt {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        flex: none;
    }
    .chip-weg {
        min-width: 44px;
        min-height: 36px;
        border: 0;
        background: none;
        color: var(--text-2);
        font-size: 1.1rem;
        cursor: pointer;
        border-radius: 999px;
    }
    .chip-weg:hover {
        color: var(--gefahr);
    }

    /* --- Fragen / Einträge ------------------------------------------- */
    .fragen,
    .eintraege {
        list-style: none;
        margin: 0;
        padding: 0;
    }
    .frage,
    .eintrag {
        padding: 16px 0;
        border-bottom: 1px solid var(--linie);
    }
    .frage:first-child,
    .eintrag:first-child {
        border-top: 1px solid var(--linie);
    }
    .eintrag.still .raster {
        opacity: 0.6;
    }
    .frage-kopf {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        margin-bottom: 8px;
    }
    .frage-knoepfe {
        display: flex;
        gap: 4px;
    }

    /* --- Discord-Vorschau -------------------------------------------- */
    .discord-vorschau {
        background: #313338;
        color: #dbdee1;
        border-radius: 10px;
        padding: 16px;
        font-size: 0.92rem;
        line-height: 1.4;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
    .verwaltung.light .discord-vorschau {
        background: #fff;
        color: #313338;
        border: 1px solid var(--linie);
    }
    .discord-vorschau .leise {
        color: #949ba4;
        margin: 0;
    }
    .dc-text {
        margin: 0;
        white-space: pre-wrap;
        word-break: break-word;
    }
    .dc-embed {
        background: #2b2d31;
        border-left: 4px solid;
        border-radius: 4px;
        padding: 12px 16px;
        display: flex;
        flex-direction: column;
        gap: 8px;
        max-width: 520px;
    }
    .verwaltung.light .dc-embed {
        background: #f2f3f5;
    }
    .dc-titel {
        margin: 0;
        font-weight: 600;
        color: #f2f3f5;
    }
    .verwaltung.light .dc-titel {
        color: #060607;
    }
    .dc-reihe {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
    }
    .dc-knopf {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        min-height: 32px;
        padding: 0 16px;
        border-radius: 8px;
        color: #fff;
        font-size: 0.875rem;
        font-weight: 500;
    }
    .dc-emoji {
        font-size: 1rem;
    }
    .dc-dropdown {
        display: flex;
        align-items: center;
        justify-content: space-between;
        max-width: 400px;
        min-height: 40px;
        padding: 0 12px;
        border-radius: 8px;
        background: #1e1f22;
        color: #949ba4;
    }
    .verwaltung.light .dc-dropdown {
        background: #e3e5e8;
        color: #5c5e66;
    }
    .dc-optionen {
        list-style: none;
        margin: 0;
        padding: 4px 0;
        max-width: 400px;
        border-radius: 8px;
        background: #2b2d31;
    }
    .verwaltung.light .dc-optionen {
        background: #f2f3f5;
    }
    .dc-optionen li {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
    }
    .dc-opt-name {
        display: block;
    }
    .dc-opt-beschr {
        display: block;
        font-size: 0.8rem;
        color: #949ba4;
    }
    .dc-inaktiv {
        margin: 0;
        font-size: 0.82rem;
        color: #949ba4;
    }

    /* --- Sperren ----------------------------------------------------- */
    .sperren {
        display: flex;
        flex-wrap: wrap;
        align-items: flex-end;
        gap: 8px;
        margin-top: 16px;
    }

    @media (max-width: 860px) {
        .raster.zwei {
            grid-template-columns: minmax(0, 1fr);
            gap: 0;
        }
        .klebend {
            position: static;
        }
    }
    @media (max-width: 560px) {
        .raster {
            grid-template-columns: minmax(0, 1fr);
        }
        .zeilenknopf {
            flex-wrap: wrap;
        }
        .auszug {
            max-width: 100%;
        }
    }
    @media (prefers-reduced-motion: reduce) {
        .spinner {
            animation-duration: 3s;
        }
    }
</style>
