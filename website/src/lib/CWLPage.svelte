<script lang="ts">
    import { onMount } from 'svelte';
    import { fade, slide } from 'svelte/transition';

    export let theme: 'dark' | 'light';
    export let apiBaseUrl: string;

    interface SideClan {
        clan_tag: string;
        name: string;
        belongs_to: string | null;
        display_index: number;
    }

    interface CWLStats {
        clan_tag: string;
        season: string;
        league_id: number | null;
        league_name: string | null;
        league_badge_url: string | null;
        rank: number | null;
    }

    interface SideClanCwlHistory {
        clan: SideClan;
        history: CWLStats[];
    }

    interface MainClan {
        tag: string;
        name?: string;
        nameDB?: string;
        index: number;
    }

    let sideClansHistory: SideClanCwlHistory[] = [];
    let mainClans: MainClan[] = [];
    let loading = true;
    let error: string | null = null;

    onMount(async () => {
        try {
            const [sideClansRes, mainClansRes] = await Promise.all([
                fetch(`${apiBaseUrl}/api/sideclans`),
                fetch(`${apiBaseUrl}/api/coc/clans`, {
                    credentials: 'include',
                }),
            ]);

            if (!sideClansRes.ok || !mainClansRes.ok) {
                throw new Error('Failed to fetch data from API');
            }

            sideClansHistory = await sideClansRes.json();
            mainClans = await mainClansRes.json();
            loading = false;
        } catch (e: any) {
            error = e.message;
            loading = false;
        }
    });

    // Reactive grouped clans
    $: groupedClans = (() => {
        const groups: Record<string, SideClanCwlHistory[]> = {};

        // Group by parent tag
        sideClansHistory.forEach((item) => {
            // Use parent tag if available, otherwise use its own tag (as it's likely a main clan)
            const parentTag = (item.clan.belongs_to || item.clan.clan_tag)
                .trim()
                .toUpperCase();

            // EXCLUSION: Don't show clans where the latest history entry is "Unranked"
            // or if there is no history at all.
            const latestLeague =
                item.history.length > 0 ? item.history[0].league_name : null;
            if (!latestLeague || latestLeague === 'Unranked') return;

            if (!groups[parentTag]) {
                groups[parentTag] = [];
            }
            groups[parentTag].push(item);
        });

        const fallbackNames: Record<string, string> = {
            '#2YUPV0UYC': 'LOST',
            '#2LU2V2LPU': 'LOST 2',
            '#2QC0QQPQ2': 'LOST 3',
            '#2YVPC20UY': 'LOST 6 EX / Vegan',
            '#2J8UG90R2': 'LOST 7',
            '#2RUJPG9JC': 'LOST 8',
            '#2820UPPQC': 'Lost F2P & LOST F2P 2',
        };

        // Convert to sorted array of groups
        return Object.entries(groups)
            .map(([parentTag, clans]) => {
                const tagFull = parentTag.trim().toUpperCase();

                // 1. Try to find the name from mainClans (fetched from API)
                let mainClan = mainClans.find((c) => {
                    const cTag = (c.tag || '').trim().toUpperCase();
                    const cTagWithHash = cTag.startsWith('#')
                        ? cTag
                        : `#${cTag}`;
                    return cTagWithHash === tagFull || cTag === tagFull;
                });

                let name = mainClan ? (mainClan.name || mainClan.nameDB) : null;

                // 2. Try to find the name from sideClansHistory (direct database entry)
                if (!name) {
                    const sideMain = sideClansHistory.find(
                        (s) => s.clan.clan_tag.trim().toUpperCase() === tagFull,
                    );
                    if (sideMain) name = sideMain.clan.name;
                }

                // 3. Specific override for F2P as requested
                if (tagFull === '#2820UPPQC') {
                    name = 'Lost F2P & LOST F2P 2';
                }

                // 4. Try fallbackNames
                if (!name) {
                    name = fallbackNames[tagFull];
                }

                // 5. Ultimate fallback to the tag itself or "Unknown Group"
                const finalName = name || parentTag || 'Other Clans';

                return {
                    parentTag,
                    parentName: finalName,
                    index: mainClan ? mainClan.index || 999 : 999,
                    clans: clans.sort((a, b) => {
                        // Priority 1: Main clan always first
                        const aTag = a.clan.clan_tag.trim().toUpperCase();
                        const bTag = b.clan.clan_tag.trim().toUpperCase();
                        if (aTag === tagFull) return -1;
                        if (bTag === tagFull) return 1;

                        // Priority 2: Use display_index from backend (0 is last)
                        if (a.clan.display_index !== b.clan.display_index) {
                            const aIdx =
                                a.clan.display_index === 0
                                    ? 1000
                                    : a.clan.display_index;
                            const bIdx =
                                b.clan.display_index === 0
                                    ? 1000
                                    : b.clan.display_index;
                            return aIdx - bIdx;
                        }

                        // Priority 3: Alphabetical fallback
                        return (a.clan.name || '').localeCompare(
                            b.clan.name || '',
                        );
                    }),
                };
            })
            .sort((a, b) => a.index - b.index);
    })();

    function getLeagueTrend(history: CWLStats[]) {
        if (history.length < 2) return null;
        const current = history[0].league_id;
        const previous = history[1].league_id;
        if (!current || !previous) return null;
        if (current > previous) return 'up';
        if (current < previous) return 'down';
        return 'same';
    }

    function getRankTrend(history: CWLStats[]) {
        if (history.length < 2) return null;
        const current = history[0].rank;
        const previous = history[1].rank;
        if (!current || !previous) return null;
        if (current < previous) return 'up'; // Rank 1 is better than Rank 2
        if (current > previous) return 'down';
        return 'same';
    }

    function getLocalBadgeUrl(leagueName: string | null) {
        if (!leagueName || leagueName === 'Unranked') return null;

        // Map "Bronze League III" to "Bronze_3", etc.
        const name = leagueName.replace(' League', '').trim();
        const parts = name.split(' ');
        if (parts.length !== 2) return null;

        const tier = parts[0]; // e.g., "Bronze"
        const rankRoman = parts[1]; // e.g., "III"

        let rankNumber = '';
        if (rankRoman === 'I') rankNumber = '1';
        else if (rankRoman === 'II') rankNumber = '2';
        else if (rankRoman === 'III') rankNumber = '3';
        else return null;

        // Ensure first letter is capitalized for the filename
        const tierFormatted =
            tier.charAt(0).toUpperCase() + tier.slice(1).toLowerCase();

        // Using relative path to assets folder
        try {
            return new URL(
                `../assets/Clash Of Clans/CWL Leagues/Icon_HV_CWL_${tierFormatted}_${rankNumber}.png`,
                import.meta.url,
            ).href;
        } catch (e) {
            return null;
        }
    }
</script>

<div class="cwl-page {theme}" in:fade={{ duration: 300 }}>
    <div class="bg-glow"></div>
    <div class="container">
        <header>
            <h1>Clankriegsliga</h1>
            <p>Übersicht der CWL-Leistung unserer Clans</p>
        </header>

        {#if loading}
            <div class="loading-container">
                <div class="spinner"></div>
                <p>Lade CWL Daten...</p>
            </div>
        {:else if error}
            <div class="error-container">
                <div class="error-card">
                    <span class="error-icon">⚠️</span>
                    <p>{error}</p>
                </div>
            </div>
        {:else}
            {#each groupedClans as { parentTag, parentName, clans }}
                <section class="clan-group">
                    <h2 class="group-title">
                        {parentName}
                    </h2>

                    <div class="clans-grid">
                        {#each clans as entry}
                            <div class="clan-stats-card">
                                <div class="clan-header">
                                    <h3 class="clan-name">{entry.clan.name}</h3>
                                    <span class="clan-tag"
                                        >{entry.clan.clan_tag}</span
                                    >
                                </div>

                                <div class="league-info">
                                    {#if entry.history.length > 0}
                                        {@const current = entry.history[0]}
                                        <div class="current-league">
                                            <div class="league-badge-container">
                                                {#if getLocalBadgeUrl(current.league_name)}
                                                    <img
                                                        src={getLocalBadgeUrl(
                                                            current.league_name,
                                                        )}
                                                        alt={current.league_name}
                                                        class="league-badge"
                                                    />
                                                {:else if current.league_badge_url}
                                                    <img
                                                        src={current.league_badge_url}
                                                        alt={current.league_name}
                                                        class="league-badge"
                                                    />
                                                {:else}
                                                    <div class="no-badge"></div>
                                                {/if}
                                            </div>
                                            <div class="league-details">
                                                <span class="league-label"
                                                    >Aktuell:</span
                                                >
                                                <div class="league-name-row">
                                                    <span class="league-value"
                                                        >{current.league_name ||
                                                            'N/A'}</span
                                                    >

                                                    {#if getLeagueTrend(entry.history) === 'up'}
                                                        <span
                                                            class="trend up"
                                                            title="Aufgestiegen"
                                                            >▲</span
                                                        >
                                                    {:else if getLeagueTrend(entry.history) === 'down'}
                                                        <span
                                                            class="trend down"
                                                            title="Abgestiegen"
                                                            >▼</span
                                                        >
                                                    {/if}
                                                </div>
                                            </div>
                                        </div>
                                    {:else}
                                        <p class="no-data">
                                            Keine CWL-Daten vorhanden
                                        </p>
                                    {/if}
                                </div>

                                {#if entry.history.length > 1}
                                    <div class="history-table-wrapper">
                                        <table class="history-table">
                                            <thead>
                                                <tr>
                                                    <th>Saison</th>
                                                    <th>Liga</th>
                                                    <th>Rank</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {#each entry.history.slice(0, 5) as stat}
                                                    <tr>
                                                        <td>{stat.season}</td>
                                                        <td>
                                                            <div
                                                                class="table-league-cell"
                                                            >
                                                                {#if getLocalBadgeUrl(stat.league_name)}
                                                                    <img
                                                                        src={getLocalBadgeUrl(
                                                                            stat.league_name,
                                                                        )}
                                                                        alt=""
                                                                        class="table-badge"
                                                                    />
                                                                {:else if stat.league_badge_url}
                                                                    <img
                                                                        src={stat.league_badge_url}
                                                                        alt=""
                                                                        class="table-badge"
                                                                    />
                                                                {/if}
                                                                <span
                                                                    >{stat.league_name ||
                                                                        '-'}</span
                                                                >
                                                            </div>
                                                        </td>
                                                        <td
                                                            >{stat.rank ||
                                                                '-'}</td
                                                        >
                                                    </tr>
                                                {/each}
                                            </tbody>
                                        </table>
                                    </div>
                                {/if}
                            </div>
                        {/each}
                    </div>
                </section>
            {/each}
        {/if}
    </div>
</div>

<style>
    .cwl-page {
        min-height: 100vh;
        padding-top: 100px;
        padding-bottom: 50px;
        color: white;
        position: relative;
        overflow: hidden;
    }

    .bg-glow {
        position: absolute;
        top: 0;
        left: 50%;
        transform: translateX(-50%);
        width: 100%;
        height: 600px;
        background: radial-gradient(
            circle at 50% 0%,
            rgba(88, 101, 242, 0.15) 0%,
            transparent 70%
        );
        pointer-events: none;
        z-index: 0;
    }

    .container {
        max-width: 1200px;
        margin: 0 auto;
        padding: 0 20px;
        position: relative;
        z-index: 1;
    }

    header {
        text-align: center;
        margin-bottom: 50px;
    }

    header h1 {
        font-size: 3rem;
        font-weight: 900;
        margin-bottom: 10px;
        background: linear-gradient(
            135deg,
            #fff 0%,
            rgba(255, 255, 255, 0.7) 100%
        );
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .cwl-page.light header h1 {
        background: linear-gradient(135deg, #1e293b 0%, #64748b 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    header p {
        color: rgba(255, 255, 255, 0.6);
        font-size: 1.2rem;
    }

    .cwl-page.light header p {
        color: #64748b;
    }

    .clan-group {
        margin-bottom: 220px;
    }

    .group-title {
        font-size: 2.2rem;
        font-weight: 950;
        margin: 120px 0 50px;
        color: #fff;
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        gap: 25px;
        text-transform: uppercase;
        letter-spacing: 4px;
        text-shadow: 0 10px 30px rgba(0, 0, 0, 0.8);
        position: relative;
    }

    .group-title::before {
        content: '';
        width: 16px;
        height: 16px;
        background: #5865f2;
        border-radius: 4px;
        transform: rotate(45deg);
        box-shadow:
            0 0 25px #5865f2,
            0 0 50px rgba(88, 101, 242, 0.3);
    }

    .group-title::after {
        content: '';
        flex: 1;
        height: 2px;
        background: linear-gradient(
            90deg,
            rgba(88, 101, 242, 0.6) 0%,
            rgba(88, 101, 242, 0.2) 50%,
            transparent 100%
        );
    }

    .cwl-page.light .group-title {
        color: #1e293b;
    }

    .clans-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
        gap: 25px;
    }

    .clan-stats-card {
        background: rgba(30, 41, 59, 0.4);
        backdrop-filter: blur(16px);
        -webkit-backdrop-filter: blur(16px);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 24px;
        padding: 30px;
        transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
        position: relative;
        overflow: hidden;
    }

    .clan-stats-card::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 4px;
        background: linear-gradient(90deg, #5865f2, #818cf8);
        opacity: 0;
        transition: opacity 0.3s ease;
    }

    .clan-stats-card:hover {
        transform: translateY(-8px);
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
        border-color: rgba(88, 101, 242, 0.3);
    }

    .clan-stats-card:hover::before {
        opacity: 1;
    }

    .cwl-page.light .clan-stats-card {
        background: rgba(255, 255, 255, 0.8);
        border-color: rgba(0, 0, 0, 0.05);
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.03);
        color: #1e293b;
    }

    .clan-header {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        margin-bottom: 25px;
        gap: 4px;
    }

    .clan-name {
        font-size: 1.6rem;
        font-weight: 800;
        margin: 0;
        letter-spacing: -0.5px;
    }

    .clan-tag {
        font-family: monospace;
        color: rgba(255, 255, 255, 0.4);
        font-size: 0.9rem;
    }

    .cwl-page.light .clan-tag {
        color: #64748b;
    }

    .current-league {
        display: flex;
        align-items: center;
        gap: 20px;
        margin-bottom: 30px;
        padding: 20px;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 20px;
    }

    .league-badge-container {
        width: 64px;
        height: 64px;
        display: flex;
        align-items: center;
        justify-content: center;
        filter: drop-shadow(0 0 10px rgba(0, 0, 0, 0.3));
    }

    .league-badge {
        width: 100%;
        height: 100%;
        object-fit: contain;
        transition: transform 0.3s ease;
    }

    .clan-stats-card:hover .league-badge {
        transform: scale(1.1) rotate(5deg);
    }

    .league-details {
        flex: 1;
        display: flex;
        flex-direction: column;
    }

    .league-label {
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: rgba(255, 255, 255, 0.5);
        margin-bottom: 2px;
    }

    .league-name-row {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .league-value {
        font-weight: 900;
        font-size: 1.3rem;
        color: #fff;
        line-height: 1.2;
    }

    .trend {
        font-size: 1rem;
        padding: 2px 6px;
        border-radius: 6px;
        background: rgba(0, 0, 0, 0.2);
    }

    .trend.up {
        color: #10b981;
    }
    .trend.down {
        color: #ef4444;
    }

    .history-table-wrapper {
        margin-top: 20px;
        background: rgba(0, 0, 0, 0.15);
        border-radius: 16px;
        padding: 5px;
    }

    .history-table {
        width: 100%;
        border-collapse: separate;
        border-spacing: 0;
        font-size: 0.95rem;
    }

    .history-table th {
        text-align: left;
        padding: 12px 15px;
        color: rgba(255, 255, 255, 0.3);
        font-weight: 600;
        text-transform: uppercase;
        font-size: 0.75rem;
        letter-spacing: 1px;
    }

    .history-table td {
        padding: 12px 15px;
        border-top: 1px solid rgba(255, 255, 255, 0.03);
        color: rgba(255, 255, 255, 0.8);
    }

    .cwl-page.light .history-table td {
        border-top-color: rgba(0, 0, 0, 0.05);
        color: #1e293b;
    }

    .table-league-cell {
        display: flex;
        align-items: center;
        gap: 10px;
        font-weight: 500;
    }

    .table-badge {
        width: 24px;
        height: 24px;
        object-fit: contain;
    }

    .cwl-page.light .history-table td {
        border-top-color: rgba(0, 0, 0, 0.05);
    }

    .loading-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 100px 0;
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 4px solid rgba(88, 101, 242, 0.2);
        border-top-color: #5865f2;
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin-bottom: 20px;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .error-card {
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid #ef4444;
        padding: 20px;
        border-radius: 15px;
        text-align: center;
        max-width: 500px;
        margin: 50px auto;
    }
</style>
