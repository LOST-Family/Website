/**
 * Zugriff auf das Ticket-Dashboard.
 *
 * Alle Anfragen gehen an das Website-Backend, das sie an den Ticket-Bot
 * weiterreicht. WER etwas sehen darf, entscheidet dort der Bot anhand der
 * echten Discord-Rollen — nicht diese Datei und nicht die Rollenhierarchie der
 * Website. Deshalb gibt es hier bewusst keine Rollenprüfung: Was zurückkommt,
 * ist bereits gefiltert.
 */
import { writable } from 'svelte/store';

export interface TicketGuild {
    id: string;
    name: string;
    icon: string | null;
    panels: number;
}

export interface TicketPanel {
    id: number;
    name: string;
    active: boolean;
    counter: number;
    open: number;
    total: number;
}

export interface TicketRow {
    id: number;
    panel: string | null;
    number: number;
    channel_id: string;
    channel_name: string;
    owner_id: string;
    owner_name: string | null;
    claimed_by: string | null;
    claimed_by_name: string | null;
    status: string;
    opened_at: string;
    closed_at: string;
    closed_by: string | null;
}

export interface TicketStats {
    visible: boolean;
    panels?: number;
    open?: number;
    closed?: number;
    unclaimed?: number;
    legacy?: number;
    per_day?: { day: string; count: number }[];
}

export interface TicketAttachment {
    filename: string;
    content_type: string | null;
    size: number;
    url: string | null;
}

export interface TicketMessage {
    author_id: string;
    author_name: string;
    bot: boolean;
    content: string | null;
    sent_at: string;
    edited: boolean;
    deleted: boolean;
    attachments: TicketAttachment[];
}

export interface TicketDetail {
    ticket: TicketRow & {
        close_reason: string | null;
        members: string[];
    };
    messages: TicketMessage[];
}

export interface LegacyRow {
    ticket_name: string;
    panel: string | null;
    owner_id: string | null;
    closed_by: string | null;
    closed_at: string;
    size: number;
    log_channel_id: string;
    log_message_id: string;
}

interface Liste<T> {
    items: T[];
    total: number;
}

/**
 * Auf welchen Servern der eingeloggte Nutzer Tickets sehen darf.
 *
 * `null` heißt "noch nicht gefragt", ein leeres Array heißt "nirgends" — die
 * Unterscheidung braucht die Navigation, damit der Eintrag nicht kurz
 * aufblitzt und wieder verschwindet.
 */
export const ticketGuilds = writable<TicketGuild[] | null>(null);

let geladen = false;

/**
 * Darf der Nutzer irgendwo die Ticket-Verwaltung öffnen? Für die Navigation:
 * ein Website-Admin ohne Discord-Rolle sieht keine Tickets, soll den Reiter
 * „Verwaltung“ aber trotzdem finden.
 */
export const verwaltungVerfuegbar = writable(false);

async function hole<T>(apiBaseUrl: string, pfad: string): Promise<T> {
    const res = await fetch(`${apiBaseUrl}${pfad}`, {
        credentials: 'include',
    });
    if (!res.ok) {
        if (res.status === 401) throw new Error('Nicht angemeldet');
        if (res.status === 503)
            throw new Error('Das Ticketsystem ist gerade nicht erreichbar');
        const koerper = await res.json().catch(() => null);
        throw new Error(koerper?.error ?? `HTTP ${res.status}`);
    }
    return res.json();
}

/** Einmal je Sitzung. Die Navigation fragt bei jedem Seitenwechsel. */
export async function ladeTicketGuilds(apiBaseUrl: string): Promise<void> {
    if (geladen) return;
    geladen = true;
    try {
        const liste = await hole<Liste<TicketGuild>>(
            apiBaseUrl,
            '/api/tickets/guilds',
        );
        ticketGuilds.set(liste.items);
        if (liste.items.length === 0) {
            const vw = await ladeVerwaltungGuilds(apiBaseUrl);
            verwaltungVerfuegbar.set(vw.length > 0);
        }
    } catch {
        // Kein Zugriff, nicht eingeloggt, Bot aus — in allen Fällen gibt es
        // für diesen Nutzer kein Dashboard, und ein Fehler in der Navigation
        // hilft ihm nicht weiter.
        ticketGuilds.set([]);
    }
}

export function vergissTicketGuilds(): void {
    geladen = false;
    ticketGuilds.set(null);
    verwaltungVerfuegbar.set(false);
}

export const ladeStats = (apiBaseUrl: string, guild: string) =>
    hole<TicketStats>(apiBaseUrl, `/api/tickets/stats?guild=${guild}`);

export const ladePanels = (apiBaseUrl: string, guild: string) =>
    hole<Liste<TicketPanel>>(apiBaseUrl, `/api/tickets/panels?guild=${guild}`);

export const ladeDetail = (apiBaseUrl: string, id: number) =>
    hole<TicketDetail>(apiBaseUrl, `/api/tickets/${id}`);

export const ladeLegacy = (
    apiBaseUrl: string,
    guild: string,
    limit: number,
    offset: number,
) =>
    hole<Liste<LegacyRow>>(
        apiBaseUrl,
        `/api/tickets/legacy?guild=${guild}&limit=${limit}&offset=${offset}`,
    );

export function ladeTickets(
    apiBaseUrl: string,
    guild: string,
    opts: { status?: string; q?: string; limit: number; offset: number },
) {
    const p = new URLSearchParams({
        guild,
        limit: String(opts.limit),
        offset: String(opts.offset),
    });
    if (opts.status) p.set('status', opts.status);
    if (opts.q) p.set('q', opts.q);
    return hole<Liste<TicketRow>>(apiBaseUrl, `/api/tickets?${p}`);
}

// ---------------------------------------------------------------------------
// Verwaltung (Panels, Menüs, Sperrliste)
//
// Auch hier prüft der Bot selbst, wer verwalten darf. Die Website reicht nur
// durch; eine Aktion, die hier angeboten wird, kann dort trotzdem mit 403
// abgelehnt werden — dann steht der Grund im `error`-Feld.
// ---------------------------------------------------------------------------

export interface VerwaltungGuild {
    id: string;
    name: string;
    icon: string | null;
}

export interface VwFrage {
    label: string;
    placeholder: string | null;
    lang: boolean;
    pflicht: boolean;
    max: number | null;
}

export interface VwPanel {
    id: number;
    name: string;
    active: boolean;
    category_opened: string | null;
    category_closed: string | null;
    name_pattern_open: string | null;
    name_pattern_closed: string | null;
    counter: number;
    counter_padding: number;
    welcome_text: string | null;
    welcome_embed_title: string | null;
    welcome_embed_text: string | null;
    log_channel_id: string | null;
    max_open_per_user: number;
    cooldown_seconds: number;
    close_on_owner_leave: boolean;
    owner_leave_message: string | null;
    dm_on_open: boolean;
    dm_on_close: boolean;
    next_name: string | null;
    support: string[];
    ping: string[];
    fragen: VwFrage[];
    open: number;
    total: number;
}

export interface VwMenuEintrag {
    panel_id: number;
    label: string | null;
    description: string | null;
    emoji: string | null;
    button_style: number;
    position: number;
}

export interface VwMenu {
    id: number;
    channel_id: string;
    channel_name: string | null;
    message_id?: string | null;
    posted: boolean;
    style: 'knoepfe' | 'dropdown';
    embed_title: string | null;
    embed_text: string | null;
    embed_color: number | null;
    placeholder: string | null;
    entries: VwMenuEintrag[];
}

export interface VwRolle {
    id: string;
    name: string;
    color: number;
    managed: boolean;
}

export interface VwKategorie {
    id: string;
    name: string;
}

export interface VwKanal {
    id: string;
    name: string;
    kategorie_id: string | null;
    kategorie: string | null;
}

export interface VwSperre {
    subject_id: string;
    is_role: boolean;
    reason: string | null;
    added_by: string | null;
    added_at: string | null;
    name: string | null;
}

export interface VwFeld {
    name: string;
    art: 'TEXT' | 'NUMBER' | 'BOOL';
}

export interface VerwaltungUebersicht {
    guild: VerwaltungGuild;
    config: { limit: number; panel_color: number; ticket_color: number };
    panels: VwPanel[];
    menus: VwMenu[];
    rollen: VwRolle[];
    kategorien: VwKategorie[];
    kanaele: VwKanal[];
    sperrliste: VwSperre[];
    felder: VwFeld[];
}

/** Server, die der Nutzer verwalten darf. Jeder Fehler heißt: keine. */
export async function ladeVerwaltungGuilds(
    apiBaseUrl: string,
): Promise<VerwaltungGuild[]> {
    try {
        const liste = await hole<Liste<VerwaltungGuild>>(
            apiBaseUrl,
            '/api/tickets/verwaltung/guilds',
        );
        return liste.items ?? [];
    } catch {
        return [];
    }
}

export const ladeVerwaltung = (apiBaseUrl: string, guild: string) =>
    hole<VerwaltungUebersicht>(
        apiBaseUrl,
        `/api/tickets/verwaltung?guild=${encodeURIComponent(guild)}`,
    );

/**
 * Eine Verwaltungsaktion ausführen. `koerper` muss `guild` enthalten.
 * Bei Fehlern wird der Text des Bots unverändert als Error geworfen — er ist
 * für Menschen formuliert und soll so angezeigt werden.
 */
export async function verwaltungAktion(
    apiBaseUrl: string,
    aktion: string,
    koerper: Record<string, unknown>,
): Promise<{ ok: boolean; meldung: string }> {
    const res = await fetch(`${apiBaseUrl}/api/tickets/verwaltung/${aktion}`, {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(koerper),
    });
    const antwort = await res.json().catch(() => null);
    if (!res.ok) {
        if (antwort?.error) throw new Error(antwort.error);
        if (res.status === 401) throw new Error('Nicht angemeldet');
        if (res.status === 503)
            throw new Error('Das Ticketsystem ist gerade nicht erreichbar');
        throw new Error(`HTTP ${res.status}`);
    }
    return {
        ok: antwort?.ok ?? true,
        meldung: antwort?.meldung ?? 'Gespeichert.',
    };
}

/**
 * Discord-Farbwert als #rrggbb. `null`, 0 und JDAs "keine Farbe"
 * (0x1FFFFFFF, alles über 0xFFFFFF) ergeben `null`.
 */
export function farbeHex(wert: number | null | undefined): string | null {
    if (wert == null || wert <= 0 || wert > 0xffffff) return null;
    return '#' + wert.toString(16).padStart(6, '0');
}

// ---------------------------------------------------------------------------
// Darstellung
// ---------------------------------------------------------------------------

/**
 * Die API liefert Postgres-Zeitstempel als "2026-09-09 21:14:03.0" — ohne
 * Zonenangabe, gemeint ist UTC (Datenbank und Bot laufen in UTC; gegen die
 * Discord-Nachrichten-IDs geprüft am 20.11.2026). Ohne das angehängte "Z"
 * liest der Browser die Angabe als Ortszeit, und jede Uhrzeit im Dashboard
 * steht eine Stunde (im Sommer zwei) zu früh da. Bringt die Angabe schon
 * eine Zone mit, bleibt sie unangetastet.
 */
export function alsDatum(roh: string): Date {
    let iso = roh.trim().replace(' ', 'T');
    if (iso.includes('T') && !/(Z|[+-]\d{2}(:?\d{2})?)$/i.test(iso)) iso += 'Z';
    return new Date(iso);
}

export function zeit(roh: string | null | undefined): string {
    if (!roh || roh === 'null') return '—';
    const d = alsDatum(roh);
    if (Number.isNaN(d.getTime())) return roh;
    return d.toLocaleString('de-DE', {
        day: '2-digit',
        month: '2-digit',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    });
}

export function datum(roh: string | null | undefined): string {
    if (!roh || roh === 'null') return '—';
    // Tagesangaben ("2026-11-19") sind UTC-Tage des Bots; als solche zeigen,
    // sonst rutscht der Balken westlich von Greenwich auf den Vortag.
    const d = alsDatum(roh);
    if (Number.isNaN(d.getTime())) return roh;
    return d.toLocaleDateString('de-DE', {
        day: '2-digit',
        month: '2-digit',
        timeZone: 'UTC',
    });
}

export function groesse(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

/** Namen der Leute, die in einem Ticket vorkommen: wer geschrieben hat, wem es
 *  gehört, wer es übernommen hat. Mehr weiß das Dashboard nicht – wer nur erwähnt
 *  wurde und nie schrieb, bleibt eine Nummer. Bots zuletzt, damit ein Mensch mit
 *  derselben ID (gibt es nicht, aber sicher ist sicher) nicht überschrieben wird;
 *  bei mehreren Namen desselben Menschen gilt der jüngste. */
export function namenAusTicket(detail: TicketDetail | null): Map<string, string> {
    const namen = new Map<string, string>();
    if (!detail) return namen;
    for (const m of detail.messages) {
        if (m.author_id && m.author_name) namen.set(m.author_id, m.author_name);
    }
    const t = detail.ticket;
    if (t.owner_id && t.owner_name && !namen.has(t.owner_id))
        namen.set(t.owner_id, t.owner_name);
    if (t.claimed_by && t.claimed_by_name && !namen.has(t.claimed_by))
        namen.set(t.claimed_by, t.claimed_by_name);
    return namen;
}

/** Discord-Erwähnungen lesbar machen; alles andere bleibt Text. Mit `namen`
 *  steht statt der Nummer der Name da, soweit er bekannt ist. */
export function erwaehnungen(
    text: string | null,
    namen?: Map<string, string>,
): string {
    if (!text) return '';
    // Ein Durchgang, damit ein eingesetzter Name nicht selbst noch einmal
    // als Erwähnung gelesen wird.
    return text.replace(/<(@!?|@&|#)(\d+)>/g, (_, art, id) =>
        art === '#'
            ? '#' + id
            : '@' + (art === '@&' ? id : (namen?.get(id) ?? id)),
    );
}
