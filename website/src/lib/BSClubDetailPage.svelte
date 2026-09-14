<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { fade } from 'svelte/transition';
    import { user, userOverride, hasRequiredRole } from './auth';
    import PlayerDetailModal from './PlayerDetailModal.svelte';
    import {
        ROLE_ORDER,
        getRoleDisplay,
        isRoleWrong,
        getPlayerName,
    } from './roleUtils';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string;
    export let clanTag: string;
    export let backPath: string = '/bs/clans';

    const dispatch = createEventDispatcher<{ navigate: string }>();

    // Brawl Stars kennt keine Arenen, keine Spenden und keine Kriegssterne.
    // Deshalb eine eigene Seite statt einer Kopie der CR-Seite: dort waere die
    // Haelfte der Felder dauerhaft leer geblieben.
    interface Club {
        tag: string;
        name: string;
        description: string;
        type: string;
        badgeId: number;
        badgeUrls?: { large: string; medium: string; small: string };
        requiredTrophies: number;
        trophies: number;
        members: number;
        // aus dem Bot
        nameDB?: string;
        maxKickpoints?: number;
        kickpointsExpireAfterDays?: number;
        kickpointReasons?: Array<{ name: string; amount: number }>;
    }

    interface Player {
        tag: string;
        name: string;
        role: string;
        trophies: number;
        icon?: { id: number };
        nameDB?: string;
        userId?: string;
        isLinked?: boolean;
        nickname?: string;
        avatar?: string;
        global_name?: string;
        username?: string;
        totalKickpoints?: number;
        activeKickpointsCount?: number;
        activeKickpointsSum?: number;
        activeKickpoints?: any[];
        in_supercell?: boolean;
        in_upstream?: boolean;
        is_new?: boolean;
        is_left?: boolean;
        isHidden?: boolean;
        upstream_name?: string;
        upstream_role?: string;
    }

    let club: Club | null = null;
    let members: Player[] = [];
    let loading = true;
    let error: string | null = null;
    let searchQuery = '';
    let selectedPlayer: Player | null = null;
    let playerDetailsLoading = false;

    function normalizeTag(tag: string | undefined | null): string {
        return (tag || '').trim().replace(/^#/, '').toUpperCase();
    }

    $: viewerIsInClub = !!(
        $user &&
        members.some((m) =>
            ($user.linked_bs_players || []).some(
                (linkedTag: string) =>
                    normalizeTag(linkedTag) === normalizeTag(m.tag),
            ),
        )
    );
    $: viewerIsCoLeader = !!(
        $user &&
        members.some(
            (m) =>
                ($user.linked_bs_players || []).some(
                    (linkedTag: string) =>
                        normalizeTag(linkedTag) === normalizeTag(m.tag),
                ) &&
                ['president', 'vicePresident', 'leader', 'coLeader'].includes(
                    m.role,
                ),
        )
    );
    $: hasPrivilegedAccess = !!(
        $user?.is_admin ||
        viewerIsCoLeader ||
        ($userOverride && hasRequiredRole($user?.highest_role, 'COLEADER'))
    );
    $: hasKickpointAccess = !!(hasPrivilegedAccess || viewerIsInClub);

    function toNumber(value: number | string | undefined): number {
        if (typeof value === 'number') return value;
        if (typeof value === 'string') {
            const parsed = Number(value);
            return Number.isFinite(parsed) ? parsed : 0;
        }
        return 0;
    }

    function getActiveKickpointSum(member: Player): number {
        if (member.activeKickpointsSum !== undefined) {
            return toNumber(member.activeKickpointsSum);
        }
        return (member.activeKickpoints || []).reduce(
            (sum, kp) => sum + toNumber(kp?.amount),
            0,
        );
    }

    function isAtKickpointLimit(member: Player): boolean {
        const max = toNumber(club?.maxKickpoints);
        return max > 0 && getActiveKickpointSum(member) >= max;
    }

    /**
     * Das Wappen kommt von Brawlify — Supercell liefert fuer Brawl Stars nur
     * die Nummer, keine Adresse. Das Backend setzt daraus schon eine
     * badgeUrls-Struktur zusammen; hier steht nur der Rueckfall.
     */
    function badgeUrl(c: Club | null): string {
        if (!c) return '';
        return (
            c.badgeUrls?.large ||
            (c.badgeId
                ? `https://cdn.brawlify.com/club-badges/regular/${c.badgeId}.png`
                : '')
        );
    }

    function icon(id: number | undefined): string {
        return id
            ? `https://cdn.brawlify.com/profile-icons/regular/${id}.png`
            : '';
    }

    function hideOnError(event: Event) {
        (event.currentTarget as HTMLImageElement).style.display = 'none';
    }

    function clubTypeDisplay(type: string | undefined): string {
        switch (type) {
            case 'open':
                return 'Offen';
            case 'inviteOnly':
                return 'Nur auf Einladung';
            case 'closed':
                return 'Geschlossen';
            default:
                return type || 'Unbekannt';
        }
    }

    async function fetchClubData() {
        loading = true;
        error = null;
        try {
            const encodedTag = encodeURIComponent(clanTag);
            const clubRes = await fetch(
                `${apiBaseUrl}/api/bs/clans/${encodedTag}`,
                { credentials: 'include' },
            );
            if (!clubRes.ok) throw new Error('Club nicht gefunden');
            club = await clubRes.json();

            const membersRes = await fetch(
                `${apiBaseUrl}/api/bs/clans/${encodedTag}/members`,
                { credentials: 'include' },
            );
            if (!membersRes.ok)
                throw new Error('Mitglieder konnten nicht geladen werden');
            const membersData = await membersRes.json();
            members = Array.isArray(membersData) ? membersData : [];

            members.sort((a, b) => {
                const rA = ROLE_ORDER[a.role] || 99;
                const rB = ROLE_ORDER[b.role] || 99;
                if (rA !== rB) return rA - rB;
                return (b.trophies || 0) - (a.trophies || 0);
            });
        } catch (e) {
            error = e instanceof Error ? e.message : 'Unbekannter Fehler';
        } finally {
            loading = false;
        }
    }

    async function selectPlayer(player: Player) {
        selectedPlayer = player;
        playerDetailsLoading = true;
        try {
            const encodedTag = encodeURIComponent(player.tag);
            const [res, kpRes, idRes] = await Promise.all([
                fetch(`${apiBaseUrl}/api/bs/players/${encodedTag}`, {
                    credentials: 'include',
                }),
                fetch(
                    `${apiBaseUrl}/api/bs/players/${encodedTag}/kickpoints/details`,
                    { credentials: 'include' },
                ),
                fetch(`${apiBaseUrl}/api/bs/players/${encodedTag}/identity`, {
                    credentials: 'include',
                }),
            ]);

            const detailed = res.ok ? await res.json() : {};
            const kickpoints = kpRes.ok ? await kpRes.json() : [];
            const identity = idRes.ok ? await idRes.json() : {};

            selectedPlayer = {
                ...player,
                ...detailed,
                ...identity,
                activeKickpoints: kickpoints,
            };
        } catch (e) {
            console.error('Spielerdetails konnten nicht geladen werden:', e);
        } finally {
            playerDetailsLoading = false;
        }
    }

    $: if (clanTag) {
        fetchClubData();
    }

    $: topByTrophies = [...members]
        .sort((a, b) => (b.trophies || 0) - (a.trophies || 0))
        .slice(0, 3);

    $: filteredMembers = members
        .filter((m) => !m.is_new && !m.isHidden)
        .filter((m) => {
            if (!searchQuery) return true;
            const q = searchQuery.toLowerCase();
            return (
                getPlayerName(m).toLowerCase().includes(q) ||
                (m.tag || '').toLowerCase().includes(q)
            );
        });

    // Nur der Bot weiss, wer eigentlich welchen Rang haben sollte. Weicht der
    // Rang im Spiel davon ab, faellt das hier auf.
    function rangFalsch(m: Player): boolean {
        return isRoleWrong(m.role, m.upstream_role, false);
    }
</script>

<div class="club-detail-page" class:light={theme === 'light'}>
    <button class="back-btn" on:click={() => dispatch('navigate', backPath.slice(1))}>
        ← Zurück
    </button>

    {#if loading}
        <div class="state">
            <div class="spinner"></div>
            <p>Club wird geladen …</p>
        </div>
    {:else if error}
        <div class="state">
            <p class="error">{error}</p>
            <button class="retry-btn" on:click={fetchClubData}
                >Erneut versuchen</button
            >
        </div>
    {:else if club}
        <div class="club-header" in:fade={{ duration: 200 }}>
            {#if badgeUrl(club)}
                <img
                    class="club-badge"
                    src={badgeUrl(club)}
                    alt=""
                    on:error={hideOnError}
                />
            {/if}
            <div class="club-titles">
                <h1>{club.nameDB || club.name}</h1>
                <span class="club-tag">{club.tag}</span>
                {#if club.description}
                    <p class="club-description">{club.description}</p>
                {/if}
            </div>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <span class="label">Trophäen gesamt</span>
                <span class="value"
                    >{(club.trophies || 0).toLocaleString('de-DE')}</span
                >
            </div>
            <div class="stat-card">
                <span class="label">Benötigte Trophäen</span>
                <span class="value"
                    >{(club.requiredTrophies || 0).toLocaleString('de-DE')}</span
                >
            </div>
            <div class="stat-card">
                <span class="label">Mitglieder</span>
                <span class="value">{members.length} / 30</span>
            </div>
            <div class="stat-card">
                <span class="label">Beitritt</span>
                <span class="value small">{clubTypeDisplay(club.type)}</span>
            </div>
        </div>

        {#if hasPrivilegedAccess && (club.maxKickpoints || club.kickpointReasons?.length)}
            <div class="panel">
                <h2>Kickpunkte</h2>
                <div class="kp-meta">
                    {#if club.maxKickpoints}
                        <span>Grenze: <strong>{club.maxKickpoints}</strong></span>
                    {/if}
                    {#if club.kickpointsExpireAfterDays}
                        <span
                            >Verfallen nach:
                            <strong>{club.kickpointsExpireAfterDays} Tagen</strong
                            ></span
                        >
                    {/if}
                </div>
                {#if club.kickpointReasons?.length}
                    <ul class="kp-reasons">
                        {#each club.kickpointReasons as reason}
                            <li>
                                <span>{reason.name}</span>
                                <span class="kp-amount">{reason.amount}</span>
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>
        {/if}

        {#if topByTrophies.length > 0}
            <div class="panel">
                <h2>Meiste Trophäen</h2>
                <ol class="top-list">
                    {#each topByTrophies as m, i}
                        <li>
                            <span class="platz">{i + 1}.</span>
                            <span class="top-name">{getPlayerName(m)}</span>
                            <span class="top-wert"
                                >{(m.trophies || 0).toLocaleString('de-DE')}</span
                            >
                        </li>
                    {/each}
                </ol>
            </div>
        {/if}

        <div class="panel">
            <div class="panel-head">
                <h2>Mitglieder ({filteredMembers.length})</h2>
                <input
                    class="search"
                    type="text"
                    placeholder="Name oder Tag …"
                    bind:value={searchQuery}
                />
            </div>

            <div class="member-list">
                {#each filteredMembers as member (member.tag)}
                    <button
                        class="member-row"
                        class:limit={hasKickpointAccess &&
                            isAtKickpointLimit(member)}
                        on:click={() => selectPlayer(member)}
                    >
                        {#if icon(member.icon?.id)}
                            <img
                                class="member-icon"
                                src={icon(member.icon?.id)}
                                alt=""
                                on:error={hideOnError}
                            />
                        {/if}
                        <span class="member-name">
                            {getPlayerName(member)}
                            {#if member.isLinked || member.userId}
                                <span class="linked" title="Mit Discord verknüpft"
                                    >●</span
                                >
                            {/if}
                        </span>
                        <span class="member-tag">{member.tag}</span>
                        <span class="member-role" class:falsch={rangFalsch(member)}>
                            {getRoleDisplay(member.role, 'bs')}
                            {#if rangFalsch(member)}
                                <span
                                    class="hinweis"
                                    title="Im Spiel steht ein anderer Rang als beim Bot: {getRoleDisplay(
                                        member.upstream_role || '',
                                        'bs',
                                    )}">⚠</span
                                >
                            {/if}
                        </span>
                        {#if hasKickpointAccess}
                            <span class="member-kp"
                                >{getActiveKickpointSum(member)} KP</span
                            >
                        {/if}
                        <span class="member-trophies"
                            >{(member.trophies || 0).toLocaleString('de-DE')}</span
                        >
                    </button>
                {:else}
                    <p class="leer">Keine Mitglieder gefunden.</p>
                {/each}
            </div>
        </div>
    {/if}
</div>

<PlayerDetailModal
    isOpen={!!selectedPlayer && !playerDetailsLoading}
    player={selectedPlayer}
    gameType="bs"
    {theme}
    onClose={() => (selectedPlayer = null)}
    {hasKickpointAccess}
    isAdmin={$user?.is_admin}
    onNavigateToProfile={(userId) => dispatch('navigate', `profile/${userId}`)}
/>

{#if selectedPlayer && playerDetailsLoading}
    <div class="modal-backdrop" transition:fade={{ duration: 200 }}>
        <div class="modal-loading">
            <div class="spinner"></div>
            <p>Spielerdetails laden …</p>
        </div>
    </div>
{/if}

<style>
    .club-detail-page {
        max-width: 1100px;
        margin: 0 auto;
        padding: 1rem 1.5rem 4rem;
        color: #dcddde;
    }

    .club-detail-page.light {
        color: #2e3338;
    }

    .back-btn,
    .retry-btn {
        background: rgba(241, 176, 25, 0.12);
        color: #f1b019;
        border: 1px solid rgba(241, 176, 25, 0.25);
        border-radius: 8px;
        padding: 0.5rem 1rem;
        cursor: pointer;
        font-size: 0.95rem;
    }

    .back-btn:hover,
    .retry-btn:hover {
        background: rgba(241, 176, 25, 0.2);
    }

    .state {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        min-height: 300px;
        justify-content: center;
    }

    .error {
        color: #ed4245;
    }

    .spinner {
        width: 36px;
        height: 36px;
        border: 3px solid rgba(241, 176, 25, 0.2);
        border-top-color: #f1b019;
        border-radius: 50%;
        animation: spin 0.9s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .club-header {
        display: flex;
        align-items: center;
        gap: 1.5rem;
        margin: 1.5rem 0;
    }

    .club-badge {
        width: 96px;
        height: 96px;
        object-fit: contain;
    }

    .club-titles h1 {
        margin: 0;
        font-size: 2rem;
    }

    .club-tag {
        color: #8e9297;
        font-family: monospace;
    }

    .club-description {
        margin: 0.5rem 0 0;
        max-width: 60ch;
        color: #b9bbbe;
        white-space: pre-wrap;
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
        gap: 1rem;
        margin-bottom: 1.5rem;
    }

    .stat-card,
    .panel {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 12px;
        padding: 1rem 1.25rem;
    }

    .club-detail-page.light .stat-card,
    .club-detail-page.light .panel {
        background: rgba(0, 0, 0, 0.03);
        border-color: rgba(0, 0, 0, 0.08);
    }

    .stat-card {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .stat-card .label {
        font-size: 0.8rem;
        color: #8e9297;
        text-transform: uppercase;
        letter-spacing: 0.04em;
    }

    .stat-card .value {
        font-size: 1.5rem;
        font-weight: 700;
        color: #f1b019;
    }

    .stat-card .value.small {
        font-size: 1.1rem;
    }

    .panel {
        margin-bottom: 1.5rem;
    }

    .panel h2 {
        margin: 0 0 0.75rem;
        font-size: 1.15rem;
    }

    .panel-head {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
        flex-wrap: wrap;
    }

    .search {
        background: rgba(0, 0, 0, 0.25);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        padding: 0.45rem 0.75rem;
        color: inherit;
        min-width: 200px;
    }

    .kp-meta {
        display: flex;
        gap: 1.5rem;
        flex-wrap: wrap;
        margin-bottom: 0.75rem;
        color: #b9bbbe;
    }

    .kp-reasons {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .kp-reasons li {
        display: flex;
        justify-content: space-between;
        gap: 1rem;
        padding: 0.35rem 0;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .kp-amount {
        color: #f1b019;
        font-weight: 600;
    }

    .top-list {
        margin: 0;
        padding: 0;
        list-style: none;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .top-list li {
        display: grid;
        grid-template-columns: 2rem 1fr auto;
        gap: 0.75rem;
        align-items: center;
    }

    .platz {
        color: #8e9297;
    }

    .top-wert {
        color: #f1b019;
        font-weight: 600;
    }

    .member-list {
        display: flex;
        flex-direction: column;
        gap: 0.3rem;
        margin-top: 0.75rem;
    }

    .member-row {
        display: grid;
        grid-template-columns: 32px minmax(120px, 1fr) auto auto auto auto;
        gap: 0.75rem;
        align-items: center;
        width: 100%;
        text-align: left;
        background: transparent;
        border: 1px solid transparent;
        border-radius: 8px;
        padding: 0.45rem 0.6rem;
        color: inherit;
        cursor: pointer;
        font-size: 0.95rem;
    }

    .member-row:hover {
        background: rgba(241, 176, 25, 0.08);
        border-color: rgba(241, 176, 25, 0.2);
    }

    .member-row.limit {
        border-color: rgba(237, 66, 69, 0.45);
    }

    .member-icon {
        width: 32px;
        height: 32px;
        border-radius: 50%;
    }

    .member-tag,
    .member-role,
    .member-kp {
        color: #8e9297;
        font-size: 0.85rem;
    }

    .member-role.falsch {
        color: #faa61a;
    }

    .member-trophies {
        color: #f1b019;
        font-weight: 600;
        min-width: 5ch;
        text-align: right;
    }

    .linked {
        color: #5865f2;
        font-size: 0.7rem;
        vertical-align: middle;
    }

    .leer {
        color: #8e9297;
    }

    .modal-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.7);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }

    .modal-loading {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        color: #dcddde;
    }
</style>
