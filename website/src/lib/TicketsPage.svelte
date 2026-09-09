<script lang="ts">
    import { onMount } from 'svelte';
    import { user, loading as authLoading, login } from './auth';
    import {
        ticketGuilds,
        ladeTicketGuilds,
        ladeStats,
        ladePanels,
        ladeTickets,
        ladeDetail,
        ladeLegacy,
        zeit,
        datum,
        groesse,
        erwaehnungen,
        type TicketGuild,
        type TicketStats,
        type TicketPanel,
        type TicketRow,
        type TicketDetail,
        type LegacyRow,
    } from './tickets';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string;

    const PRO_SEITE = 25;

    let guild: TicketGuild | null = null;
    let reiter: 'offen' | 'alle' | 'altdaten' = 'offen';

    let stats: TicketStats | null = null;
    let panels: TicketPanel[] = [];

    let zeilen: TicketRow[] = [];
    let gesamt = 0;
    let seite = 0;
    let suche = '';
    let sucheEntwurf = '';

    let altdaten: LegacyRow[] = [];
    let altdatenGesamt = 0;
    let altdatenSeite = 0;

    let detail: TicketDetail | null = null;
    let detailLaedt = false;

    let laedt = true;
    let fehler: string | null = null;

    // Der Status-Filter hängt am Reiter, nicht an einem eigenen Bedienelement:
    // "offen" ist der Alltagsblick, "alle" der Nachschlageblick.
    $: status = reiter === 'offen' ? 'open' : '';

    onMount(async () => {
        await ladeTicketGuilds(apiBaseUrl);
        const liste = $ticketGuilds ?? [];
        if (liste.length > 0) {
            await serverWechseln(liste[0]);
        } else {
            laedt = false;
        }
    });

    async function serverWechseln(g: TicketGuild) {
        guild = g;
        detail = null;
        seite = 0;
        altdatenSeite = 0;
        await alles();
    }

    async function alles() {
        if (!guild) return;
        laedt = true;
        fehler = null;
        try {
            const [s, p] = await Promise.all([
                ladeStats(apiBaseUrl, guild.id),
                ladePanels(apiBaseUrl, guild.id),
            ]);
            stats = s;
            panels = p.items;
            await liste();
        } catch (e) {
            fehler = e instanceof Error ? e.message : 'Unbekannter Fehler';
        } finally {
            laedt = false;
        }
    }

    async function liste() {
        if (!guild) return;
        try {
            const res = await ladeTickets(apiBaseUrl, guild.id, {
                status,
                q: suche,
                limit: PRO_SEITE,
                offset: seite * PRO_SEITE,
            });
            zeilen = res.items;
            gesamt = res.total;
        } catch (e) {
            fehler = e instanceof Error ? e.message : 'Unbekannter Fehler';
        }
    }

    async function listeAltdaten() {
        if (!guild) return;
        try {
            const res = await ladeLegacy(
                apiBaseUrl,
                guild.id,
                PRO_SEITE,
                altdatenSeite * PRO_SEITE,
            );
            altdaten = res.items;
            altdatenGesamt = res.total;
        } catch (e) {
            fehler = e instanceof Error ? e.message : 'Unbekannter Fehler';
        }
    }

    async function reiterWechseln(neu: typeof reiter) {
        reiter = neu;
        detail = null;
        if (neu === 'altdaten') {
            altdatenSeite = 0;
            await listeAltdaten();
        } else {
            seite = 0;
            await liste();
        }
    }

    async function suchen() {
        suche = sucheEntwurf.trim();
        seite = 0;
        await liste();
    }

    async function blaettern(richtung: number) {
        if (reiter === 'altdaten') {
            altdatenSeite = Math.max(0, altdatenSeite + richtung);
            await listeAltdaten();
        } else {
            seite = Math.max(0, seite + richtung);
            await liste();
        }
    }

    async function oeffnen(id: number) {
        detailLaedt = true;
        try {
            detail = await ladeDetail(apiBaseUrl, id);
        } catch (e) {
            fehler = e instanceof Error ? e.message : 'Unbekannter Fehler';
        } finally {
            detailLaedt = false;
        }
    }

    function discordLink(channelId: string): string {
        return `https://discord.com/channels/${guild?.id}/${channelId}`;
    }

    function altdatenLink(z: LegacyRow): string {
        return `https://discord.com/channels/${guild?.id}/${z.log_channel_id}/${z.log_message_id}`;
    }

    // Höchster Tageswert als Bezugsgröße der Balken. Mindestens 1, sonst
    // teilt eine ruhige Woche durch null.
    $: spitze = Math.max(1, ...(stats?.per_day ?? []).map((t) => t.count));

    $: seitenZahl =
        reiter === 'altdaten'
            ? Math.ceil(altdatenGesamt / PRO_SEITE)
            : Math.ceil(gesamt / PRO_SEITE);
    $: aktuelleSeite = reiter === 'altdaten' ? altdatenSeite : seite;
</script>

<div class="tickets-page" class:light={theme === 'light'}>
    {#if $authLoading}
        <div class="mitte"><div class="spinner"></div></div>
    {:else if !$user}
        <div class="mitte">
            <div class="karte-leer">
                <div class="icon">🔒</div>
                <h2>Anmeldung nötig</h2>
                <p>
                    Das Ticket-Dashboard zeigt dir genau die Bereiche, die du
                    auch in Discord siehst. Dafür musst du angemeldet sein.
                </p>
                <button class="knopf" on:click={login}>
                    Mit Discord anmelden
                </button>
            </div>
        </div>
    {:else if $ticketGuilds !== null && $ticketGuilds.length === 0}
        <div class="mitte">
            <div class="karte-leer">
                <div class="icon">🎫</div>
                <h2>Kein Zugriff</h2>
                <p>
                    Tickets sieht nur, wer in Discord die entsprechende
                    Team-Rolle trägt. Das Dashboard entscheidet das nicht selbst
                    — es fragt den Ticket-Bot, und der prüft deine echten
                    Rollen.
                </p>
            </div>
        </div>
    {:else}
        <header class="kopf">
            <div>
                <h1>Tickets</h1>
                <p class="unter">
                    Was du hier siehst, entspricht deinen Rollen auf Discord.
                </p>
            </div>

            {#if ($ticketGuilds ?? []).length > 1}
                <div class="server-wahl">
                    {#each $ticketGuilds ?? [] as g (g.id)}
                        <button
                            class="server"
                            class:aktiv={guild?.id === g.id}
                            on:click={() => serverWechseln(g)}
                        >
                            {#if g.icon}
                                <img src={g.icon} alt="" />
                            {/if}
                            {g.name}
                        </button>
                    {/each}
                </div>
            {/if}
        </header>

        {#if fehler}
            <div class="fehler">
                {fehler}
                <button class="klein" on:click={alles}>Erneut versuchen</button>
            </div>
        {/if}

        {#if laedt}
            <div class="mitte"><div class="spinner"></div></div>
        {:else if detail}
            <!-- Verlauf eines einzelnen Tickets -->
            <div class="verlauf">
                <button class="zurueck" on:click={() => (detail = null)}>
                    ← Zurück zur Liste
                </button>

                <div class="verlauf-kopf">
                    <h2>{detail.ticket.channel_name}</h2>
                    <dl>
                        <div>
                            <dt>Bereich</dt>
                            <dd>{detail.ticket.panel ?? '—'}</dd>
                        </div>
                        <div>
                            <dt>Eröffner</dt>
                            <dd>
                                {detail.ticket.owner_name ??
                                    detail.ticket.owner_id}
                            </dd>
                        </div>
                        <div>
                            <dt>Geöffnet</dt>
                            <dd>{zeit(detail.ticket.opened_at)}</dd>
                        </div>
                        <div>
                            <dt>Status</dt>
                            <dd>{detail.ticket.status}</dd>
                        </div>
                        {#if detail.ticket.closed_at && detail.ticket.closed_at !== 'null'}
                            <div>
                                <dt>Geschlossen</dt>
                                <dd>{zeit(detail.ticket.closed_at)}</dd>
                            </div>
                        {/if}
                        <div>
                            <dt>Nachrichten</dt>
                            <dd>{detail.messages.length}</dd>
                        </div>
                    </dl>
                    <a
                        class="discord"
                        href={discordLink(detail.ticket.channel_id)}
                        target="_blank"
                        rel="noreferrer">In Discord öffnen ↗</a
                    >
                </div>

                {#if detail.messages.length === 0}
                    <p class="hinweis">
                        Für dieses Ticket wurde kein Verlauf aufgezeichnet. Bei
                        Tickets, die vor der Umstellung geschlossen wurden, ist
                        das normal — deren Verlauf liegt weiterhin als
                        Ticket-Tool-Transcript im Log-Kanal.
                    </p>
                {/if}

                <div class="nachrichten">
                    {#each detail.messages as m, i (i)}
                        <article class="msg" class:geloescht={m.deleted}>
                            <div class="msg-kopf">
                                <span class="autor">{m.author_name}</span>
                                {#if m.bot}<span class="bot">Bot</span>{/if}
                                <time>{zeit(m.sent_at)}</time>
                                {#if m.edited}<span class="marke"
                                        >bearbeitet</span
                                    >{/if}
                                {#if m.deleted}<span class="marke rot"
                                        >später gelöscht</span
                                    >{/if}
                            </div>
                            {#if m.content}
                                <p class="text">{erwaehnungen(m.content)}</p>
                            {/if}
                            {#each m.attachments as a (a.filename + a.url)}
                                {#if a.url && a.content_type?.startsWith('image/')}
                                    <a
                                        href={a.url}
                                        target="_blank"
                                        rel="noreferrer"
                                    >
                                        <img
                                            class="anhang-bild"
                                            src={a.url}
                                            alt={a.filename}
                                            loading="lazy"
                                        />
                                    </a>
                                {:else if a.url}
                                    <a
                                        class="anhang"
                                        href={a.url}
                                        target="_blank"
                                        rel="noreferrer"
                                    >
                                        {a.filename}
                                        <span>{groesse(a.size)}</span>
                                    </a>
                                {:else}
                                    <span class="anhang fehlt">
                                        {a.filename} — Link nicht abrufbar
                                    </span>
                                {/if}
                            {/each}
                        </article>
                    {/each}
                </div>
            </div>
        {:else}
            <!-- Übersicht -->
            {#if stats?.visible}
                <div class="kennzahlen">
                    <div class="zahl">
                        <span class="wert">{stats.open}</span>
                        <span class="name">offen</span>
                    </div>
                    <div class="zahl" class:warnung={(stats.unclaimed ?? 0) > 0}>
                        <span class="wert">{stats.unclaimed}</span>
                        <span class="name">unbeansprucht</span>
                    </div>
                    <div class="zahl">
                        <span class="wert">{stats.closed}</span>
                        <span class="name">geschlossen</span>
                    </div>
                    <div class="zahl">
                        <span class="wert">{stats.legacy}</span>
                        <span class="name">aus Ticket Tool</span>
                    </div>
                </div>

                {#if (stats.per_day ?? []).length > 0}
                    <section class="block">
                        <h3>Eröffnet in den letzten 30 Tagen</h3>
                        <div class="kurve">
                            {#each stats.per_day ?? [] as tag (tag.day)}
                                <div
                                    class="balken"
                                    title="{tag.day}: {tag.count}"
                                >
                                    <div
                                        class="fuellung"
                                        style="height: {(tag.count / spitze) *
                                            100}%"
                                    ></div>
                                    <span class="tag">{datum(tag.day)}</span>
                                </div>
                            {/each}
                        </div>
                    </section>
                {/if}
            {/if}

            {#if panels.length > 0}
                <section class="block">
                    <h3>Bereiche</h3>
                    <div class="panel-gitter">
                        {#each panels as p (p.id)}
                            <div class="panel" class:still={!p.active}>
                                <span class="panel-name">{p.name}</span>
                                <span class="panel-zahlen">
                                    <b>{p.open}</b> offen · {p.total} gesamt
                                </span>
                                {#if !p.active}
                                    <span class="panel-marke">stillgelegt</span>
                                {/if}
                            </div>
                        {/each}
                    </div>
                </section>
            {/if}

            <section class="block">
                <div class="reiter">
                    <button
                        class:aktiv={reiter === 'offen'}
                        on:click={() => reiterWechseln('offen')}>Offen</button
                    >
                    <button
                        class:aktiv={reiter === 'alle'}
                        on:click={() => reiterWechseln('alle')}>Alle</button
                    >
                    <button
                        class:aktiv={reiter === 'altdaten'}
                        on:click={() => reiterWechseln('altdaten')}
                        >Aus Ticket Tool</button
                    >

                    {#if reiter !== 'altdaten'}
                        <form class="suche" on:submit|preventDefault={suchen}>
                            <input
                                type="search"
                                placeholder="Kanalname oder Discord-ID"
                                bind:value={sucheEntwurf}
                            />
                            <button type="submit">Suchen</button>
                        </form>
                    {/if}
                </div>

                {#if reiter === 'altdaten'}
                    <p class="hinweis">
                        Diese Transcripts stammen aus Ticket Tool und liegen
                        weiterhin als Datei im jeweiligen Log-Kanal. Der Bot
                        führt nur ein Verzeichnis darüber — der Link geht direkt
                        zur Nachricht in Discord.
                    </p>
                    <table>
                        <thead>
                            <tr>
                                <th>Ticket</th>
                                <th>Bereich</th>
                                <th>Geschlossen</th>
                                <th>Größe</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each altdaten as z (z.log_message_id)}
                                <tr>
                                    <td class="stark">{z.ticket_name}</td>
                                    <td>{z.panel ?? '—'}</td>
                                    <td>{zeit(z.closed_at)}</td>
                                    <td>{groesse(z.size)}</td>
                                    <td
                                        ><a
                                            href={altdatenLink(z)}
                                            target="_blank"
                                            rel="noreferrer">öffnen ↗</a
                                        ></td
                                    >
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                    {#if altdaten.length === 0}
                        <p class="hinweis">Nichts gefunden.</p>
                    {/if}
                {:else}
                    <table>
                        <thead>
                            <tr>
                                <th>Ticket</th>
                                <th>Bereich</th>
                                <th>Eröffner</th>
                                <th>Betreut von</th>
                                <th>Geöffnet</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each zeilen as z (z.id)}
                                <tr
                                    class="klickbar"
                                    on:click={() => oeffnen(z.id)}
                                >
                                    <td class="stark">
                                        {z.channel_name}
                                        {#if z.status !== 'open'}
                                            <span class="marke"
                                                >{z.status === 'deleted'
                                                    ? 'gelöscht'
                                                    : 'geschlossen'}</span
                                            >
                                        {/if}
                                    </td>
                                    <td>{z.panel ?? '—'}</td>
                                    <td>{z.owner_name ?? z.owner_id}</td>
                                    <td>
                                        {#if z.claimed_by}
                                            {z.claimed_by_name ?? z.claimed_by}
                                        {:else}
                                            <span class="offen">niemand</span>
                                        {/if}
                                    </td>
                                    <td>{zeit(z.opened_at)}</td>
                                    <td>
                                        <a
                                            href={discordLink(z.channel_id)}
                                            target="_blank"
                                            rel="noreferrer"
                                            on:click|stopPropagation
                                            >Discord ↗</a
                                        >
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                    {#if zeilen.length === 0}
                        <p class="hinweis">
                            {suche
                                ? 'Nichts gefunden.'
                                : 'Hier ist gerade nichts offen.'}
                        </p>
                    {/if}
                {/if}

                {#if seitenZahl > 1}
                    <div class="blaettern">
                        <button
                            disabled={aktuelleSeite === 0}
                            on:click={() => blaettern(-1)}>← Zurück</button
                        >
                        <span>Seite {aktuelleSeite + 1} von {seitenZahl}</span>
                        <button
                            disabled={aktuelleSeite + 1 >= seitenZahl}
                            on:click={() => blaettern(1)}>Weiter →</button
                        >
                    </div>
                {/if}
            </section>
        {/if}

        {#if detailLaedt}
            <div class="mitte"><div class="spinner"></div></div>
        {/if}
    {/if}
</div>

<style>
    .tickets-page {
        min-height: 100vh;
        padding: 2rem 1.5rem 4rem;
        max-width: 1200px;
        margin: 0 auto;
        color: #dcddde;
    }
    .tickets-page.light {
        color: #2e3338;
    }

    .mitte {
        display: flex;
        justify-content: center;
        align-items: center;
        min-height: 300px;
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid rgba(88, 101, 242, 0.15);
        border-left-color: #5865f2;
        border-radius: 50%;
        animation: dreh 1s linear infinite;
    }
    @keyframes dreh {
        to {
            transform: rotate(360deg);
        }
    }

    .karte-leer {
        text-align: center;
        max-width: 460px;
        padding: 2.5rem;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 14px;
    }
    .tickets-page.light .karte-leer {
        background: #fff;
        border-color: #e2e8f0;
    }
    .karte-leer .icon {
        font-size: 2.5rem;
        margin-bottom: 0.75rem;
    }
    .karte-leer h2 {
        margin: 0 0 0.5rem;
        font-size: 1.35rem;
    }
    .karte-leer p {
        color: #9aa0a6;
        line-height: 1.55;
        margin: 0 0 1.25rem;
    }

    .knopf,
    .klein {
        background: #5865f2;
        color: #fff;
        border: none;
        border-radius: 8px;
        padding: 0.7rem 1.4rem;
        font-weight: 600;
        cursor: pointer;
    }
    .knopf:hover,
    .klein:hover {
        background: #4752c4;
    }
    .klein {
        padding: 0.35rem 0.8rem;
        font-size: 0.85rem;
        margin-left: 0.75rem;
    }

    .kopf {
        display: flex;
        flex-wrap: wrap;
        gap: 1rem;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 1.75rem;
    }
    .kopf h1 {
        margin: 0;
        font-size: 2rem;
    }
    .unter {
        margin: 0.35rem 0 0;
        color: #9aa0a6;
        font-size: 0.92rem;
    }

    .server-wahl {
        display: flex;
        gap: 0.5rem;
        flex-wrap: wrap;
    }
    .server {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 0.9rem;
        border-radius: 10px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        background: rgba(255, 255, 255, 0.03);
        color: inherit;
        cursor: pointer;
        font-size: 0.9rem;
    }
    .server img {
        width: 22px;
        height: 22px;
        border-radius: 50%;
    }
    .server.aktiv {
        border-color: #5865f2;
        background: rgba(88, 101, 242, 0.15);
    }
    .tickets-page.light .server {
        background: #fff;
        border-color: #e2e8f0;
    }

    .fehler {
        background: rgba(237, 66, 69, 0.12);
        border: 1px solid rgba(237, 66, 69, 0.4);
        color: #ff9a9c;
        padding: 0.85rem 1rem;
        border-radius: 10px;
        margin-bottom: 1.5rem;
    }

    .kennzahlen {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
        gap: 1rem;
        margin-bottom: 1.75rem;
    }
    .zahl {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 12px;
        padding: 1.1rem 1.25rem;
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }
    .tickets-page.light .zahl {
        background: #fff;
        border-color: #e2e8f0;
    }
    .zahl.warnung {
        border-color: rgba(250, 166, 26, 0.5);
    }
    .zahl .wert {
        font-size: 1.9rem;
        font-weight: 700;
        line-height: 1;
    }
    .zahl .name {
        color: #9aa0a6;
        font-size: 0.85rem;
    }

    .block {
        margin-bottom: 2rem;
    }
    .block h3 {
        margin: 0 0 0.85rem;
        font-size: 1.05rem;
        color: #b9bbbe;
        font-weight: 600;
    }
    .tickets-page.light .block h3 {
        color: #475569;
    }

    .kurve {
        display: flex;
        align-items: flex-end;
        gap: 3px;
        height: 130px;
        padding-bottom: 1.4rem;
        position: relative;
    }
    .balken {
        flex: 1;
        height: 100%;
        display: flex;
        align-items: flex-end;
        position: relative;
        min-width: 6px;
    }
    .fuellung {
        width: 100%;
        background: linear-gradient(180deg, #5865f2, #3c45a5);
        border-radius: 3px 3px 0 0;
        min-height: 2px;
    }
    .balken .tag {
        position: absolute;
        bottom: -1.25rem;
        left: 50%;
        transform: translateX(-50%);
        font-size: 0.62rem;
        color: #72767d;
        white-space: nowrap;
    }
    /* Bei 30 Balken überlappen die Beschriftungen — nur jede fünfte zeigen. */
    .balken:not(:nth-child(5n + 1)) .tag {
        display: none;
    }

    .panel-gitter {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
        gap: 0.75rem;
    }
    .panel {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 10px;
        padding: 0.8rem 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }
    .tickets-page.light .panel {
        background: #fff;
        border-color: #e2e8f0;
    }
    .panel.still {
        opacity: 0.55;
    }
    .panel-name {
        font-weight: 600;
    }
    .panel-zahlen {
        font-size: 0.85rem;
        color: #9aa0a6;
    }
    .panel-marke {
        font-size: 0.7rem;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        color: #72767d;
    }

    .reiter {
        display: flex;
        gap: 0.5rem;
        align-items: center;
        flex-wrap: wrap;
        margin-bottom: 1rem;
    }
    .reiter > button {
        padding: 0.5rem 1rem;
        border-radius: 8px;
        border: 1px solid transparent;
        background: rgba(255, 255, 255, 0.04);
        color: #b9bbbe;
        cursor: pointer;
        font-size: 0.9rem;
    }
    .reiter > button.aktiv {
        background: rgba(88, 101, 242, 0.18);
        border-color: #5865f2;
        color: #fff;
    }
    .tickets-page.light .reiter > button.aktiv {
        color: #1e293b;
    }

    .suche {
        margin-left: auto;
        display: flex;
        gap: 0.4rem;
    }
    .suche input {
        padding: 0.5rem 0.75rem;
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.12);
        background: rgba(0, 0, 0, 0.25);
        color: inherit;
        min-width: 220px;
    }
    .tickets-page.light .suche input {
        background: #fff;
        border-color: #cbd5e1;
    }
    .suche button {
        padding: 0.5rem 0.9rem;
        border-radius: 8px;
        border: none;
        background: #5865f2;
        color: #fff;
        cursor: pointer;
    }

    table {
        width: 100%;
        border-collapse: collapse;
        font-size: 0.9rem;
    }
    th {
        text-align: left;
        padding: 0.6rem 0.75rem;
        color: #72767d;
        font-weight: 600;
        font-size: 0.78rem;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    }
    td {
        padding: 0.7rem 0.75rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }
    .tickets-page.light td {
        border-bottom-color: #eef2f7;
    }
    tr.klickbar {
        cursor: pointer;
    }
    tr.klickbar:hover td {
        background: rgba(88, 101, 242, 0.08);
    }
    .stark {
        font-weight: 600;
    }
    .offen {
        color: #faa61a;
    }
    td a,
    .discord {
        color: #5865f2;
        text-decoration: none;
    }
    td a:hover,
    .discord:hover {
        text-decoration: underline;
    }

    .marke {
        font-size: 0.68rem;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        color: #72767d;
        margin-left: 0.4rem;
    }
    .marke.rot {
        color: #ed4245;
    }

    .hinweis {
        color: #9aa0a6;
        font-size: 0.9rem;
        line-height: 1.55;
        margin: 1rem 0;
    }

    .blaettern {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 1rem;
        margin-top: 1.25rem;
        font-size: 0.85rem;
        color: #9aa0a6;
    }
    .blaettern button {
        padding: 0.4rem 0.9rem;
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.12);
        background: transparent;
        color: inherit;
        cursor: pointer;
    }
    .blaettern button:disabled {
        opacity: 0.35;
        cursor: default;
    }

    .zurueck {
        background: none;
        border: none;
        color: #5865f2;
        cursor: pointer;
        padding: 0;
        font-size: 0.9rem;
        margin-bottom: 1rem;
    }

    .verlauf-kopf {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 12px;
        padding: 1.25rem;
        margin-bottom: 1.5rem;
    }
    .tickets-page.light .verlauf-kopf {
        background: #fff;
        border-color: #e2e8f0;
    }
    .verlauf-kopf h2 {
        margin: 0 0 0.9rem;
        font-size: 1.3rem;
    }
    .verlauf-kopf dl {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
        gap: 0.85rem;
        margin: 0 0 1rem;
    }
    .verlauf-kopf dt {
        color: #72767d;
        font-size: 0.72rem;
        text-transform: uppercase;
        letter-spacing: 0.04em;
    }
    .verlauf-kopf dd {
        margin: 0.15rem 0 0;
        font-weight: 500;
    }

    .nachrichten {
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
    }
    .msg {
        padding: 0.75rem 0.9rem;
        border-radius: 10px;
        background: rgba(255, 255, 255, 0.02);
        border-left: 3px solid rgba(88, 101, 242, 0.5);
    }
    .tickets-page.light .msg {
        background: #fff;
        border: 1px solid #eef2f7;
        border-left: 3px solid rgba(88, 101, 242, 0.5);
    }
    .msg.geloescht {
        opacity: 0.6;
        border-left-color: #ed4245;
    }
    .msg-kopf {
        display: flex;
        align-items: baseline;
        gap: 0.5rem;
        flex-wrap: wrap;
        margin-bottom: 0.3rem;
    }
    .autor {
        font-weight: 600;
    }
    .bot {
        font-size: 0.62rem;
        background: #5865f2;
        color: #fff;
        padding: 0.05rem 0.3rem;
        border-radius: 4px;
        text-transform: uppercase;
    }
    .msg-kopf time {
        font-size: 0.75rem;
        color: #72767d;
    }
    .text {
        margin: 0;
        white-space: pre-wrap;
        word-break: break-word;
        line-height: 1.5;
    }

    .anhang-bild {
        max-width: min(100%, 420px);
        border-radius: 8px;
        margin-top: 0.5rem;
        display: block;
    }
    .anhang {
        display: inline-block;
        margin-top: 0.5rem;
        padding: 0.4rem 0.7rem;
        border-radius: 8px;
        background: rgba(88, 101, 242, 0.1);
        color: #5865f2;
        font-size: 0.85rem;
        text-decoration: none;
    }
    .anhang span {
        color: #72767d;
        margin-left: 0.4rem;
    }
    .anhang.fehlt {
        background: rgba(255, 255, 255, 0.04);
        color: #72767d;
    }

    @media (max-width: 720px) {
        .suche {
            margin-left: 0;
            width: 100%;
        }
        .suche input {
            flex: 1;
            min-width: 0;
        }
        table {
            font-size: 0.82rem;
        }
        th:nth-child(4),
        td:nth-child(4) {
            display: none;
        }
    }
</style>
