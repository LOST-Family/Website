<script lang="ts">
    import { onMount } from 'svelte';
    import ClansSection from './ClansSection.svelte';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string;

    interface Clan {
        tag: string;
        nameDB: string;
        index: number;
        badgeUrl: string;
    }

    let allClans: Clan[] = [];
    let loading = true;
    let error: string | null = null;

    $: f2pClans = allClans.filter((clan) =>
        (clan.nameDB || '').toUpperCase().includes('F2P')
    );

    $: extraClans = allClans.filter((clan) => {
        const name = (clan.nameDB || '').toUpperCase();
        return name.includes('GP') || name.includes('ANTHRAZIT');
    });

    $: normalClans = allClans.filter(
        (clan) => !f2pClans.includes(clan) && !extraClans.includes(clan)
    );

    onMount(async () => {
        try {
            const response = await fetch(`${apiBaseUrl}/api/clans`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            allClans = await response.json();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Unknown error';
        } finally {
            loading = false;
        }
    });
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
        </div>
    {:else}
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
                description="LOST GP ist eine option zwischen F2P und einem normalen Clan. Hier sind Goldpässe und Eventpässe erlaubt. Anthrazit dient hauptsächlich für 2. Accounts"
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
</style>
