<script lang="ts">
    import { onMount } from 'svelte';
    import { fade, slide } from 'svelte/transition';
    import type { GameType } from './auth';

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

    export let apiBaseUrl: string = '';
    export let theme: 'dark' | 'light' = 'dark';
    export let clansData: Clan[] | null = null;
    export let gameType: GameType = 'coc';

    // Exported stats for parent component
    export let clanCount: number = 0;
    export let playerCount: number = 0;

    $: apiPrefix = gameType === 'coc' ? '/api/coc' : '/api/cr';

    interface Clan {
        tag: string;
        nameDB: string;
        index: number;
        badgeUrl: string;
    }

    interface Player {
        tag: string;
        userId: string | null;
        roleInClan: string;
        clanDB: string | null;
        nameDB: string;
        isHidden: boolean;
    }

    interface ClanWithMembers extends Clan {
        members: Player[];
        loading: boolean;
        error: string | null;
        isCollapsed: boolean;
    }

    let clans: ClanWithMembers[] = [];
    let mainLoading = true;
    let mainError: string | null = null;

    // Update exported counts reactively
    $: clanCount = clans.length;
    $: playerCount = clans.reduce((sum, c) => sum + c.members.length, 0);

    const roleOrder: Record<string, number> = {
        ANFÜHRER: 10,
        'ANFÜHRER 2': 11,
        'ANFÜHRER 3': 12,
        'ANFÜHRER 4': 13,
        'ANFÜHRER 5': 14,
        'ANFÜHRER 6': 15,
        'ANFÜHRER 7': 16,
        'ANFÜHRER 8': 17,
        'ANFÜHRER GP': 18,
        'ANFÜHRER ANTHRAZIT': 19,
        'VIZE 1': 20,
        'VIZE 2': 21,
        'VIZE 3': 22,
        'VIZE 4': 23,
        'VIZE 5': 24,
        'VIZE 6': 25,
        'VIZE 7': 26,
        'VIZE 8': 27,
        'VIZE GP': 28,
        'VIZE ANTHRAZIT': 29,
        'ÄLTESTER 1': 30,
        'ÄLTESTER 2': 31,
        'ÄLTESTER 3': 32,
        'ÄLTESTER 4': 33,
        'ÄLTESTER 5': 34,
        'ÄLTESTER 6': 35,
        'ÄLTESTER 7': 36,
        'ÄLTESTER 8': 37,
        'ÄLTESTER GP': 38,
        'ÄLTESTER ANTHRAZIT': 39,
        MITGLIED: 40,
        'MITGLIED 2': 41,
        'MITGLIED 3': 42,
        'MITGLIED 4': 43,
        'MITGLIED 5': 44,
        'MITGLIED 6': 45,
        'MITGLIED 7': 46,
        'MITGLIED 8': 47,
        'MITGLIED GP': 48,
        'MITGLIED ANTHRAZIT': 49,
        'ANFÜHRER CR': 60,
        'ANFÜHRER CR 2': 61,
        'ANFÜHRER CR 3': 62,
        'ANFÜHRER CR 4': 63,
        'ANFÜHRER CR 5': 64,
        'VIZE CR 1': 70,
        'VIZE CR 2': 71,
        'VIZE CR 3': 72,
        'VIZE CR 4': 73,
        'VIZE CR 5': 74,
        'ÄLTESTER CR 1': 80,
        'ÄLTESTER CR 2': 81,
        'ÄLTESTER CR 3': 82,
        'ÄLTESTER CR 4': 83,
        'ÄLTESTER CR 5': 84,
        'MITGLIED CR': 90,
        'MITGLIED CR 2': 91,
        'MITGLIED CR 3': 92,
        'MITGLIED CR 4': 93,
        'MITGLIED CR 5': 94,
    };

    const roleColors: Record<string, string> = {
        ANFÜHRER: '#c90000',
        'ANFÜHRER 2': '#05762b',
        'ANFÜHRER 3': '#c89e00',
        'ANFÜHRER 4': '#691a97',
        'ANFÜHRER 5': '#024885',
        'ANFÜHRER 6': '#b54800',
        'ANFÜHRER 7': '#007076',
        'ANFÜHRER 8': '#d100c7',
        'ANFÜHRER GP': '#a5025a',
        'ANFÜHRER ANTHRAZIT': '#3d3a3f',
        'VIZE 1': '#f61313',
        'VIZE 2': '#02990b',
        'VIZE 3': '#eec010',
        'VIZE 4': '#9f03ce',
        'VIZE 5': '#075aca',
        'VIZE 6': '#ff7600',
        'VIZE 7': '#00b3c6',
        'VIZE 8': '#fb2bff',
        'VIZE GP': '#df0c5d',
        'VIZE ANTHRAZIT': '#817d7d',
        'ÄLTESTER 1': '#f53d4a',
        'ÄLTESTER 2': '#51ff84',
        'ÄLTESTER 3': '#e1e459',
        'ÄLTESTER 4': '#a76cd8',
        'ÄLTESTER 5': '#56a3d8',
        'ÄLTESTER 6': '#f8994f',
        'ÄLTESTER 7': '#5bb2be',
        'ÄLTESTER 8': '#f261ff',
        'ÄLTESTER GP': '#e04dbd',
        'ÄLTESTER ANTHRAZIT': '#546e7a',
        MITGLIED: '#f58190',
        'MITGLIED 2': '#a0f5b1',
        'MITGLIED 3': '#fdf277',
        'MITGLIED 4': '#d9a6f9',
        'MITGLIED 5': '#9eb7f5',
        'MITGLIED 6': '#f6c17b',
        'MITGLIED 7': '#9ed6e2',
        'MITGLIED 8': '#ff88ee',
        'MITGLIED GP': '#f38ac5',
        'MITGLIED ANTHRAZIT': '#546e7a',
        'ANFÜHRER CR': '#227e3f',
        'ANFÜHRER CR 2': '#227e3f',
        'ANFÜHRER CR 3': '#227e3f',
        'ANFÜHRER CR 4': '#227e3f',
        'ANFÜHRER CR 5': '#227e3f',
        'VIZE CR 1': '#289c4d',
        'VIZE CR 2': '#289c4d',
        'VIZE CR 3': '#289c4d',
        'VIZE CR 4': '#289c4d',
        'VIZE CR 5': '#289c4d',
        'ÄLTESTER CR 1': '#00ff9e',
        'ÄLTESTER CR 2': '#93d128',
        'ÄLTESTER CR 3': '#992d22',
        'ÄLTESTER CR 4': '#13598b',
        'ÄLTESTER CR 5': '#ce9201',
        'MITGLIED CR': '#42ffb7',
        'MITGLIED CR 2': '#a4cc60',
        'MITGLIED CR 3': '#be4d40',
        'MITGLIED CR 4': '#2183c9',
        'MITGLIED CR 5': '#fabf05',
    };

    function getDisplayRole(role: string): string {
        if (role.includes('ANFÜHRER')) return 'Anführer';
        if (role.includes('VIZE')) return 'Vize-Anführer';
        if (role.includes('ÄLTESTER')) return 'Ältester';
        if (role.includes('MITGLIED')) return 'Mitglied';
        return role;
    }

    // Helper to get display name - use nameAPI, nameDB, or tag
    function getPlayerName(player: Player): string {
        return player.nameDB || player.tag || 'Unknown';
    }

    onMount(async () => {
        try {
            // 1. Fetch Clans (filtered by game type) or use provided data
            let fetchedClanData: Clan[] = [];
            if (clansData) {
                fetchedClanData = clansData;
            } else {
                const response = await fetch(`${apiBaseUrl}${apiPrefix}/clans`);
                if (!response.ok) throw new Error(`HTTP ${response.status}`);
                fetchedClanData = await response.json();
            }

            // sort the clanData by index
            fetchedClanData.sort((a, b) => a.index - b.index);

            // Initialize with loading state
            clans = fetchedClanData.map((clan) => ({
                ...clan,
                members: [],
                loading: true,
                error: null,
                isCollapsed: true,
            }));
            mainLoading = false;
            // 2. Fetch Members for ALL clans in parallel
            await Promise.all(
                clans.map(async (clan, index) => {
                    try {
                        const encodedTag = encodeURIComponent(clan.tag);
                        const res = await fetch(
                            `${apiBaseUrl}${apiPrefix}/clans/${encodedTag}/members-lite`
                        );

                        if (!res.ok) throw new Error(`Failed to load`);

                        let members: Player[] = await res.json();

                        if (Array.isArray(members)) {
                            // Normalize and sort - keep all players that have a tag
                            members = members
                                .filter((m) => m && m.tag && !m.isHidden)
                                .map((m) => {
                                    const baseRole = m.roleInClan
                                        ? m.roleInClan.toUpperCase()
                                        : 'NOTINCLAN';
                                    let computedRole = baseRole;

                                    // Standard roles from API (CoC and CR)
                                    const standardRoles = [
                                        'LEADER',
                                        'COLEADER',
                                        'CO-LEADER',
                                        'ELDER',
                                        'ADMIN',
                                        'MEMBER',
                                        'LEADER',
                                        'COLEADER',
                                        'ELDER',
                                        'MEMBER',
                                        // CR specific (camelCase from Supercell API)
                                        'COLEADER',
                                    ];

                                    // Normalize for matching
                                    const upperRole = baseRole.toUpperCase();
                                    const isCoCStandard = [
                                        'LEADER',
                                        'COLEADER',
                                        'CO-LEADER',
                                        'ELDER',
                                        'ADMIN',
                                        'MEMBER',
                                    ].includes(upperRole);

                                    const isCRStandard = [
                                        'LEader', // Sometimes mixed case in CR bot
                                        'COLEADER',
                                        'ELDER',
                                        'MEMBER',
                                    ]
                                        .map((r) => r.toUpperCase())
                                        .includes(upperRole);

                                    if (isCoCStandard || isCRStandard) {
                                        const clanIndex = clan.index;
                                        const clanNameUpper = (
                                            clan.nameDB || ''
                                        ).toUpperCase();

                                        if (gameType === 'cr') {
                                            if (upperRole === 'LEADER') {
                                                computedRole =
                                                    clanIndex === 1
                                                        ? 'ANFÜHRER CR'
                                                        : `ANFÜHRER CR ${clanIndex}`;
                                            } else if (
                                                upperRole === 'COLEADER' ||
                                                upperRole === 'CO-LEADER'
                                            ) {
                                                computedRole = `VIZE CR ${clanIndex}`;
                                            } else if (
                                                upperRole === 'ELDER' ||
                                                upperRole === 'ADMIN'
                                            ) {
                                                computedRole = `ÄLTESTER CR ${clanIndex}`;
                                            } else if (upperRole === 'MEMBER') {
                                                computedRole =
                                                    clanIndex === 1
                                                        ? 'MITGLIED CR'
                                                        : `MITGLIED CR ${clanIndex}`;
                                            }
                                        } else {
                                            const isGP =
                                                clanNameUpper.includes('GP');
                                            const isAnthrazit =
                                                clanNameUpper.includes(
                                                    'ANTHRAZIT'
                                                );

                                            if (isGP) {
                                                if (upperRole === 'LEADER')
                                                    computedRole =
                                                        'ANFÜHRER GP';
                                                else if (
                                                    upperRole === 'COLEADER' ||
                                                    upperRole === 'CO-LEADER'
                                                )
                                                    computedRole = 'VIZE GP';
                                                else if (
                                                    upperRole === 'ELDER' ||
                                                    upperRole === 'ADMIN'
                                                )
                                                    computedRole =
                                                        'ÄLTESTER GP';
                                                else if (upperRole === 'MEMBER')
                                                    computedRole =
                                                        'MITGLIED GP';
                                            } else if (isAnthrazit) {
                                                if (upperRole === 'LEADER')
                                                    computedRole =
                                                        'ANFÜHRER ANTHRAZIT';
                                                else if (
                                                    upperRole === 'COLEADER' ||
                                                    upperRole === 'CO-LEADER'
                                                )
                                                    computedRole =
                                                        'VIZE ANTHRAZIT';
                                                else if (
                                                    upperRole === 'ELDER' ||
                                                    upperRole === 'ADMIN'
                                                )
                                                    computedRole =
                                                        'ÄLTESTER ANTHRAZIT';
                                                else if (upperRole === 'MEMBER')
                                                    computedRole =
                                                        'MITGLIED ANTHRAZIT';
                                            } else {
                                                if (upperRole === 'LEADER') {
                                                    computedRole =
                                                        clanIndex === 1
                                                            ? 'ANFÜHRER'
                                                            : `ANFÜHRER ${clanIndex}`;
                                                } else if (
                                                    upperRole === 'COLEADER' ||
                                                    upperRole === 'CO-LEADER'
                                                ) {
                                                    computedRole = `VIZE ${clanIndex}`;
                                                } else if (
                                                    upperRole === 'ELDER' ||
                                                    upperRole === 'ADMIN'
                                                ) {
                                                    computedRole = `ÄLTESTER ${clanIndex}`;
                                                } else if (
                                                    upperRole === 'MEMBER'
                                                ) {
                                                    computedRole =
                                                        clanIndex === 1
                                                            ? 'MITGLIED'
                                                            : `MITGLIED ${clanIndex}`;
                                                }
                                            }
                                        }
                                    }

                                    return {
                                        ...m,
                                        roleInClan: computedRole,
                                    };
                                })
                                .sort((a, b) => {
                                    const roleA = roleOrder[a.roleInClan] ?? 99;
                                    const roleB = roleOrder[b.roleInClan] ?? 99;
                                    return roleA - roleB;
                                });

                            clans[index].members = members;
                        }
                    } catch (e) {
                        clans[index].error = 'Could not load members';
                        console.error(
                            `Error loading members for ${clan.tag}:`,
                            e
                        );
                    } finally {
                        clans[index].loading = false;
                        // Trigger reactivity
                        clans = [...clans];
                    }
                })
            );
        } catch (e) {
            mainError = e instanceof Error ? e.message : 'Unknown error';
            mainLoading = false;
        }
    });

    function getPlayersByRole(members: Player[]) {
        const grouped = new Map<string, Player[]>();
        for (const member of members) {
            const role = member.roleInClan || 'NOTINCLAN';
            if (!grouped.has(role)) grouped.set(role, []);
            grouped.get(role)!.push(member);
        }
        return [...grouped.entries()].sort(
            (a, b) => (roleOrder[a[0]] ?? 99) - (roleOrder[b[0]] ?? 99)
        );
    }

    function toggleClan(index: number) {
        clans[index].isCollapsed = !clans[index].isCollapsed;
        clans = [...clans];
    }

    function getClanBanner(clanName: string): string {
        const name = clanName.toUpperCase();

        // Clash Royale clans use their own banners
        if (gameType === 'cr') {
            if (name === 'LOST') return bannerCR1;
            if (name.includes('2') || name.includes('II')) return bannerCR2;
            if (name.includes('3') || name.includes('III')) return bannerCR3;
            if (name.includes('4') || name.includes('IV')) return bannerCR4;
            if (name.includes('5') || name.includes('V')) return bannerCR5;
            return bannerDefault;
        }

        if (name.includes('F2P 2') || name.includes('F2P2')) return bannerF2P2;
        if (name.includes('F2P')) return bannerF2P;
        if (name.includes('GP')) return bannerGP;
        if (name.includes('3') || name.includes('III')) return banner3;
        if (name.includes('4') || name.includes('IV')) return banner4;
        if (name.includes('5') || name.includes('V')) return banner5;
        if (name.includes('6') || name.includes('VI')) return banner6;
        if (name.includes('7') || name.includes('VII')) return banner7;
        if (name.includes('8') || name.includes('VIII')) return banner8;
        if (name.includes('ANTHRAZIT')) return bannerAnthrazit;
        return bannerDefault;
    }
</script>

<div class="card-container" class:light={theme === 'light'}>
    {#if mainLoading}
        <div class="status-message">
            <div class="spinner"></div>
            <p>Loading clans...</p>
        </div>
    {:else if mainError}
        <div class="status-message error">
            <p>⚠️ {mainError}</p>
        </div>
    {:else}
        <div class="clans-grid">
            {#each clans as clan, i}
                <div
                    class="clan-card-item"
                    class:expanded={!clan.isCollapsed}
                    transition:fade
                >
                    <div
                        class="clan-card-header"
                        on:click={() => toggleClan(i)}
                        role="button"
                        tabindex="0"
                        on:keydown={(e) => e.key === 'Enter' && toggleClan(i)}
                    >
                        {#if clan.badgeUrl}
                            <img
                                src={clan.badgeUrl}
                                alt="Badge"
                                class="clan-badge"
                            />
                        {/if}
                        <div class="clan-titles">
                            <h3 class="clan-name">
                                {clan.nameDB || 'Unknown Clan'}
                            </h3>
                            <span class="clan-tag">{clan.tag}</span>
                        </div>
                        {#if !clan.loading}
                            <div class="member-count-badge">
                                {clan.members.length} Mitglieder
                            </div>
                        {/if}
                        <div
                            class="collapse-icon"
                            class:is-collapsed={clan.isCollapsed}
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="16"
                                height="16"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                ><path d="m6 9 6 6 6-6" /></svg
                            >
                        </div>
                    </div>

                    {#if clan.isCollapsed}
                        <div
                            class="clan-banner-container"
                            on:click={() => toggleClan(i)}
                            role="button"
                            tabindex="0"
                            on:keydown={(e) =>
                                e.key === 'Enter' && toggleClan(i)}
                            transition:slide
                        >
                            <img
                                src={getClanBanner(clan.nameDB || '')}
                                alt="LOST Clan Banner"
                                class="clan-banner"
                            />
                            <div class="banner-overlay" transition:fade>
                                <span>Mitglieder anzeigen</span>
                            </div>
                        </div>
                    {:else}
                        <div class="clan-card-body" transition:slide>
                            {#if clan.loading}
                                <div class="loading-members">
                                    <div class="mini-spinner"></div>
                                </div>
                            {:else if clan.error}
                                <div class="member-error">
                                    <span>{clan.error}</span>
                                </div>
                            {:else if clan.members.length === 0}
                                <div class="empty-members">
                                    <span>Keine Mitglieder</span>
                                </div>
                            {:else}
                                <div class="roles-flow">
                                    {#each getPlayersByRole(clan.members) as [role, players]}
                                        <div class="role-section">
                                            <div class="role-header">
                                                <span
                                                    class="role-dot"
                                                    style="background-color: {roleColors[
                                                        role
                                                    ] ||
                                                        (theme === 'light'
                                                            ? '#666'
                                                            : '#aaa')}"
                                                ></span>
                                                <span class="role-title"
                                                    >{getDisplayRole(
                                                        role
                                                    )}</span
                                                >
                                                <span class="role-count"
                                                    >{players.length}</span
                                                >
                                            </div>

                                            <div class="members-list">
                                                {#each players as player}
                                                    <div
                                                        class="member-item"
                                                        transition:fade
                                                    >
                                                        <div
                                                            class="member-avatar"
                                                            style="border-color: {roleColors[
                                                                role
                                                            ] ||
                                                                (theme ===
                                                                'light'
                                                                    ? '#eee'
                                                                    : '#444')}"
                                                        >
                                                            {getPlayerName(
                                                                player
                                                            )
                                                                .charAt(0)
                                                                .toUpperCase()}
                                                        </div>
                                                        <span
                                                            class="member-name"
                                                            style="color: {roleColors[
                                                                role
                                                            ] || 'inherit'}"
                                                            title={getPlayerName(
                                                                player
                                                            )}
                                                        >
                                                            {getPlayerName(
                                                                player
                                                            )}
                                                        </span>
                                                    </div>
                                                {/each}
                                            </div>
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .card-container {
        width: 100%;
        max-width: 1400px;
        margin: 0 auto;
        font-family: 'Inter', system-ui, sans-serif;
    }

    .status-message {
        padding: 48px;
        display: flex;
        flex-direction: column;
        align-items: center;
        color: #b9bbbe;
        text-align: center;
        gap: 16px;
        transition: color 0.4s ease;
    }

    .card-container.light .status-message {
        color: #5c6470;
    }

    .status-message.error {
        color: #ed4245;
    }

    /* Grid layout for clans */
    .clans-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
        gap: 20px;
        align-items: start;
    }

    .clan-card-item {
        background: #2f3136;
        border-radius: 8px;
        overflow: hidden;
        border: 1px solid #202225;
        display: flex;
        flex-direction: column;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        height: fit-content;
    }

    .clan-card-item.expanded {
        border-color: #5865f2;
        box-shadow: 0 4px 20px rgba(88, 101, 242, 0.15);
        z-index: 10;
    }

    .card-container.light .clan-card-item {
        background: #ffffff;
        border: 1px solid #e3e5e8;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
    }

    .clan-card-item:hover {
        transform: translateY(-4px);
        box-shadow: 0 12px 24px rgba(0, 0, 0, 0.4);
        border-color: #4f545c;
    }

    .clan-card-item.expanded:hover {
        border-color: #7d8ef5;
        box-shadow: 0 12px 32px rgba(88, 101, 242, 0.25);
    }

    .card-container.light .clan-card-item:hover {
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
    }

    .clan-card-header {
        padding: 16px 20px;
        background: #292b2f;
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 8px;
        border-bottom: 1px solid #202225;
        transition: all 0.4s ease;
        cursor: pointer;
        user-select: none;
    }

    .clan-card-header:hover {
        background: #32353b;
    }

    .card-container.light .clan-card-header:hover {
        background: #f8f9fa;
    }

    .collapse-icon {
        margin-left: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #72767d;
        transition: transform 0.3s ease;
    }

    .collapse-icon.is-collapsed {
        transform: rotate(-90deg);
    }

    /* Banner styles */
    .clan-banner-container {
        position: relative;
        width: 100%;
        height: 120px;
        cursor: pointer;
        overflow: hidden;
    }

    .clan-banner {
        width: 100%;
        height: 100%;
        object-fit: cover;
        transition: transform 0.5s ease;
    }

    .banner-overlay {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.3s ease;
    }

    .banner-overlay span {
        color: white;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 1px;
        font-size: 0.8rem;
        padding: 8px 16px;
        border: 1px solid rgba(255, 255, 255, 0.5);
        border-radius: 4px;
        background: rgba(0, 0, 0, 0.2);
    }

    .clan-banner-container:hover .clan-banner {
        transform: scale(1.05);
    }

    .clan-banner-container:hover .banner-overlay {
        opacity: 1;
    }

    .card-container.light .clan-card-header {
        background: linear-gradient(180deg, #f2f3f5 0%, #e9ecef 100%);
        border-bottom: 1px solid #e3e5e8;
    }

    .clan-badge {
        width: 48px;
        height: 48px;
        object-fit: contain;
        flex-shrink: 0;
    }

    .clan-titles {
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .clan-name {
        margin: 0;
        font-size: 1.1rem;
        color: #fff;
        font-weight: 700;
        transition: color 0.4s ease;
    }

    .card-container.light .clan-name {
        color: #1a1a2e;
    }

    .clan-tag {
        font-size: 0.75rem;
        color: #72767d;
        font-family: monospace;
        transition: color 0.4s ease;
    }

    .card-container.light .clan-tag {
        color: #5c6470;
    }

    .member-count-badge {
        margin-left: auto;
        font-size: 0.75rem;
        color: #b9bbbe;
        background: rgba(0, 0, 0, 0.3);
        padding: 4px 10px;
        border-radius: 12px;
        transition: all 0.4s ease;
    }

    .card-container.light .member-count-badge {
        background: rgba(88, 101, 242, 0.15);
        color: #5865f2;
    }

    .clan-card-body {
        padding: 16px;
        flex-grow: 1;
        max-height: 600px;
        overflow-y: auto;
        transition: background 0.4s ease;
        scrollbar-gutter: stable;
    }

    .card-container.light .clan-card-body {
        background: #ffffff;
    }

    .loading-members,
    .empty-members,
    .member-error {
        padding: 24px;
        text-align: center;
        color: #b9bbbe;
        font-size: 0.9rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        transition: color 0.4s ease;
    }

    .card-container.light .loading-members,
    .card-container.light .empty-members {
        color: #5c6470;
    }

    .member-error {
        color: #ed4245;
    }

    .roles-flow {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .role-section {
        background: rgba(0, 0, 0, 0.2);
        border-radius: 8px;
        padding: 12px;
        transition: all 0.3s ease;
        border: 1px solid rgba(255, 255, 255, 0.03);
    }

    .role-section:hover {
        background: rgba(0, 0, 0, 0.25);
        border-color: rgba(255, 255, 255, 0.08);
    }

    .card-container.light .role-section {
        background: rgba(88, 101, 242, 0.05);
        border: 1px solid rgba(88, 101, 242, 0.1);
    }

    .role-header {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 10px;
        padding-bottom: 8px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        transition: border-color 0.4s ease;
    }

    .card-container.light .role-header {
        border-bottom: 1px solid rgba(0, 0, 0, 0.08);
    }

    .role-dot {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        flex-shrink: 0;
    }

    .role-title {
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 1px;
        font-weight: 700;
        color: #b9bbbe;
        transition: color 0.4s ease;
    }

    .card-container.light .role-title {
        color: #4f5660;
    }

    .role-count {
        margin-left: auto;
        font-size: 0.75rem;
        color: #b9bbbe;
        background: #202225;
        padding: 2px 10px;
        border-radius: 12px;
        transition: all 0.4s ease;
    }

    .card-container.light .role-count {
        background: rgba(0, 0, 0, 0.06);
        color: #5c6470;
    }

    .members-list {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
        gap: 8px;
    }

    .member-item {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 4px 8px;
        background: rgba(180, 180, 180, 0.1);
        border-radius: 6px;
        transition: all 0.2s ease;
    }

    .card-container.light .member-item {
        background: rgba(0, 0, 0, 0.04);
    }

    .member-item:hover {
        background: #40444b;
        transform: translateX(4px);
    }

    .card-container.light .member-item:hover {
        background: #e3e5e8;
    }

    .member-avatar {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        background: #202225;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #b9bbbe;
        font-weight: 600;
        font-size: 0.7rem;
        flex-shrink: 0;
        border: 2px solid;
        transition: background 0.4s ease;
    }

    .card-container.light .member-avatar {
        background: #ffffff;
        color: #4f5660;
    }

    .member-name {
        color: #dcddde;
        font-size: 0.85rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        flex: 1;
        transition: color 0.4s ease;
        text-shadow: 0.3px 0.3px 0.3px rgba(0, 0, 0, 0.8);
    }

    .card-container.light .member-name {
        color: #2e3338;
        text-shadow: 0.5px 0.5px 0px rgba(255, 255, 255, 0.8);
    }

    .spinner,
    .mini-spinner {
        border: 2px solid rgba(255, 255, 255, 0.1);
        border-left-color: #5865f2;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    .card-container.light .spinner,
    .card-container.light .mini-spinner {
        border: 2px solid rgba(88, 101, 242, 0.2);
        border-left-color: #5865f2;
    }

    .spinner {
        width: 32px;
        height: 32px;
        border-width: 3px;
    }

    .mini-spinner {
        width: 20px;
        height: 20px;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    /* Custom Scrollbar */
    .clan-card-body::-webkit-scrollbar {
        width: 6px;
    }
    .clan-card-body::-webkit-scrollbar-track {
        background: transparent;
    }
    .clan-card-body::-webkit-scrollbar-thumb {
        background: #18191c;
        border-radius: 3px;
    }

    .card-container.light .clan-card-body::-webkit-scrollbar-thumb {
        background: #c3cfe2;
    }

    /* Responsive adjustments */
    @media (max-width: 900px) {
        .clans-grid {
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        }
    }

    @media (max-width: 600px) {
        .clans-grid {
            grid-template-columns: 1fr;
        }
    }
</style>
