<script lang="ts">
    import { onMount, createEventDispatcher } from 'svelte';
    import { fade } from 'svelte/transition';
    import { user, loading as authLoading } from './auth';

    // Import banners
    import banner3 from '../assets/Clans/Clash of Clans/Lost-X-3.png';
    import banner4 from '../assets/Clans/Clash of Clans/Lost-X-4.png';
    import banner5 from '../assets/Clans/Clash of Clans/Lost-X-5.png';
    import banner6 from '../assets/Clans/Clash of Clans/Lost-X-6.png';
    import banner7 from '../assets/Clans/Clash of Clans/Lost-X-7.png';
    import banner8 from '../assets/Clans/Clash of Clans/Lost-X-8.png';
    import bannerF2P from '../assets/Clans/Clash of Clans/Lost-X-f2p.png';
    import bannerF2P2 from '../assets/Clans/Clash of Clans/Lost-X-f2p2.png';
    import bannerGP from '../assets/Clans/Clash of Clans/Lost-X-gp.png';
    import bannerAnthrazit from '../assets/Clans/Clash of Clans/Lost-X-anthrazit.png';
    import bannerDefault from '../assets/Assets/banner-lost.png';

    // Clash Royale banners
    import bannerCR1 from '../assets/Clans/Clash Royale/Lost_1.png';
    import bannerCR2 from '../assets/Clans/Clash Royale/Lost_2.png';
    import bannerCR3 from '../assets/Clans/Clash Royale/Lost_3.png';
    import bannerCR4 from '../assets/Clans/Clash Royale/Lost_4.png';
    import bannerCR5 from '../assets/Clans/Clash Royale/Lost_5.png';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string;

    const dispatch = createEventDispatcher<{ navigate: string }>();

    interface Clan {
        tag: string;
        nameDB: string;
        index: number;
        badgeUrl: string;
        gameType?: 'coc' | 'cr';
    }

    let allClans: Clan[] = [];
    let loading = true;
    let error: string | null = null;

    $: cocClans = allClans.filter((c) => c.gameType === 'coc');
    $: crClans = allClans.filter((c) => c.gameType === 'cr');

    function getClanBanner(clanName: string, gameType?: 'coc' | 'cr'): string {
        const name = (clanName || '').toUpperCase();

        if (gameType === 'cr') {
            if (name === 'LOST') return bannerCR1;
            if (name.includes('4') || name.includes('IV')) return bannerCR4;
            if (name.includes('5') || name.includes('V')) return bannerCR5;
            if (name.includes('3') || name.includes('III')) return bannerCR3;
            if (name.includes('2') || name.includes('II')) return bannerCR2;
            return bannerDefault;
        }

        if (name.includes('F2P 2') || name.includes('F2P2')) return bannerF2P2;
        if (name.includes('F2P')) return bannerF2P;
        if (name.includes('GP')) return bannerGP;
        if (name.includes('8') || name.includes('VIII')) return banner8;
        if (name.includes('7') || name.includes('VII')) return banner7;
        if (name.includes('6') || name.includes('VI')) return banner6;
        if (name.includes('4') || name.includes('IV')) return banner4;
        if (name.includes('5') || name.includes('V')) return banner5;
        if (name.includes('3') || name.includes('III')) return banner3;
        if (name.includes('ANTHRAZIT')) return bannerAnthrazit;
        return bannerDefault;
    }

    function getClanColor(name: string, index: number): string {
        const n = (name || '').toUpperCase();
        if (n.includes('GP')) return '#a5025a';
        if (n.includes('ANTHRAZIT')) return '#3d3a3f';

        // Priority to name-based coloring to fix potential index swaps
        if (n.includes('F2P 2') || n.includes('F2P2')) return '#05762b'; // Green
        if (n.includes('F2P')) return '#c90000'; // Red
        if (n.includes('8') || n.includes('VIII')) return '#d100c7'; // Pink
        if (n.includes('7') || n.includes('VII')) return '#007076'; // Teal
        if (n.includes('6') || n.includes('VI')) return '#b54800'; // Orange
        if (n.includes('4') || n.includes('IV')) return '#691a97'; // Purple
        if (n.includes('5') || n.includes('V')) return '#024885'; // Dark Blue
        if (n.includes('3') || n.includes('III')) return '#c89e00'; // Gold

        // Fallback to index-based coloring to match ClanCard.svelte
        if (index === 1) return '#c90000'; // First clan is always Red
        if (index === 2) return '#05762b'; // Second is Green
        if (index === 3) return '#c89e00'; // Third is Gold
        if (index === 4) return '#691a97'; // Purple
        if (index === 5) return '#024885'; // Dark Blue
        if (index === 6) return '#b54800'; // Orange
        if (index === 7) return '#007076'; // Teal
        if (index === 8) return '#d100c7'; // Pink

        return '#c90000';
    }

    async function loadClans() {
        if (!$user?.is_admin) {
            error = 'Administrator-Zutritt verweigert.';
            loading = false;
            return;
        }

        loading = true;
        error = null;
        try {
            const [cocRes, crRes] = await Promise.all([
                fetch(`${apiBaseUrl}/api/coc/clans`),
                fetch(`${apiBaseUrl}/api/cr/clans`),
            ]);

            if (!cocRes.ok) {
                if (cocRes.status === 403)
                    throw new Error(
                        'Keine Berechtigung (Admin für CoC erforderlich)'
                    );
                throw new Error(`CoC HTTP ${cocRes.status}`);
            }
            if (!crRes.ok) {
                if (crRes.status === 403)
                    throw new Error(
                        'Keine Berechtigung (Admin für CR erforderlich)'
                    );
                throw new Error(`CR HTTP ${crRes.status}`);
            }

            const cocData: Clan[] = await cocRes.json();
            const crData: Clan[] = await crRes.json();

            // Sort by index in the right order
            allClans = [
                ...cocData.map((c) => ({ ...c, gameType: 'coc' as const })),
                ...crData.map((c) => ({ ...c, gameType: 'cr' as const })),
            ].sort((a, b) => {
                // First group by game type
                if (a.gameType !== b.gameType) {
                    return a.gameType === 'coc' ? -1 : 1;
                }
                // Then sort by index
                return a.index - b.index;
            });
        } catch (e) {
            error = e instanceof Error ? e.message : 'Unbekannter Fehler';
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        if (!$authLoading) {
            if ($user?.is_admin) {
                loadClans();
            } else {
                error = 'Administrator-Zutritt verweigert.';
                loading = false;
            }
        }
    });

    $: if (!$authLoading && !allClans.length && !error && loading) {
        if ($user?.is_admin) {
            loadClans();
        } else {
            error = 'Administrator-Zutritt verweigert.';
            loading = false;
        }
    }

    function navigateToClan(clan: Clan) {
        const type = clan.gameType || 'coc';
        dispatch('navigate', `${type}/clan/${clan.tag.replace('#', '')}`);
    }
</script>

<div class="admin-clans-page" class:light={theme === 'light'}>
    <div class="container">
        <header class="page-header">
            <div class="header-content">
                <div class="title-group">
                    <h1 class="page-title">Clan Auswahl</h1>
                    <p class="page-subtitle">
                        Alle Clans der LOST Family (Admin Ansicht)
                    </p>
                </div>
                <button
                    class="back-btn"
                    on:click={() => dispatch('navigate', 'home')}
                >
                    Zurück zur Startseite
                </button>
            </div>
        </header>

        {#if loading}
            <div class="state-container" in:fade>
                <div class="spinner"></div>
                <p>Clans werden geladen...</p>
            </div>
        {:else if error}
            <div class="state-container error" in:fade>
                {#if error === 'Administrator-Zutritt verweigert.'}
                    <div class="access-denied-container">
                        <div class="lock-icon">
                            <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <rect
                                    x="3"
                                    y="11"
                                    width="18"
                                    height="11"
                                    rx="2"
                                    ry="2"
                                />
                                <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                            </svg>
                        </div>
                        <h3>Zutritt verweigert</h3>
                        <p>
                            Du benötigst Administrator-Rechte, um auf diese
                            Seite zuzugreifen.
                        </p>
                        <div class="deny-actions">
                            <button
                                class="action-btn secondary"
                                on:click={() => dispatch('navigate', 'home')}
                            >
                                Zurück zur Startseite
                            </button>
                            <button
                                class="action-btn"
                                on:click={() => dispatch('navigate', 'clans')}
                            >
                                Zu den Clans
                            </button>
                        </div>
                    </div>
                {:else}
                    <svg
                        class="error-icon"
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
                    <p>{error}</p>
                    <button class="retry-btn" on:click={loadClans}
                        >Erneut versuchen</button
                    >
                {/if}
            </div>
        {:else}
            <div class="sections-container">
                {#if cocClans.length > 0}
                    <section class="game-section">
                        <div class="section-header-inline">
                            <div class="game-icon coc">⚔️</div>
                            <h2 class="section-subtitle">Clash of Clans</h2>
                        </div>
                        <div class="clan-grid">
                            {#each cocClans as clan}
                                <button
                                    class="clan-selector-card"
                                    on:click={() => navigateToClan(clan)}
                                    style="--clan-color: {getClanColor(
                                        clan.nameDB,
                                        clan.index,
                                    )}"
                                >
                                    <div class="card-banner">
                                        <img
                                            src={getClanBanner(
                                                clan.nameDB,
                                                clan.gameType,
                                            )}
                                            alt="Banner"
                                        />
                                        <div class="banner-overlay"></div>
                                    </div>
                                    <div class="card-content">
                                        <div class="clan-badge">
                                            {#if clan.badgeUrl}
                                                <img
                                                    src={clan.badgeUrl}
                                                    alt={clan.nameDB ||
                                                        clan.tag}
                                                />
                                            {:else}
                                                <div class="badge-placeholder">
                                                    {(
                                                        clan.nameDB || 'C'
                                                    ).charAt(0)}
                                                </div>
                                            {/if}
                                        </div>
                                        <div class="clan-info">
                                            <span class="clan-name"
                                                >{clan.nameDB || clan.tag}</span
                                            >
                                            <span class="clan-tag"
                                                >{clan.tag}</span
                                            >
                                        </div>
                                        <div class="clan-meta">
                                            <span
                                                class="game-tag"
                                                class:cr={clan.gameType ===
                                                    'cr'}
                                                >{clan.gameType?.toUpperCase() ||
                                                    'CoC'}</span
                                            >
                                            <span class="clan-index-badge"
                                                >#{clan.index}</span
                                            >
                                        </div>
                                    </div>
                                </button>
                            {/each}
                        </div>
                    </section>
                {/if}

                {#if crClans.length > 0}
                    <section class="game-section">
                        <div class="section-header-inline">
                            <div class="game-icon cr">👑</div>
                            <h2 class="section-subtitle">Clash Royale</h2>
                        </div>
                        <div class="clan-grid">
                            {#each crClans as clan}
                                <button
                                    class="clan-selector-card"
                                    on:click={() => navigateToClan(clan)}
                                    style="--clan-color: {getClanColor(
                                        clan.nameDB,
                                        clan.index,
                                    )}"
                                >
                                    <div class="card-banner">
                                        <img
                                            src={getClanBanner(
                                                clan.nameDB,
                                                clan.gameType,
                                            )}
                                            alt="Banner"
                                        />
                                        <div class="banner-overlay"></div>
                                    </div>
                                    <div class="card-content">
                                        <div class="clan-badge">
                                            {#if clan.badgeUrl}
                                                <img
                                                    src={clan.badgeUrl}
                                                    alt={clan.nameDB ||
                                                        clan.tag}
                                                />
                                            {:else}
                                                <div class="badge-placeholder">
                                                    {(
                                                        clan.nameDB || 'C'
                                                    ).charAt(0)}
                                                </div>
                                            {/if}
                                        </div>
                                        <div class="clan-info">
                                            <span class="clan-name"
                                                >{clan.nameDB || clan.tag}</span
                                            >
                                            <span class="clan-tag"
                                                >{clan.tag}</span
                                            >
                                        </div>
                                        <div class="clan-meta">
                                            <span
                                                class="game-tag"
                                                class:cr={clan.gameType ===
                                                    'cr'}
                                                >{clan.gameType?.toUpperCase() ||
                                                    'CoC'}</span
                                            >
                                            <span class="clan-index-badge"
                                                >#{clan.index}</span
                                            >
                                        </div>
                                    </div>
                                </button>
                            {/each}
                        </div>
                    </section>
                {/if}
            </div>
        {/if}
    </div>
</div>

<style>
    .admin-clans-page {
        min-height: 100vh;
        background: #0a0a0f;
        padding: 6rem 2rem 4rem;
        color: white;
    }

    .admin-clans-page.light {
        background: #f8fafc;
        color: #1e293b;
    }

    .container {
        max-width: 1000px;
        margin: 0 auto;
    }

    .page-header {
        margin-bottom: 3rem;
        padding-bottom: 2rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    }

    .light .page-header {
        border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    }

    .header-content {
        display: flex;
        justify-content: space-between;
        align-items: flex-end;
    }

    .page-title {
        font-size: 2.5rem;
        font-weight: 800;
        margin: 0;
        background: linear-gradient(135deg, #fff 0%, #a5b4fc 100%);
        -webkit-background-clip: text;
        background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .light .page-title {
        background: linear-gradient(135deg, #1e293b 0%, #475569 100%);
        -webkit-background-clip: text;
        background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .page-subtitle {
        color: rgba(255, 255, 255, 0.6);
        margin: 0.5rem 0 0;
        font-size: 1.1rem;
    }

    .light .page-subtitle {
        color: #64748b;
    }

    .back-btn {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: white;
        padding: 0.75rem 1.5rem;
        border-radius: 12px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .light .back-btn {
        background: white;
        border: 1px solid #e2e8f0;
        color: #1e293b;
    }

    .back-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        transform: translateY(-2px);
    }

    .light .back-btn:hover {
        background: #f1f5f9;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
    }

    .state-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 4rem;
        text-align: center;
        gap: 1.5rem;
        min-height: 400px;
    }

    .access-denied-container {
        max-width: 400px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.5rem;
        padding: 3rem;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 24px;
        box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
    }

    .light .access-denied-container {
        background: white;
        border-color: #e2e8f0;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.05);
    }

    .lock-icon {
        width: 80px;
        height: 80px;
        background: rgba(239, 68, 68, 0.1);
        color: #ef4444;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 20px;
        margin-bottom: 0.5rem;
    }

    .lock-icon svg {
        width: 40px;
        height: 40px;
    }

    .access-denied-container h3 {
        font-size: 1.75rem;
        font-weight: 800;
        margin: 0;
        color: white;
    }

    .light .access-denied-container h3 {
        color: #1e293b;
    }

    .access-denied-container p {
        color: rgba(255, 255, 255, 0.6);
        font-size: 1.1rem;
        line-height: 1.5;
        margin: 0;
    }

    .light .access-denied-container p {
        color: #64748b;
    }

    .deny-actions {
        display: flex;
        gap: 1rem;
        margin-top: 1rem;
    }

    .action-btn {
        padding: 0.8rem 1.75rem;
        border-radius: 12px;
        font-weight: 700;
        cursor: pointer;
        transition: all 0.2s ease;
        border: none;
        background: #5865f2;
        color: white;
    }

    .action-btn:hover {
        background: #4752c4;
        transform: translateY(-2px);
    }

    .action-btn.secondary {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: white;
    }

    .light .action-btn.secondary {
        background: #f1f5f9;
        border-color: #e2e8f0;
        color: #475569;
    }

    .action-btn.secondary:hover {
        background: rgba(255, 255, 255, 0.1);
    }

    .light .action-btn.secondary:hover {
        background: #e2e8f0;
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid rgba(88, 101, 242, 0.2);
        border-top-color: #5865f2;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .sections-container {
        display: flex;
        flex-direction: column;
        gap: 4rem;
    }

    .game-section {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .section-header-inline {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding-bottom: 1rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .light .section-header-inline {
        border-bottom: 1px solid rgba(0, 0, 0, 0.05);
    }

    .game-icon {
        font-size: 1.5rem;
        width: 42px;
        height: 42px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 12px;
    }

    .light .game-icon {
        background: white;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
    }

    .section-subtitle {
        font-size: 1.75rem;
        font-weight: 700;
        margin: 0;
        color: white;
    }

    .light .section-subtitle {
        color: #1e293b;
    }

    .clan-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
        gap: 2rem;
    }

    .clan-selector-card {
        display: flex;
        flex-direction: column;
        background: #1a1a1f;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        cursor: pointer;
        transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
        text-align: left;
        width: 100%;
        position: relative;
        overflow: hidden;
        padding: 0;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    }

    .light .clan-selector-card {
        background: white;
        border: 1px solid #e2e8f0;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
    }

    .clan-selector-card:hover {
        transform: translateY(-8px);
        border-color: var(--clan-color);
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
    }

    .light .clan-selector-card:hover {
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
    }

    .card-banner {
        position: relative;
        width: 100%;
        height: 100px;
        overflow: hidden;
    }

    .card-banner img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        transition: transform 0.6s ease;
    }

    .clan-selector-card:hover .card-banner img {
        transform: scale(1.1);
    }

    .banner-overlay {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: linear-gradient(
            180deg,
            transparent 0%,
            rgba(26, 26, 31, 1) 100%
        );
    }

    .light .banner-overlay {
        background: linear-gradient(
            180deg,
            transparent 0%,
            rgba(255, 255, 255, 1) 100%
        );
    }

    .card-content {
        display: flex;
        align-items: center;
        gap: 1.25rem;
        padding: 0 1.5rem 1.5rem;
        margin-top: -20px;
        position: relative;
        z-index: 1;
    }

    .clan-badge {
        width: 64px;
        height: 64px;
        flex-shrink: 0;
        background: #1a1a1f;
        padding: 6px;
        border-radius: 16px;
        border: 2px solid var(--clan-color);
        box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
    }

    .light .clan-badge {
        background: white;
        box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
    }

    .clan-badge img {
        width: 100%;
        height: 100%;
        object-fit: contain;
    }

    .badge-placeholder {
        width: 100%;
        height: 100%;
        background: var(--clan-color);
        color: white;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 800;
        font-size: 1.5rem;
    }

    .clan-info {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
        flex: 1;
        min-width: 0;
    }

    .clan-name {
        font-weight: 800;
        font-size: 1.15rem;
        color: white;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .light .clan-name {
        color: #1a202c;
    }

    .clan-tag {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.4);
        font-family: 'JetBrains Mono', ui-monospace, monospace;
    }

    .light .clan-tag {
        color: #718096;
    }
</style>
