<script lang="ts">
    import { onMount, createEventDispatcher } from 'svelte';
    import ClansSection from './ClansSection.svelte';
    import { user } from './auth';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string;

    const dispatch = createEventDispatcher<{ navigate: string }>();

    interface Clan {
        tag: string;
        nameDB: string;
        index: number;
        badgeUrl: string;
    }

    let userClans: { coc: Clan[], cr: Clan[] } = { coc: [], cr: [] };
    let loading = true;
    let error: string | null = null;

    async function loadMyClans() {
        if (!$user) {
            loading = false;
            return;
        }

        loading = true;
        error = null;
        try {
            const [accountsRes, cocClansRes, crClansRes] = await Promise.all([
                fetch(`${apiBaseUrl}/api/me/accounts`, { credentials: 'include' }),
                fetch(`${apiBaseUrl}/api/coc/clans`),
                fetch(`${apiBaseUrl}/api/cr/clans`)
            ]);

            if (!accountsRes.ok || !cocClansRes.ok || !crClansRes.ok) {
                throw new Error('Fehler beim Laden der Daten');
            }

            const accounts = await accountsRes.json();
            const allCocClans: Clan[] = await cocClansRes.json();
            const allCrClans: Clan[] = await crClansRes.json();

            const officialCocTags = new Set(allCocClans.map(c => c.tag));
            const officialCrTags = new Set(allCrClans.map(c => c.tag));

            const cocAccountClans = new Set<string>();
            const cocAccounts = accounts.coc || (Array.isArray(accounts) ? accounts : []);
            cocAccounts.forEach((acc: any) => {
                if (acc.clan && officialCocTags.has(acc.clan.tag)) {
                    cocAccountClans.add(acc.clan.tag);
                }
            });

            const crAccountClans = new Set<string>();
            const crAccounts = accounts.cr || [];
            crAccounts.forEach((acc: any) => {
                if (acc.clan && officialCrTags.has(acc.clan.tag)) {
                    crAccountClans.add(acc.clan.tag);
                }
            });

            userClans = {
                coc: allCocClans.filter(c => cocAccountClans.has(c.tag)),
                cr: allCrClans.filter(c => crAccountClans.has(c.tag))
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

    // CoC groupings for my clans
    $: f2pClans = userClans.coc.filter((clan) =>
        (clan.nameDB || '').toUpperCase().includes('F2P')
    );

    $: extraClans = userClans.coc.filter((clan) => {
        const name = (clan.nameDB || '').toUpperCase();
        return name.includes('GP') || name.includes('ANTHRAZIT');
    });

    $: normalCocClans = userClans.coc.filter(
        (clan) => !f2pClans.includes(clan) && !extraClans.includes(clan)
    );
</script>

<div class="my-clans-page" class:light={theme === 'light'}>
    <div class="page-header">
        <h1>Deine Clans</h1>
        <p>Hier siehst du alle Clans der LOST Family, in denen du Mitglied bist.</p>
    </div>

    {#if loading}
        <div class="loading-state">
            <div class="spinner"></div>
            <p>Deine Clans werden geladen...</p>
        </div>
    {:else if error}
        <div class="error-state">
            <p>Fehler beim Laden deiner Clans: {error}</p>
            <button class="retry-btn" on:click={loadMyClans}>Erneut versuchen</button>
        </div>
    {:else if userClans.coc.length === 0 && userClans.cr.length === 0}
        <div class="empty-state">
            <div class="empty-icon">🏰</div>
            <h3>Keine Clans gefunden</h3>
            <p>Du bist derzeit in keinem Clan der LOST Family.</p>
            <button class="action-btn" on:click={() => dispatch('navigate', 'coc/clans')}>
                Clans entdecken
            </button>
        </div>
    {:else}
        {#if userClans.coc.length > 0}
            <div class="game-section">
                {#if f2pClans.length > 0}
                    <ClansSection
                        {theme}
                        {apiBaseUrl}
                        gameType="coc"
                        title="Deine F2P Clans"
                        description=""
                        clansData={f2pClans}
                    />
                {/if}

                {#if extraClans.length > 0}
                    <ClansSection
                        {theme}
                        {apiBaseUrl}
                        gameType="coc"
                        title="Deine Extra Clans"
                        description=""
                        clansData={extraClans}
                    />
                {/if}

                {#if normalCocClans.length > 0}
                    <ClansSection
                        {theme}
                        {apiBaseUrl}
                        gameType="coc"
                        title="Deine CoC Clans"
                        description=""
                        clansData={normalCocClans}
                    />
                {/if}
            </div>
        {/if}

        {#if userClans.cr.length > 0}
            <div class="game-section">
                <ClansSection
                    {theme}
                    {apiBaseUrl}
                    gameType="cr"
                    title="Deine Clash Royale Clans"
                    description=""
                    clansData={userClans.cr}
                />
            </div>
        {/if}
    {/if}
</div>

<style>
    .my-clans-page {
        min-height: 100vh;
        padding-top: 4rem;
        padding-bottom: 4rem;
        max-width: 1200px;
        margin: 0 auto;
        padding-left: 1rem;
        padding-right: 1rem;
    }

    .page-header {
        text-align: center;
        margin-bottom: 4rem;
    }

    .page-header h1 {
        font-size: 3rem;
        font-weight: 800;
        margin-bottom: 1rem;
        background: linear-gradient(135deg, #fff 0%, #b9bbbe 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .light .page-header h1 {
        background: linear-gradient(135deg, #2e3338 0%, #4f5660 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .page-header p {
        color: #b9bbbe;
        font-size: 1.25rem;
    }

    .light .page-header p {
        color: #4f5660;
    }

    .loading-state,
    .error-state,
    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        min-height: 400px;
        color: #b9bbbe;
        text-align: center;
    }

    .light .loading-state,
    .light .error-state,
    .light .empty-state {
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

    .retry-btn, .action-btn {
        margin-top: 1.5rem;
        padding: 0.75rem 1.5rem;
        background: #5865f2;
        color: white;
        border: none;
        border-radius: 8px;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.2s;
    }

    .retry-btn:hover, .action-btn:hover {
        background: #4752c4;
    }

    .empty-icon {
        font-size: 4rem;
        margin-bottom: 1.5rem;
    }

    .empty-state h3 {
        font-size: 2rem;
        margin-bottom: 0.5rem;
        color: white;
    }

    .light .empty-state h3 {
        color: #2e3338;
    }

    .game-section {
        margin-bottom: 4rem;
    }
</style>
