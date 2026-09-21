<script lang="ts">
    // Automatische Erinnerungen und Kickpunkte eines Clans — im Bot heißen sie
    // "Listening Events". Sie hängen am Clan, nicht am Spieler; deshalb ein
    // eigener Abschnitt auf der Clanseite und nicht das Spieler-Fenster.
    //
    // Der Bot ist die Wahrheit: Die Formularlogik hier gräut Felder aus und
    // blendet sie ein, sie entscheidet aber nichts. Jede Kombination wird
    // drüben noch einmal geprüft, und dessen Begründung ist die, die der
    // Benutzer zu sehen bekommt.
    import { onMount } from 'svelte';
    import { slide, fade } from 'svelte/transition';

    export let apiBaseUrl: string;
    export let clanTag: string;
    export let theme: 'dark' | 'light' = 'dark';
    /** Ab Vize. Darunter ist der Abschnitt gar nicht erst eingebunden. */
    export let canManage: boolean = false;
    /** Kickpunktgründe des Clans, für die Auswahl im Formular. */
    export let kickpointReasons: Array<{ name: string; amount?: number }> = [];

    interface LEvent {
        id: number;
        clanTag: string;
        clanName: string | null;
        clanTagValid: boolean;
        type: string | null;
        duration: number;
        durationText: string;
        actionType: string | null;
        channelId: string | null;
        channelName: string | null;
        kickpointReason: string | null;
        punishedViolation: string | null;
        customMessage: string | null;
        status: string;
        statusLabel: string;
        fireText: string;
        lastRun: string;
        values: Record<string, number>;
    }

    interface Kanal {
        id: string;
        name: string;
        category: string | null;
        thread: boolean;
    }

    let events: LEvent[] = [];
    let kanaele: Kanal[] = [];
    let sideclans: Array<{ clan_tag: string; name: string }> = [];
    let laden = true;
    let ladefehler: string | null = null;
    let offen = false;

    let statusFilter = '';
    let typFilter = '';

    // Formular
    let formularOffen = false;
    let bearbeiteId: number | null = null;
    let speichert = false;
    let meldung: { art: 'ok' | 'fehler'; text: string } | null = null;
    let loeschKandidat: number | null = null;

    let fClan = '';
    let fTyp = 'cwlday';
    let fDauer = '0';
    let fAktion = 'infomessage';
    let fKanal = '';
    let fGrund = '';
    let fNachricht = '';
    let fSchwelle: string = '';
    let fSterne = '1';
    let fModus = '1';
    let fFreieVersuche = '0';
    let fCapitalPeak = '10';
    let fAndereDistrikte = '6';
    let fBeideBestrafen = '1';
    let fErzwingeKickpunkte = '2';
    let fPerfekterKrieg = '2';
    let fMinWins = '';
    let fMinCw = '8';
    let fListen = '0';
    let fOhneLeader = '0';

    const TYPEN: Array<[string, string]> = [
        ['cw', 'Clan War'],
        ['cwlday', 'CWL-Kampftag'],
        ['cwlend', 'CWL-Ende'],
        ['raid', 'Raid-Wochenende'],
        ['cs', 'Clan Games'],
        ['seasonend', 'Season-Ende'],
        ['fixtimeinterval', 'Fester Zeitpunkt'],
    ];

    const AKTIONEN: Array<[string, string]> = [
        ['infomessage', 'Erinnerung posten'],
        ['kickpoint', 'Kickpunkte vergeben'],
        ['custommessage', 'Eigene Nachricht'],
        ['cwdonator', 'CW-Spender ermitteln'],
        ['filler', 'Auffüller ermitteln'],
        ['raidfails', 'District-Analyse'],
        ['raidfails_kickpoint', 'District-Analyse (Kickpunkte)'],
        ['starfails', 'Schlechte Angriffe melden'],
        ['starfails_kickpoint', 'Schlechte Angriffe bestrafen'],
        ['cwcount', 'CW-Teilnahme zählen'],
        ['cwcount_kickpoint', 'CW-Teilnahme bestrafen'],
    ];

    /** Dieselben Einschränkungen, die der Bot durchsetzt — hier nur, damit die
     *  Auswahl gar nicht erst unmögliche Kombinationen anbietet. */
    function aktionenFuer(typ: string): Array<[string, string]> {
        return AKTIONEN.filter(([schluessel]) => {
            if (schluessel === 'cwdonator' || schluessel === 'filler')
                return typ === 'cw';
            if (
                schluessel === 'raidfails' ||
                schluessel === 'raidfails_kickpoint'
            )
                return typ === 'raid';
            if (
                schluessel === 'starfails' ||
                schluessel === 'starfails_kickpoint'
            )
                return typ === 'cw' || typ === 'cwlday';
            if (schluessel === 'cwcount' || schluessel === 'cwcount_kickpoint')
                return typ === 'seasonend';
            return true;
        });
    }

    $: verfuegbareAktionen = aktionenFuer(fTyp);
    // Wechselt der Typ, kann die gewählte Aktion unmöglich geworden sein.
    $: if (
        fTyp &&
        !verfuegbareAktionen.some(([schluessel]) => schluessel === fAktion)
    ) {
        fAktion = 'infomessage';
    }

    $: brauchtGrund = [
        'kickpoint',
        'raidfails_kickpoint',
        'starfails_kickpoint',
        'cwcount_kickpoint',
    ].includes(fAktion);
    $: brauchtAngriffe =
        fTyp === 'cw' && ['infomessage', 'kickpoint'].includes(fAktion);
    $: brauchtPunkte =
        fTyp === 'cs' && ['infomessage', 'kickpoint'].includes(fAktion);
    $: brauchtWins =
        fTyp === 'seasonend' && ['infomessage', 'kickpoint'].includes(fAktion);
    $: brauchtSterne = ['starfails', 'starfails_kickpoint'].includes(fAktion);
    $: brauchtDistrikte = ['raidfails', 'raidfails_kickpoint'].includes(
        fAktion,
    );
    $: brauchtCwAnzahl = ['cwcount', 'cwcount_kickpoint'].includes(fAktion);
    $: brauchtNachricht = fAktion === 'custommessage';
    $: brauchtSpenderOptionen = fAktion === 'cwdonator';
    $: perfekterKriegMoeglich =
        fAktion === 'kickpoint' && (fTyp === 'cw' || fTyp === 'cwlday');
    // Die District-Analyse wertet den Endstand aus und muss deshalb genau zum
    // Raid-Ende laufen. Der Bot lehnt alles andere ab.
    $: dauerFest = brauchtDistrikte;
    $: if (dauerFest) fDauer = '0';

    /** Clans, für die ein Event angelegt werden darf: der Clan selbst und seine
     *  Nebenclans — dort hängen die CWL-Erinnerungen. */
    $: waehlbareClans = [
        { tag: clanTag, name: 'Dieser Clan' },
        ...sideclans.map((s) => ({ tag: s.clan_tag, name: s.name })),
    ];

    $: gefiltert = events.filter(
        (e) =>
            (!statusFilter || e.status === statusFilter) &&
            (!typFilter || e.type === typFilter),
    );

    /** Nach Clan gruppiert, Hauptclan zuerst. */
    $: gruppen = (() => {
        const karte = new Map<string, LEvent[]>();
        for (const e of gefiltert) {
            const schluessel = e.clanTag;
            if (!karte.has(schluessel)) karte.set(schluessel, []);
            karte.get(schluessel)!.push(e);
        }
        const normal = (t: string) =>
            (t || '').replace(/^#/, '').toUpperCase();
        return [...karte.entries()].sort(([a], [b]) =>
            normal(a) === normal(clanTag)
                ? -1
                : normal(b) === normal(clanTag)
                  ? 1
                  : a.localeCompare(b),
        );
    })();

    function typName(typ: string | null): string {
        return TYPEN.find(([s]) => s === typ)?.[1] ?? (typ ?? 'unbekannt');
    }

    function aktionName(aktion: string | null): string {
        return (
            AKTIONEN.find(([s]) => s === aktion)?.[1] ?? (aktion ?? 'unbekannt')
        );
    }

    /** Kanalauswahl, nach Kategorie gruppiert. Der Discord hat über 400 Kanäle;
     *  eine flache Liste davon ist keine Auswahl mehr, sondern ein Suchspiel.
     *
     *  Ergänzt um die Kanäle, die die vorhandenen Events schon benutzen: ein
     *  archivierter Thread steht nicht im Cache des Bots, er kann aber sehr wohl
     *  dorthin schreiben — ohne diese Ergänzung fiele beim Ändern eines solchen
     *  Events der eigene Kanal aus der Auswahl. */
    $: kanalAuswahl = (() => {
        const bekannt = new Map<
            string,
            { name: string; kategorie: string }
        >();
        for (const k of kanaele) {
            bekannt.set(k.id, {
                name: (k.thread ? '🧵 ' : '# ') + k.name,
                kategorie: k.category ?? 'Ohne Kategorie',
            });
        }
        for (const e of events) {
            if (e.channelId && !bekannt.has(e.channelId)) {
                bekannt.set(e.channelId, {
                    name: e.channelName
                        ? '🧵 ' + e.channelName
                        : `Unbekannt (${e.channelId})`,
                    kategorie: 'Von Events benutzt',
                });
            }
        }
        const nachKategorie = new Map<
            string,
            Array<{ id: string; name: string }>
        >();
        for (const [id, k] of bekannt) {
            if (!nachKategorie.has(k.kategorie))
                nachKategorie.set(k.kategorie, []);
            nachKategorie.get(k.kategorie)!.push({ id, name: k.name });
        }
        for (const liste of nachKategorie.values()) {
            liste.sort((a, b) => a.name.localeCompare(b.name));
        }
        return [...nachKategorie.entries()].sort(([a], [b]) =>
            a.localeCompare(b),
        );
    })();

    async function laden_() {
        laden = true;
        ladefehler = null;
        try {
            const kodiert = encodeURIComponent(clanTag);
            const antwort = await fetch(
                `${apiBaseUrl}/api/coc/clans/${kodiert}/listeningevents`,
                { credentials: 'include' },
            );
            if (!antwort.ok) {
                throw new Error(
                    antwort.status === 403
                        ? 'Dafür brauchst du mindestens Vize.'
                        : `Die Events konnten nicht geladen werden (${antwort.status}).`,
                );
            }
            const daten = await antwort.json();
            events = Array.isArray(daten) ? daten : [];
        } catch (e) {
            ladefehler =
                e instanceof Error ? e.message : 'Unbekannter Fehler';
        } finally {
            laden = false;
        }
    }

    async function nebensachenLaden() {
        // Beides ist Beiwerk für das Formular; scheitert es, bleibt die Liste
        // trotzdem benutzbar und der Kanal muss eben als ID eingetragen werden.
        try {
            const a = await fetch(`${apiBaseUrl}/api/guild/channels`, {
                credentials: 'include',
            });
            if (a.ok) kanaele = await a.json();
        } catch {
            /* egal */
        }
        try {
            const a = await fetch(`${apiBaseUrl}/api/sideclans`, {
                credentials: 'include',
            });
            if (a.ok) {
                const alle = await a.json();
                const normal = (t: string) =>
                    (t || '').replace(/^#/, '').toUpperCase();
                sideclans = (Array.isArray(alle) ? alle : []).filter(
                    (s: any) => normal(s.belongs_to) === normal(clanTag),
                );
            }
        } catch {
            /* egal */
        }
    }

    function formularZuruecksetzen() {
        bearbeiteId = null;
        fClan = clanTag;
        fTyp = 'cwlday';
        fDauer = '0';
        fAktion = 'infomessage';
        fKanal = '';
        fGrund = '';
        fNachricht = '';
        fSchwelle = '';
        fSterne = '1';
        fModus = '1';
        fFreieVersuche = '0';
        fCapitalPeak = '10';
        fAndereDistrikte = '6';
        fBeideBestrafen = '1';
        fErzwingeKickpunkte = '2';
        fPerfekterKrieg = '2';
        fMinWins = '';
        fMinCw = '8';
        fListen = '0';
        fOhneLeader = '0';
    }

    function neuAnlegen() {
        formularZuruecksetzen();
        meldung = null;
        formularOffen = true;
    }

    function bearbeiten(e: LEvent) {
        formularZuruecksetzen();
        meldung = null;
        bearbeiteId = e.id;
        fClan = e.clanTag;
        fTyp = e.type ?? 'cwlday';
        fDauer = e.duration === -1 ? 'start' : String(e.duration);
        // Die gespeicherte Aktion kennt raidfails_kickpoint nicht — die
        // Unterscheidung steckt darin, ob ein Kickpunktgrund dranhängt.
        fAktion =
            e.actionType === 'raidfails' && e.kickpointReason
                ? 'raidfails_kickpoint'
                : (e.actionType ?? 'infomessage');
        fKanal = e.channelId ?? '';
        fGrund = e.kickpointReason ?? '';
        fNachricht = e.customMessage ?? '';
        const w = e.values ?? {};
        if (w.thresholdOrAttacks !== undefined)
            fSchwelle = String(w.thresholdOrAttacks);
        if (w.starCount !== undefined) fSterne = String(w.starCount);
        if (w.punishmentMode !== undefined) fModus = String(w.punishmentMode);
        if (w.capitalPeakMax !== undefined)
            fCapitalPeak = String(w.capitalPeakMax);
        if (w.otherDistrictsMax !== undefined)
            fAndereDistrikte = String(w.otherDistrictsMax);
        if (w.penalizeBoth !== undefined)
            fBeideBestrafen = String(w.penalizeBoth);
        if (w.raid_force_kickpoints !== undefined)
            fErzwingeKickpunkte = String(w.raid_force_kickpoints);
        if (w.ignore_perfect_war !== undefined)
            fPerfekterKrieg = String(w.ignore_perfect_war);
        if (w.wins_threshold !== undefined) fMinWins = String(w.wins_threshold);
        if (w.cw_min_count !== undefined) fMinCw = String(w.cw_min_count);
        if (w.starfails_free_hits !== undefined)
            fFreieVersuche = String(w.starfails_free_hits);
        if (w.useLists !== undefined) fListen = String(w.useLists);
        if (w.excludeLeaders !== undefined)
            fOhneLeader = String(w.excludeLeaders);
        formularOffen = true;
    }

    function zahlOderNichts(wert: string): number | undefined {
        const t = (wert ?? '').trim();
        if (t === '') return undefined;
        const n = Number(t);
        return Number.isFinite(n) ? n : undefined;
    }

    function koerperBauen(): Record<string, unknown> {
        const k: Record<string, unknown> = {
            clanTag: fClan,
            type: fTyp,
            duration: fDauer.trim() === '' ? 0 : fDauer.trim(),
            actionType: fAktion,
            channelId: fKanal,
        };
        if (brauchtGrund) k.kickpointReason = fGrund;
        if (brauchtNachricht) k.customMessage = fNachricht;
        if (brauchtAngriffe || brauchtPunkte)
            k.thresholdOrAttacks = zahlOderNichts(fSchwelle);
        if (brauchtSterne) {
            k.starCount = zahlOderNichts(fSterne);
            // In der CWL hat jeder genau einen Angriff, der Modus wirkt dort
            // nicht — der Bot speichert trotzdem einen Wert, also 1.
            k.punishmentMode = fTyp === 'cwlday' ? 1 : zahlOderNichts(fModus);
            k.starfailsFreeHits = zahlOderNichts(fFreieVersuche);
        }
        if (brauchtDistrikte) {
            k.capitalPeakMax = zahlOderNichts(fCapitalPeak);
            k.otherDistrictsMax = zahlOderNichts(fAndereDistrikte);
            k.penalizeBoth = zahlOderNichts(fBeideBestrafen);
            if (brauchtGrund)
                k.raidForceKickpoints = zahlOderNichts(fErzwingeKickpunkte);
        }
        if (perfekterKriegMoeglich)
            k.ignorePerfectWar = zahlOderNichts(fPerfekterKrieg);
        if (brauchtWins) {
            const w = zahlOderNichts(fMinWins);
            if (w !== undefined) k.winsThreshold = w;
        }
        if (brauchtCwAnzahl) k.cwMinCount = zahlOderNichts(fMinCw);
        if (brauchtSpenderOptionen) {
            k.useLists = zahlOderNichts(fListen);
            k.excludeLeaders = zahlOderNichts(fOhneLeader);
        }
        if (bearbeiteId !== null) k.id = bearbeiteId;
        return k;
    }

    async function aktion(
        name: 'add' | 'edit' | 'remove',
        koerper: Record<string, unknown>,
    ): Promise<{ ok: boolean; text: string }> {
        try {
            const antwort = await fetch(
                `${apiBaseUrl}/api/coc/manage/listeningevents/${name}`,
                {
                    method: 'POST',
                    credentials: 'include',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(koerper),
                },
            );
            let daten: any = null;
            try {
                daten = await antwort.json();
            } catch {
                // Manche Fehler kommen ohne Körper; dann zählt nur der Status.
            }
            if (antwort.ok) {
                return {
                    ok: true,
                    text: daten?.warning
                        ? `${daten?.message ?? 'Erledigt'} — ${daten.warning}`
                        : (daten?.message ?? 'Erledigt'),
                };
            }
            return {
                ok: false,
                text: verstaendlich(antwort.status, daten?.error ?? ''),
            };
        } catch {
            return {
                ok: false,
                text: 'Der Server war nicht erreichbar. Bitte noch einmal versuchen.',
            };
        }
    }

    /** Die Begründung des Bots ist die verbindliche und wird durchgereicht.
     *  Nur wo er gar keine liefert, steht hier ein eigener Satz. */
    function verstaendlich(status: number, grund: string): string {
        if (grund && grund.trim() !== '') return grund;
        if (status === 401)
            return 'Du bist nicht mehr angemeldet. Bitte neu einloggen.';
        if (status === 403) return 'Dafür fehlen dir die Rechte.';
        if (status === 404) return 'Das gibt es nicht (mehr).';
        if (status === 502)
            return 'Der Bot ist gerade nicht erreichbar. Nichts wurde geändert.';
        return `Unerwarteter Fehler (${status}).`;
    }

    async function speichern() {
        speichert = true;
        meldung = null;
        const ergebnis = await aktion(
            bearbeiteId === null ? 'add' : 'edit',
            koerperBauen(),
        );
        speichert = false;
        meldung = { art: ergebnis.ok ? 'ok' : 'fehler', text: ergebnis.text };
        if (ergebnis.ok) {
            formularOffen = false;
            await laden_();
        }
    }

    async function loeschen(id: number) {
        speichert = true;
        const ergebnis = await aktion('remove', { id });
        speichert = false;
        loeschKandidat = null;
        meldung = { art: ergebnis.ok ? 'ok' : 'fehler', text: ergebnis.text };
        if (ergebnis.ok) await laden_();
    }

    function aufklappen() {
        offen = !offen;
        if (offen && events.length === 0 && !ladefehler) {
            laden_();
            nebensachenLaden();
        }
    }

    onMount(() => {
        formularZuruecksetzen();
    });
</script>

<section class="le-section" class:light={theme === 'light'}>
    <button class="le-kopf" on:click={aufklappen} aria-expanded={offen}>
        <div class="le-titel">
            <h2>Automatische Erinnerungen</h2>
            <p class="le-unter">
                Was der Bot von selbst postet und bestraft — für diesen Clan und
                seine CWL-Ableger
            </p>
        </div>
        <span class="le-pfeil" class:auf={offen}>▾</span>
    </button>

    {#if offen}
        <div class="le-inhalt" transition:slide={{ duration: 200 }}>
            {#if laden}
                <p class="le-hinweis">Wird geladen …</p>
            {:else if ladefehler}
                <p class="le-fehler">{ladefehler}</p>
                <button class="le-knopf" on:click={laden_}
                    >Erneut versuchen</button
                >
            {:else}
                <div class="le-leiste">
                    <select bind:value={typFilter} aria-label="Typ filtern">
                        <option value="">Alle Typen</option>
                        {#each TYPEN as [schluessel, name]}
                            <option value={schluessel}>{name}</option>
                        {/each}
                    </select>
                    <select
                        bind:value={statusFilter}
                        aria-label="Status filtern"
                    >
                        <option value="">Jeder Status</option>
                        <option value="SCHEDULED">Geplant</option>
                        <option value="WAITING">Wartet auf Event</option>
                        <option value="FIRED">Feuerzeit vorbei</option>
                        <option value="MISSED">Verpasst</option>
                    </select>
                    <span class="le-zahl"
                        >{gefiltert.length} von {events.length}</span
                    >
                    <div class="le-luecke"></div>
                    {#if canManage}
                        <button class="le-knopf primaer" on:click={neuAnlegen}>
                            + Neu
                        </button>
                    {/if}
                    <button class="le-knopf" on:click={laden_}>
                        Aktualisieren
                    </button>
                </div>

                {#if meldung}
                    <p
                        class="le-meldung"
                        class:fehler={meldung.art === 'fehler'}
                        transition:fade={{ duration: 150 }}
                    >
                        {meldung.text}
                    </p>
                {/if}

                {#if formularOffen}
                    <div class="le-formular" transition:slide={{ duration: 200 }}>
                        <h3>
                            {bearbeiteId === null
                                ? 'Neues Event'
                                : `Event ${bearbeiteId} ändern`}
                        </h3>

                        <div class="le-felder">
                            <label>
                                Clan
                                <select bind:value={fClan}>
                                    {#each waehlbareClans as c}
                                        <option value={c.tag}
                                            >{c.name} ({c.tag})</option
                                        >
                                    {/each}
                                </select>
                            </label>

                            <label>
                                Worauf gehört wird
                                <select bind:value={fTyp}>
                                    {#each TYPEN as [schluessel, name]}
                                        <option value={schluessel}>{name}</option
                                        >
                                    {/each}
                                </select>
                            </label>

                            <label>
                                Was dann passiert
                                <select bind:value={fAktion}>
                                    {#each verfuegbareAktionen as [schluessel, name]}
                                        <option value={schluessel}>{name}</option
                                        >
                                    {/each}
                                </select>
                            </label>

                            <label>
                                Vorlauf
                                <input
                                    type="text"
                                    bind:value={fDauer}
                                    disabled={dauerFest}
                                    placeholder="0, 2h, 1d oder start"
                                />
                                <small>
                                    {#if dauerFest}
                                        Die District-Analyse läuft immer genau
                                        zum Raid-Ende.
                                    {:else}
                                        Wie lange <em>vor</em> dem Ende.
                                        <code>0</code> = genau zum Ende,
                                        <code>2h</code> = zwei Stunden vorher{fTyp ===
                                        'cw'
                                            ? ', '
                                            : '.'}{#if fTyp === 'cw'}<code
                                                >start</code
                                            > = beim Kriegsbeginn.{/if}
                                    {/if}
                                </small>
                            </label>

                            <label>
                                Kanal
                                <select bind:value={fKanal}>
                                    <option value=""
                                        >— bitte auswählen —</option
                                    >
                                    {#each kanalAuswahl as [kategorie, liste]}
                                        <optgroup label={kategorie}>
                                            {#each liste as k}
                                                <option value={k.id}
                                                    >{k.name}</option
                                                >
                                            {/each}
                                        </optgroup>
                                    {/each}
                                </select>
                                <small
                                    >🧵 sind Threads. Archivierte Threads stehen
                                    nur dann hier, wenn sie schon von einem Event
                                    benutzt werden.</small
                                >
                            </label>

                            {#if brauchtGrund}
                                <label>
                                    Kickpunktgrund
                                    <select bind:value={fGrund}>
                                        <option value=""
                                            >— bitte auswählen —</option
                                        >
                                        {#each kickpointReasons as g}
                                            <option value={g.name}
                                                >{g.name}{g.amount !== undefined
                                                    ? ` (${g.amount})`
                                                    : ''}</option
                                            >
                                        {/each}
                                    </select>
                                    {#if kickpointReasons.length === 0}
                                        <small
                                            >Dieser Clan hat noch keine Gründe
                                            hinterlegt.</small
                                        >
                                    {/if}
                                </label>
                            {/if}

                            {#if brauchtAngriffe}
                                <label>
                                    Benötigte Angriffe
                                    <select bind:value={fSchwelle}>
                                        <option value="1">1</option>
                                        <option value="2">2</option>
                                    </select>
                                </label>
                            {/if}

                            {#if brauchtPunkte}
                                <label>
                                    Punkte-Schwelle
                                    <input
                                        type="number"
                                        min="0"
                                        bind:value={fSchwelle}
                                        placeholder="z. B. 4000"
                                    />
                                </label>
                            {/if}

                            {#if brauchtWins}
                                <label>
                                    Minimum Wins
                                    <input
                                        type="number"
                                        min="0"
                                        bind:value={fMinWins}
                                        placeholder="leer = Clan-Einstellung"
                                    />
                                </label>
                            {/if}

                            {#if brauchtCwAnzahl}
                                <label>
                                    Minimum CWs pro Season
                                    <input
                                        type="number"
                                        min="1"
                                        max="999"
                                        bind:value={fMinCw}
                                    />
                                </label>
                            {/if}

                            {#if brauchtSterne}
                                <label>
                                    Angriffe mit genau … Sternen
                                    <select bind:value={fSterne}>
                                        <option value="0">0 ★</option>
                                        <option value="1">1 ★</option>
                                        <option value="2">2 ★</option>
                                    </select>
                                </label>
                                <label>
                                    Freie Fehlversuche
                                    <input
                                        type="number"
                                        min="0"
                                        max="20"
                                        bind:value={fFreieVersuche}
                                    />
                                    <small
                                        >0 = jeder schlechte Angriff zählt.</small
                                    >
                                </label>
                                {#if fTyp !== 'cwlday'}
                                    <label>
                                        Modus
                                        <select bind:value={fModus}>
                                            <option value="1"
                                                >Einmal pro Spieler</option
                                            >
                                            <option value="2"
                                                >Pro schlechtem Angriff</option
                                            >
                                            <option value="3"
                                                >Nur wenn alle schlecht</option
                                            >
                                        </select>
                                    </label>
                                {/if}
                            {/if}

                            {#if brauchtDistrikte}
                                <label>
                                    Maximale Angriffe auf Capital Peak
                                    <input
                                        type="number"
                                        min="1"
                                        bind:value={fCapitalPeak}
                                    />
                                </label>
                                <label>
                                    Maximale Angriffe auf andere Distrikte
                                    <input
                                        type="number"
                                        min="1"
                                        bind:value={fAndereDistrikte}
                                    />
                                </label>
                                {#if brauchtGrund}
                                    <label>
                                        Bei Gleichstand beide bestrafen
                                        <select bind:value={fBeideBestrafen}>
                                            <option value="1">Ja</option>
                                            <option value="2">Nein</option>
                                        </select>
                                    </label>
                                    <label>
                                        Kickpunkte trotz unbestätigter Daten
                                        <select
                                            bind:value={fErzwingeKickpunkte}
                                        >
                                            <option value="2"
                                                >Nein (Standard)</option
                                            >
                                            <option value="1">Ja</option>
                                        </select>
                                    </label>
                                {/if}
                            {/if}

                            {#if perfekterKriegMoeglich}
                                <label>
                                    Kickpunkte auch bei perfektem Krieg
                                    <select bind:value={fPerfekterKrieg}>
                                        <option value="2"
                                            >Nein (Standard)</option
                                        >
                                        <option value="1">Ja</option>
                                    </select>
                                </label>
                            {/if}

                            {#if brauchtSpenderOptionen}
                                <label>
                                    Listenbasierte Verteilung
                                    <select bind:value={fListen}>
                                        <option value="0">Nein</option>
                                        <option value="1">Ja</option>
                                    </select>
                                </label>
                                <label>
                                    Leader ausschließen
                                    <select bind:value={fOhneLeader}>
                                        <option value="0">Nein</option>
                                        <option value="1">Ja</option>
                                    </select>
                                </label>
                            {/if}

                            {#if brauchtNachricht}
                                <label class="breit">
                                    Nachricht
                                    <textarea
                                        rows="3"
                                        maxlength="2000"
                                        bind:value={fNachricht}
                                        placeholder="Was der Bot posten soll …"
                                    ></textarea>
                                </label>
                            {/if}
                        </div>

                        <div class="le-formular-fuss">
                            <button
                                class="le-knopf primaer"
                                on:click={speichern}
                                disabled={speichert}
                            >
                                {speichert
                                    ? 'Wird gespeichert …'
                                    : bearbeiteId === null
                                      ? 'Anlegen'
                                      : 'Speichern'}
                            </button>
                            <button
                                class="le-knopf"
                                on:click={() => (formularOffen = false)}
                                disabled={speichert}
                            >
                                Abbrechen
                            </button>
                        </div>
                    </div>
                {/if}

                {#if gefiltert.length === 0}
                    <p class="le-hinweis">
                        {events.length === 0
                            ? 'Für diesen Clan ist nichts eingerichtet.'
                            : 'Nichts passt zu den Filtern.'}
                    </p>
                {:else}
                    {#each gruppen as [tag, liste]}
                        <h3 class="le-gruppe">
                            {liste[0].clanName ?? tag}
                            <span class="le-tag">{tag}</span>
                            {#if !liste[0].clanTagValid}
                                <span class="le-warnung"
                                    >ungültiger Clan-Tag — diese Events können
                                    nie feuern</span
                                >
                            {/if}
                        </h3>
                        <div class="le-liste">
                            {#each liste as e (e.id)}
                                <article
                                    class="le-karte"
                                    class:verpasst={e.status === 'MISSED'}
                                >
                                    <header>
                                        <span
                                            class="le-status s-{e.status.toLowerCase()}"
                                            >{e.statusLabel}</span
                                        >
                                        <strong>{typName(e.type)}</strong>
                                        <span class="le-trenner">→</span>
                                        <span>{aktionName(e.actionType)}</span>
                                        <span class="le-id">#{e.id}</span>
                                    </header>

                                    <dl>
                                        <div>
                                            <dt>Vorlauf</dt>
                                            <dd>{e.durationText}</dd>
                                        </div>
                                        <div>
                                            <dt>Kanal</dt>
                                            <dd>
                                                {#if e.channelName}
                                                    {e.channelName}
                                                {:else}
                                                    <span class="le-warnung"
                                                        >unbekannt ({e.channelId})</span
                                                    >
                                                {/if}
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>Feuert</dt>
                                            <dd>{e.fireText}</dd>
                                        </div>
                                        <div>
                                            <dt>Zuletzt</dt>
                                            <dd>
                                                {e.lastRun.replace(/\*\*/g, '')}
                                            </dd>
                                        </div>
                                        {#if e.kickpointReason}
                                            <div>
                                                <dt>Kickpunktgrund</dt>
                                                <dd>
                                                    {e.kickpointReason}
                                                    {#if e.punishedViolation}
                                                        <small
                                                            >für {e.punishedViolation}</small
                                                        >
                                                    {/if}
                                                </dd>
                                            </div>
                                        {/if}
                                        {#if e.customMessage}
                                            <div class="breit">
                                                <dt>Nachricht</dt>
                                                <dd>{e.customMessage}</dd>
                                            </div>
                                        {/if}
                                    </dl>

                                    {#if canManage}
                                        <footer>
                                            {#if loeschKandidat === e.id}
                                                <span class="le-frage"
                                                    >Wirklich löschen?</span
                                                >
                                                <button
                                                    class="le-knopf gefahr"
                                                    on:click={() =>
                                                        loeschen(e.id)}
                                                    disabled={speichert}
                                                    >Ja, löschen</button
                                                >
                                                <button
                                                    class="le-knopf"
                                                    on:click={() =>
                                                        (loeschKandidat = null)}
                                                    >Abbrechen</button
                                                >
                                            {:else}
                                                <button
                                                    class="le-knopf"
                                                    on:click={() =>
                                                        bearbeiten(e)}
                                                    >Ändern</button
                                                >
                                                <button
                                                    class="le-knopf gefahr"
                                                    on:click={() =>
                                                        (loeschKandidat = e.id)}
                                                    >Löschen</button
                                                >
                                            {/if}
                                        </footer>
                                    {/if}
                                </article>
                            {/each}
                        </div>
                    {/each}
                {/if}
            {/if}
        </div>
    {/if}
</section>

<style>
    .le-section {
        --le-rand: rgba(255, 255, 255, 0.08);
        --le-karte: rgba(20, 20, 30, 0.7);
        --le-dim: rgba(255, 255, 255, 0.6);
        --le-akzent: #5865f2;
        margin-top: 2rem;
        border: 1px solid var(--le-rand);
        border-radius: 16px;
        background: var(--le-karte);
        overflow: hidden;
    }
    .le-section.light {
        --le-rand: rgba(0, 0, 0, 0.08);
        --le-karte: #ffffff;
        --le-dim: rgba(0, 0, 0, 0.6);
        color: #111;
    }

    .le-kopf {
        width: 100%;
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 1.25rem 1.5rem;
        background: none;
        border: none;
        color: inherit;
        cursor: pointer;
        text-align: left;
    }
    .le-titel h2 {
        margin: 0;
        font-size: 1.25rem;
    }
    .le-unter {
        margin: 0.25rem 0 0;
        font-size: 0.85rem;
        color: var(--le-dim);
    }
    .le-pfeil {
        margin-left: auto;
        transition: transform 0.2s;
        font-size: 1.2rem;
    }
    .le-pfeil.auf {
        transform: rotate(180deg);
    }

    .le-inhalt {
        padding: 0 1.5rem 1.5rem;
    }

    .le-leiste {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-wrap: wrap;
        margin-bottom: 1rem;
    }
    .le-luecke {
        flex: 1;
    }
    .le-zahl {
        font-size: 0.85rem;
        color: var(--le-dim);
    }

    .le-knopf {
        padding: 0.45rem 0.9rem;
        border-radius: 8px;
        border: 1px solid var(--le-rand);
        background: transparent;
        color: inherit;
        cursor: pointer;
        font-size: 0.85rem;
    }
    .le-knopf:hover:not(:disabled) {
        border-color: var(--le-akzent);
    }
    .le-knopf:disabled {
        opacity: 0.5;
        cursor: default;
    }
    .le-knopf.primaer {
        background: var(--le-akzent);
        border-color: var(--le-akzent);
        color: #fff;
    }
    .le-knopf.gefahr:hover:not(:disabled) {
        border-color: #e74c3c;
        color: #e74c3c;
    }

    .le-hinweis,
    .le-fehler {
        color: var(--le-dim);
        font-size: 0.9rem;
    }
    .le-fehler {
        color: #e74c3c;
    }

    .le-meldung {
        padding: 0.6rem 0.9rem;
        border-radius: 8px;
        border: 1px solid rgba(46, 204, 113, 0.4);
        background: rgba(46, 204, 113, 0.08);
        font-size: 0.9rem;
    }
    .le-meldung.fehler {
        border-color: rgba(231, 76, 60, 0.4);
        background: rgba(231, 76, 60, 0.08);
    }

    .le-formular {
        border: 1px solid var(--le-rand);
        border-radius: 12px;
        padding: 1rem;
        margin-bottom: 1.25rem;
    }
    .le-formular h3 {
        margin: 0 0 0.75rem;
        font-size: 1rem;
    }
    .le-felder {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
        gap: 0.75rem;
    }
    .le-felder label {
        display: flex;
        flex-direction: column;
        gap: 0.3rem;
        font-size: 0.85rem;
        color: var(--le-dim);
    }
    .le-felder label.breit {
        grid-column: 1 / -1;
    }
    .le-felder small {
        font-size: 0.75rem;
        line-height: 1.35;
    }
    .le-felder code {
        font-size: 0.75rem;
        padding: 0 0.2rem;
        border-radius: 3px;
        background: rgba(128, 128, 128, 0.2);
    }
    .le-formular-fuss {
        display: flex;
        gap: 0.5rem;
        margin-top: 1rem;
    }

    select,
    input,
    textarea {
        padding: 0.45rem 0.6rem;
        border-radius: 8px;
        border: 1px solid var(--le-rand);
        background: rgba(128, 128, 128, 0.1);
        color: inherit;
        font-size: 0.9rem;
        font-family: inherit;
    }
    select:disabled,
    input:disabled {
        opacity: 0.5;
    }

    .le-gruppe {
        margin: 1.5rem 0 0.75rem;
        font-size: 0.95rem;
        display: flex;
        align-items: baseline;
        gap: 0.5rem;
        flex-wrap: wrap;
    }
    .le-tag {
        font-size: 0.8rem;
        color: var(--le-dim);
        font-weight: normal;
    }
    .le-warnung {
        color: #e67e22;
        font-size: 0.8rem;
        font-weight: normal;
    }

    .le-liste {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
        gap: 0.75rem;
    }

    .le-karte {
        border: 1px solid var(--le-rand);
        border-radius: 12px;
        padding: 0.9rem;
    }
    .le-karte.verpasst {
        border-color: rgba(231, 76, 60, 0.5);
    }
    .le-karte header {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        flex-wrap: wrap;
        font-size: 0.9rem;
        margin-bottom: 0.6rem;
    }
    .le-trenner,
    .le-id {
        color: var(--le-dim);
    }
    .le-id {
        margin-left: auto;
        font-size: 0.75rem;
    }

    .le-status {
        font-size: 0.7rem;
        padding: 0.15rem 0.45rem;
        border-radius: 999px;
        border: 1px solid currentColor;
    }
    .s-scheduled {
        color: #2ecc71;
    }
    .s-waiting {
        color: #95a5a6;
    }
    .s-fired {
        color: #3498db;
    }
    .s-missed {
        color: #e74c3c;
    }

    .le-karte dl {
        margin: 0;
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
        gap: 0.4rem 0.75rem;
    }
    .le-karte dl div.breit {
        grid-column: 1 / -1;
    }
    .le-karte dt {
        font-size: 0.7rem;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        color: var(--le-dim);
    }
    .le-karte dd {
        margin: 0.1rem 0 0;
        font-size: 0.85rem;
    }
    .le-karte dd small {
        display: block;
        color: var(--le-dim);
        font-size: 0.75rem;
    }

    .le-karte footer {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-top: 0.8rem;
        flex-wrap: wrap;
    }
    .le-frage {
        font-size: 0.85rem;
        color: #e67e22;
    }
</style>
