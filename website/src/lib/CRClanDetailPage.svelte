<script lang="ts">
    import { onMount, createEventDispatcher } from 'svelte';
    import { fade, slide, scale } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
    import { user, userOverride, hasRequiredRole } from './auth';
    import PlayerDetailModal from './PlayerDetailModal.svelte';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string;
    export let clanTag: string;
    export let backPath: string = '/cr/clans';

    const dispatch = createEventDispatcher<{
        navigate: string;
    }>();

    interface Clan {
        tag: string;
        name: string;
        type: string;
        description: string;
        location: { name: string };
        badgeUrls: { large: string; medium: string; small: string };
        badgeId: number;
        clanScore: number;
        clanWarTrophies: number;
        requiredTrophies: number;
        donationsPerWeek: number;
        members: number;
        clanChestStatus?: string;
        clanChestLevel?: number;
        // Upstream data
        maxKickpoints?: number;
        minSeasonWins?: number;
        kickpointsExpireAfterDays?: number;
        kickpointReasons?: Array<{ name: string; amount: number }>;
    }

    interface Player {
        tag: string;
        name: string;
        role: string;
        expLevel: number;
        arena?: { id: number; name: string };
        trophies: number;
        donations: number;
        donationsReceived: number;
        clanRank: number;
        previousClanRank: number;
        clanChestPoints?: number;
        // Upstream data (if available)
        userId?: string;
        isLinked?: boolean;
        discordId?: string;
        nickname?: string;
        avatar?: string;
        totalKickpoints?: number;
        activeKickpointsCount?: number;
        activeKickpointsSum?: number;
        activeKickpoints?: any[];
        // Mixed API Data
        in_supercell?: boolean;
        in_upstream?: boolean;
        is_diff?: boolean;
        is_new?: boolean;
        is_left?: boolean;
        is_dirty?: boolean;
        isHidden?: boolean;
        upstream_name?: string;
        upstream_role?: string;
        upstream_expLevel?: number;
    }

    let clan: Clan | null = null;
    let members: Player[] = [];
    let loading = true;
    let error: string | null = null;
    let searchQuery = '';
    let selectedPlayer: Player | null = null;
    let playerDetailsLoading = false;

    $: viewerIsCoLeader = !!(
        $user &&
        members.some(
            (m) =>
                ($user.linked_cr_players || []).includes(m.tag) &&
                (m.role === 'coLeader' || m.role === 'leader')
        )
    );
    $: hasPrivilegedAccess = !!(
        $user?.is_admin ||
        viewerIsCoLeader ||
        ($userOverride && hasRequiredRole($user?.highest_role, 'COLEADER'))
    );

    const roleOrder: Record<string, number> = {
        leader: 1,
        coLeader: 2,
        elder: 3,
        member: 4,
    };

    function getRoleDisplay(role: string): string {
        switch (role) {
            case 'leader':
                return 'Anführer';
            case 'coLeader':
                return 'Vize-Anführer';
            case 'elder':
                return 'Ältester';
            case 'member':
                return 'Mitglied';
            default:
                return role;
        }
    }

    async function fetchClanData() {
        loading = true;
        try {
            const encodedTag = encodeURIComponent(clanTag);
            const clanRes = await fetch(
                `${apiBaseUrl}/api/cr/clans/${encodedTag}`,
                { credentials: 'include' }
            );
            if (!clanRes.ok) throw new Error('Clan nicht gefunden');
            clan = await clanRes.json();

            const membersRes = await fetch(
                `${apiBaseUrl}/api/cr/clans/${encodedTag}/members`,
                { credentials: 'include' }
            );
            if (!membersRes.ok)
                throw new Error('Mitglieder konnten nicht geladen werden');
            const membersData = await membersRes.json();
            members = Array.isArray(membersData) ? membersData : [];

            // Sort members by role, then trophies
            members.sort((a, b) => {
                const rA = roleOrder[a.role] || 99;
                const rB = roleOrder[b.role] || 99;
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
            const [res, idRes] = await Promise.all([
                fetch(`${apiBaseUrl}/api/cr/players/${encodedTag}`, {
                    credentials: 'include',
                }),
                fetch(`${apiBaseUrl}/api/cr/players/${encodedTag}/identity`, {
                    credentials: 'include',
                }),
            ]);

            const detailedPlayer = res.ok ? await res.json() : {};
            const identity = idRes.ok ? await idRes.json() : {};

            selectedPlayer = {
                ...player,
                ...detailedPlayer,
                ...identity,
            };
        } catch (e) {
            console.error('Failed to fetch player details:', e);
        } finally {
            playerDetailsLoading = false;
        }
    }

    function closePlayerDetails() {
        selectedPlayer = null;
    }

    $: if (clanTag) {
        fetchClanData();
    }

    $: sortedByDonations = [...members]
        .sort((a, b) => (b.donations || 0) - (a.donations || 0))
        .slice(0, 3);
    $: sortedByTrophies = [...members]
        .sort((a, b) => (b.trophies || 0) - (a.trophies || 0))
        .slice(0, 3);

    $: filteredMembers = members
        .filter(
            (m) => m.in_supercell && !m.is_new && !m.is_dirty && !m.isHidden
        ) // Hide diff & hidden members from main list
        .filter((m) => {
            const nameMatch =
                m.name?.toLowerCase().includes(searchQuery.toLowerCase()) ??
                false;
            const tagMatch =
                m.tag?.toLowerCase().includes(searchQuery.toLowerCase()) ??
                false;
            return nameMatch || tagMatch;
        });

    $: newMembers = members.filter((m) => m.is_new && !m.isHidden);
    $: leftMembers = members.filter((m) => m.is_left && !m.isHidden);
    $: changedMembers = members.filter((m) => m.is_dirty && !m.isHidden);
    $: hasDifferences =
        newMembers.length > 0 ||
        leftMembers.length > 0 ||
        changedMembers.length > 0;
</script>

<div
    class="clan-detail-page cr-theme"
    class:light={theme === 'light'}
    in:fade={{ duration: 300 }}
>
    {#if loading}
        <div class="loading-overlay">
            <div class="spinner"></div>
            <p>Clan-Daten werden geladen...</p>
        </div>
    {:else if error}
        <div class="error-container">
            <div class="error-content">
                <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                >
                    <circle cx="12" cy="12" r="10" /><line
                        x1="12"
                        y1="8"
                        x2="12"
                        y2="12"
                    /><line x1="12" y1="16" x2="12.01" y2="16" />
                </svg>
                <h2>Hoppla!</h2>
                <p>{error}</p>
                <button class="retry-btn" on:click={fetchClanData}
                    >Erneut versuchen</button
                >
            </div>
        </div>
    {:else if clan}
        <div class="page-content">
            <!-- Hero Header Section -->
            <header class="clan-hero">
                <button
                    class="back-btn"
                    on:click={() =>
                        dispatch(
                            'navigate',
                            backPath.startsWith('/')
                                ? backPath.substring(1)
                                : backPath
                        )}
                >
                    <svg
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="3"
                    >
                        <path d="M19 12H5M12 19l-7-7 7-7" />
                    </svg>
                    {#if backPath === '/my-clans'}
                        Deine Clans
                    {:else if backPath === '/admin/clans'}
                        Clan Admin
                    {:else}
                        Alle Clans
                    {/if}
                </button>
                <div
                    class="hero-bg"
                    style="background-image: url({clan.badgeUrls?.large || ''})"
                ></div>
                <div class="hero-overlay"></div>
                <div class="hero-content">
                    <div class="badge-container">
                        <img
                            src={clan.badgeUrls?.large || ''}
                            alt={clan.name}
                            class="clan-badge"
                        />
                    </div>
                    <div class="clan-info-main">
                        <div class="title-row">
                            <h1>{clan.name}</h1>
                            <span class="tag-pill">{clan.tag}</span>
                        </div>
                        <p class="clan-desc">
                            {clan.description ||
                                'Keine Beschreibung verfügbar.'}
                        </p>
                        <div class="quick-stats">
                            <div class="q-stat">
                                <span class="q-label">Clan Score</span>
                                <span class="q-value"
                                    >{clan.clanScore?.toLocaleString() ??
                                        '0'}</span
                                >
                            </div>
                            <div class="q-stat">
                                <span class="q-label">Mitglieder</span>
                                <span class="q-value"
                                    >{clan.members ?? '0'}/50</span
                                >
                            </div>
                            <div class="q-stat">
                                <span class="q-label">Typ</span>
                                <span class="q-value"
                                    >{clan.type === 'inviteOnly'
                                        ? 'Auf Einladung'
                                        : clan.type === 'open'
                                          ? 'Offen'
                                          : 'Geschlossen'}</span
                                >
                            </div>
                        </div>
                    </div>
                </div>
            </header>

            <div class="layout-grid">
                <!-- Sidebar -->
                <aside class="sidebar">
                    <div class="sidebar-sticky">
                        <section class="info-card highlight-card">
                            <h3>Top Mitglieder</h3>
                            <div class="top-lists">
                                <div class="top-list-section">
                                    <div class="top-list-header">
                                        <svg
                                            viewBox="0 0 24 24"
                                            fill="currentColor"
                                            class="t-icon trophies"
                                        >
                                            <path
                                                d="M19 5h-2V3H7v2H5c-1.1 0-2 .9-2 2v3c0 2.55 1.92 4.63 4.39 4.94.63 1.5 1.98 2.63 3.61 2.96V19H7v2h10v-2h-4v-3.1c1.63-.33 2.98-1.46 3.61-2.96C19.08 12.63 21 10.55 21 8V7c0-1.1-.9-2-2-2zM5 10V7h2v3c0 1.1-.9 2-2 2zm14 0c-1.1 0-2-.9-2-2V7h2v3z"
                                            />
                                        </svg>
                                        <span>Trophäen</span>
                                    </div>
                                    <div class="mini-rank-list">
                                        {#each sortedByTrophies as player, i}
                                            <div
                                                class="rank-item"
                                                on:click={() =>
                                                    selectPlayer(player)}
                                                on:keydown={(e) =>
                                                    e.key === 'Enter' &&
                                                    selectPlayer(player)}
                                                role="button"
                                                tabindex="0"
                                            >
                                                <span class="rank-num"
                                                    >{i + 1}</span
                                                >
                                                <span class="rank-name"
                                                    >{player.name}</span
                                                >
                                                <span class="rank-val"
                                                    >{player.trophies}</span
                                                >
                                            </div>
                                        {/each}
                                    </div>
                                </div>

                                <div class="top-list-divider"></div>

                                <div class="top-list-section">
                                    <div class="top-list-header">
                                        <svg
                                            viewBox="0 0 24 24"
                                            fill="currentColor"
                                            class="t-icon donation"
                                        >
                                            <path
                                                d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
                                            />
                                        </svg>
                                        <span>Top Spender</span>
                                    </div>
                                    <div class="mini-rank-list">
                                        {#each sortedByDonations as player, i}
                                            <div
                                                class="rank-item"
                                                on:click={() =>
                                                    selectPlayer(player)}
                                                on:keydown={(e) =>
                                                    e.key === 'Enter' &&
                                                    selectPlayer(player)}
                                                role="button"
                                                tabindex="0"
                                            >
                                                <span class="rank-num"
                                                    >{i + 1}</span
                                                >
                                                <span class="rank-name"
                                                    >{player.name}</span
                                                >
                                                <span class="rank-val"
                                                    >▲{player.donations}</span
                                                >
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            </div>
                        </section>

                        <section class="info-card">
                            <h3>Clan Informationen</h3>
                            <div class="info-grid">
                                <div class="info-item">
                                    <span class="label">Standort</span>
                                    <span class="value"
                                        >{clan.location?.name ||
                                            'Unbekannt'}</span
                                    >
                                </div>
                                <div class="info-item">
                                    <span class="label">Benötigte Trophäen</span
                                    >
                                    <span class="value"
                                        >{clan.requiredTrophies ?? '0'}</span
                                    >
                                </div>
                                <div class="info-item">
                                    <span class="label">Spenden/Woche</span>
                                    <span class="value"
                                        >{clan.donationsPerWeek ?? '0'}</span
                                    >
                                </div>
                                <div class="info-item">
                                    <span class="label">Kriegs-Trophäen</span>
                                    <span class="value"
                                        >{clan.clanWarTrophies ?? '0'}</span
                                    >
                                </div>
                            </div>
                        </section>

                        {#if clan.kickpointReasons && clan.kickpointReasons.length > 0}
                            <section
                                class="info-card reasons-card"
                                in:slide={{ duration: 400, delay: 200 }}
                            >
                                <div class="card-header">
                                    <svg
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        class="h-icon"
                                    >
                                        <path
                                            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                                        />
                                    </svg>
                                    <h3>Regelwerk / Kickpoints</h3>
                                </div>
                                <div class="reasons-list">
                                    {#each clan.kickpointReasons as reason}
                                        <div class="reason-item">
                                            <span class="reason-name"
                                                >{reason.name}</span
                                            >
                                            <span class="reason-amount"
                                                >+{reason.amount}</span
                                            >
                                        </div>
                                    {/each}
                                </div>
                                {#if clan.maxKickpoints}
                                    <div class="max-kp-info">
                                        Maximale Kickpoints: <span class="val"
                                            >{clan.maxKickpoints}</span
                                        >
                                    </div>
                                {/if}
                                {#if clan.kickpointsExpireAfterDays}
                                    <div class="expiry-info">
                                        Verfall nach {clan.kickpointsExpireAfterDays}
                                        Tagen
                                    </div>
                                {/if}
                            </section>
                        {/if}
                    </div>
                </aside>

                <!-- Members Section -->
                <main class="members-main">
                    <div class="section-header-row">
                        <div class="title-group">
                            <h2>Clan Mitglieder</h2>
                            <p class="subtitle">
                                {filteredMembers.length} von {clan.members ??
                                    '0'} Mitgliedern angezeigt
                            </p>
                        </div>
                        <div class="search-box">
                            <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <circle cx="11" cy="11" r="8" /><line
                                    x1="21"
                                    y1="21"
                                    x2="16.65"
                                    y2="16.65"
                                />
                            </svg>
                            <input
                                type="text"
                                placeholder="Mitglieder suchen..."
                                bind:value={searchQuery}
                            />
                        </div>
                    </div>

                    <div class="members-grid">
                        {#each filteredMembers as member (member.tag)}
                            {@const total =
                                (member.donations || 0) +
                                (member.donationsReceived || 0)}
                            {@const percent =
                                total > 0
                                    ? ((member.donations || 0) / total) * 100
                                    : 50}
                            <div
                                class="member-card"
                                class:is-linked={member.userId}
                                on:click={() => selectPlayer(member)}
                                on:keydown={(e) =>
                                    e.key === 'Enter' && selectPlayer(member)}
                                role="button"
                                tabindex="0"
                            >
                                <div class="m-card-content">
                                    <div class="card-glow"></div>
                                    <div class="m-rank-indicator">
                                        {members.indexOf(member) + 1}
                                    </div>
                                    <div class="m-avatar-container">
                                        {#if member.arena}
                                            <div
                                                class="arena-badge"
                                                title={member.arena.name}
                                            >
                                                Arena {member.arena.id}
                                            </div>
                                        {:else}
                                            <div class="no-league"></div>
                                        {/if}
                                    </div>
                                    <div class="m-main-info">
                                        <h4 class="m-name">{member.name}</h4>
                                        <div class="m-sub-info">
                                            <span class="m-role-label"
                                                >{getRoleDisplay(
                                                    member.role
                                                )}</span
                                            >
                                            <span class="dot">•</span>
                                            <span class="m-tag-small"
                                                >{member.tag}</span
                                            >
                                        </div>
                                    </div>
                                    <div class="m-points-info">
                                        {#if hasPrivilegedAccess && member.activeKickpointsSum !== undefined && member.activeKickpointsSum > 0}
                                            <div
                                                class="m-kp-badge"
                                                title="Aktive Kickpoints"
                                                in:scale
                                            >
                                                <svg
                                                    viewBox="0 0 24 24"
                                                    fill="currentColor"
                                                    class="kp-icon"
                                                >
                                                    <path
                                                        d="M12 2L1 21h22L12 2zm0 3.99L19.53 19H4.47L12 5.99zM11 16h2v2h-2zm0-6h2v4h-2z"
                                                    />
                                                </svg>
                                                <span
                                                    >{member.activeKickpointsSum}</span
                                                >
                                                {#if clan?.maxKickpoints}
                                                    <span class="max-sep"
                                                        >/</span
                                                    >
                                                    <span class="max-val"
                                                        >{clan.maxKickpoints}</span
                                                    >
                                                {/if}
                                            </div>
                                        {/if}
                                        <div class="m-th-badge">
                                            LVL {member.expLevel}
                                        </div>
                                        <div class="m-stat-group">
                                            <div
                                                class="m-trophies-badge m-trophies-small"
                                                title="Trophäen"
                                            >
                                                <svg
                                                    viewBox="0 0 24 24"
                                                    fill="currentColor"
                                                >
                                                    <path
                                                        d="M19 5h-2V3H7v2H5c-1.1 0-2 .9-2 2v3c0 2.55 1.92 4.63 4.39 4.94.63 1.5 1.98 2.63 3.61 2.96V19H7v2h10v-2h-4v-3.1c1.63-.33 2.98-1.46 3.61-2.96C19.08 12.63 21 10.55 21 8V7c0-1.1-.9-2-2-2zM5 10V7h2v3c0 1.1-.9 2-2 2zm14 0c-1.1 0-2-.9-2-2V7h2v3z"
                                                    />
                                                </svg>
                                                <span
                                                    >{member.trophies ||
                                                        0}</span
                                                >
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div class="m-card-footer">
                                    <div class="donation-bar-container">
                                        <span class="don-label">Spenden</span>
                                        <div class="donation-bar">
                                            <div
                                                class="don-fill given"
                                                style="width: {percent}%"
                                            ></div>
                                        </div>
                                        <div class="don-values">
                                            <span class="given-val"
                                                >▲ {member.donations || 0}</span
                                            >
                                            <span class="rec-val"
                                                >▼ {member.donationsReceived ||
                                                    0}</span
                                            >
                                        </div>
                                    </div>
                                </div>
                            </div>
                        {/each}
                    </div>

                    {#if hasDifferences}
                        <div class="diff-section" transition:slide>
                            <div class="section-header-row">
                                <div class="title-group">
                                    <h3>Memberstatus</h3>
                                </div>
                            </div>

                            <div class="diff-grid-layout">
                                {#if leftMembers.length > 0}
                                    <div class="diff-category">
                                        <h4>
                                            <span class="indicator-dot left"
                                            ></span>
                                            Mitglied, ingame nicht im Clan ({leftMembers.length})
                                        </h4>
                                        <div class="diff-cards">
                                            {#each leftMembers as m}
                                                <div
                                                    class="mini-diff-card left"
                                                >
                                                    <div class="m-info">
                                                        <span class="m-name"
                                                            >{m.name ||
                                                                'Unbekannt'}</span
                                                        >
                                                        <span class="m-tag"
                                                            >{m.tag}</span
                                                        >
                                                    </div>
                                                </div>
                                            {/each}
                                        </div>
                                    </div>
                                {/if}

                                {#if newMembers.length > 0}
                                    <div class="diff-category">
                                        <h4>
                                            <span class="indicator-dot new"
                                            ></span>
                                            Kein Mitglied, ingame im Clan ({newMembers.length})
                                        </h4>
                                        <div class="diff-cards">
                                            {#each newMembers as m}
                                                <div class="mini-diff-card new">
                                                    <div class="m-info">
                                                        <span class="m-name"
                                                            >{m.name}</span
                                                        >
                                                        <span class="m-tag"
                                                            >{m.tag}</span
                                                        >
                                                    </div>
                                                </div>
                                            {/each}
                                        </div>
                                    </div>
                                {/if}
                            </div>

                            {#if changedMembers.length > 0}
                                <div class="diff-table-container">
                                    <div class="table-header">
                                        <h4>
                                            Im Clan, falsche Rolle / Daten ({changedMembers.length})
                                        </h4>
                                    </div>
                                    <table class="diff-table">
                                        <thead>
                                            <tr>
                                                <th>Spieler</th>
                                                <th>Feld</th>
                                                <th>Ingame</th>
                                                <th>Datenbank</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {#each changedMembers as m}
                                                {#if m.name !== m.upstream_name && m.upstream_name}
                                                    <tr class="row-diff">
                                                        <td
                                                            ><strong
                                                                >{m.name}</strong
                                                            >
                                                            <span
                                                                class="tag-small"
                                                                >({m.tag})</span
                                                            ></td
                                                        >
                                                        <td>Name</td>
                                                        <td class="val-sc"
                                                            >{m.name}</td
                                                        >
                                                        <td class="val-up"
                                                            >{m.upstream_name}</td
                                                        >
                                                    </tr>
                                                {/if}
                                                {#if m.upstream_role && !(m.role === m.upstream_role || (m.role === 'elder' && m.upstream_role === 'admin') || (m.role === 'admin' && m.upstream_role === 'elder'))}
                                                    <tr class="row-diff">
                                                        <td
                                                            ><strong
                                                                >{m.name}</strong
                                                            >
                                                            <span
                                                                class="tag-small"
                                                                >({m.tag})</span
                                                            ></td
                                                        >
                                                        <td>Rolle</td>
                                                        <td class="val-sc"
                                                            >{getRoleDisplay(
                                                                m.role
                                                            )}</td
                                                        >
                                                        <td class="val-up"
                                                            >{getRoleDisplay(
                                                                m.upstream_role
                                                            )}</td
                                                        >
                                                    </tr>
                                                {/if}
                                                {#if m.upstream_expLevel && String(m.expLevel) !== String(m.upstream_expLevel)}
                                                    <tr class="row-diff">
                                                        <td
                                                            ><strong
                                                                >{m.name}</strong
                                                            >
                                                            <span
                                                                class="tag-small"
                                                                >({m.tag})</span
                                                            ></td
                                                        >
                                                        <td>Level</td>
                                                        <td class="val-sc"
                                                            >{m.expLevel}</td
                                                        >
                                                        <td class="val-up"
                                                            >{m.upstream_expLevel}</td
                                                        >
                                                    </tr>
                                                {/if}
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                            {/if}
                        </div>
                    {/if}
                </main>
            </div>
        </div>
    {/if}

    <PlayerDetailModal
        isOpen={!!selectedPlayer && !playerDetailsLoading}
        player={selectedPlayer}
        gameType="cr"
        {theme}
        onClose={closePlayerDetails}
        {hasPrivilegedAccess}
        isAdmin={$user?.is_admin}
        onNavigateToProfile={(userId) =>
            dispatch('navigate', `profile/${userId}`)}
    />

    {#if selectedPlayer && playerDetailsLoading}
        <div class="modal-backdrop" transition:fade={{ duration: 200 }}>
            <div class="modal-loading-minimal">
                <div class="spinner"></div>
                <p>Spielerdetails laden...</p>
            </div>
        </div>
    {/if}
</div>

<style>
    .modal-loading-minimal {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(8px);
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        z-index: 2100;
        color: white;
        gap: 1.5rem;
    }

    .clan-detail-page {
        min-height: 100vh;
        padding-bottom: 4rem;
    }

    .cr-theme .clan-hero::before {
        background: linear-gradient(
            135deg,
            rgba(88, 101, 242, 0.2),
            rgba(114, 137, 218, 0.1)
        );
    }

    .cr-theme .hero-overlay {
        background: linear-gradient(
            to bottom,
            rgba(10, 10, 15, 0.6) 0%,
            rgba(10, 10, 15, 0.95) 100%
        );
    }

    .loading-overlay {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        min-height: 60vh;
        color: #b9bbbe;
    }

    .spinner {
        width: 48px;
        height: 48px;
        border: 4px solid rgba(88, 101, 242, 0.2);
        border-top-color: #5865f2;
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin-bottom: 1rem;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .error-container {
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 60vh;
    }

    .error-content {
        text-align: center;
        padding: 3rem;
        background: rgba(30, 41, 59, 0.5);
        border-radius: 24px;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .error-content svg {
        width: 64px;
        height: 64px;
        color: #ed4245;
        margin-bottom: 1rem;
    }

    .error-content h2 {
        color: #fff;
        margin-bottom: 0.5rem;
    }

    .error-content p {
        color: #b9bbbe;
        margin-bottom: 1.5rem;
    }

    .retry-btn {
        padding: 0.75rem 2rem;
        background: #5865f2;
        color: white;
        border: none;
        border-radius: 12px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
    }

    .retry-btn:hover {
        background: #4752c4;
        transform: translateY(-2px);
    }

    /* Hero */
    .clan-hero {
        position: relative;
        padding: 8rem 2rem 4rem;
        overflow: hidden;
    }

    .hero-bg {
        position: absolute;
        inset: 0;
        background-size: cover;
        background-position: center;
        filter: blur(60px);
        opacity: 0.3;
        transform: scale(1.2);
    }

    .hero-overlay {
        position: absolute;
        inset: 0;
        background: linear-gradient(
            to bottom,
            rgba(10, 10, 15, 0.6) 0%,
            rgba(10, 10, 15, 0.95) 100%
        );
    }

    .hero-content {
        position: relative;
        display: flex;
        align-items: center;
        gap: 2.5rem;
        max-width: 1200px;
        margin: 0 auto;
    }

    .back-btn {
        position: absolute;
        top: 6rem;
        left: 2rem;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.75rem 1.25rem;
        background: rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 12px;
        color: #fff;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        z-index: 10;
    }

    .back-btn:hover {
        background: rgba(255, 255, 255, 0.2);
        transform: translateX(-4px);
    }

    .back-btn svg {
        width: 20px;
        height: 20px;
    }

    .badge-container {
        position: relative;
    }

    .clan-badge {
        width: 140px;
        height: 140px;
        border-radius: 24px;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
    }

    .clan-info-main {
        flex: 1;
    }

    .title-row {
        display: flex;
        align-items: center;
        gap: 1rem;
        flex-wrap: wrap;
        margin-bottom: 0.75rem;
    }

    .title-row h1 {
        font-size: 2.5rem;
        font-weight: 800;
        color: #fff;
        margin: 0;
    }

    .tag-pill {
        padding: 0.4rem 1rem;
        background: rgba(88, 101, 242, 0.2);
        border: 1px solid rgba(88, 101, 242, 0.3);
        border-radius: 20px;
        color: #8b9eff;
        font-size: 0.875rem;
        font-weight: 600;
    }

    .clan-desc {
        color: rgba(255, 255, 255, 0.7);
        font-size: 1.1rem;
        line-height: 1.6;
        margin-bottom: 1.5rem;
        max-width: 600px;
    }

    .quick-stats {
        display: flex;
        gap: 2rem;
    }

    .q-stat {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .q-label {
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .q-value {
        font-size: 1.25rem;
        font-weight: 700;
        color: #fff;
    }

    /* Layout */
    .layout-grid {
        display: grid;
        grid-template-columns: 320px 1fr;
        gap: 2rem;
        max-width: 1400px;
        margin: 0 auto;
        padding: 0 2rem;
    }

    @media (max-width: 1024px) {
        .layout-grid {
            grid-template-columns: 1fr;
        }
    }

    /* Sidebar */
    .sidebar-sticky {
        position: sticky;
        top: 100px;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .info-card {
        background: rgba(30, 41, 59, 0.5);
        backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 20px;
        padding: 1.5rem;
    }

    .info-card h3 {
        font-size: 1rem;
        font-weight: 700;
        color: #fff;
        margin-bottom: 1rem;
    }

    .reasons-card {
        border-color: rgba(237, 66, 69, 0.3);
    }

    .reasons-card .card-header {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-bottom: 1.25rem;
    }

    .reasons-card h3 {
        margin: 0;
    }

    .h-icon {
        width: 20px;
        height: 20px;
        color: #ed4245;
    }

    .reasons-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        margin-bottom: 1.25rem;
    }

    .reason-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1rem;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 12px;
        font-size: 0.9rem;
    }

    .reason-name {
        color: rgba(255, 255, 255, 0.9);
    }

    .reason-amount {
        color: #ed4245;
        font-weight: 700;
    }

    .max-kp-info,
    .expiry-info {
        font-size: 0.8rem;
        padding: 0.5rem 0.75rem;
        border-radius: 8px;
        margin-top: 0.5rem;
    }

    .max-kp-info {
        background: rgba(237, 66, 69, 0.15);
        color: #ff787b;
        border: 1px solid rgba(237, 66, 69, 0.2);
    }

    .max-kp-info .val {
        font-weight: 800;
    }

    .expiry-info {
        color: rgba(255, 255, 255, 0.4);
        font-style: italic;
    }

    .highlight-card {
        background: linear-gradient(
            135deg,
            rgba(88, 101, 242, 0.15),
            rgba(30, 41, 59, 0.5)
        );
        border-color: rgba(88, 101, 242, 0.3);
    }

    .top-lists {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .top-list-header {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.6);
        margin-bottom: 0.5rem;
    }

    .t-icon {
        width: 16px;
        height: 16px;
    }

    .t-icon.trophies {
        color: #fbbf24;
    }
    .t-icon.donation {
        color: #f87171;
    }

    .mini-rank-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .rank-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.5rem 0.75rem;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.2s;
    }

    .rank-item:hover {
        background: rgba(255, 255, 255, 0.1);
    }

    .rank-num {
        font-size: 0.75rem;
        font-weight: 700;
        color: rgba(255, 255, 255, 0.4);
        width: 1.5rem;
    }

    .rank-name {
        flex: 1;
        font-size: 0.875rem;
        color: #fff;
        font-weight: 500;
    }

    .rank-val {
        font-size: 0.8rem;
        color: #fbbf24;
        font-weight: 600;
    }

    .top-list-divider {
        height: 1px;
        background: rgba(255, 255, 255, 0.1);
    }

    .info-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    .info-item {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .info-item .label {
        font-size: 0.7rem;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
    }

    .info-item .value {
        font-size: 0.9rem;
        color: #fff;
        font-weight: 600;
    }

    /* Members Section */
    .members-main {
        min-height: 100vh;
    }

    .section-header-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 2rem;
        flex-wrap: wrap;
        gap: 1rem;
    }

    .title-group h2 {
        font-size: 1.5rem;
        font-weight: 800;
        color: #fff;
        margin: 0;
    }

    .subtitle {
        color: rgba(255, 255, 255, 0.5);
        font-size: 0.875rem;
        margin-top: 0.25rem;
    }

    .search-box {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem 1.25rem;
        background: rgba(30, 41, 59, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
    }

    .search-box svg {
        width: 20px;
        height: 20px;
        color: rgba(255, 255, 255, 0.4);
    }

    .search-box input {
        background: none;
        border: none;
        color: #fff;
        font-size: 0.9rem;
        outline: none;
        width: 200px;
    }

    .search-box input::placeholder {
        color: rgba(255, 255, 255, 0.4);
    }

    /* Member Cards */
    .members-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
        gap: 1.25rem;
    }

    .member-card {
        background: rgba(30, 41, 59, 0.4);
        backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 16px;
        overflow: hidden;
        cursor: pointer;
        transition: all 0.3s ease;
        position: relative;
    }

    .member-card:hover {
        transform: translateY(-4px);
        border-color: rgba(88, 101, 242, 0.4);
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
    }

    .member-card.is-linked {
        border-color: rgba(88, 101, 242, 0.3);
    }

    .m-card-content {
        padding: 1.25rem;
        display: flex;
        align-items: center;
        gap: 1rem;
        position: relative;
    }

    .card-glow {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 2px;
        background: linear-gradient(90deg, #5865f2, #7289da);
        opacity: 0;
        transition: opacity 0.3s;
    }

    .member-card:hover .card-glow {
        opacity: 1;
    }

    .m-rank-indicator {
        position: absolute;
        top: 0.5rem;
        right: 0.75rem;
        font-size: 0.7rem;
        font-weight: 700;
        color: rgba(255, 255, 255, 0.3);
    }

    .m-avatar-container {
        flex-shrink: 0;
    }

    .arena-badge {
        padding: 0.5rem 0.75rem;
        background: linear-gradient(135deg, #5865f2, #7289da);
        border-radius: 10px;
        font-size: 0.7rem;
        font-weight: 700;
        color: #fff;
    }

    .no-league {
        width: 48px;
        height: 48px;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 10px;
        border: 1px dashed rgba(255, 255, 255, 0.1);
    }

    .m-main-info {
        flex: 1;
        min-width: 0;
    }

    .m-name {
        font-size: 1rem;
        font-weight: 700;
        color: #fff;
        margin: 0 0 0.25rem 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .m-sub-info {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.5);
    }

    .m-role-label {
        color: #5865f2;
        font-weight: 600;
    }

    .m-kp-badge {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.35rem 0.6rem;
        background: rgba(237, 66, 69, 0.15);
        border: 1px solid rgba(237, 66, 69, 0.3);
        border-radius: 8px;
        font-size: 0.75rem;
        font-weight: 800;
        color: #ed4245;
        box-shadow: 0 4px 12px rgba(237, 66, 69, 0.2);
    }

    .kp-icon {
        width: 14px;
        height: 14px;
    }

    .max-sep {
        opacity: 0.4;
        margin: 0 -0.1rem;
    }

    .max-val {
        opacity: 0.7;
        font-size: 0.7rem;
    }

    .dot {
        opacity: 0.3;
    }

    .m-points-info {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 0.5rem;
    }

    .m-th-badge {
        padding: 0.35rem 0.75rem;
        background: rgba(88, 101, 242, 0.2);
        border: 1px solid rgba(88, 101, 242, 0.3);
        border-radius: 8px;
        font-size: 0.75rem;
        font-weight: 700;
        color: #8b9eff;
    }

    .m-stat-group {
        display: flex;
        gap: 0.5rem;
    }

    .m-trophies-badge {
        display: flex;
        align-items: center;
        gap: 0.3rem;
        padding: 0.25rem 0.5rem;
        background: rgba(251, 191, 36, 0.1);
        border-radius: 6px;
        font-size: 0.7rem;
        font-weight: 600;
        color: #fbbf24;
    }

    .m-trophies-badge svg {
        width: 12px;
        height: 12px;
    }

    /* Card Footer */
    .m-card-footer {
        padding: 0.75rem 1.25rem 1rem;
        border-top: 1px solid rgba(255, 255, 255, 0.05);
    }

    .donation-bar-container {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .don-label {
        font-size: 0.7rem;
        color: rgba(255, 255, 255, 0.4);
        text-transform: uppercase;
    }

    .donation-bar {
        height: 6px;
        background: rgba(255, 255, 255, 0.1);
        border-radius: 3px;
        overflow: hidden;
    }

    .don-fill {
        height: 100%;
        border-radius: 3px;
        transition: width 0.3s ease;
    }

    .don-fill.given {
        background: linear-gradient(90deg, #3ba55c, #57d47e);
    }

    .don-values {
        display: flex;
        justify-content: space-between;
        font-size: 0.75rem;
    }

    .given-val {
        color: #3ba55c;
    }
    .rec-val {
        color: #5865f2;
    }

    /* Modal */
    .modal-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.8);
        backdrop-filter: blur(8px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        padding: 2rem;
    }

    .modal-loading-minimal {
        background: rgba(255, 255, 255, 0.05);
        padding: 2rem;
        border-radius: 20px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.5rem;
    }

    /* Light theme */
    .clan-detail-page.light {
        background: linear-gradient(180deg, #f8fafc, #f1f5f9);
    }

    .clan-detail-page.light .hero-overlay {
        background: linear-gradient(
            to bottom,
            rgba(248, 250, 252, 0.6) 0%,
            rgba(248, 250, 252, 0.98) 100%
        );
    }

    .clan-detail-page.light .title-row h1,
    .clan-detail-page.light .title-group h2,
    .clan-detail-page.light .info-card h3,
    .clan-detail-page.light .rank-name,
    .clan-detail-page.light .info-item .value,
    .clan-detail-page.light .m-name {
        color: #1e293b;
    }

    .clan-detail-page.light .clan-desc,
    .clan-detail-page.light .subtitle,
    .clan-detail-page.light .q-label,
    .clan-detail-page.light .info-item .label,
    .clan-detail-page.light .m-sub-info,
    .clan-detail-page.light .don-label {
        color: #64748b;
    }

    .clan-detail-page.light .q-value {
        color: #1e293b;
    }

    .clan-detail-page.light .reason-item {
        background: rgba(0, 0, 0, 0.03);
    }

    .clan-detail-page.light .reason-name {
        color: #1e293b;
    }

    .clan-detail-page.light .info-card,
    .clan-detail-page.light .member-card,
    .clan-detail-page.light .search-box {
        background: rgba(255, 255, 255, 0.8);
        border-color: rgba(0, 0, 0, 0.08);
    }

    .clan-detail-page.light .back-btn {
        background: rgba(0, 0, 0, 0.05);
        border-color: rgba(0, 0, 0, 0.1);
        color: #1e293b;
    }

    .clan-detail-page.light .m-rank-indicator {
        background: rgba(0, 0, 0, 0.05);
        color: rgba(0, 0, 0, 0.4);
    }

    .clan-detail-page.light .m-tag-small {
        color: rgba(0, 0, 0, 0.4);
    }

    .clan-detail-page.light .m-th-badge {
        background: rgba(0, 0, 0, 0.05);
        color: rgba(0, 0, 0, 0.6);
    }

    .clan-detail-page.light .m-trophies-badge {
        background: rgba(217, 119, 6, 0.1);
        border-color: rgba(217, 119, 6, 0.2);
        color: #d97706;
    }

    .clan-detail-page.light .rank-val,
    .clan-detail-page.light .rank-num {
        color: #1e293b;
    }

    /* Diff Section Styles */
    .diff-section {
        margin-top: 2rem;
        padding-top: 2rem;
        border-top: 1px solid rgba(255, 255, 255, 0.08);
    }

    .diff-section .section-header-row {
        border-bottom: none;
        padding-bottom: 0;
        margin-bottom: 1.5rem;
    }

    .diff-section .title-group h3 {
        margin: 0;
        font-size: 1.8rem;
        font-weight: 800;
        letter-spacing: -0.01em;
    }

    .light .diff-section {
        border-top-color: rgba(0, 0, 0, 0.08);
    }

    .diff-grid-layout {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 2rem;
        margin-top: 2rem;
    }

    .diff-category h4 {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-bottom: 1.25rem;
        font-size: 1.1rem;
        font-weight: 700;
        opacity: 0.8;
    }

    .indicator-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
    }
    .indicator-dot.new {
        background: #10b981;
        box-shadow: 0 0 10px #10b981;
    }
    .indicator-dot.left {
        background: #4b5563;
    }

    .diff-cards {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .mini-diff-card {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 1.25rem;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 12px;
        transition: all 0.2s;
    }

    .mini-diff-card.new {
        border-left: 4px solid #10b981;
    }
    .mini-diff-card.left {
        border-left: 4px solid #4b5563;
        opacity: 0.7;
    }

    .mini-diff-card .m-info {
        display: flex;
        flex-direction: column;
    }

    .mini-diff-card .m-name {
        font-weight: 700;
        font-size: 1rem;
    }

    .mini-diff-card .m-tag {
        font-size: 0.75rem;
        font-family: 'JetBrains Mono', monospace;
        opacity: 0.5;
    }

    .diff-table-container {
        margin-top: 3rem;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        overflow: hidden;
    }

    .diff-table {
        width: 100%;
        border-collapse: collapse;
    }

    .diff-table th {
        text-align: left;
        padding: 1rem 1.5rem;
        background: rgba(255, 255, 255, 0.05);
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    }

    .diff-table td {
        padding: 1rem 1.5rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .val-sc {
        color: #3b82f6;
        font-weight: 700;
    }
    .val-up {
        color: #9ca3af;
        text-decoration: line-through;
        opacity: 0.6;
    }

    @media (max-width: 768px) {
        .hero-content {
            flex-direction: column;
            text-align: center;
        }

        .clan-info-main {
            display: flex;
            flex-direction: column;
            align-items: center;
        }

        .title-row {
            justify-content: center;
        }

        .quick-stats {
            justify-content: center;
        }

        .members-grid {
            grid-template-columns: 1fr;
        }

        .section-header-row {
            flex-direction: column;
            align-items: stretch;
        }

        .search-box {
            width: 100%;
        }

        .search-box input {
            width: 100%;
        }
    }
</style>
