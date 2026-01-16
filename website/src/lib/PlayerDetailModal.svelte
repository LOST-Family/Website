<script lang="ts">
    import { fade, scale, slide } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
    import type { GameType } from './auth';

    export let player: any;
    export let gameType: GameType;
    export let theme: 'dark' | 'light' = 'dark';
    export let isOpen: boolean = false;
    export let onClose: () => void;
    export let otherAccounts: any[] = [];
    export let hasPrivilegedAccess: boolean = false;
    export let isAdmin: boolean = false;
    export let onNavigateToProfile: ((userId: string) => void) | null = null;
    export let onSelectOtherAccount: ((acc: any) => void) | null = null;

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') onClose();
    }
</script>

{#if isOpen && player}
    <div
        class="modal-backdrop"
        on:click|self={onClose}
        on:keydown={handleKeydown}
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
                on:click={onClose}
                aria-label="Schließen"
                title="Schließen"
            >
                <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="3.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path d="M18 6L6 18" />
                    <path d="M6 6l12 12" />
                </svg>
            </button>

            <div class="modal-scroll-area">
                <div class="modal-header">
                    <div class="player-info-large">
                        {#if gameType === 'coc'}
                            <img
                                src={player.leagueTier?.iconUrls?.large ||
                                    player.league?.iconUrls?.large ||
                                    player.league?.iconUrls?.medium}
                                alt={player.leagueTier?.name ||
                                    player.league?.name}
                                class="league-icon-large"
                            />
                        {:else if player.arena}
                            <div class="arena-icon-container">
                                <div class="arena-id-label">
                                    A{player.arena.id.toString().slice(-2)}
                                </div>
                            </div>
                        {/if}
                        <div class="player-titles">
                            <div class="title-with-link">
                                <h2>
                                    {player.nameDB || player.name}
                                </h2>
                                {#if player.userId && onNavigateToProfile && isAdmin}
                                    <button
                                        class="view-profile-btn"
                                        on:click={() =>
                                            onNavigateToProfile(player.userId)}
                                        title="Profil ansehen"
                                    >
                                        <svg
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2"
                                        >
                                            <path
                                                d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14L21 3"
                                            />
                                        </svg>
                                    </button>
                                {/if}
                            </div>
                            <p class="tag">{player.tag}</p>
                            {#if player.clan}
                                <div class="clan-info-small">
                                    <img
                                        src={player.clan.badgeUrls?.small ||
                                            player.clan.badgeUrls?.medium}
                                        alt=""
                                    />
                                    <span>{player.clan.name}</span>
                                </div>
                            {/if}
                        </div>
                    </div>
                    <div class="th-display">
                        {#if gameType === 'coc'}
                            <div class="th-level">
                                TH {player.townHallLevel}
                            </div>
                            {#if player.builderHallLevel}
                                <div class="bh-level">
                                    BH {player.builderHallLevel}
                                </div>
                            {/if}
                        {:else}
                            <div class="th-level cr-level">
                                {player.arena?.name ||
                                    `Arena ${player.arena?.id || '?'}`}
                            </div>
                        {/if}
                    </div>
                </div>

                <div class="modal-body">
                    <div class="stats-grid-large">
                        {#if gameType === 'coc'}
                            <div class="stat-card">
                                <span class="label">EXP Level</span>
                                <span class="value">{player.expLevel}</span>
                            </div>
                            <div class="stat-card">
                                <span class="label">Kriegssterne</span>
                                <span class="value">{player.warStars || 0}</span
                                >
                            </div>
                            <div class="stat-card">
                                <span class="label">Spenden</span>
                                <span class="value"
                                    >{player.donations || 0}</span
                                >
                            </div>
                            <div class="stat-card">
                                <span class="label">Erhaltene Spenden</span>
                                <span class="value"
                                    >{player.donationsReceived || 0}</span
                                >
                            </div>
                        {:else}
                            <div class="stat-card">
                                <span class="label">EXP Level</span>
                                <span class="value">{player.expLevel}</span>
                            </div>
                            <div class="stat-card">
                                <span class="label">Total Wins</span>
                                <span class="value">{player.wins || 0}</span>
                            </div>
                            <div class="stat-card">
                                <span class="label">Donations</span>
                                <span class="value"
                                    >{player.donations || 0}</span
                                >
                            </div>
                            <div class="stat-card">
                                <span class="label">Trophies</span>
                                <span class="value">{player.trophies || 0}</span
                                >
                            </div>
                        {/if}
                        {#if hasPrivilegedAccess}
                            <div class="stat-card">
                                <span class="label">Kickpunkte</span>
                                <span
                                    class="value {(player.activeKickpointsSum ||
                                        0) >= 10
                                        ? 'danger'
                                        : ''}"
                                >
                                    {player.activeKickpointsSum || 0}
                                </span>
                            </div>
                            <div class="stat-card">
                                <span class="label"
                                    >Gesamtanzahl Kickpunkte</span
                                >
                                <span class="value">
                                    {player.totalKickpoints || 0}
                                </span>
                            </div>
                        {/if}
                    </div>

                    <!-- Other Accounts -->
                    {#if player.playerAccounts && player.playerAccounts.length > 1}
                        <div
                            class="detail-section"
                            in:slide={{ duration: 300, delay: 300 }}
                        >
                            <h3>
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    width="20"
                                    height="20"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    ><path
                                        d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"
                                    /><circle cx="9" cy="7" r="4" /><path
                                        d="M22 21v-2a4 4 0 0 0-3-3.87"
                                    /><path
                                        d="M16 3.13a4 4 0 0 1 0 7.75"
                                    /></svg
                                >
                                Other Linked Accounts
                            </h3>
                            <div class="other-accounts-grid">
                                {#each player.playerAccounts as acc}
                                    {#if acc.tag !== player.tag}
                                        <button
                                            class="other-account-btn"
                                            on:click={() => {
                                                onClose();
                                                window.location.href = `/clash/player/${acc.tag.replace('#', '')}`;
                                            }}
                                        >
                                            <span class="acc-name"
                                                >{acc.name || acc.tag}</span
                                            >
                                            <span class="acc-tag"
                                                >{acc.tag}</span
                                            >
                                        </button>
                                    {/if}
                                {/each}
                            </div>
                        </div>
                    {/if}

                    <!-- Kickpoints History -->
                    {#if hasPrivilegedAccess && ((player.kickpoints && player.kickpoints.length > 0) || (player.activeKickpoints && player.activeKickpoints.length > 0))}
                        <div
                            class="detail-section"
                            in:slide={{ duration: 300, delay: 400 }}
                        >
                            <h3>Kickpunkte</h3>
                            <div class="kickpoints-list">
                                {#each player.kickpoints || player.activeKickpoints || [] as kp}
                                    <div class="kp-item">
                                        <div class="kp-header">
                                            <span class="kp-reason"
                                                >{kp.reason}</span
                                            >
                                            <span
                                                class="kp-amount {kp.amount > 0
                                                    ? 'positive'
                                                    : 'negative'}"
                                            >
                                                {kp.amount > 0
                                                    ? '+'
                                                    : ''}{kp.amount}
                                            </span>
                                        </div>
                                        <div class="kp-footer">
                                            <span class="kp-date"
                                                >{new Date(
                                                    kp.date || kp.createdAt
                                                ).toLocaleDateString()}</span
                                            >
                                            {#if kp.description}
                                                <span class="kp-desc"
                                                    >{kp.description}</span
                                                >
                                            {/if}
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    {#if gameType === 'cr' && player.currentDeck}
                        <div class="detail-section">
                            <h3>Aktuelles Deck</h3>
                            <div class="deck-grid">
                                {#each player.currentDeck as card}
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

                    {#if gameType === 'cr' && player.badges && player.badges.length > 0}
                        <div class="detail-section">
                            <h3>Abzeichen</h3>
                            <div class="badges-grid-cr">
                                {#each player.badges as badge}
                                    <div
                                        class="badge-item-cr"
                                        title={badge.name}
                                    >
                                        <img
                                            src={badge.iconUrls?.large}
                                            alt={badge.name}
                                        />
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    {#if player.heroes && player.heroes.length > 0}
                        <div class="detail-section">
                            <h3>Helden</h3>
                            <div class="items-grid">
                                {#each player.heroes as hero}
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

                    {#if player.labels && player.labels.length > 0}
                        <div class="detail-section">
                            <h3>Labels</h3>
                            <div class="labels-list">
                                {#each player.labels as label}
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

                    {#if player.troops && player.troops.length > 0}
                        <div class="detail-section">
                            <h3>Truppen</h3>
                            <div class="items-grid">
                                {#each player.troops.filter((t: any) => !t.name.startsWith('Super ') && !['Sneaky Goblin', 'Rocket Balloon', 'Inferno Dragon', 'Ice Hound'].includes(t.name)) as troop}
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

                    {#if otherAccounts && otherAccounts.length > 0}
                        <div class="detail-section">
                            <h3>Weitere Accounts</h3>
                            <div class="other-accounts-list">
                                {#each otherAccounts as acc}
                                    <button
                                        class="other-acc-card"
                                        on:click={() =>
                                            onSelectOtherAccount?.(acc)}
                                    >
                                        <img
                                            src={acc.clan?.badgeUrls?.small ||
                                                ''}
                                            alt=""
                                            class="acc-badge"
                                        />
                                        <div class="acc-info">
                                            <div class="acc-name">
                                                {acc.nameDB || acc.name}
                                            </div>
                                            <div class="acc-tag">{acc.tag}</div>
                                        </div>
                                    </button>
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
    /* Modal Styles */
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.85);
        backdrop-filter: blur(12px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 2000;
        padding: 1rem;
    }

    .modal-content {
        background: #111827;
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 28px;
        width: 100%;
        max-width: 800px;
        max-height: 90vh;
        position: relative;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7);
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .modal-content.light {
        background: white;
        border-color: rgba(0, 0, 0, 0.1);
        color: #1f2937;
    }

    .modal-scroll-area {
        overflow-y: auto;
        flex-grow: 1;
        scrollbar-width: thin;
        scrollbar-color: rgba(59, 130, 246, 0.3) transparent;
    }

    .modal-scroll-area::-webkit-scrollbar {
        width: 6px;
    }

    .modal-scroll-area::-webkit-scrollbar-thumb {
        background-color: rgba(59, 130, 246, 0.3);
        border-radius: 10px;
    }

    .close-modal {
        --btn-size: 40px;
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        width: var(--btn-size);
        height: var(--btn-size);
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.2rem;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 20px;
        color: rgba(255, 255, 255, 0.6);
        cursor: pointer;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        z-index: 100;
        backdrop-filter: blur(12px);
    }

    .light .close-modal {
        background: rgba(0, 0, 0, 0.04);
        border-color: rgba(0, 0, 0, 0.08);
        color: rgba(0, 0, 0, 0.5);
    }

    .close-modal:hover {
        background: rgba(239, 68, 68, 0.15);
        border-color: rgba(239, 68, 68, 0.4);
        color: #ef4444;
        transform: scale(1.05);
        border-radius: 50%;
    }

    .close-modal:active {
        transform: scale(0.92);
    }

    .light .close-modal:hover {
        background: rgba(239, 68, 68, 0.1);
        border-color: rgba(239, 68, 68, 0.3);
        color: #ef4444;
    }

    .close-modal svg {
        width: 54px;
        height: 54px;
        transition: transform 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    }

    .close-modal:hover svg {
        transform: rotate(90deg);
    }

    .modal-header {
        padding: 3rem 3rem 2rem;
        background: linear-gradient(
            to bottom,
            rgba(59, 130, 246, 0.1),
            transparent
        );
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: 2rem;
    }

    .player-info-large {
        display: flex;
        align-items: center;
        gap: 2rem;
    }

    .league-icon-large {
        width: 100px;
        height: 100px;
        filter: drop-shadow(0 0 20px rgba(59, 130, 246, 0.4));
    }

    .player-titles {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .title-with-link {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .player-titles h2 {
        margin: 0;
        font-size: 2.25rem;
        font-weight: 800;
        letter-spacing: -0.02em;
    }

    .view-profile-btn {
        background: rgba(59, 130, 246, 0.1);
        border: 1px solid rgba(59, 130, 246, 0.2);
        color: #3b82f6;
        width: 32px;
        height: 32px;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s;
        padding: 0;
    }

    .view-profile-btn:hover {
        background: #3b82f6;
        color: white;
    }

    .view-profile-btn svg {
        width: 18px;
        height: 18px;
    }

    .player-titles .tag {
        margin: 0;
        font-family: 'JetBrains Mono', monospace;
        color: #3b82f6;
        font-weight: 600;
        font-size: 1.1rem;
    }

    .clan-info-small {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-top: 0.75rem;
        font-size: 1rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.8);
        background: rgba(255, 255, 255, 0.05);
        padding: 0.4rem 0.8rem;
        border-radius: 12px;
        width: fit-content;
    }

    .light .clan-info-small {
        background: rgba(0, 0, 0, 0.05);
        color: rgba(0, 0, 0, 0.8);
    }

    .clan-info-small img {
        width: 24px;
        height: 24px;
    }

    .th-display {
        text-align: right;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .th-level,
    .bh-level {
        background: #3b82f6;
        color: white;
        padding: 0.6rem 1.25rem;
        border-radius: 14px;
        font-weight: 800;
        font-size: 1rem;
        box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
    }

    .bh-level {
        background: #8b5cf6;
        box-shadow: 0 4px 12px rgba(139, 92, 246, 0.3);
    }

    .cr-level {
        background: linear-gradient(135deg, #5865f2, #7289da);
        box-shadow: 0 4px 12px rgba(88, 101, 242, 0.3);
    }

    .modal-body {
        padding: 0 3rem 3rem;
    }

    .stats-grid-large {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 1.25rem;
        margin-bottom: 3rem;
    }

    .stat-card {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        padding: 1.5rem;
        border-radius: 20px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5rem;
        transition: all 0.3s ease;
    }

    .stat-card:hover {
        background: rgba(255, 255, 255, 0.06);
        border-color: rgba(59, 130, 246, 0.3);
        transform: translateY(-2px);
    }

    .light .stat-card {
        background: #f9fafb;
        border-color: rgba(0, 0, 0, 0.05);
    }

    .stat-card .label {
        font-size: 0.7rem;
        text-transform: uppercase;
        color: rgba(255, 255, 255, 0.4);
        font-weight: 700;
        letter-spacing: 0.1em;
    }

    .light .stat-card .label {
        color: rgba(0, 0, 0, 0.5);
    }

    .stat-card .value {
        font-size: 1.5rem;
        font-weight: 800;
        color: #3b82f6;
    }

    .detail-section {
        margin-top: 3rem;
    }

    .detail-section h3 {
        font-size: 1.25rem;
        font-weight: 800;
        margin-bottom: 1.5rem;
        display: flex;
        align-items: center;
        gap: 1rem;
        color: white;
    }

    .light .detail-section h3 {
        color: #111827;
    }

    .detail-section h3::after {
        content: '';
        flex-grow: 1;
        height: 1px;
        background: linear-gradient(
            to right,
            rgba(255, 255, 255, 0.1),
            transparent
        );
    }

    .light .detail-section h3::after {
        background: linear-gradient(to right, rgba(0, 0, 0, 0.1), transparent);
    }

    /* Kickpoints History */
    .kickpoints-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .kp-item {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        padding: 1rem;
        border-radius: 12px;
    }

    .light .kp-item {
        background: #f8fafc;
        border-color: rgba(0, 0, 0, 0.05);
    }

    .kp-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.25rem;
    }

    .kp-reason {
        font-weight: 700;
        font-size: 0.95rem;
    }

    .kp-amount {
        font-weight: 800;
        font-size: 0.9rem;
    }

    .kp-amount.positive {
        color: #10b981;
    }
    .kp-amount.negative {
        color: #ef4444;
    }

    .kp-footer {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .kp-date {
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.4);
        font-family: 'JetBrains Mono', monospace;
    }

    .light .kp-date {
        color: rgba(0, 0, 0, 0.4);
    }

    .kp-desc {
        font-size: 0.85rem;
        color: rgba(255, 255, 255, 0.7);
    }

    .light .kp-desc {
        color: rgba(0, 0, 0, 0.7);
    }

    /* Other Accounts */
    .other-accounts-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
        gap: 0.75rem;
    }

    .other-account-btn {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        padding: 0.75rem 1rem;
        border-radius: 12px;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 0.25rem;
        cursor: pointer;
        transition: all 0.2s;
        text-align: left;
        color: inherit;
    }

    .light .other-account-btn {
        background: #f8fafc;
        border-color: rgba(0, 0, 0, 0.05);
    }

    .other-account-btn:hover {
        background: rgba(59, 130, 246, 0.1);
        border-color: rgba(59, 130, 246, 0.3);
        transform: translateY(-2px);
    }

    .acc-name {
        font-weight: 700;
        font-size: 0.9rem;
    }

    .acc-tag {
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.4);
        font-family: 'JetBrains Mono', monospace;
    }

    .light .acc-tag {
        color: rgba(0, 0, 0, 0.4);
    }

    /* Items Grids */
    .items-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
        gap: 1rem;
    }

    .item-badge {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        padding: 1.25rem;
        border-radius: 16px;
        position: relative;
        overflow: hidden;
    }

    .item-name {
        display: block;
        font-size: 0.9rem;
        font-weight: 700;
        margin-bottom: 0.4rem;
    }

    .item-level {
        font-size: 0.85rem;
        color: rgba(255, 255, 255, 0.5);
        font-weight: 600;
    }

    .progress-bar {
        position: absolute;
        bottom: 0;
        left: 0;
        height: 4px;
        background: #3b82f6;
        box-shadow: 0 0 10px rgba(59, 130, 246, 0.5);
    }

    .other-accounts-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .other-acc-card {
        display: flex;
        align-items: center;
        gap: 1rem;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        padding: 1rem;
        border-radius: 16px;
        cursor: pointer;
        width: 100%;
        text-align: left;
        transition: all 0.2s;
    }

    .other-acc-card:hover {
        background: rgba(255, 255, 255, 0.06);
        border-color: rgba(59, 130, 246, 0.3);
        transform: scale(1.02);
    }

    .acc-badge {
        width: 32px;
        height: 32px;
    }

    .acc-info {
        flex-grow: 1;
    }

    .acc-name {
        font-weight: 700;
        font-size: 1rem;
    }

    .acc-tag {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.4);
        font-family: 'JetBrains Mono', monospace;
    }

    /* CR Specific */
    .arena-icon-container {
        width: 100px;
        height: 100px;
        background: rgba(88, 101, 242, 0.1);
        border: 2px solid rgba(88, 101, 242, 0.3);
        border-radius: 24px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .arena-id-label {
        font-size: 2rem;
        font-weight: 900;
        color: #5865f2;
        text-shadow: 0 0 20px rgba(88, 101, 242, 0.5);
    }

    .deck-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 1rem;
    }

    .card-item {
        background: rgba(255, 255, 255, 0.03);
        border-radius: 16px;
        padding: 0.75rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.75rem;
        transition: all 0.2s;
    }

    .card-item:hover {
        transform: translateY(-5px);
        background: rgba(255, 255, 255, 0.08);
    }

    .card-item img {
        width: 100%;
        height: auto;
        border-radius: 8px;
    }

    .card-level {
        font-size: 0.8rem;
        font-weight: 800;
        color: #fff;
        background: #5865f2;
        padding: 0.25rem 0.6rem;
        border-radius: 8px;
    }

    .badges-grid-cr {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(70px, 1fr));
        gap: 1.25rem;
    }

    .badge-item-cr {
        position: relative;
        aspect-ratio: 1;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 12px;
        padding: 0.5rem;
    }

    .badge-item-cr img {
        width: 100%;
        height: 100%;
        object-fit: contain;
    }

    .labels-list {
        display: flex;
        flex-wrap: wrap;
        gap: 1rem;
    }

    .label-badge {
        background: rgba(255, 255, 255, 0.05);
        padding: 0.6rem 1.25rem;
        border-radius: 24px;
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-size: 0.95rem;
        font-weight: 700;
        transition: all 0.2s;
    }

    .label-badge:hover {
        background: rgba(255, 255, 255, 0.1);
        transform: scale(1.05);
    }

    .label-badge img {
        width: 24px;
        height: 24px;
    }

    @media (max-width: 768px) {
        .modal-header {
            flex-direction: column;
            padding: 2.5rem 2rem 1.5rem;
            gap: 1.5rem;
        }

        .th-display {
            text-align: left;
            flex-direction: row;
        }

        .modal-body {
            padding: 0 2rem 2.5rem;
        }

        .stats-grid-large {
            grid-template-columns: repeat(2, 1fr);
        }

        .player-info-large {
            gap: 1.25rem;
        }

        .league-icon-large {
            width: 70px;
            height: 70px;
        }

        .player-titles h2 {
            font-size: 1.75rem;
        }
    }
</style>
