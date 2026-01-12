<script lang="ts">
    import { onMount, createEventDispatcher } from 'svelte';
    import { fade, slide, scale } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
    import { user } from './auth';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string;
    export let clanTag: string;

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
        clanLevel: number;
        clanPoints: number;
        clanVersusPoints: number;
        requiredTrophies: number;
        warFrequency: string;
        warWinstreak: number;
        warWins: number;
        warTies?: number;
        warLosses?: number;
        isWarLogPublic: boolean;
        members: number;
        chatLanguage?: { name: string };
    }

    interface Player {
        tag: string;
        name: string;
        role: string;
        townHallLevel: number;
        expLevel: number;
        league?: { iconUrls: { medium: string; small: string }; name: string };
        trophies: number;
        versusTrophies: number;
        donations: number;
        donationsReceived: number;
        // Alliance/Upstream data (if unfiltered)
        userId?: string;
        discordId?: string;
        nickname?: string;
        avatar?: string;
        totalKickpoints?: number;
        activeKickpoints?: any[];
        linked_players?: string[]; // If we fetch full user profile
    }

    let clan: Clan | null = null;
    let members: Player[] = [];
    let loading = true;
    let error: string | null = null;
    let searchQuery = '';
    let selectedPlayer: Player | null = null;
    let playerDetailsLoading = false;
    let playerOtherAccounts: any[] = [];

    const roleOrder: Record<string, number> = {
        'leader': 1,
        'coLeader': 2,
        'admin': 3,
        'member': 4
    };

    function getRoleDisplay(role: string): string {
        switch (role) {
            case 'leader': return 'Anführer';
            case 'coLeader': return 'Vize-Anführer';
            case 'admin': return 'Ältester';
            case 'member': return 'Mitglied';
            default: return role;
        }
    }

    async function fetchClanData() {
        loading = true;
        try {
            const encodedTag = encodeURIComponent(clanTag);
            const clanRes = await fetch(`${apiBaseUrl}/api/clans/${encodedTag}`, { credentials: 'include' });
            if (!clanRes.ok) throw new Error('Clan nicht gefunden');
            clan = await clanRes.json();

            const membersRes = await fetch(`${apiBaseUrl}/api/clans/${encodedTag}/members`, { credentials: 'include' });
            if (!membersRes.ok) throw new Error('Mitglieder konnten nicht geladen werden');
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
        playerOtherAccounts = [];
        
        try {
            const encodedTag = encodeURIComponent(player.tag);
            // Fetch detailed player info (unfiltered if admin/co)
            const res = await fetch(`${apiBaseUrl}/api/players/${encodedTag}`, { credentials: 'include' });
            if (res.ok) {
                const detailedPlayer = await res.json();
                selectedPlayer = { ...player, ...detailedPlayer };
                
                // If the player has a userId, fetch their other accounts
                if (selectedPlayer.userId) {
                    const userRes = await fetch(`${apiBaseUrl}/api/users/${selectedPlayer.userId}`, { credentials: 'include' });
                    if (userRes.ok) {
                        const userData = await userRes.json();
                        // Filter out the currently selected player from "other accounts"
                        playerOtherAccounts = (userData.playerAccounts || []).filter((acc: any) => acc.tag !== player.tag);
                    }
                }
            }
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

    $: sortedByDonations = [...members].sort((a, b) => (b.donations || 0) - (a.donations || 0)).slice(0, 3);
    $: sortedByTrophies = [...members].sort((a, b) => (b.trophies || 0) - (a.trophies || 0)).slice(0, 3);

    $: filteredMembers = members.filter(m => {
        const nameMatch = m.name?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false;
        const tagMatch = m.tag?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false;
        return nameMatch || tagMatch;
    });
</script>

<div class="clan-detail-page" class:light={theme === 'light'} in:fade={{ duration: 300 }}>
    {#if loading}
        <div class="loading-overlay">
            <div class="spinner"></div>
            <p>Clan-Daten werden geladen...</p>
        </div>
    {:else if error}
        <div class="error-container">
            <div class="error-content">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
                </svg>
                <h2>Hoppla!</h2>
                <p>{error}</p>
                <button class="retry-btn" on:click={fetchClanData}>Erneut versuchen</button>
            </div>
        </div>
    {:else if clan}
        <div class="page-content">
            <!-- Hero Header Section -->
            <header class="clan-hero">
                <button class="back-btn" on:click={() => dispatch('navigate', 'coc/clans')}>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                        <path d="M19 12H5M12 19l-7-7 7-7"/>
                    </svg>
                    Alle Clans
                </button>
                <div class="hero-bg" style="background-image: url({clan.badgeUrls?.large || ''})"></div>
                <div class="hero-overlay"></div>
                <div class="hero-content">
                    <div class="badge-container">
                        <img src={clan.badgeUrls?.large || ''} alt={clan.name} class="clan-badge" />
                        <div class="level-badge" transition:scale>LVL {clan.clanLevel}</div>
                    </div>
                    <div class="clan-info-main">
                        <div class="title-row">
                            <h1>{clan.name}</h1>
                            <span class="tag-pill">{clan.tag}</span>
                        </div>
                        <p class="clan-desc">{clan.description || 'Keine Beschreibung verfügbar.'}</p>
                        <div class="quick-stats">
                            <div class="q-stat">
                                <span class="q-label">Punkte</span>
                                <span class="q-value">{clan.clanPoints?.toLocaleString() ?? '0'}</span>
                            </div>
                            <div class="q-stat">
                                <span class="q-label">Mitglieder</span>
                                <span class="q-value">{clan.members ?? '0'}/50</span>
                            </div>
                            <div class="q-stat">
                                <span class="q-label">Typ</span>
                                <span class="q-value">{clan.type === 'inviteOnly' ? 'Auf Einladung' : (clan.type === 'open' ? 'Offen' : 'Geschlossen')}</span>
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
                                        <svg viewBox="0 0 24 24" fill="currentColor" class="t-icon trophies">
                                            <path d="M19 5h-2V3H7v2H5c-1.1 0-2 .9-2 2v3c0 2.55 1.92 4.63 4.39 4.94.63 1.5 1.98 2.63 3.61 2.96V19H7v2h10v-2h-4v-3.1c1.63-.33 2.98-1.46 3.61-2.96C19.08 12.63 21 10.55 21 8V7c0-1.1-.9-2-2-2zM5 10V7h2v3c0 1.1-.9 2-2 2zm14 0c-1.1 0-2-.9-2-2V7h2v3z"/>
                                        </svg>
                                        <span>Trophäen</span>
                                    </div>
                                    <div class="mini-rank-list">
                                        {#each sortedByTrophies as player, i}
                                            <div class="rank-item" on:click={() => selectPlayer(player)}>
                                                <span class="rank-num">{i+1}</span>
                                                <span class="rank-name">{player.name}</span>
                                                <span class="rank-val">{player.trophies}</span>
                                            </div>
                                        {/each}
                                    </div>
                                </div>

                                <div class="top-list-divider"></div>

                                <div class="top-list-section">
                                    <div class="top-list-header">
                                        <svg viewBox="0 0 24 24" fill="currentColor" class="t-icon donation">
                                            <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
                                        </svg>
                                        <span>Top Spender</span>
                                    </div>
                                    <div class="mini-rank-list">
                                        {#each sortedByDonations as player, i}
                                            <div class="rank-item" on:click={() => selectPlayer(player)}>
                                                <span class="rank-num">{i+1}</span>
                                                <span class="rank-name">{player.name}</span>
                                                <span class="rank-val">▲{player.donations}</span>
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
                                    <span class="value">{clan.location?.name || 'Unbekannt'}</span>
                                </div>
                                <div class="info-item">
                                    <span class="label">Sprache</span>
                                    <span class="value">{clan.chatLanguage?.name || 'Deutsch'}</span>
                                </div>
                                <div class="info-item">
                                    <span class="label">Benötigte Trophäen</span>
                                    <span class="value">{clan.requiredTrophies ?? '0'}</span>
                                </div>
                                <div class="info-item">
                                    <span class="label">Kriegsfrequenz</span>
                                    <span class="value">{clan.warFrequency === 'always' ? 'Immer' : (clan.warFrequency === 'never' ? 'Nie' : 'Regelmäßig')}</span>
                                </div>
                                <div class="info-item">
                                    <span class="label">Kriegssiege</span>
                                    <span class="value">{clan.warWins ?? '0'}</span>
                                </div>
                                <div class="info-item">
                                    <span class="label">Winstreak</span>
                                    <span class="value">{clan.warWinstreak ?? '0'}</span>
                                </div>
                            </div>
                        </section>
                    </div>
                </aside>

                <!-- Members Section -->
                <main class="members-main">
                    <div class="section-header-row">
                        <div class="title-group">
                            <h2>Clan Mitglieder</h2>
                            <p class="subtitle">{filteredMembers.length} von {clan.members ?? '0'} Mitgliedern angezeigt</p>
                        </div>
                        <div class="search-box">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
                            </svg>
                            <input type="text" placeholder="Mitglieder suchen..." bind:value={searchQuery} />
                        </div>
                    </div>

                    <div class="members-grid">
                        {#each filteredMembers as member (member.tag)}
                            {@const total = (member.donations || 0) + (member.donationsReceived || 0)}
                            {@const percent = total > 0 ? ((member.donations || 0) / total) * 100 : 50}
                            <div 
                                class="member-card" 
                                class:is-linked={member.userId}
                                on:click={() => selectPlayer(member)}
                                on:keydown={(e) => e.key === 'Enter' && selectPlayer(member)}
                                role="button"
                                tabindex="0"
                            >
                                <div class="m-card-content">
                                    <div class="m-rank-indicator">{members.indexOf(member) + 1}</div>
                                    <div class="m-avatar-container">
                                        {#if member.league}
                                            <img src={member.league.iconUrls.small} alt={member.league.name} class="league-icon" />
                                        {:else}
                                            <div class="no-league"></div>
                                        {/if}
                                    </div>
                                    <div class="m-main-info">
                                        <h4 class="m-name">{member.name}</h4>
                                        <div class="m-sub-info">
                                            <span class="m-role-label">{getRoleDisplay(member.role)}</span>
                                            <span class="dot">•</span>
                                            <span class="m-tag-small">{member.tag}</span>
                                        </div>
                                    </div>
                                    <div class="m-points-info">
                                        <div class="m-trophies-badge">
                                            <svg viewBox="0 0 24 24" fill="currentColor">
                                                <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                                            </svg>
                                            <span>{member.trophies}</span>
                                        </div>
                                        <div class="m-th-sm">RH {member.townHallLevel}</div>
                                    </div>
                                </div>
                                
                                <div class="m-card-footer">
                                    <div class="donation-bar-container">
                                        <div class="donation-stats">
                                            <span>▲ {member.donations}</span>
                                            <span>▼ {member.donationsReceived}</span>
                                        </div>
                                        <div class="donation-bar">
                                            <div class="bar-fill" style="width: {percent}%"></div>
                                        </div>
                                    </div>
                                </div>

                                {#if member.userId}
                                    <div class="linked-indicator" title="Verknüpft mit Discord">
                                        <svg viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03z"/>
                                        </svg>
                                    </div>
                                {/if}
                            </div>
                        {:else}
                            <div class="no-results">Keine Mitglieder für diese Suche gefunden.</div>
                        {/each}
                    </div>
                </main>
            </div>
        </div>
    {/if}

    <!-- Player Details Overlay -->
    {#if selectedPlayer}
        <div class="overlay" on:click|self={closePlayerDetails} transition:fade={{ duration: 200 }}>
            <div class="drawer" transition:slide={{ axis: 'x', duration: 400, easing: quintOut }}>
                <button class="close-btn" on:click={closePlayerDetails}>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
                    </svg>
                </button>

                {#if playerDetailsLoading}
                    <div class="drawer-loading">
                        <div class="spinner"></div>
                    </div>
                {:else}
                    <div class="drawer-content">
                        <div class="player-hero">
                            <div class="p-hero-top">
                                {#if selectedPlayer.league}
                                    <img src={selectedPlayer.league.iconUrls.medium} alt={selectedPlayer.league.name} class="p-league-img" />
                                {/if}
                                <div class="p-title">
                                    <h2>{selectedPlayer.name}</h2>
                                    <span class="p-tag">{selectedPlayer.tag}</span>
                                </div>
                            </div>
                            <div class="p-role-badge" style="background: {theme === 'light' ? 'rgba(88, 101, 242, 0.1)' : 'rgba(88, 101, 242, 0.2)'}">
                                {getRoleDisplay(selectedPlayer.role)}
                            </div>
                        </div>

                        {#if selectedPlayer.userId}
                            <section class="discord-profile">
                                <div class="d-header">
                                    <div class="d-avatar-box">
                                        {#if selectedPlayer.avatar}
                                            <img src={selectedPlayer.avatar} alt="Discord Avatar" />
                                        {:else}
                                            <div class="d-placeholder">{(selectedPlayer.nickname || selectedPlayer.name).charAt(0)}</div>
                                        {/if}
                                    </div>
                                    <div class="d-info">
                                        <div class="d-name">{selectedPlayer.nickname || 'Unbekannt'}</div>
                                        <div class="d-status">Verknüpft</div>
                                    </div>
                                </div>
                            </section>
                        {/if}

                        <section class="detail-section">
                            <h3>Spiel-Statistiken</h3>
                            <div class="stat-list">
                                <div class="s-item">
                                    <span class="s-label">Rathaus</span>
                                    <span class="s-value">Level {selectedPlayer.townHallLevel}</span>
                                </div>
                                <div class="s-item">
                                    <span class="s-label">Erfahrung</span>
                                    <span class="s-value">Lvl {selectedPlayer.expLevel}</span>
                                </div>
                                <div class="s-item">
                                    <span class="s-label">Trophäen</span>
                                    <span class="s-value">{selectedPlayer.trophies}</span>
                                </div>
                                <div class="s-item">
                                    <span class="s-label">Spenden</span>
                                    <span class="s-value">▲ {selectedPlayer.donations} / ▼ {selectedPlayer.donationsReceived}</span>
                                </div>
                                {#if selectedPlayer.totalKickpoints !== undefined}
                                    <div class="s-item highlighted">
                                        <span class="s-label">Kickpunkte</span>
                                        <span class="s-value">{(selectedPlayer.activeKickpoints || []).reduce((a, b) => a + (b.amount || 0), 0)} (Gesamt: {selectedPlayer.totalKickpoints})</span>
                                    </div>
                                {/if}
                            </div>
                        </section>

                        {#if selectedPlayer.activeKickpoints && selectedPlayer.activeKickpoints.length > 0}
                            <section class="detail-section">
                                <h3>Kickpunkt Details</h3>
                                <div class="kp-list">
                                    {#each selectedPlayer.activeKickpoints as kp}
                                        <div class="kp-detail-item">
                                            <div class="kp-detail-header">
                                                <span class="kp-amount">+{kp.amount}</span>
                                                <span class="kp-date">{new Date(kp.date).toLocaleDateString('de-DE')}</span>
                                            </div>
                                            {#if kp.reason}
                                                <div class="kp-reason">{kp.reason}</div>
                                            {/if}
                                            {#if kp.description}
                                                <div class="kp-desc">{kp.description}</div>
                                            {/if}
                                        </div>
                                    {/each}
                                </div>
                            </section>
                        {/if}

                        {#if playerOtherAccounts.length > 0}
                            <section class="detail-section">
                                <h3>Weitere Accounts ({playerOtherAccounts.length})</h3>
                                <div class="other-accounts">
                                    {#each playerOtherAccounts as acc}
                                        <div class="acc-mini-card" on:click={() => selectPlayer(acc)}>
                                            <img src={acc.clan?.badgeUrls?.small || ''} alt="" class="acc-badge" />
                                            <div class="acc-info">
                                                <div class="acc-name">{acc.nameDB}</div>
                                                <div class="acc-clan">{acc.clan?.name || 'Kein Clan'}</div>
                                            </div>
                                            <div class="acc-tag">{acc.tag}</div>
                                        </div>
                                    {/each}
                                </div>
                            </section>
                        {/if}
                    </div>
                {/if}
            </div>
        </div>
    {/if}
</div>

<style>
    :root {
        --accent-color: #5865f2;
        --accent-hover: #4752c4;
        --bg-dark: #0a0a0f;
        --bg-card-dark: rgba(20, 20, 30, 0.7);
        --border-dark: rgba(255, 255, 255, 0.08);
        --text-dim: rgba(255, 255, 255, 0.6);
        
        --bg-light: #f8fafc;
        --bg-card-light: #ffffff;
        --border-light: rgba(0, 0, 0, 0.08);
        --text-dim-light: rgba(0, 0, 0, 0.6);
    }

    .clan-detail-page {
        min-height: 100vh;
        background-color: var(--bg-dark);
        color: #fff;
        padding-bottom: 4rem;
        font-family: 'Inter', system-ui, -apple-system, sans-serif;
    }

    .clan-detail-page.light {
        background-color: var(--bg-light);
        color: #1a1a2e;
    }

    /* Hero Section */
    .clan-hero {
        position: relative;
        height: 400px;
        display: flex;
        align-items: flex-end;
        padding: 0 5% 4rem;
        overflow: hidden;
    }

    .hero-bg {
        position: absolute;
        top: 0; left: 0; right: 0; bottom: 0;
        background-size: cover;
        background-position: center;
        filter: blur(60px) opacity(0.25);
        transform: scale(1.2);
    }

    .hero-overlay {
        position: absolute;
        bottom: 0; left: 0; right: 0;
        height: 100%;
        background: linear-gradient(0deg, var(--bg-dark) 0%, transparent 100%);
    }

    .light .hero-overlay {
        background: linear-gradient(0deg, var(--bg-light) 0%, transparent 100%);
    }

    .hero-content {
        position: relative;
        z-index: 2;
        display: flex;
        gap: 3rem;
        align-items: center;
        width: 100%;
        max-width: 1400px;
        margin: 0 auto;
    }

    .badge-container {
        position: relative;
        flex-shrink: 0;
    }

    .clan-badge {
        width: 180px;
        height: 180px;
        filter: drop-shadow(0 15px 35px rgba(0,0,0,0.6));
    }

    .level-badge {
        position: absolute;
        bottom: -5px;
        left: 50%;
        transform: translateX(-50%);
        background: var(--accent-color);
        padding: 6px 16px;
        border-radius: 30px;
        font-weight: 800;
        font-size: 0.95rem;
        box-shadow: 0 8px 20px rgba(88, 101, 242, 0.4);
        border: 2px solid rgba(255, 255, 255, 0.1);
        white-space: nowrap;
    }

    .clan-info-main {
        flex: 1;
    }

    .title-row {
        display: flex;
        align-items: center;
        gap: 1.5rem;
        margin-bottom: 0.75rem;
        flex-wrap: wrap;
    }

    .clan-info-main h1 {
        font-size: 4rem;
        font-weight: 900;
        margin: 0;
        line-height: 1;
        letter-spacing: -0.02em;
    }

    .tag-pill {
        background: rgba(255, 255, 255, 0.1);
        padding: 6px 14px;
        border-radius: 8px;
        font-family: 'JetBrains Mono', monospace;
        font-size: 1.1rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.8);
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .light .tag-pill {
        background: rgba(0, 0, 0, 0.05);
        color: rgba(0, 0, 0, 0.6);
        border-color: rgba(0, 0, 0, 0.05);
    }

    .clan-desc {
        font-size: 1.15rem;
        color: var(--text-dim);
        max-width: 700px;
        margin-bottom: 2rem;
        line-height: 1.6;
    }

    .light .clan-desc {
        color: var(--text-dim-light);
    }

    .quick-stats {
        display: flex;
        gap: 3rem;
    }

    .q-stat {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .q-label {
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        font-weight: 600;
        opacity: 0.5;
    }

    .q-value {
        font-size: 1.6rem;
        font-weight: 800;
    }

    .back-btn {
        position: absolute;
        top: 2rem;
        left: 2rem;
        z-index: 10;
        background: rgba(255, 255, 255, 0.05);
        backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: #fff;
        padding: 0.75rem 1.25rem;
        border-radius: 12px;
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
    }

    .light .back-btn {
        background: rgba(0, 0, 0, 0.05);
        border-color: rgba(0, 0, 0, 0.1);
        color: #1a1a2e;
    }

    .back-btn:hover {
        background: rgba(255, 255, 255, 0.12);
        transform: translateX(-5px);
    }

    .back-btn svg { width: 20px; }

    /* Main Grid Layout */
    .layout-grid {
        display: grid;
        grid-template-columns: 320px 1fr;
        gap: 3rem;
        max-width: 1400px;
        margin: 0 auto;
        padding: 0 5%;
    }

    .sidebar {
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    .sidebar-sticky {
        position: sticky;
        top: 6rem;
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    .info-card {
        background: var(--bg-card-dark);
        backdrop-filter: blur(15px);
        border: 1px solid var(--border-dark);
        border-radius: 24px;
        padding: 2rem;
        box-shadow: 0 20px 40px rgba(0,0,0,0.2);
    }

    .light .info-card {
        background: var(--bg-card-light);
        border-color: var(--border-light);
        box-shadow: 0 10px 30px rgba(0,0,0,0.05);
    }

    .info-card h3 {
        margin: 0 0 1.5rem 0;
        font-size: 1.2rem;
        font-weight: 800;
        letter-spacing: -0.01em;
    }

    /* Top Lists Styling */
    .top-lists {
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    .top-list-header {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-weight: 700;
        margin-bottom: 1rem;
        font-size: 0.95rem;
        color: var(--accent-color);
    }

    .t-icon { width: 20px; height: 20px; }
    .trophies { color: #ffcc00; }
    .donation { color: #00ff88; }

    .mini-rank-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .rank-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem 1rem;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 14px;
        cursor: pointer;
        transition: all 0.2s;
        border: 1px solid transparent;
    }

    .light .rank-item {
        background: rgba(0, 0, 0, 0.03);
    }

    .rank-item:hover {
        background: rgba(88, 101, 242, 0.1);
        border-color: rgba(88, 101, 242, 0.2);
        transform: scale(1.02);
    }

    .rank-num {
        font-weight: 900;
        font-size: 0.9rem;
        color: var(--accent-color);
        width: 1.2rem;
    }

    .rank-name {
        flex: 1;
        font-weight: 600;
        font-size: 0.95rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .rank-val {
        font-weight: 800;
        font-size: 0.9rem;
        font-variant-numeric: tabular-nums;
    }

    .top-list-divider {
        height: 1px;
        background: linear-gradient(90deg, transparent, var(--border-dark), transparent);
    }

    .light .top-list-divider {
        background: linear-gradient(90deg, transparent, var(--border-light), transparent);
    }

    .info-grid {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .info-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .info-item .label {
        font-size: 0.9rem;
        font-weight: 500;
        color: var(--text-dim);
    }

    .light .info-item .label { color: var(--text-dim-light); }

    .info-item .value {
        font-weight: 700;
        font-size: 0.95rem;
    }

    /* Members Main Section */
    .members-main {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
    }

    .section-header-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding-bottom: 1rem;
        border-bottom: 1px solid var(--border-dark);
    }

    .light .section-header-row {
        border-bottom-color: var(--border-light);
    }

    .title-group h2 {
        font-size: 2.2rem;
        font-weight: 900;
        margin: 0;
        letter-spacing: -0.02em;
    }

    .subtitle {
        margin: 0.25rem 0 0 0;
        color: var(--text-dim);
        font-weight: 500;
    }

    .light .subtitle { color: var(--text-dim-light); }

    .search-box {
        position: relative;
        width: 350px;
    }

    .search-box svg {
        position: absolute;
        left: 14px;
        top: 50%;
        transform: translateY(-50%);
        width: 20px;
        opacity: 0.5;
    }

    .search-box input {
        width: 100%;
        padding: 12px 16px 12px 46px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid var(--border-dark);
        border-radius: 16px;
        color: #fff;
        font-size: 1rem;
        outline: none;
        transition: all 0.3s;
    }

    .light .search-box input {
        background: #fff;
        color: #1a1a2e;
        border-color: var(--border-light);
    }

    .search-box input:focus {
        border-color: var(--accent-color);
        background: rgba(88, 101, 242, 0.05);
        box-shadow: 0 0 0 4px rgba(88, 101, 242, 0.1);
    }

    /* Members Grid */
    .members-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
        gap: 1.5rem;
    }

    .member-card {
        background: var(--bg-card-dark);
        border: 1px solid var(--border-dark);
        border-radius: 24px;
        padding: 1.5rem;
        cursor: pointer;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        position: relative;
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .light .member-card {
        background: var(--bg-card-light);
        border-color: var(--border-light);
        box-shadow: 0 10px 20px rgba(0,0,0,0.03);
    }

    .member-card:hover {
        transform: translateY(-8px);
        border-color: var(--accent-color);
        box-shadow: 0 20px 40px rgba(0,0,0,0.3);
    }

    .m-card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .league-icon {
        width: 44px;
        height: 44px;
        filter: drop-shadow(0 4px 8px rgba(0,0,0,0.2));
    }

    .m-trophies-badge {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        background: rgba(0, 0, 0, 0.3);
        padding: 6px 12px;
        border-radius: 12px;
        font-weight: 800;
        font-size: 0.95rem;
        border: 1px solid rgba(255, 255, 255, 0.05);
    }

    .m-trophies-badge svg {
        width: 16px;
        color: #ffcc00;
    }

    .m-card-body {
        flex: 1;
    }

    .m-name {
        margin: 0 0 4px 0;
        font-size: 1.3rem;
        font-weight: 800;
        letter-spacing: -0.01em;
    }

    .m-role-label {
        font-size: 0.85rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--accent-color);
    }

    .m-stats-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-top: 1rem;
        padding-top: 1rem;
        border-top: 1px solid var(--border-dark);
    }

    .light .m-stats-row {
        border-top-color: var(--border-light);
    }

    .m-th-badge {
        background: rgba(255, 255, 255, 0.05);
        padding: 4px 10px;
        border-radius: 8px;
        font-weight: 700;
        font-size: 0.85rem;
        color: var(--text-dim);
    }

    .light .m-th-badge {
        background: rgba(0, 0, 0, 0.05);
        color: var(--text-dim-light);
    }

    .m-donations {
        font-weight: 800;
        color: #00ff88;
        font-size: 0.95rem;
    }

    .linked-indicator {
        position: absolute;
        top: 1rem;
        right: 1.5rem;
        color: var(--accent-color);
        opacity: 0.6;
    }

    .linked-indicator svg { width: 18px; }

    /* Drawer / Overlay Styles */
    .overlay {
        position: fixed;
        top: 0; left: 0; right: 0; bottom: 0;
        background: rgba(0, 0, 0, 0.8);
        backdrop-filter: blur(8px);
        z-index: 1000;
        display: flex;
        justify-content: flex-end;
    }

    .drawer {
        width: 100%;
        max-width: 550px;
        height: 100%;
        background: var(--bg-dark);
        border-left: 1px solid var(--border-dark);
        position: relative;
        padding: 4rem 2.5rem;
        overflow-y: auto;
        box-shadow: -20px 0 60px rgba(0,0,0,0.5);
    }

    .light .drawer {
        background: var(--bg-light);
        border-left-color: var(--border-light);
    }

    .close-btn {
        position: absolute;
        top: 1.5rem;
        right: 1.5rem;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid var(--border-dark);
        color: #fff;
        width: 44px;
        height: 44px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s;
    }

    .light .close-btn {
        background: rgba(0, 0, 0, 0.05);
        border-color: var(--border-light);
        color: #1a1a2e;
    }

    .close-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        transform: rotate(90deg);
    }

    /* Player Detail Content */
    .player-hero {
        margin-bottom: 3rem;
    }

    .p-hero-top {
        display: flex;
        align-items: center;
        gap: 1.5rem;
        margin-bottom: 1.5rem;
    }

    .p-league-img { width: 80px; height: 80px; }

    .p-title h2 {
        font-size: 2.5rem;
        font-weight: 900;
        margin: 0;
        line-height: 1;
    }

    .p-tag {
        font-family: 'JetBrains Mono', monospace;
        color: var(--text-dim);
        font-size: 1.1rem;
    }

    .p-role-badge {
        display: inline-block;
        padding: 6px 16px;
        border-radius: 10px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--accent-color);
    }

    .detail-section {
        margin-bottom: 3rem;
    }

    .detail-section h3 {
        font-size: 0.95rem;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: var(--text-dim);
        margin-bottom: 1.5rem;
        border-bottom: 1px solid var(--border-dark);
        padding-bottom: 0.75rem;
    }

    .stat-list {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    .s-item {
        background: rgba(255, 255, 255, 0.03);
        padding: 1.25rem;
        border-radius: 18px;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
        border: 1px solid var(--border-dark);
    }

    .light .s-item { background: #fff; border-color: var(--border-light); }

    .s-item.highlighted {
        background: rgba(88, 101, 242, 0.1);
        border-color: rgba(88, 101, 242, 0.2);
    }

    .s-label {
        font-size: 0.8rem;
        font-weight: 600;
        color: var(--text-dim);
        text-transform: uppercase;
    }

    .s-value {
        font-size: 1.2rem;
        font-weight: 800;
    }

    /* Discord Card inside Drawer */
    .discord-profile {
        background: #5865f2;
        border-radius: 24px;
        padding: 1.5rem;
        margin-bottom: 3rem;
        color: white;
        box-shadow: 0 10px 25px rgba(88, 101, 242, 0.3);
    }

    .d-header {
        display: flex;
        align-items: center;
        gap: 1.25rem;
    }

    .d-avatar-box img {
        width: 64px;
        height: 64px;
        border-radius: 50%;
        border: 3px solid rgba(255, 255, 255, 0.2);
    }

    .d-name { font-size: 1.4rem; font-weight: 800; }
    .d-status { font-size: 0.9rem; opacity: 0.8; font-weight: 600; }

    /* Other Accounts */
    .other-accounts {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .acc-mini-card {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 1rem;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 16px;
        cursor: pointer;
        transition: all 0.2s;
        border: 1px solid var(--border-dark);
    }

    .acc-mini-card:hover {
        background: rgba(255, 255, 255, 0.08);
        transform: translateX(8px);
    }

    .acc-badge { width: 44px; height: 44px; }
    .acc-info { flex: 1; }
    .acc-name { font-weight: 800; font-size: 1.1rem; }
    .acc-clan { font-size: 0.85rem; opacity: 0.6; }
    .acc-tag { font-family: monospace; font-size: 0.8rem; opacity: 0.4; }

    /* Kickpoint Styling */
    .kp-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .kp-detail-item {
        background: rgba(244, 63, 94, 0.05);
        border-radius: 16px;
        padding: 1.25rem;
        border: 1px solid rgba(244, 63, 94, 0.1);
        border-left: 4px solid #f43f5e;
    }

    .kp-detail-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.5rem;
    }

    .kp-amount { font-weight: 900; color: #f43f5e; font-size: 1.3rem; }
    .kp-date { font-size: 0.85rem; font-weight: 600; opacity: 0.5; }
    .kp-reason { font-weight: 800; font-size: 1.1rem; margin-bottom: 0.25rem; }
    .kp-desc { font-size: 0.95rem; line-height: 1.5; opacity: 0.8; }

    /* Loading / Spinner */
    .loading-overlay {
        height: 80vh;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 1.5rem;
    }

    .spinner {
        width: 50px;
        height: 50px;
        border: 4px solid rgba(88, 101, 242, 0.1);
        border-top-color: var(--accent-color);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin { to { transform: rotate(360deg); } }

    /* Responsive */
    @media (max-width: 1200px) {
        .layout-grid {
            grid-template-columns: 1fr;
        }
        .sidebar {
            order: 2;
        }
        .sidebar-sticky {
            position: relative;
            top: 0;
        }
        .members-main {
            order: 1;
        }
    }

    @media (max-width: 768px) {
        .clan-hero {
            height: auto;
            padding-top: 6rem;
        }
        .hero-content {
            flex-direction: column;
            text-align: center;
            gap: 1.5rem;
        }
        .clan-info-main h1 { font-size: 2.5rem; }
        .quick-stats { justify-content: center; flex-wrap: wrap; gap: 1.5rem; }
        
        .section-header-row {
            flex-direction: column;
            align-items: flex-start;
            gap: 1.5rem;
        }
        .search-box { width: 100%; }
    }
</style>

