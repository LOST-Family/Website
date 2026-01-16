<script lang="ts">
    import { onMount, createEventDispatcher } from 'svelte';
    import ClansSection from './ClansSection.svelte';
    import ClanDetailPage from './ClanDetailPage.svelte';
    import CRClanDetailPage from './CRClanDetailPage.svelte';
    import type { GameType } from './auth';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string;
    export let clanTag: string | null = null;
    export let gameType: GameType = 'coc';
    export let backPath: string =
        gameType === 'coc' ? '/coc/clans' : '/cr/clans';

    const dispatch = createEventDispatcher<{ navigate: string }>();

    interface Clan {
        tag: string;
        nameDB: string;
        index: number;
        badgeUrl: string;
    }

    let allClans: Clan[] = [];
    let loading = true;
    let error: string | null = null;

    $: apiPrefix = gameType === 'coc' ? '/api/coc' : '/api/cr';
    $: gameName = gameType === 'coc' ? 'Clash of Clans' : 'Clash Royale';

    $: filteredClans = clanTag
        ? allClans.filter((c) => c.tag === clanTag)
        : allClans;

    // CoC specific clan groupings
    $: f2pClans =
        gameType === 'coc'
            ? filteredClans.filter((clan) =>
                  (clan.nameDB || '').toUpperCase().includes('F2P')
              )
            : [];

    $: extraClans =
        gameType === 'coc'
            ? filteredClans.filter((clan) => {
                  const name = (clan.nameDB || '').toUpperCase();
                  return name.includes('GP') || name.includes('ANTHRAZIT');
              })
            : [];

    $: normalClans =
        gameType === 'coc'
            ? filteredClans.filter(
                  (clan) =>
                      !f2pClans.includes(clan) && !extraClans.includes(clan)
              )
            : filteredClans;

    async function loadClans() {
        loading = true;
        error = null;
        try {
            const response = await fetch(`${apiBaseUrl}${apiPrefix}/clans`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            const data = await response.json();
            allClans = Array.isArray(data)
                ? data.sort((a, b) => (a.index || 0) - (b.index || 0))
                : [];
        } catch (e) {
            error = e instanceof Error ? e.message : 'Unknown error';
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadClans();
    });

    // Reload when game type changes
    $: if (apiPrefix) {
        loadClans();
    }
</script>

<div class="clans-page" class:light={theme === 'light'}>
    {#if loading}
        <div class="loading-state">
            <div class="spinner"></div>
            <p>Clans werden geladen...</p>
        </div>
    {:else if error}
        <div class="error-state">
            <p>Fehler beim Laden der Clans: {error}</p>
            <button class="retry-btn" on:click={loadClans}
                >Erneut versuchen</button
            >
        </div>
    {:else if clanTag}
        {#if gameType === 'coc'}
            <ClanDetailPage
                {theme}
                {apiBaseUrl}
                {clanTag}
                {backPath}
                on:navigate
            />
        {:else}
            <CRClanDetailPage
                {theme}
                {apiBaseUrl}
                {clanTag}
                {backPath}
                on:navigate
            />
        {/if}
    {:else if gameType === 'coc'}
        {#if f2pClans.length > 0}
            <ClansSection
                {theme}
                {apiBaseUrl}
                gameType="coc"
                title="F2P Clans"
                description="Unsere Free-to-Play Clans in Clash of Clans. Free-to-Play heißt, dass kein Cent in einen Account investiert wurde."
                clansData={f2pClans}
            />
        {/if}

        {#if extraClans.length > 0}
            <ClansSection
                {theme}
                {apiBaseUrl}
                gameType="coc"
                title="Extra Clans"
                description="LOST GP ist eine option zwischen einem F2P Clan und einem normalen Clan. Hier sind Goldpässe und Eventpässe erlaubt. Anthrazit dient hauptsächlich für 2. Accounts"
                clansData={extraClans}
            />
        {/if}

        {#if normalClans.length > 0}
            <ClansSection
                {theme}
                {apiBaseUrl}
                gameType="coc"
                title="Normale Clans"
                description="In diesen Clans sind alle Spieler willkommen. LOST 3 ist unser Push-Clan und versucht jede Saison Spitzenergebnisse zu erzielen."
                clansData={normalClans}
            />
        {/if}
    {:else}
        <!-- Clash Royale clans -->
        {#if normalClans.length > 0}
            <ClansSection
                {theme}
                {apiBaseUrl}
                gameType="cr"
                title="Clash Royale Clans"
                description="Unsere Clans in Clash Royale. Tritt einem Clan bei und kämpfe gemeinsam!"
                clansData={normalClans}
            />
        {:else}
            <div class="empty-state">
                <div class="empty-icon">🏰</div>
                <h3>Keine Clash Royale Clans</h3>
                <p>Derzeit sind keine Clash Royale Clans verfügbar.</p>
            </div>
        {/if}
    {/if}
</div>

<style>
    .clans-page {
        min-height: 100vh;
        padding-top: 2rem;
        padding-bottom: 4rem;
    }

    .loading-state,
    .error-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        min-height: 400px;
        color: #b9bbbe;
    }

    .clans-page.light .loading-state,
    .clans-page.light .error-state {
        color: #4f5660;
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid rgba(88, 101, 242, 0.1);
        border-left-color: #5865f2;
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin-bottom: 1rem;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .retry-btn {
        margin-top: 1rem;
        padding: 0.75rem 1.5rem;
        background: #5865f2;
        color: white;
        border: none;
        border-radius: 8px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
    }

    .retry-btn:hover {
        background: #4752c4;
        transform: translateY(-2px);
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        min-height: 300px;
        text-align: center;
        padding: 2rem;
    }

    .empty-icon {
        font-size: 4rem;
        margin-bottom: 1rem;
    }

    .empty-state h3 {
        color: #fff;
        font-size: 1.5rem;
        margin-bottom: 0.5rem;
    }

    .clans-page.light .empty-state h3 {
        color: #1e293b;
    }

    .empty-state p {
        color: #b9bbbe;
    }

    .clans-page.light .empty-state p {
        color: #64748b;
    }
</style>
