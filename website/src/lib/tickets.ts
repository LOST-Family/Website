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
// Darstellung
// ---------------------------------------------------------------------------

/** Die API liefert Postgres-Zeitstempel als "2026-09-09 21:14:03.0". */
export function zeit(roh: string | null | undefined): string {
    if (!roh || roh === 'null') return '—';
    const d = new Date(roh.replace(' ', 'T'));
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
    const d = new Date(roh.replace(' ', 'T'));
    if (Number.isNaN(d.getTime())) return roh;
    return d.toLocaleDateString('de-DE', { day: '2-digit', month: '2-digit' });
}

export function groesse(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

/** Discord-Erwähnungen lesbar machen; alles andere bleibt Text. */
export function erwaehnungen(text: string | null): string {
    if (!text) return '';
    return text
        .replace(/<@!?(\d+)>/g, '@$1')
        .replace(/<#(\d+)>/g, '#$1')
        .replace(/<@&(\d+)>/g, '@$1');
}
