<script lang="ts">
    import { onMount } from 'svelte';
    import { fade, scale, slide } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
    import { user, loading } from './auth';
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
                }
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
                    { credentials: 'include' }
                ),
                fetch(
                    `${apiBaseUrl}${apiPrefix}/players/${encodedTag}/identity`,
                    { credentials: 'include' }
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
                                <svg viewBox="0 0 24 24" fill="currentColor">
                                    <path
                                        d="M12 2L2 7l10 5 10-5-10-5zm0 9l2.5-1.25L12 8.5l-2.5 1.25L12 11zm0 2.5l-5-2.5-5 2.5L12 22l10-8.5-5-2.5-5 2.5z"
                                    />
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
                                        <div class="stat-item">
                                            <span class="stat-label">Clan</span>
                                            <span class="stat-value">
                                                {#if player.clanDB?.badgeUrl}
                                                    <img
                                                        src={player.clanDB
                                                            .badgeUrl}
                                                        alt=""
                                                        style="width: 25px; height: 25px; vertical-align: middle; margin-right: 4px; display: inline-block;"
                                                    />
                                                {/if}
                                                {player.clanDB?.nameDB ||
                                                    'Kein Clan'}
                                            </span>
                                        </div>
                                        <div class="stat-item">
                                            <span class="stat-label"
                                                >Aktueller Clan</span
                                            >
                                            <span class="stat-value">
                                                {#if player.clan?.badgeUrls?.large}
                                                    <img
                                                        src={player.clan
                                                            .badgeUrls.large}
                                                        alt=""
                                                        style="width: 25px; height: 25px; vertical-align: middle; margin-right: 4px; display: inline-block;"
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
                                                            kp.date
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
                                <svg viewBox="0 0 24 24" fill="currentColor">
                                    <path
                                        d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"
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
                                                        style="width: 25px; height: 25px; vertical-align: middle; margin-right: 4px; display: inline-block;"
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
                                                            kp.date
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

{#if selectedPlayer}
    <div
        class="modal-backdrop"
        on:click|self={closePlayerModal}
        on:keydown={(e) => e.key === 'Escape' && closePlayerModal()}
        class:light={theme === 'light'}
        role="button"
        tabindex="-1"
        transition:fade={{ duration: 250 }}
    >
        <div
            class="modal-content"
            class:light={theme === 'light'}
            transition:scale={{
                duration: 400,
                delay: 50,
                opacity: 0,
                start: 0.95,
                easing: quintOut,
            }}
        >
            <button
                class="close-modal"
                on:click={closePlayerModal}
                aria-label="Schließen"
            >
                <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
            </button>

            <div class="modal-scroll-area">
                <div class="modal-header">
                    <div class="player-info-large">
                        {#if selectedGameType === 'coc'}
                            <img
                                src={selectedPlayer.leagueTier?.iconUrls?.large}
                                alt={selectedPlayer.leagueTier?.name}
                                class="league-icon-large"
                            />
                        {:else if selectedPlayer.arena}
                            <div class="arena-icon-container">
                                <div class="arena-id-label">
                                    A{selectedPlayer.arena.id
                                        .toString()
                                        .slice(-2)}
                                </div>
                            </div>
                        {/if}
                        <div class="player-titles">
                            <h2>
                                {selectedPlayer.nameDB || selectedPlayer.name}
                            </h2>
                            <p class="tag">{selectedPlayer.tag}</p>
                            {#if selectedPlayer.clan}
                                <div class="clan-info-small">
                                    <img
                                        src={selectedPlayer.clan.badgeUrls
                                            ?.small}
                                        alt=""
                                    />
                                    <span>{selectedPlayer.clan.name}</span>
                                </div>
                            {/if}
                        </div>
                    </div>
                    <div class="th-display">
                        {#if selectedGameType === 'coc'}
                            <div class="th-level">
                                TH {selectedPlayer.townHallLevel}
                            </div>
                            {#if selectedPlayer.builderHallLevel}
                                <div class="bh-level">
                                    BH {selectedPlayer.builderHallLevel}
                                </div>
                            {/if}
                        {:else}
                            <div class="th-level cr-level">
                                {selectedPlayer.arena?.name ||
                                    `Arena ${selectedPlayer.arena?.id || '?'}`}
                            </div>
                        {/if}
                    </div>
                </div>

                <div class="modal-body">
                    <div class="stats-grid-large">
                        <div class="stat-card">
                            <span class="label">EXP Level</span>
                            <span class="value">{selectedPlayer.expLevel}</span>
                        </div>
                        {#if selectedGameType === 'coc'}
                            <div class="stat-card">
                                <span class="label">Kriegssterne</span>
                                <span class="value"
                                    >{selectedPlayer.warStars}</span
                                >
                            </div>
                        {:else}
                            <div class="stat-card">
                                <span class="label">Trophäen</span>
                                <span class="value"
                                    >{selectedPlayer.trophies || 0}</span
                                >
                            </div>
                        {/if}
                        <div class="stat-card">
                            <span class="label">Spenden</span>
                            <span class="value"
                                >{selectedPlayer.donations || 0}</span
                            >
                        </div>
                        <div class="stat-card">
                            <span class="label">Erhaltene Spenden</span>
                            <span class="value"
                                >{selectedPlayer.donationsReceived || 0}</span
                            >
                        </div>
                    </div>

                    {#if selectedGameType === 'cr' && selectedPlayer.currentDeck}
                        <div class="detail-section">
                            <h3>Aktuelles Deck</h3>
                            <div class="deck-grid">
                                {#each selectedPlayer.currentDeck as card}
                                    <div class="card-item" title={card.name}>
                                        <img
                                            src={card.iconUrls?.medium}
                                            alt={card.name}
                                        />
                                        <span class="card-level"
                                            >LVL {card.level +
                                                (card.rarity === 'legendary'
                                                    ? 8
                                                    : card.rarity === 'epic'
                                                      ? 5
                                                      : card.rarity === 'rare'
                                                        ? 2
                                                        : 0)}</span
                                        >
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    {#if selectedGameType === 'cr' && selectedPlayer.badges && selectedPlayer.badges.length > 0}
                        <div class="detail-section">
                            <h3>Abzeichen</h3>
                            <div class="badges-grid-cr">
                                {#each selectedPlayer.badges as badge}
                                    <div
                                        class="badge-item-cr"
                                        title={badge.name}
                                    >
                                        <img
                                            src={badge.iconUrls?.large}
                                            alt={badge.name}
                                        />
                                        {#if badge.level}
                                            <span class="badge-level-cr"
                                                >Lvl {badge.level}</span
                                            >
                                        {/if}
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    {#if selectedPlayer.heroes && selectedPlayer.heroes.length > 0}
                        <div class="detail-section">
                            <h3>Helden</h3>
                            <div class="items-grid">
                                {#each selectedPlayer.heroes as hero}
                                    <div class="item-badge" title={hero.name}>
                                        <span class="item-name"
                                            >{hero.name}</span
                                        >
                                        <span class="item-level"
                                            >{hero.level} / {hero.maxLevel}</span
                                        >
                                        <div
                                            class="progress-bar"
                                            style="width: {(hero.level /
                                                hero.maxLevel) *
                                                100}%"
                                        ></div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    {#if selectedPlayer.labels && selectedPlayer.labels.length > 0}
                        <div class="detail-section">
                            <h3>Labels</h3>
                            <div class="labels-list">
                                {#each selectedPlayer.labels as label}
                                    <div class="label-badge">
                                        <img
                                            src={label.iconUrls?.small}
                                            alt={label.name}
                                        />
                                        <span>{label.name}</span>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    {#if selectedPlayer.troops && selectedPlayer.troops.length > 0}
                        <div class="detail-section">
                            <h3>Truppen</h3>
                            <div class="items-grid">
                                {#each selectedPlayer.troops.filter((t: any) => !t.name.startsWith('Super ') && !['Sneaky Goblin', 'Rocket Balloon', 'Inferno Dragon', 'Ice Hound'].includes(t.name)) as troop}
                                    <div class="item-badge" title={troop.name}>
                                        <span class="item-name"
                                            >{troop.name}</span
                                        >
                                        <span class="item-level"
                                            >{troop.level} / {troop.maxLevel}</span
                                        >
                                        <div
                                            class="progress-bar"
                                            style="width: {(troop.level /
                                                troop.maxLevel) *
                                                100}%"
                                        ></div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    {#if selectedPlayer.spells && selectedPlayer.spells.length > 0}
                        <div class="detail-section">
                            <h3>Zauber</h3>
                            <div class="items-grid">
                                {#each selectedPlayer.spells as spell}
                                    <div class="item-badge" title={spell.name}>
                                        <span class="item-name"
                                            >{spell.name}</span
                                        >
                                        <span class="item-level"
                                            >{spell.level} / {spell.maxLevel}</span
                                        >
                                        <div
                                            class="progress-bar"
                                            style="width: {(spell.level /
                                                spell.maxLevel) *
                                                100}%"
                                        ></div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    .profile-page {
        min-height: calc(100vh - 64px);
        padding: 3rem 1rem;
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

    .container {
        max-width: 1000px;
        margin: 0 auto;
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
        color: rgba(255, 255, 255, 0.4);
        background: rgba(255, 255, 255, 0.05);
        padding: 0.25rem 0.75rem;
        border-radius: 20px;
    }

    .light .account-count {
        color: rgba(0, 0, 0, 0.4);
        background: rgba(0, 0, 0, 0.05);
    }

    .accounts-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
        gap: 1.5rem;
    }

    .account-card {
        position: relative;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 20px;
        padding: 1.5rem;
        overflow: hidden;
        transition: all 0.3s ease;
        cursor: pointer;
        outline: none;
    }

    .account-card:focus {
        outline: none;
    }

    .account-card:hover {
        transform: translateY(-5px);
        background: rgba(255, 255, 255, 0.04);
        border-color: rgba(255, 255, 255, 0.1);
    }

    .account-card.light {
        background: white;
        border-color: rgba(0, 0, 0, 0.05);
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.03);
    }

    .account-card.light:hover {
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.06);
    }

    .card-glow {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 100px;
        background: radial-gradient(
            600px circle at var(--x, 50%) var(--y, 0%),
            rgba(59, 130, 246, 0.1),
            transparent 40%
        );
        pointer-events: none;
    }

    .player-header {
        display: flex;
        align-items: center;
        gap: 1rem;
        margin-bottom: 1.5rem;
    }

    .league-icon {
        width: 48px;
        height: 48px;
        filter: drop-shadow(0 0 8px rgba(0, 0, 0, 0.3));
    }

    .player-main {
        flex-grow: 1;
    }

    .player-name {
        margin: 0;
        font-size: 1.25rem;
        font-weight: 700;
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
        grid-template-columns: repeat(3, 1fr);
        gap: 0.5rem;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 12px;
        padding: 1rem;
        margin-bottom: 1.25rem;
    }

    .light .player-stats {
        background: #f8fafc;
    }

    .stat-item {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
    }

    .stat-label {
        font-size: 0.7rem;
        text-transform: uppercase;
        color: rgba(255, 255, 255, 0.3);
        letter-spacing: 0.05em;
        margin-bottom: 0.25rem;
    }

    .light .stat-label {
        color: rgba(0, 0, 0, 0.4);
    }

    .stat-value {
        font-size: 0.95rem;
        font-weight: 700;
    }

    .stat-value.danger {
        color: #ef4444;
    }

    .role-badge-small {
        font-size: 0.7rem;
        padding: 0.2rem 0.5rem;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 6px;
        color: rgba(255, 255, 255, 0.6);
        text-transform: uppercase;
        font-weight: 700;
    }

    .light .role-badge-small {
        background: rgba(0, 0, 0, 0.03);
        border-color: rgba(0, 0, 0, 0.08);
        color: rgba(0, 0, 0, 0.5);
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

    .loading-state,
    .error-state,
    .empty-state {
        text-align: center;
        padding: 5rem 2rem;
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

    /* Modal Styles */
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.8);
        backdrop-filter: blur(8px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        padding: 1rem;
    }

    .modal-content {
        background: #1e293b;
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 24px;
        width: 100%;
        max-width: 800px;
        max-height: 90vh;
        position: relative;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .modal-content.light {
        background: white;
        border-color: rgba(0, 0, 0, 0.1);
        color: #1e293b;
    }

    .modal-scroll-area {
        overflow-y: auto;
        flex-grow: 1;
    }

    .close-modal {
        position: absolute;
        top: 1.25rem;
        right: 1.25rem;
        background: none;
        border: none;
        color: rgba(255, 255, 255, 0.4);
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s ease;
        z-index: 10;
        padding: 0;
    }

    .light .close-modal {
        color: rgba(0, 0, 0, 0.4);
    }

    .close-modal:hover {
        color: white;
        transform: scale(1.1);
    }

    .light .close-modal:hover {
        color: black;
    }

    .close-modal svg {
        width: 100%;
        height: 100%;
    }

    .modal-header {
        padding: 2.5rem;
        background: linear-gradient(
            to bottom,
            rgba(59, 130, 246, 0.1),
            transparent
        );
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
    }

    .player-info-large {
        display: flex;
        align-items: center;
        gap: 1.5rem;
    }

    .league-icon-large {
        width: 80px;
        height: 80px;
        filter: drop-shadow(0 0 15px rgba(59, 130, 246, 0.3));
    }

    .player-titles h2 {
        margin: 0;
        font-size: 2rem;
        font-weight: 800;
    }

    .player-titles .tag {
        margin: 0.25rem 0 0;
        font-family: 'JetBrains Mono', monospace;
        color: rgba(255, 255, 255, 0.4);
    }

    .light .player-titles .tag {
        color: rgba(0, 0, 0, 0.4);
    }

    .clan-info-small {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-top: 0.5rem;
        font-size: 0.9rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.7);
    }

    .light .clan-info-small {
        color: rgba(0, 0, 0, 0.7);
    }

    .clan-info-small img {
        width: 20px;
        height: 20px;
    }

    .th-display {
        text-align: right;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .th-level,
    .bh-level {
        background: #3b82f6;
        color: white;
        padding: 0.5rem 1rem;
        border-radius: 12px;
        font-weight: 800;
        font-size: 0.9rem;
    }

    .cr-level {
        background: linear-gradient(135deg, #5865f2, #7289da);
    }

    .arena-icon-container {
        width: 80px;
        height: 80px;
        background: rgba(88, 101, 242, 0.1);
        border: 2px solid rgba(88, 101, 242, 0.3);
        border-radius: 16px;
        display: flex;
        align-items: center;
        justify-content: center;
        position: relative;
    }

    .arena-id-label {
        font-size: 1.2rem;
        font-weight: 900;
        color: #5865f2;
        text-shadow: 0 0 15px rgba(88, 101, 242, 0.4);
    }

    .deck-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 1rem;
        margin-top: 1rem;
    }

    .card-item {
        background: rgba(255, 255, 255, 0.03);
        border-radius: 12px;
        padding: 0.5rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5rem;
        transition: transform 0.2s;
    }

    .card-item:hover {
        transform: translateY(-5px);
        background: rgba(255, 255, 255, 0.05);
    }

    .card-item img {
        width: 100%;
        height: auto;
        border-radius: 8px;
    }

    .card-level {
        font-size: 0.75rem;
        font-weight: 800;
        color: #fff;
        background: #5865f2;
        padding: 0.2rem 0.5rem;
        border-radius: 6px;
    }

    .badges-grid-cr {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(60px, 1fr));
        gap: 1rem;
        margin-top: 1rem;
    }

    .badge-item-cr {
        position: relative;
        aspect-ratio: 1;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .badge-item-cr img {
        width: 100%;
        height: 100%;
        object-fit: contain;
    }

    .badge-level-cr {
        position: absolute;
        bottom: -5px;
        right: -5px;
        background: #febd31;
        color: #000;
        font-size: 0.65rem;
        font-weight: 900;
        padding: 0.1rem 0.3rem;
        border-radius: 4px;
        border: 2px solid #202225;
    }

    .bh-level {
        background: #8b5cf6;
    }

    .modal-body {
        padding: 0 2.5rem 2.5rem;
    }

    .stats-grid-large {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
        gap: 1rem;
        margin-bottom: 2.5rem;
    }

    .stat-card {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        padding: 1.25rem;
        border-radius: 16px;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .light .stat-card {
        background: #f8fafc;
        border-color: rgba(0, 0, 0, 0.05);
    }

    .stat-card .label {
        font-size: 0.75rem;
        text-transform: uppercase;
        color: rgba(255, 255, 255, 0.4);
        font-weight: 600;
        letter-spacing: 0.05em;
    }

    .light .stat-card .label {
        color: rgba(0, 0, 0, 0.4);
    }

    .stat-card .value {
        font-size: 1.25rem;
        font-weight: 700;
        color: #3b82f6;
    }

    .detail-section {
        margin-top: 2rem;
    }

    .detail-section h3 {
        font-size: 1.1rem;
        font-weight: 700;
        margin-bottom: 1.25rem;
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .detail-section h3::after {
        content: '';
        flex-grow: 1;
        height: 1px;
        background: rgba(255, 255, 255, 0.1);
    }

    .light .detail-section h3::after {
        background: rgba(0, 0, 0, 0.1);
    }

    .items-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1rem;
    }

    .item-badge {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        padding: 1rem;
        border-radius: 12px;
        position: relative;
        overflow: hidden;
    }

    .light .item-badge {
        background: white;
        border-color: rgba(0, 0, 0, 0.05);
    }

    .item-name {
        display: block;
        font-size: 0.85rem;
        font-weight: 600;
        margin-bottom: 0.25rem;
    }

    .item-level {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.5);
    }

    .light .item-level {
        color: rgba(0, 0, 0, 0.5);
    }

    .progress-bar {
        position: absolute;
        bottom: 0;
        left: 0;
        height: 3px;
        background: #3b82f6;
        opacity: 0.5;
    }

    .labels-list {
        display: flex;
        flex-wrap: wrap;
        gap: 0.75rem;
    }

    .label-badge {
        background: rgba(255, 255, 255, 0.05);
        padding: 0.5rem 1rem;
        border-radius: 20px;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.85rem;
        font-weight: 600;
    }

    .light .label-badge {
        background: #f1f5f9;
    }

    .label-badge img {
        width: 20px;
        height: 20px;
    }

    /* Game-specific styles */
    .section-title-group {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .game-icon {
        width: 32px;
        height: 32px;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .game-icon svg {
        width: 20px;
        height: 20px;
    }

    .coc-icon {
        background: rgba(59, 165, 92, 0.15);
        color: #3ba55c;
    }

    .cr-icon {
        background: rgba(88, 101, 242, 0.15);
        color: #5865f2;
    }

    .coc-count {
        background: rgba(59, 165, 92, 0.1);
        color: #3ba55c;
    }

    .cr-count {
        background: rgba(88, 101, 242, 0.1);
        color: #5865f2;
    }

    .coc-card:hover {
        border-color: rgba(59, 165, 92, 0.3);
    }

    .cr-card:hover {
        border-color: rgba(88, 101, 242, 0.3);
    }

    .coc-glow {
        background: radial-gradient(
            600px circle at var(--x, 50%) var(--y, 0%),
            rgba(59, 165, 92, 0.1),
            transparent 40%
        );
    }

    .cr-glow {
        background: radial-gradient(
            600px circle at var(--x, 50%) var(--y, 0%),
            rgba(88, 101, 242, 0.1),
            transparent 40%
        );
    }

    .cr-section {
        margin-top: 2.5rem;
        padding-top: 2.5rem;
        border-top: 1px solid rgba(255, 255, 255, 0.05);
    }

    .light .cr-section {
        border-top-color: rgba(0, 0, 0, 0.05);
    }

    .arena-badge,
    .level-badge {
        padding: 0.5rem 0.75rem;
        background: linear-gradient(135deg, #5865f2, #7289da);
        border-radius: 10px;
        font-size: 0.75rem;
        font-weight: 700;
        color: #fff;
        min-width: 48px;
        text-align: center;
    }

    .cr-role {
        background: rgba(88, 101, 242, 0.15);
        color: #5865f2;
        border-color: rgba(88, 101, 242, 0.3);
    }

    @media (max-width: 640px) {
        .modal-header {
            flex-direction: column;
            gap: 1.5rem;
            padding: 2rem;
        }

        .th-display {
            text-align: left;
            flex-direction: row;
        }

        .modal-body {
            padding: 0 2rem 2rem;
        }

        .player-info-large {
            gap: 1rem;
        }

        .league-icon-large {
            width: 60px;
            height: 60px;
        }

        .player-titles h2 {
            font-size: 1.5rem;
        }
    }
</style>
