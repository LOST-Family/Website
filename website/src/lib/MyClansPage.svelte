<script lang="ts">
    import { onMount, createEventDispatcher } from 'svelte';
    import { fade } from 'svelte/transition';
    import { user } from './auth';

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

    let userClans: { coc: Clan[]; cr: Clan[] } = { coc: [], cr: [] };
    let loading = true;
    let error: string | null = null;

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
        if (n.includes('F2P 2') || n.includes('F2P2')) return '#05762b';
        if (n.includes('F2P')) return '#c90000';
        if (n.includes('8') || n.includes('VIII')) return '#d100c7';
        if (n.includes('7') || n.includes('VII')) return '#007076';
        if (n.includes('6') || n.includes('VI')) return '#b54800';
        if (n.includes('4') || n.includes('IV')) return '#691a97';
        if (n.includes('5') || n.includes('V')) return '#024885';
        if (n.includes('3') || n.includes('III')) return '#c89e00';

        if (index === 1) return '#c90000';
        if (index === 2) return '#05762b';
        if (index === 3) return '#c89e00';
        if (index === 4) return '#691a97';
        if (index === 5) return '#024885';
        if (index === 6) return '#b54800';
        if (index === 7) return '#007076';
        if (index === 8) return '#d100c7';

        return '#c90000';
    }

    async function loadMyClans() {
        if (!$user) {
            loading = false;
            return;
        }

        loading = true;
        error = null;
        try {
            const [accountsRes, cocClansRes, crClansRes] = await Promise.all([
                fetch(`${apiBaseUrl}/api/me/accounts`, {
                    credentials: 'include',
                }),
                fetch(`${apiBaseUrl}/api/coc/clans`),
                fetch(`${apiBaseUrl}/api/cr/clans`),
            ]);

            if (!accountsRes.ok) {
                throw new Error('Fehler beim Laden der Accounts');
            }

            const accounts = await accountsRes.json();
            const allCocClans: Clan[] = cocClansRes.ok
                ? await cocClansRes.json()
                : [];
            const allCrClans: Clan[] = crClansRes.ok
                ? await crClansRes.json()
                : [];

            const officialCocTags = new Set(
                allCocClans.map((c) => c.tag.toUpperCase()),
            );
            const officialCrTags = new Set(
                allCrClans.map((c) => c.tag.toUpperCase()),
            );

            const cocAccountClans = new Set<string>();
            const cocAccounts =
                accounts.coc || (Array.isArray(accounts) ? accounts : []);
            cocAccounts.forEach((acc: any) => {
                // Determine clan based on Upstream API (upstream_clan or clanDB) if available.
                // We exclusively use upstream data now to avoid showing non-LOST clans from Supercell API.
                const clan =
                    acc.upstream_clan && acc.upstream_clan.tag
                        ? acc.upstream_clan
                        : acc.clanDB && acc.clanDB.tag
                          ? acc.clanDB
                          : null;

                if (clan && officialCocTags.has(clan.tag.toUpperCase())) {
                    cocAccountClans.add(clan.tag.toUpperCase());
                }
            });

            const crAccountClans = new Set<string>();
            const crAccounts = accounts.cr || [];
            crAccounts.forEach((acc: any) => {
                // Determine clan based on Upstream API (upstream_clan or clanDB) if available.
                // We exclusively use upstream data now to avoid showing non-LOST clans from Supercell API.
                const clan =
                    acc.upstream_clan && acc.upstream_clan.tag
                        ? acc.upstream_clan
                        : acc.clanDB && acc.clanDB.tag
                          ? acc.clanDB
                          : null;

                if (clan && officialCrTags.has(clan.tag.toUpperCase())) {
                    crAccountClans.add(clan.tag.toUpperCase());
                }
            });

            userClans = {
                coc: allCocClans
                    .filter((c) => cocAccountClans.has(c.tag.toUpperCase()))
                    .map((c) => ({ ...c, gameType: 'coc' as const }))
                    .sort((a, b) => (a.index || 0) - (b.index || 0)),
                cr: allCrClans
                    .filter((c) => crAccountClans.has(c.tag.toUpperCase()))
                    .map((c) => ({ ...c, gameType: 'cr' as const }))
                    .sort((a, b) => (a.index || 0) - (b.index || 0)),
            };
        } catch (e) {
            error = e instanceof Error ? e.message : 'Unbekannter Fehler';
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadMyClans();
    });

    $: if ($user) {
        loadMyClans();
    }

    function navigateToClan(clan: Clan) {
        const type = clan.gameType || 'coc';
        dispatch('navigate', `${type}/clan/${clan.tag.replace('#', '')}`);
    }

    $: cocClans = userClans.coc;
    $: crClans = userClans.cr;
</script>

<div class="my-clans-page" class:light={theme === 'light'}>
    <div class="container">
        <header class="page-header">
            <div class="header-content">
                <div class="title-group">
                    <h1 class="page-title">Deine Clans</h1>
                    <p class="page-subtitle">
                        Hier siehst du alle Clans der LOST Family, in denen du
                        Mitglied bist.
                    </p>
                </div>
            </div>
        </header>

        {#if loading}
            <div class="state-container">
                <div class="spinner"></div>
                <p>Deine Clans werden geladen...</p>
            </div>
        {:else if error}
            <div class="state-container error">
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
                <button class="retry-btn" on:click={loadMyClans}
                    >Erneut versuchen</button
                >
            </div>
        {:else if cocClans.length === 0 && crClans.length === 0}
            <div class="empty-state" in:fade>
                <div class="empty-icon">🏰</div>
                <h3>Keine Clans gefunden</h3>
                <p>Du bist derzeit in keinem Clan der LOST Family.</p>
                <button
                    class="action-btn"
                    on:click={() => dispatch('navigate', 'coc/clans')}
                >
                    Clans entdecken
                </button>
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
                                            {#if clan.index > 0}
                                                <span class="clan-index-badge"
                                                    >#{clan.index}</span
                                                >
                                            {/if}
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
                                            {#if clan.index > 0}
                                                <span class="clan-index-badge"
                                                    >#{clan.index}</span
                                                >
                                            {/if}
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
    .my-clans-page {
        min-height: 100vh;
        background: #0a0a0f;
        padding: 6rem 2rem 4rem;
        color: white;
    }

    .my-clans-page.light {
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
        font-size: 3rem;
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

    .state-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 4rem;
        text-align: center;
        gap: 1.5rem;
        color: #b9bbbe;
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

    .clan-meta {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 0.5rem;
    }

    .game-tag {
        font-size: 0.65rem;
        font-weight: 800;
        padding: 2px 8px;
        border-radius: 6px;
        background: #3ba55c;
        color: white;
        text-transform: uppercase;
    }

    .game-tag.cr {
        background: #5865f2;
    }

    .clan-index-badge {
        font-size: 0.75rem;
        font-weight: 700;
        color: rgba(255, 255, 255, 0.3);
    }

    .light .clan-index-badge {
        color: #cbd5e1;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 4rem;
        text-align: center;
        gap: 1.5rem;
        color: #b9bbbe;
    }

    .light .empty-state {
        color: #64748b;
    }

    .empty-icon {
        font-size: 4rem;
    }

    .empty-state h3 {
        font-size: 1.75rem;
        margin: 0;
        color: white;
    }

    .light .empty-state h3 {
        color: #1e293b;
    }

    .retry-btn,
    .action-btn {
        margin-top: 1rem;
        padding: 0.75rem 1.75rem;
        background: #5865f2;
        color: white;
        border: none;
        border-radius: 12px;
        font-weight: 700;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .retry-btn:hover,
    .action-btn:hover {
        background: #4752c4;
        transform: translateY(-2px);
    }
</style>
