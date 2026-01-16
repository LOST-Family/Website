<script lang="ts">
    import { onMount } from 'svelte';
    import { fade, scale, slide } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
    import { user, loading, fetchUser } from './auth';
    import PlayerDetailModal from './PlayerDetailModal.svelte';
    import type { GameType } from './auth';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string = '';
    export let viewUserId: string | null = null;

    let cocPlayerAccounts: any[] = [];
    let crPlayerAccounts: any[] = [];
    let accountsLoading = false;
    let accountsError: string | null = null;
    let userError: string | null = null;
    let selectedPlayer: any = null;
    let selectedGameType: GameType = 'coc';
    let hasInitialFetched = false;
    let viewedUser: any = null;

    async function fetchViewedUser() {
        if (!viewUserId) return;
        userError = null;
        try {
            const response = await fetch(
                `${apiBaseUrl}/api/users/${viewUserId}`,
                {
                    credentials: 'include',
                },
            );
            if (response.ok) {
                viewedUser = await response.json();
            } else {
                userError = 'Benutzer konnte nicht geladen werden';
            }
        } catch (error) {
            console.error('Failed to fetch viewed user:', error);
            userError = 'Netzwerkfehler beim Laden des Profils';
        }
    }

    async function fetchPlayerAccounts() {
        if (accountsLoading) return;
        accountsLoading = true;
        hasInitialFetched = true;
        accountsError = null;
        try {
            const url = viewUserId
                ? `${apiBaseUrl}/api/users/${viewUserId}/accounts`
                : `${apiBaseUrl}/api/me/accounts`;

            const response = await fetch(url, {
                credentials: 'include',
            });
            if (response.ok) {
                const data = await response.json();
                // Backend now returns { coc: [...], cr: [...] }
                if (data && typeof data === 'object' && !Array.isArray(data)) {
                    cocPlayerAccounts = data.coc || [];
                    crPlayerAccounts = data.cr || [];
                } else {
                    // Fallback for old format (array = CoC accounts)
                    cocPlayerAccounts = Array.isArray(data) ? data : [];
                    crPlayerAccounts = [];
                }

                // If viewing own accounts, refresh global user state to get updated linked_players
                if (!viewUserId) {
                    fetchUser();
                }
            } else {
                accountsError = 'Fehler beim Laden der Accounts';
            }
        } catch (error) {
            console.error('Failed to fetch player accounts:', error);
            accountsError = 'Netzwerkfehler beim Laden der Accounts';
        } finally {
            accountsLoading = false;
        }
    }

    async function openPlayerModal(player: any, gameType: GameType) {
        selectedPlayer = player;
        selectedGameType = gameType;
        document.body.style.overflow = 'hidden';

        try {
            const encodedTag = encodeURIComponent(player.tag);
            const apiPrefix = gameType === 'coc' ? '/api/coc' : '/api/cr';

            // Fetch identity and kickpoints in parallel for ALL games
            const [kpRes, idRes] = await Promise.all([
                fetch(
                    `${apiBaseUrl}${apiPrefix}/players/${encodedTag}/kickpoints/details`,
                    { credentials: 'include' },
                ),
                fetch(
                    `${apiBaseUrl}${apiPrefix}/players/${encodedTag}/identity`,
                    { credentials: 'include' },
                ),
            ]);

            let kickpoints = [];
            if (kpRes.ok) kickpoints = await kpRes.json();

            let identity = {};
            if (idRes.ok) identity = await idRes.json();

            if (selectedPlayer && selectedPlayer.tag === player.tag) {
                selectedPlayer = {
                    ...selectedPlayer,
                    ...identity,
                    activeKickpoints: kickpoints,
                };
            }
        } catch (error) {
            console.error('Failed to fetch player details:', error);
        }
    }

    function closePlayerModal() {
        selectedPlayer = null;
        document.body.style.overflow = '';
    }

    $: displayUser = viewedUser || ($user && !viewUserId ? $user : null);
    $: isViewingOthers = viewUserId && $user && viewUserId !== $user.discord_id;

    onMount(() => {
        if ($user || viewUserId) {
            if (viewUserId) fetchViewedUser();
            fetchPlayerAccounts();
        }

        return () => {
            document.body.style.overflow = '';
        };
    });

    let lastId = viewUserId;
    $: if (viewUserId !== lastId) {
        lastId = viewUserId;
        hasInitialFetched = false;
        if (viewUserId) fetchViewedUser();
        else viewedUser = null;
        fetchPlayerAccounts();
    }

    $: if (
        ($user || viewUserId) &&
        !accountsLoading &&
        !hasInitialFetched &&
        !accountsError &&
        !userError
    ) {
        if (viewUserId) fetchViewedUser();
        fetchPlayerAccounts();
    }
</script>

<div class="profile-page" class:light={theme === 'light'}>
    <div class="container">
        {#if $loading || (viewUserId && !viewedUser && !accountsError && !userError)}
            <div class="loading-state">
                <div class="spinner"></div>
                <p>Lade Profil...</p>
            </div>
        {:else if !displayUser}
            <div class="error-state">
                <h2>
                    {isViewingOthers
                        ? 'Benutzer nicht gefunden'
                        : 'Nicht angemeldet'}
                </h2>
                <p>
                    {isViewingOthers
                        ? userError ||
                          'Der angeforderte Benutzer konnte nicht gefunden werden.'
                        : 'Bitte melde dich mit Discord an, um dein Profil zu sehen.'}
                </p>
            </div>
        {:else}
            {#if isViewingOthers}
                <div class="admin-view-banner" in:slide>
                    <svg
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                    >
                        <path
                            d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"
                        /><circle cx="12" cy="12" r="3" />
                    </svg>
                    <span
                        >Admin-Ansicht: Du betrachtest das Profil eines anderen
                        Mitglieds</span
                    >
                </div>
            {/if}

            <div class="profile-header">
                <div class="user-info">
                    <div class="avatar-container">
                        {#if displayUser.avatar}
                            <img
                                src={displayUser.avatar}
                                alt={displayUser.username}
                                class="avatar"
                            />
                        {:else}
                            <div class="avatar-placeholder">
                                {(
                                    displayUser.nickname ||
                                    displayUser.global_name ||
                                    displayUser.username
                                )
                                    .charAt(0)
                                    .toUpperCase()}
                            </div>
                        {/if}
                        <div class="status-badge" title="Online"></div>
                    </div>
                    <div class="user-details">
                        <h1>
                            {displayUser.nickname ||
                                displayUser.global_name ||
                                displayUser.username}
                        </h1>
                        <p class="discord-tag">@{displayUser.username}</p>
                        <div class="badges">
                            {#if displayUser.highest_role && displayUser.highest_role !== 'ADMIN'}
                                <span class="badge role-badge"
                                    >{displayUser.highest_role}</span
                                >
                            {/if}
                            {#if displayUser.is_admin}
                                <span class="badge admin-badge">Admin</span>
                            {/if}
                        </div>
                    </div>
                </div>
            </div>

            <div class="profile-content">
                <!-- CoC Accounts Section -->
                <section class="accounts-section coc-section">
                    <div class="section-header">
                        <div class="section-title-group">
                            <div class="game-icon coc-icon">
                                <svg
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                >
                                    <path
                                        d="M14.5 17.5 3 6V3h3l11.5 11.5"
                                    /><path d="m13 19 6-6" /><path
                                        d="m16 16 4 4"
                                    /><path d="m19 21 1-1" /><path
                                        d="M9.5 17.5 21 6V3h-3L6.5 14.5"
                                    /><path d="m11 19-6-6" /><path
                                        d="m8 16-4 4"
                                    /><path d="m5 21-1-1" />
                                </svg>
                            </div>
                            <h2>Clash of Clans Accounts</h2>
                        </div>
                        <span class="account-count coc-count"
                            >{cocPlayerAccounts.length} Accounts</span
                        >
                    </div>

                    {#if accountsLoading}
                        <div class="accounts-loading">
                            <div class="loading-grid">
                                {#each Array(2) as _}
                                    <div class="skeleton-card"></div>
                                {/each}
                            </div>
                        </div>
                    {:else if accountsError}
                        <div class="error-message">
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
                            <p>{accountsError}</p>
                            <button
                                on:click={fetchPlayerAccounts}
                                class="retry-btn">Erneut versuchen</button
                            >
                        </div>
                    {:else if cocPlayerAccounts.length === 0}
                        <div class="empty-state">
                            <div class="empty-icon">⚔️</div>
                            <p>Keine Clash of Clans Accounts verknüpft.</p>
                            <p class="sub-text">
                                Verknüpfe deine Accounts auf unserem Discord
                                Server.
                            </p>
                        </div>
                    {:else}
                        <div class="accounts-grid">
                            {#each cocPlayerAccounts as player}
                                {@const activePoints =
                                    player.activeKickpointsSum || 0}
                                <div
                                    class="account-card coc-card"
                                    class:light={theme === 'light'}
                                    on:click={() =>
                                        openPlayerModal(player, 'coc')}
                                    on:keydown={(e) =>
                                        e.key === 'Enter' &&
                                        openPlayerModal(player, 'coc')}
                                    role="button"
                                    tabindex="0"
                                >
                                    <div class="card-glow coc-glow"></div>
                                    <div class="player-header">
                                        <div class="player-rank">
                                            <img
                                                src={player.leagueTier?.iconUrls
                                                    ?.large ||
                                                    player.league?.iconUrls
                                                        ?.large}
                                                alt={player.leagueTier?.name ||
                                                    player.league?.name}
                                                class="league-icon"
                                            />
                                        </div>
                                        <div class="player-main">
                                            <h3 class="player-name">
                                                {player.nameDB || player.name}
                                            </h3>
                                            <p class="player-tag">
                                                {player.tag}
                                            </p>
                                        </div>
                                        {#if player.roleInClan && player.roleInClan !== 'NOTINCLAN'}
                                            <div class="role-badge-small">
                                                {player.roleInClan}
                                            </div>
                                        {/if}
                                    </div>

                                    <div class="player-stats">
                                        <div class="stat-item">
                                            <span class="stat-label"
                                                >Rathaus-Level</span
                                            >
                                            <span class="stat-value"
                                                >{player.townHallLevel}</span
                                            >
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label"
                                                >Kickpunkte</span
                                            >
                                            <span
                                                class="stat-value"
                                                class:danger={activePoints >=
                                                    (player.clanDB
                                                        ?.maxKickpoints || 9)}
                                            >
                                                {activePoints} / {player.clanDB
                                                    ?.maxKickpoints || '-'}
                                            </span>
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label"
                                                >Gesamtanzahl Kickpunkte</span
                                            >
                                            <span class="stat-value"
                                                >{player.totalKickpoints}</span
                                            >
                                        </div>
                                        <div class="stat-item span-3">
                                            <span class="stat-label">Clan</span>
                                            <span class="stat-value">
                                                {#if player.clanDB?.badgeUrl}
                                                    <img
                                                        src={player.clanDB
                                                            .badgeUrl}
                                                        alt=""
                                                        class="small-badge"
                                                    />
                                                {/if}
                                                {player.clanDB?.nameDB ||
                                                    'Kein Clan'}
                                            </span>
                                        </div>
                                        <div class="stat-item span-3">
                                            <span class="stat-label"
                                                >Aktueller Clan</span
                                            >
                                            <span class="stat-value">
                                                {#if player.clan?.badgeUrls?.large}
                                                    <img
                                                        src={player.clan
                                                            .badgeUrls.large}
                                                        alt=""
                                                        class="small-badge"
                                                    />
                                                {/if}
                                                {player.clan?.name ||
                                                    'Kein Clan'}
                                            </span>
                                        </div>
                                    </div>

                                    {#if player.activeKickpoints && player.activeKickpoints.length > 0}
                                        <div class="kickpoints-details">
                                            <span class="details-title"
                                                >Aktive Kickpunkte:</span
                                            >
                                            {#each player.activeKickpoints as kp}
                                                <div class="kp-reason">
                                                    <span class="kp-amount"
                                                        >+{kp.amount}</span
                                                    >
                                                    <span class="kp-desc"
                                                        >{kp.description}</span
                                                    >
                                                    <span class="kp-date"
                                                        >{new Date(
                                                            kp.date,
                                                        ).toLocaleDateString()}</span
                                                    >
                                                </div>
                                            {/each}
                                        </div>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    {/if}
                </section>

                <!-- CR Accounts Section -->
                <section class="accounts-section cr-section">
                    <div class="section-header">
                        <div class="section-title-group">
                            <div class="game-icon cr-icon">
                                <svg
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                >
                                    <path
                                        d="m2 4 3 12h14l3-12-6 7-4-7-4 7-6-7zm3 16h14"
                                    />
                                </svg>
                            </div>
                            <h2>Clash Royale Accounts</h2>
                        </div>
                        <span class="account-count cr-count"
                            >{crPlayerAccounts.length} Accounts</span
                        >
                    </div>

                    {#if accountsLoading}
                        <div class="accounts-loading">
                            <div class="loading-grid">
                                {#each Array(2) as _}
                                    <div class="skeleton-card"></div>
                                {/each}
                            </div>
                        </div>
                    {:else if crPlayerAccounts.length === 0}
                        <div class="empty-state">
                            <div class="empty-icon">👑</div>
                            <p>Keine Clash Royale Accounts verknüpft.</p>
                            <p class="sub-text">
                                Verknüpfe deine Accounts auf unserem Discord
                                Server.
                            </p>
                        </div>
                    {:else}
                        <div class="accounts-grid">
                            {#each crPlayerAccounts as player}
                                {@const activePoints =
                                    player.activeKickpointsSum || 0}
                                <div
                                    class="account-card cr-card"
                                    class:light={theme === 'light'}
                                    on:click={() =>
                                        openPlayerModal(player, 'cr')}
                                    on:keydown={(e) =>
                                        e.key === 'Enter' &&
                                        openPlayerModal(player, 'cr')}
                                    role="button"
                                    tabindex="0"
                                >
                                    <div class="card-glow cr-glow"></div>
                                    <div class="player-header">
                                        <div class="player-rank">
                                            {#if player.arena}
                                                <div class="arena-badge">
                                                    {player.arena.name ||
                                                        `Arena ${player.arena.id}`}
                                                </div>
                                            {:else}
                                                <div class="level-badge">
                                                    LVL {player.expLevel || '?'}
                                                </div>
                                            {/if}
                                        </div>
                                        <div class="player-main">
                                            <h3 class="player-name">
                                                {player.nameDB || player.name}
                                            </h3>
                                            <p class="player-tag">
                                                {player.tag}
                                            </p>
                                        </div>
                                        {#if player.role && player.role !== 'notMember'}
                                            <div
                                                class="role-badge-small cr-role"
                                            >
                                                {player.role}
                                            </div>
                                        {/if}
                                    </div>

                                    <div class="player-stats">
                                        <div class="stat-item">
                                            <span class="stat-label">Level</span
                                            >
                                            <span class="stat-value"
                                                >{player.expLevel || '-'}</span
                                            >
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label"
                                                >Kickpunkte</span
                                            >
                                            <span
                                                class="stat-value"
                                                class:danger={activePoints >=
                                                    (player.clanDB
                                                        ?.maxKickpoints || 9)}
                                            >
                                                {activePoints} / {player.clanDB
                                                    ?.maxKickpoints || '-'}
                                            </span>
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label"
                                                >Gesamtanzahl Kickpunkte</span
                                            >
                                            <span class="stat-value"
                                                >{player.totalKickpoints ||
                                                    0}</span
                                            >
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label"
                                                >Trophäen</span
                                            >
                                            <span class="stat-value"
                                                >{player.trophies || '-'}</span
                                            >
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label"
                                                >Bestrang</span
                                            >
                                            <span class="stat-value"
                                                >{player.bestTrophies ||
                                                    '-'}</span
                                            >
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label">Clan</span>
                                            <span class="stat-value">
                                                {#if player.clan?.badgeUrls?.large}
                                                    <img
                                                        src={player.clan
                                                            .badgeUrls.large}
                                                        alt=""
                                                        class="small-badge"
                                                    />
                                                {/if}
                                                {player.clan?.name ||
                                                    'Kein Clan'}
                                            </span>
                                        </div>
                                    </div>

                                    {#if player.activeKickpoints && player.activeKickpoints.length > 0}
                                        <div class="kickpoints-details">
                                            <span class="details-title"
                                                >Aktive Kickpunkte:</span
                                            >
                                            {#each player.activeKickpoints as kp}
                                                <div class="kp-reason">
                                                    <span class="kp-amount"
                                                        >+{kp.amount}</span
                                                    >
                                                    <span class="kp-desc"
                                                        >{kp.description}</span
                                                    >
                                                    <span class="kp-date"
                                                        >{new Date(
                                                            kp.date,
                                                        ).toLocaleDateString()}</span
                                                    >
                                                </div>
                                            {/each}
                                        </div>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    {/if}
                </section>
            </div>
        {/if}
    </div>
</div>

<PlayerDetailModal
    isOpen={!!selectedPlayer}
    player={selectedPlayer}
    gameType={selectedGameType}
    {theme}
    onClose={closePlayerModal}
    hasPrivilegedAccess={!!(
        $user?.is_admin ||
        (viewUserId && $user && viewUserId === $user.discord_id)
    )}
/>

<style>
    .profile-page {
        color: white;
        position: relative;
        z-index: 1;
    }

    .profile-page.light {
        color: #1e293b;
    }

    /* Custom Scrollbar */
    :global(::-webkit-scrollbar) {
        width: 10px;
        height: 10px;
    }

    :global(::-webkit-scrollbar-track) {
        background: transparent;
    }

    :global(::-webkit-scrollbar-thumb) {
        background: rgba(59, 130, 246, 0.3);
        border-radius: 5px;
        border: 2px solid transparent;
        background-clip: content-box;
    }

    :global(::-webkit-scrollbar-thumb:hover) {
        background: rgba(59, 130, 246, 0.5);
    }

    :global(::-webkit-scrollbar-corner) {
        background: transparent;
    }

    :global(::-webkit-scrollbar-button) {
        display: none;
    }

    :global(::-webkit-resizer) {
        display: none;
    }

    /* Firefox */
    :global(*) {
        scrollbar-width: thin;
        scrollbar-color: rgba(59, 130, 246, 0.3) transparent;
    }

    .profile-page {
        min-height: 100vh;
        background: radial-gradient(
                circle at top right,
                rgba(59, 130, 246, 0.05),
                transparent 400px
            ),
            radial-gradient(
                circle at bottom left,
                rgba(59, 130, 246, 0.03),
                transparent 400px
            );
        color: white;
        padding-bottom: 5rem;
    }

    .profile-page.light {
        background: #f1f5f9;
        color: #1e293b;
    }

    .container {
        max-width: 1200px;
        margin: 0 auto;
        padding: 0 1.5rem;
    }

    .profile-content {
        display: flex;
        flex-direction: column;
        gap: 4rem;
    }

    .accounts-section {
        animation: fadeIn 0.5s ease-out;
    }

    @keyframes fadeIn {
        from {
            opacity: 0;
            transform: translateY(20px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    .section-title-group {
        display: flex;
        align-items: center;
        gap: 1.25rem;
    }

    .game-icon {
        width: 52px;
        height: 52px;
        border-radius: 16px;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
        transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .game-icon:hover {
        transform: scale(1.1) rotate(-5deg);
    }

    .coc-icon {
        background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
        color: white;
        border: 1px solid rgba(245, 158, 11, 0.3);
    }

    .cr-icon {
        background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
        color: white;
        border: 1px solid rgba(59, 130, 246, 0.3);
    }

    .game-icon svg {
        width: 28px;
        height: 28px;
        filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.2));
    }

    .profile-header {
        background: rgba(255, 255, 255, 0.03);
        backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 24px;
        padding: 2.5rem;
        margin-bottom: 2.5rem;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
    }

    .light .profile-header {
        background: white;
        border: 1px solid rgba(0, 0, 0, 0.05);
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.05);
    }

    .user-info {
        display: flex;
        align-items: center;
        gap: 2rem;
    }

    .avatar-container {
        position: relative;
        flex-shrink: 0;
    }

    .avatar {
        width: 100px;
        height: 100px;
        border-radius: 50%;
        border: 4px solid #3b82f6;
        object-fit: cover;
    }

    .avatar-placeholder {
        width: 100px;
        height: 100px;
        border-radius: 50%;
        background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 2.5rem;
        font-weight: bold;
        color: white;
        border: 4px solid rgba(255, 255, 255, 0.1);
    }

    .status-badge {
        position: absolute;
        bottom: 5px;
        right: 5px;
        width: 18px;
        height: 18px;
        border-radius: 50%;
        background: #22c55e;
        border: 3px solid rgba(20, 20, 30, 1);
    }

    .light .status-badge {
        border-color: white;
    }

    .user-details h1 {
        margin: 0;
        font-size: 2.25rem;
        font-weight: 800;
        letter-spacing: -0.02em;
    }

    .discord-tag {
        margin: 0.25rem 0 1rem;
        font-size: 1.1rem;
        color: rgba(255, 255, 255, 0.5);
        font-family: 'JetBrains Mono', monospace;
    }

    .light .discord-tag {
        color: rgba(0, 0, 0, 0.4);
    }

    .badges {
        display: flex;
        gap: 0.75rem;
    }

    .badge {
        padding: 0.4rem 0.8rem;
        border-radius: 8px;
        font-size: 0.85rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .role-badge {
        background: rgba(59, 130, 246, 0.15);
        color: #60a5fa;
        border: 1px solid rgba(59, 130, 246, 0.2);
    }

    .admin-badge {
        background: rgba(244, 63, 94, 0.15);
        color: #fb7185;
        border: 1px solid rgba(244, 63, 94, 0.2);
    }

    .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1.5rem;
    }

    .section-header h2 {
        font-size: 1.5rem;
        font-weight: 700;
        margin: 0;
    }

    .account-count {
        font-size: 0.9rem;
        padding: 0.25rem 0.75rem;
        border-radius: 20px;
        font-weight: 600;
    }

    .coc-count {
        color: #f59e0b;
        background: rgba(245, 158, 11, 0.1);
        border: 1px solid rgba(245, 158, 11, 0.2);
    }

    .cr-count {
        color: #60a5fa;
        background: rgba(59, 130, 246, 0.1);
        border: 1px solid rgba(59, 130, 246, 0.2);
    }

    .light .account-count {
        background: rgba(0, 0, 0, 0.05);
        border: none;
    }

    .light .coc-count {
        color: #d97706;
    }

    .light .cr-count {
        color: #2563eb;
    }

    .accounts-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
        gap: 2rem;
    }

    .account-card {
        position: relative;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 24px;
        padding: 2rem;
        overflow: hidden;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        cursor: pointer;
        outline: none;
        display: flex;
        flex-direction: column;
    }

    .account-card:focus {
        outline: none;
    }

    .account-card.coc-card:hover {
        border-color: rgba(245, 158, 11, 0.3);
        box-shadow: 0 10px 30px rgba(245, 158, 11, 0.05);
    }

    .account-card.cr-card:hover {
        border-color: rgba(59, 130, 246, 0.3);
        box-shadow: 0 10px 30px rgba(59, 130, 246, 0.05);
    }

    .account-card.light {
        background: white;
        border-color: rgba(0, 0, 0, 0.05);
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.03);
    }

    .account-card.light:hover {
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.06);
    }

    .light .player-name {
        color: #1e293b;
    }

    .light .stat-value {
        color: #334155;
    }

    .light .small-badge {
        filter: none;
    }

    .card-glow {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 150px;
        pointer-events: none;
        opacity: 0.5;
    }

    .coc-glow {
        background: radial-gradient(
            circle at 100% 0%,
            rgba(245, 158, 11, 0.1),
            transparent 70%
        );
    }

    .cr-glow {
        background: radial-gradient(
            circle at 100% 0%,
            rgba(59, 130, 246, 0.1),
            transparent 70%
        );
    }

    .player-header {
        display: flex;
        align-items: center;
        gap: 1.25rem;
        margin-bottom: 2rem;
        position: relative;
    }

    .league-icon {
        width: 56px;
        height: 56px;
        filter: drop-shadow(0 4px 12px rgba(0, 0, 0, 0.4));
    }

    .player-main {
        flex-grow: 1;
    }

    .player-name {
        margin: 0;
        font-size: 1.4rem;
        font-weight: 800;
        color: white;
        letter-spacing: -0.01em;
    }

    .player-tag {
        margin: 0;
        font-size: 0.85rem;
        color: rgba(255, 255, 255, 0.4);
        font-family: 'JetBrains Mono', monospace;
    }

    .light .player-tag {
        color: rgba(0, 0, 0, 0.4);
    }

    .player-stats {
        display: grid;
        grid-template-columns: repeat(6, 1fr);
        gap: 1.5rem 0.5rem;
        background: rgba(0, 0, 0, 0.2);
        border: 1px solid rgba(255, 255, 255, 0.03);
        border-radius: 20px;
        padding: 1.5rem 1rem;
        margin-bottom: 2rem;
    }

    .light .player-stats {
        background: #f8fafc;
    }

    .stat-item {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        grid-column: span 2;
    }

    .stat-item.span-3 {
        grid-column: span 3;
    }

    .stat-label {
        font-size: 0.65rem;
        text-transform: uppercase;
        color: rgba(255, 255, 255, 0.3);
        letter-spacing: 0.08em;
        margin-bottom: 0.5rem;
        font-weight: 700;
        white-space: nowrap;
    }

    .light .stat-label {
        color: rgba(0, 0, 0, 0.4);
    }

    .stat-value {
        font-size: 1.1rem;
        font-weight: 800;
        color: #ffffff;
        display: flex;
        align-items: center;
        gap: 0.4rem;
    }

    .small-badge {
        width: 20px;
        height: 20px;
        object-fit: contain;
    }

    .stat-value.danger {
        color: #f87171;
    }

    .role-badge-small {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.3);
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .cr-card .role-badge-small {
        color: #60a5fa;
    }

    .arena-badge,
    .level-badge {
        background: #2563eb;
        color: white;
        padding: 0.3rem 0.75rem;
        border-radius: 10px;
        font-size: 0.7rem;
        font-weight: 800;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        box-shadow: 0 4px 10px rgba(37, 99, 235, 0.2);
    }

    .light .arena-badge,
    .light .level-badge {
        background: rgba(59, 130, 246, 0.1);
        color: #2563eb;
    }

    .kickpoints-details {
        margin-top: 1rem;
        padding-top: 1rem;
        border-top: 1px solid rgba(255, 255, 255, 0.05);
    }

    .light .kickpoints-details {
        border-top-color: rgba(0, 0, 0, 0.05);
    }

    .details-title {
        display: block;
        font-size: 0.75rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.4);
        margin-bottom: 0.5rem;
    }

    .light .details-title {
        color: rgba(0, 0, 0, 0.4);
    }

    .kp-reason {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-size: 0.8rem;
        margin-bottom: 0.4rem;
    }

    .kp-amount {
        color: #ef4444;
        font-weight: 700;
        flex-shrink: 0;
    }

    .kp-desc {
        flex-grow: 1;
        color: rgba(255, 255, 255, 0.8);
    }

    .light .kp-desc {
        color: rgba(0, 0, 0, 0.8);
    }

    .kp-date {
        font-size: 0.7rem;
        color: rgba(255, 255, 255, 0.3);
        font-family: 'JetBrains Mono', monospace;
    }

    .light .kp-date {
        color: rgba(0, 0, 0, 0.3);
    }

    .loading-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
        gap: 1.5rem;
    }

    .skeleton-card {
        height: 200px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 20px;
        position: relative;
        overflow: hidden;
    }

    .skeleton-card::after {
        content: '';
        position: absolute;
        top: 0;
        right: 0;
        bottom: 0;
        left: 0;
        transform: translateX(-100%);
        background: linear-gradient(
            90deg,
            transparent,
            rgba(255, 255, 255, 0.05),
            transparent
        );
        animation: shimmer 1.5s infinite;
    }

    @keyframes shimmer {
        100% {
            transform: translateX(100%);
        }
    }

    .error-message {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        padding: 3rem;
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.2);
        border-radius: 20px;
        color: #fca5a5;
    }

    .error-message svg {
        width: 48px;
        height: 48px;
    }

    .retry-btn {
        padding: 0.6rem 1.5rem;
        background: #ef4444;
        color: white;
        border: none;
        border-radius: 10px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
    }

    .retry-btn:hover {
        background: #dc2626;
        transform: scale(1.05);
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 4rem 2rem;
        text-align: center;
        background: rgba(255, 255, 255, 0.02);
        border: 2px dashed rgba(255, 255, 255, 0.05);
        border-radius: 24px;
        margin: 1.5rem 0;
        animation: fadeIn 0.5s ease-out;
    }

    .light .empty-state {
        background: rgba(0, 0, 0, 0.02);
        border-color: rgba(0, 0, 0, 0.05);
    }

    .empty-icon {
        font-size: 4rem;
        margin-bottom: 1.5rem;
        filter: drop-shadow(0 0 15px rgba(255, 255, 255, 0.1));
    }

    .light .empty-icon {
        filter: drop-shadow(0 0 15px rgba(0, 0, 0, 0.1));
    }

    .empty-state p {
        margin: 0.5rem 0;
        font-size: 1.2rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.8);
    }

    .empty-state .sub-text {
        font-size: 0.95rem;
        color: rgba(255, 255, 255, 0.4);
    }

    .light .empty-state p {
        color: #1e293b;
    }

    .light .empty-state .sub-text {
        color: #64748b;
    }
    }

    .admin-view-banner {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        background: rgba(59, 130, 246, 0.1);
        border: 1px solid rgba(59, 130, 246, 0.2);
        color: #60a5fa;
        padding: 0.75rem 1rem;
        border-radius: 0.75rem;
        margin-bottom: 2rem;
        font-weight: 500;
    }

    .admin-view-banner svg {
        width: 1.25rem;
        height: 1.25rem;
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid rgba(255, 255, 255, 0.1);
        border-top-color: #3b82f6;
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin: 0 auto 1.5rem;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    @media (max-width: 768px) {
        .user-info {
            flex-direction: column;
            text-align: center;
            gap: 1.5rem;
        }

        .profile-header {
            padding: 2rem 1.5rem;
        }

        .accounts-grid {
            grid-template-columns: 1fr;
        }
    }
</style>
