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
                    stroke-width="3"
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
                                                    kp.date || kp.createdAt,
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

                    <!-- Heroes -->
                    {#if player.heroes && player.heroes.length > 0}
                        <div class="detail-section">
                            <h3>Helden</h3>
                            <div class="items-grid">
                                {#each player.heroes as hero}
                                    <div
                                        class="item-badge"
                                        class:maxed={hero.level ===
                                            hero.maxLevel}
                                        title={hero.name}
                                    >
                                        <span class="item-name"
                                            >{hero.name}</span
                                        >
                                        <span class="item-level"
                                            >{hero.level} / {hero.maxLevel}</span
                                        >
                                        <div class="progress-track">
                                            <div
                                                class="progress-bar"
                                                style="width: {(hero.level /
                                                    hero.maxLevel) *
                                                    100}%"
                                            ></div>
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    <!-- Hero Equipment -->
                    {#if player.heroEquipment && player.heroEquipment.length > 0}
                        <div class="detail-section">
                            <h3>Helden-Ausrüstung</h3>
                            <div class="items-grid">
                                {#each player.heroEquipment as eq}
                                    <div
                                        class="item-badge"
                                        class:maxed={eq.level === eq.maxLevel}
                                        title={eq.name}
                                    >
                                        <span class="item-name">{eq.name}</span>
                                        <span class="item-level"
                                            >{eq.level} / {eq.maxLevel}</span
                                        >
                                        <div class="progress-track">
                                            <div
                                                class="progress-bar"
                                                style="width: {(eq.level /
                                                    eq.maxLevel) *
                                                    100}%"
                                            ></div>
                                        </div>
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
                                        {#if label.iconUrls?.small}
                                            <img
                                                src={label.iconUrls.small}
                                                alt={label.name}
                                            />
                                        {/if}
                                        <span>{label.name}</span>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    <!-- Troops -->
                    {#if player.troops && player.troops.length > 0}
                        <div class="detail-section">
                            <h3>Truppen</h3>
                            <div class="items-grid">
                                {#each player.troops.filter((t: any) => !t.name.startsWith('Super ') && !['Sneaky Goblin', 'Rocket Balloon', 'Inferno Dragon', 'Ice Hound'].includes(t.name)) as troop}
                                    <div
                                        class="item-badge"
                                        class:maxed={troop.level ===
                                            troop.maxLevel}
                                        title={troop.name}
                                    >
                                        <span class="item-name"
                                            >{troop.name}</span
                                        >
                                        <span class="item-level"
                                            >{troop.level} / {troop.maxLevel}</span
                                        >
                                        <div class="progress-track">
                                            <div
                                                class="progress-bar"
                                                style="width: {(troop.level /
                                                    troop.maxLevel) *
                                                    100}%"
                                            ></div>
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    <!-- Spells -->
                    {#if player.spells && player.spells.length > 0}
                        <div class="detail-section">
                            <h3>Zauber</h3>
                            <div class="items-grid">
                                {#each player.spells as spell}
                                    <div
                                        class="item-badge"
                                        class:maxed={spell.level ===
                                            spell.maxLevel}
                                        title={spell.name}
                                    >
                                        <span class="item-name"
                                            >{spell.name}</span
                                        >
                                        <span class="item-level"
                                            >{spell.level} / {spell.maxLevel}</span
                                        >
                                        <div class="progress-track">
                                            <div
                                                class="progress-bar"
                                                style="width: {(spell.level /
                                                    spell.maxLevel) *
                                                    100}%"
                                            ></div>
                                        </div>
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
        background: #0f172a;
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 32px;
        width: 100%;
        max-width: 850px;
        max-height: 90vh;
        position: relative;
        box-shadow:
            0 30px 60px -12px rgba(0, 0, 0, 0.8),
            0 0 40px rgba(59, 130, 246, 0.1);
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .modal-content.light {
        background: #f8fafc;
        border-color: rgba(0, 0, 0, 0.08);
        color: #1e293b;
    }

    .modal-scroll-area {
        overflow-y: auto;
        flex-grow: 1;
        scrollbar-width: thin;
        scrollbar-color: rgba(59, 130, 246, 0.3) transparent;
    }

    .modal-scroll-area::-webkit-scrollbar {
        width: 8px;
    }

    .modal-scroll-area::-webkit-scrollbar-thumb {
        background-color: rgba(59, 130, 246, 0.2);
        border-radius: 10px;
        border: 2px solid transparent;
        background-clip: content-box;
    }

    .modal-scroll-area::-webkit-scrollbar-thumb:hover {
        background-color: rgba(59, 130, 246, 0.4);
    }

    .close-modal {
        --btn-size: 56px;
        position: absolute;
        top: 1.5rem;
        right: 1.5rem;
        width: var(--btn-size);
        height: var(--btn-size);
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        background: rgba(255, 255, 255, 0.1);
        border: 2px solid rgba(255, 255, 255, 0.1);
        border-radius: 50%;
        color: white;
        cursor: pointer;
        transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
        z-index: 100;
        backdrop-filter: blur(12px);
    }

    .light .close-modal {
        background: rgba(0, 0, 0, 0.05);
        border-color: rgba(0, 0, 0, 0.1);
        color: #1e293b;
    }

    .close-modal:hover {
        background: #ef4444;
        border-color: #ef4444;
        color: white;
        transform: rotate(90deg) scale(1.1);
        box-shadow: 0 0 30px rgba(239, 68, 68, 0.5);
    }

    .close-modal svg {
        width: 36px;
        height: 36px;
        stroke-width: 3px;
    }

    .modal-header {
        padding: 4rem 3rem 3rem;
        background: linear-gradient(
            to bottom,
            rgba(59, 130, 246, 0.15),
            rgba(15, 23, 42, 0)
        );
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 2rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .player-info-large {
        display: flex;
        align-items: center;
        gap: 2.5rem;
    }

    .league-icon-large {
        width: 120px;
        height: 120px;
        filter: drop-shadow(0 0 30px rgba(59, 130, 246, 0.4));
        transition: transform 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    }

    .league-icon-large:hover {
        transform: scale(1.1) rotate(5deg);
    }

    .player-titles h2 {
        margin: 0;
        font-size: 2.75rem;
        font-weight: 900;
        letter-spacing: -0.03em;
        background: linear-gradient(to bottom right, #fff, #94a3b8);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        line-height: 1;
    }

    .light .player-titles h2 {
        background: linear-gradient(to bottom right, #1e293b, #64748b);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .player-titles .tag {
        margin: 0.5rem 0 0;
        font-family: 'JetBrains Mono', monospace;
        color: #3b82f6;
        font-weight: 700;
        font-size: 1.25rem;
        letter-spacing: 1px;
    }

    .clan-info-small {
        display: flex;
        align-items: center;
        gap: 1rem;
        margin-top: 1.5rem;
        font-size: 1.15rem;
        font-weight: 850;
        color: white;
        background: linear-gradient(
            135deg,
            rgba(30, 41, 59, 0.8),
            rgba(15, 23, 42, 0.9)
        );
        padding: 0.85rem 1.6rem;
        border-radius: 20px;
        border: 1px solid rgba(59, 130, 246, 0.25);
        width: fit-content;
        backdrop-filter: blur(12px);
        box-shadow: 0 12px 30px -8px rgba(0, 0, 0, 0.5);
        transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    }

    .clan-info-small:hover {
        border-color: #3b82f6;
        transform: translateY(-3px) scale(1.02);
        box-shadow: 0 15px 35px -10px rgba(59, 130, 246, 0.3);
    }

    .light .clan-info-small {
        background: rgba(255, 255, 255, 0.9);
        border-color: rgba(0, 0, 0, 0.1);
        color: #0f172a;
    }

    .clan-info-small img {
        width: 38px;
        height: 38px;
        filter: drop-shadow(0 0 12px rgba(59, 130, 246, 0.4));
    }

    .th-display {
        text-align: right;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .th-level,
    .bh-level {
        background: linear-gradient(135deg, #3b82f6, #2563eb);
        color: white;
        padding: 0.75rem 1.5rem;
        border-radius: 18px;
        font-weight: 900;
        font-size: 1.1rem;
        box-shadow: 0 8px 20px -5px rgba(59, 130, 246, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .bh-level {
        background: linear-gradient(135deg, #8b5cf6, #7c3aed);
        box-shadow: 0 8px 20px -5px rgba(139, 92, 246, 0.5);
    }

    .cr-level {
        background: linear-gradient(135deg, #5865f2, #4752c4);
        box-shadow: 0 8px 20px -5px rgba(88, 101, 242, 0.5);
    }

    .modal-body {
        padding: 0 3rem 4rem;
    }

    .stats-grid-large {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 1.5rem;
        margin-bottom: 4rem;
    }

    .stat-card {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        padding: 1.5rem;
        border-radius: 24px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5rem;
        transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
        backdrop-filter: blur(8px);
    }

    .stat-card:hover {
        background: rgba(255, 255, 255, 0.06);
        border-color: #3b82f6;
        transform: translateY(-4px) scale(1.02);
        box-shadow: 0 12px 24px -8px rgba(0, 0, 0, 0.5);
    }

    .light .stat-card {
        background: #f9fafb;
        border-color: rgba(0, 0, 0, 0.05);
    }

    .stat-card .label {
        font-size: 0.75rem;
        text-transform: uppercase;
        color: rgba(255, 255, 255, 0.5);
        font-weight: 700;
        letter-spacing: 0.05em;
    }

    .light .stat-card .label {
        color: rgba(0, 0, 0, 0.5);
    }

    .stat-card .value {
        font-size: 1.75rem;
        font-weight: 900;
        color: #3b82f6;
        text-shadow: 0 0 20px rgba(59, 130, 246, 0.2);
    }

    .stat-card .value.danger {
        color: #ef4444;
        text-shadow: 0 0 20px rgba(239, 68, 68, 0.2);
    }

    .detail-section {
        margin-top: 4rem;
    }

    .detail-section h3 {
        font-size: 1.5rem;
        font-weight: 900;
        margin-bottom: 2rem;
        display: flex;
        align-items: center;
        gap: 1.25rem;
        color: white;
        letter-spacing: -0.01em;
    }

    .light .detail-section h3 {
        color: #111827;
    }

    .detail-section h3::after {
        content: '';
        flex-grow: 1;
        height: 2px;
        background: linear-gradient(
            to right,
            rgba(59, 130, 246, 0.3),
            transparent
        );
        border-radius: 2px;
    }

    .light .detail-section h3::after {
        background: linear-gradient(
            to right,
            rgba(59, 130, 246, 0.2),
            transparent
        );
    }

    /* Kickpoints History */
    .kickpoints-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .kp-item {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        padding: 1.25rem;
        border-radius: 20px;
        transition: all 0.2s;
    }

    .kp-item:hover {
        background: rgba(255, 255, 255, 0.05);
        transform: translateX(4px);
    }

    .light .kp-item {
        background: #f8fafc;
        border-color: rgba(0, 0, 0, 0.05);
    }

    .kp-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.5rem;
    }

    .kp-reason {
        font-weight: 800;
        font-size: 1.1rem;
        color: white;
    }

    .light .kp-reason {
        color: #1e293b;
    }

    .kp-amount {
        font-weight: 900;
        font-size: 1rem;
        padding: 0.25rem 0.75rem;
        border-radius: 10px;
    }

    .kp-amount.positive {
        color: #10b981;
        background: rgba(16, 185, 129, 0.1);
    }
    .kp-amount.negative {
        color: #ef4444;
        background: rgba(239, 68, 68, 0.1);
    }

    .kp-footer {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .kp-date {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.5);
        font-family: 'JetBrains Mono', monospace;
    }

    .light .kp-date {
        color: rgba(0, 0, 0, 0.4);
    }

    .kp-desc {
        font-size: 0.9rem;
        color: rgba(255, 255, 255, 0.8);
        line-height: 1.5;
    }

    .light .kp-desc {
        color: rgba(0, 0, 0, 0.7);
    }

    /* Other Accounts */
    .other-accounts-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1rem;
    }

    .other-account-btn {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        padding: 1.25rem;
        border-radius: 20px;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 0.5rem;
        cursor: pointer;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        text-align: left;
        color: inherit;
    }

    .light .other-account-btn {
        background: #f8fafc;
        border-color: rgba(0, 0, 0, 0.05);
    }

    .other-account-btn:hover {
        background: rgba(59, 130, 246, 0.1);
        border-color: #3b82f6;
        transform: translateY(-4px);
        box-shadow: 0 10px 20px -10px rgba(59, 130, 246, 0.5);
    }

    .acc-name {
        font-weight: 800;
        font-size: 1rem;
    }

    .acc-tag {
        font-size: 0.8rem;
        color: #3b82f6;
        font-family: 'JetBrains Mono', monospace;
        font-weight: 600;
    }

    .light .acc-tag {
        color: #2563eb;
    }

    /* Items Grids */
    .items-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1.25rem;
    }

    .item-badge {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        padding: 1.5rem;
        border-radius: 24px;
        position: relative;
        overflow: hidden;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        backdrop-filter: blur(4px);
    }

    .item-badge:hover {
        background: rgba(255, 255, 255, 0.06);
        border-color: rgba(255, 255, 255, 0.2);
        transform: translateY(-4px);
    }

    .item-badge.maxed {
        border-color: #fbbf24;
        background: linear-gradient(
            135deg,
            rgba(251, 191, 36, 0.1),
            rgba(180, 83, 9, 0.05)
        );
        box-shadow: 0 8px 24px -8px rgba(251, 191, 36, 0.25);
    }

    .item-badge.maxed::after {
        content: 'MAX';
        position: absolute;
        top: 0.6rem;
        right: 0.6rem;
        background: #fbbf24;
        color: #000;
        font-size: 0.65rem;
        font-weight: 900;
        padding: 0.2rem 0.5rem;
        border-radius: 6px;
        box-shadow: 0 0 10px rgba(251, 191, 36, 0.4);
    }

    .item-name {
        display: block;
        font-size: 1rem;
        font-weight: 800;
        margin-bottom: 0.5rem;
        color: white;
    }

    .light .item-name {
        color: #1e293b;
    }

    .item-level {
        font-size: 0.9rem;
        color: rgba(255, 255, 255, 0.5);
        font-weight: 700;
        font-family: 'JetBrains Mono', monospace;
    }

    .item-badge.maxed .item-level {
        color: #fbbf24;
    }

    .progress-track {
        height: 6px;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 3px;
        margin-top: 1rem;
        overflow: hidden;
    }

    .progress-bar {
        position: relative; /* Remove absolute positioning */
        height: 100%;
        background: #3b82f6;
        box-shadow: 0 0 12px rgba(59, 130, 246, 0.5);
        border-radius: 3px;
        transition: width 1s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .item-badge.maxed .progress-bar {
        background: linear-gradient(to right, #f59e0b, #fbbf24);
        box-shadow: 0 0 12px rgba(251, 191, 36, 0.6);
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
