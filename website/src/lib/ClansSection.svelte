<script lang="ts">
    import ClanCard from './ClanCard.svelte';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string;
    export let gameType: 'coc' | 'cr';
    export let title: string;
    export let description: string;
    export let clansData: any[] | null = null;

    let clanCount = 0;
    let playerCount = 0;

    const isCoc = gameType === 'coc';
</script>

<section
    class="content-section"
    id="{gameType}-clans"
    class:light={theme === 'light'}
>
    <div class="section-container full-width">
        <div class="section-header with-stats">
            <div class="section-header-left">
                <div
                    class="section-icon clan-icon"
                    class:coc={isCoc}
                    class:cr={!isCoc}
                >
                    {#if isCoc}
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path
                                d="M12 2L2 7l10 5 10-5-10-5zm0 9l2.5-1.25L12 8.5l-2.5 1.25L12 11zm0 2.5l-5-2.5-5 2.5L12 22l10-8.5-5-2.5-5 2.5z"
                            />
                        </svg>
                    {:else}
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path
                                d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"
                            />
                        </svg>
                    {/if}
                </div>
                <div class="section-title-group">
                    <h2 class="section-title">{title}</h2>
                    <p class="section-description">{description}</p>
                </div>
            </div>
            {#if clanCount > 0}
                <div class="section-stats">
                    <span class="stat-badge" class:coc={isCoc} class:cr={!isCoc}
                        >{clanCount} Clans</span
                    >
                    <span class="stat-badge" class:coc={isCoc} class:cr={!isCoc}
                        >{playerCount} Mitglieder</span
                    >
                </div>
            {/if}
        </div>

        <ClanCard
            {apiBaseUrl}
            {theme}
            {clansData}
            bind:clanCount
            bind:playerCount
        />
    </div>
</section>

<style>
    .content-section {
        padding: 4rem 0;
        position: relative;
    }

    .content-section::before {
        content: '';
        position: absolute;
        top: 0;
        left: 50%;
        transform: translateX(-50%);
        width: 120px;
        height: 1px;
        background: linear-gradient(
            90deg,
            transparent,
            rgba(88, 101, 242, 0.5),
            transparent
        );
    }

    .section-container.full-width {
        max-width: none;
        padding: 0 2rem;
    }

    .section-header {
        display: flex;
        align-items: center;
        gap: 1rem;
        margin-bottom: 2.5rem;
        max-width: 1400px;
        margin-left: auto;
        margin-right: auto;
    }

    .section-header.with-stats {
        flex-wrap: wrap;
    }

    .section-header-left {
        display: flex;
        align-items: center;
        gap: 1rem;
        min-width: 0;
        flex: 1;
    }

    .section-stats {
        display: flex;
        gap: 12px;
        margin-left: 2rem;
        flex-shrink: 0;
    }

    .stat-badge {
        padding: 8px 18px;
        border-radius: 20px;
        font-size: 0.875rem;
        font-weight: 600;
        transition: all 0.4s ease;
    }

    .stat-badge.coc {
        background: rgba(59, 165, 92, 0.15);
        color: #3ba55c;
        border: 1px solid rgba(59, 165, 92, 0.2);
    }

    .stat-badge.cr {
        background: rgba(88, 101, 242, 0.15);
        color: #5865f2;
        border: 1px solid rgba(88, 101, 242, 0.2);
    }

    .content-section.light .stat-badge.coc {
        background: rgba(59, 165, 92, 0.1);
        color: #2d8049;
        border: 1px solid rgba(59, 165, 92, 0.15);
    }

    .content-section.light .stat-badge.cr {
        background: rgba(88, 101, 242, 0.1);
        color: #4752c4;
        border: 1px solid rgba(88, 101, 242, 0.15);
    }

    .section-icon {
        width: 48px;
        height: 48px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        flex-shrink: 0;
    }

    .section-icon.clan-icon.coc {
        background: linear-gradient(135deg, #3ba55c 0%, #2d8049 100%);
        box-shadow: 0 8px 24px rgba(59, 165, 92, 0.3);
    }

    .section-icon.clan-icon.cr {
        background: linear-gradient(135deg, #5865f2 0%, #4752c4 100%);
        box-shadow: 0 8px 24px rgba(88, 101, 242, 0.3);
    }

    .section-icon svg {
        width: 24px;
        height: 24px;
    }

    .section-title-group {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        min-width: 0;
    }

    .section-title {
        margin: 0;
        font-size: 1.75rem;
        font-weight: 700;
        color: #fff;
        transition: color 0.4s ease;
    }

    .content-section.light .section-title {
        color: #0f172a;
    }

    .section-description {
        margin: 0;
        font-size: 0.95rem;
        color: rgba(255, 255, 255, 0.5);
        transition: color 0.4s ease;
        line-height: 1.5;
        max-width: 800px;
    }

    .content-section.light .section-description {
        color: #64748b;
    }

    @media (max-width: 800px) {
        .content-section {
            padding: 3rem 0;
        }

        .section-container.full-width {
            padding: 0 1rem;
        }

        .section-header {
            padding: 0 1rem;
        }

        .section-header.with-stats {
            flex-direction: row;
            flex-wrap: wrap;
            align-items: center;
        }

        .section-header-left {
            flex-direction: column;
            align-items: flex-start;
            gap: 0.75rem;
        }

        .section-stats {
            margin-left: 0;
            margin-top: 0.5rem;
            width: 100%;
            justify-content: flex-start;
        }

        .stat-badge {
            padding: 6px 14px;
            font-size: 0.8rem;
        }

        .section-title {
            font-size: 1.5rem;
        }
    }
</style>
