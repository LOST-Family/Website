<script lang="ts">
  import { onMount } from 'svelte';
  import { user, loading } from './auth';
  
  export let theme: 'dark' | 'light' = 'dark';
  export let apiBaseUrl: string = '';
  
  let playerAccounts: any[] = [];
  let accountsLoading = false;
  let accountsError: string | null = null;

  async function fetchPlayerAccounts() {
    accountsLoading = true;
    try {
      const response = await fetch(`${apiBaseUrl}/api/me/accounts`, {
        credentials: 'include',
      });
      if (response.ok) {
        playerAccounts = await response.json();
      } else {
        accountsError = 'Fehler beim Laden der Accounts';
      }
    } catch (error) {
      console.error('Failed to fetch player accounts:', error);
      accountsError = 'Netzwerkfehler beim Laden der Accounts';
    } finally {
      accountsLoading = false;
    }
  }

  onMount(() => {
    if ($user) {
      fetchPlayerAccounts();
    }
  });

  $: if ($user && !accountsLoading && playerAccounts.length === 0 && !accountsError) {
    fetchPlayerAccounts();
  }
</script>

<div class="profile-page" class:light={theme === 'light'}>
  <div class="container">
    {#if $loading}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Lade Profil...</p>
      </div>
    {:else if !$user}
      <div class="error-state">
        <h2>Nicht angemeldet</h2>
        <p>Bitte melde dich mit Discord an, um dein Profil zu sehen.</p>
      </div>
    {:else}
      <div class="profile-header">
        <div class="user-info">
          <div class="avatar-container">
            {#if $user.avatar}
              <img src={$user.avatar} alt={$user.username} class="avatar" />
            {:else}
              <div class="avatar-placeholder">
                {($user.nickname || $user.global_name || $user.username).charAt(0).toUpperCase()}
              </div>
            {/if}
            <div class="status-badge" title="Online"></div>
          </div>
          <div class="user-details">
            <h1>{$user.nickname || $user.global_name || $user.username}</h1>
            <p class="discord-tag">@{$user.username}</p>
            <div class="badges">
              {#if $user.highest_role}
                <span class="badge role-badge">{$user.highest_role}</span>
              {/if}
              {#if $user.is_admin}
                <span class="badge admin-badge">Admin</span>
              {/if}
            </div>
          </div>
        </div>
      </div>

      <div class="profile-content">
        <section class="accounts-section">
          <div class="section-header">
            <h2>Verknüpfte Accounts</h2>
            <span class="account-count">{playerAccounts.length} Accounts</span>
          </div>

          {#if accountsLoading}
            <div class="accounts-loading">
              <div class="loading-grid">
                {#each Array(2) as _}
                  <div class="skeleton-card"></div>
                {/each}
              </div>
            </div>
          {:else if accountsError}
            <div class="error-message">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
              <p>{accountsError}</p>
              <button on:click={fetchPlayerAccounts} class="retry-btn">Erneut versuchen</button>
            </div>
          {:else if playerAccounts.length === 0}
            <div class="empty-state">
              <div class="empty-icon">🎮</div>
              <p>Keine Clash of Clans Accounts verknüpft.</p>
              <p class="sub-text">Verknüpfe deine Accounts auf unserem Discord Server.</p>
            </div>
          {:else}
            <div class="accounts-grid">
              {#each playerAccounts as player}
                {@const activePoints = (player.activeKickpoints || []).reduce((acc: number, kp: any) => acc + (kp.amount || 0), 0)}
                <div class="account-card" class:light={theme === 'light'}>
                  <div class="card-glow"></div>
                  <div class="player-header">
                    <div class="player-rank">
                      {#if player.clanDB?.badgeUrl}
                        <img src={player.clanDB.badgeUrl} alt={player.clanDB.nameDB} class="league-icon" />
                      {:else}
                        <div class="league-icon-placeholder">🎮</div>
                      {/if}
                    </div>
                    <div class="player-main">
                      <h3 class="player-name">{player.nameDB}</h3>
                      <p class="player-tag">{player.tag}</p>
                    </div>
                    {#if player.roleInClan && player.roleInClan !== 'NOTINCLAN'}
                      <div class="role-badge-small">
                        {player.roleInClan}
                      </div>
                    {/if}
                  </div>
                  
                  <div class="player-stats">
                    <div class="stat-item">
                      <span class="stat-label">Kickpunkte</span>
                      <span class="stat-value" class:danger={activePoints >= (player.clanDB?.maxKickpoints || 9)}>
                        {activePoints} / {player.clanDB?.maxKickpoints || '-'}
                      </span>
                    </div>
                    <div class="stat-item">
                      <span class="stat-label">Clan</span>
                      <span class="stat-value">{player.clanDB?.nameDB || 'Kein Clan'}</span>
                    </div>
                    <div class="stat-item">
                      <span class="stat-label">Gesamtanzahl Kickpunkte</span>
                      <span class="stat-value">{player.totalKickpoints}</span>
                    </div>
                  </div>

                  {#if player.activeKickpoints && player.activeKickpoints.length > 0}
                    <div class="kickpoints-details">
                      <span class="details-title">Aktive Kickpunkte:</span>
                      {#each player.activeKickpoints as kp}
                        <div class="kp-reason">
                          <span class="kp-amount">+{kp.amount}</span>
                          <span class="kp-desc">{kp.description}</span>
                          <span class="kp-date">{new Date(kp.date).toLocaleDateString()}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </section>
      </div>
    {/if}
  </div>
</div>

<style>
  .profile-page {
    min-height: calc(100vh - 64px);
    padding: 3rem 1rem;
    color: white;
    position: relative;
    z-index: 1;
  }

  .profile-page.light {
    color: #1e293b;
  }

  .container {
    max-width: 1000px;
    margin: 0 auto;
  }

  .profile-header {
    background: rgba(255, 255, 255, 0.03);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 24px;
    padding: 2.5rem;
    margin-bottom: 2.5rem;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
  }

  .light .profile-header {
    background: white;
    border: 1px solid rgba(0, 0, 0, 0.05);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.05);
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: 2rem;
  }

  .avatar-container {
    position: relative;
    flex-shrink: 0;
  }

  .avatar {
    width: 100px;
    height: 100px;
    border-radius: 50%;
    border: 4px solid #3b82f6;
    object-fit: cover;
  }

  .avatar-placeholder {
    width: 100px;
    height: 100px;
    border-radius: 50%;
    background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2.5rem;
    font-weight: bold;
    color: white;
    border: 4px solid rgba(255, 255, 255, 0.1);
  }

  .status-badge {
    position: absolute;
    bottom: 5px;
    right: 5px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #22c55e;
    border: 3px solid rgba(20, 20, 30, 1);
  }

  .light .status-badge {
    border-color: white;
  }

  .user-details h1 {
    margin: 0;
    font-size: 2.25rem;
    font-weight: 800;
    letter-spacing: -0.02em;
  }

  .discord-tag {
    margin: 0.25rem 0 1rem;
    font-size: 1.1rem;
    color: rgba(255, 255, 255, 0.5);
    font-family: 'JetBrains Mono', monospace;
  }

  .light .discord-tag {
    color: rgba(0, 0, 0, 0.4);
  }

  .badges {
    display: flex;
    gap: 0.75rem;
  }

  .badge {
    padding: 0.4rem 0.8rem;
    border-radius: 8px;
    font-size: 0.85rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .role-badge {
    background: rgba(59, 130, 246, 0.15);
    color: #60a5fa;
    border: 1px solid rgba(59, 130, 246, 0.2);
  }

  .admin-badge {
    background: rgba(244, 63, 94, 0.15);
    color: #fb7185;
    border: 1px solid rgba(244, 63, 94, 0.2);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .section-header h2 {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0;
  }

  .account-count {
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.4);
    background: rgba(255, 255, 255, 0.05);
    padding: 0.25rem 0.75rem;
    border-radius: 20px;
  }

  .light .account-count {
    color: rgba(0, 0, 0, 0.4);
    background: rgba(0, 0, 0, 0.05);
  }

  .accounts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1.5rem;
  }

  .account-card {
    position: relative;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 20px;
    padding: 1.5rem;
    overflow: hidden;
    transition: all 0.3s ease;
  }

  .account-card:hover {
    transform: translateY(-5px);
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .account-card.light {
    background: white;
    border-color: rgba(0, 0, 0, 0.05);
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.03);
  }

  .account-card.light:hover {
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.06);
  }

  .card-glow {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 100px;
    background: radial-gradient(600px circle at var(--x, 50%) var(--y, 0%), rgba(59, 130, 246, 0.1), transparent 40%);
    pointer-events: none;
  }

  .player-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .league-icon {
    width: 48px;
    height: 48px;
    filter: drop-shadow(0 0 8px rgba(0, 0, 0, 0.3));
  }

  .player-main {
    flex-grow: 1;
  }

  .player-name {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 700;
  }

  .player-tag {
    margin: 0;
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.4);
    font-family: 'JetBrains Mono', monospace;
  }

  .light .player-tag {
    color: rgba(0, 0, 0, 0.4);
  }

  .th-badge {
    background: #475569;
    padding: 0.25rem 0.6rem;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 800;
    color: white;
  }

  .player-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 12px;
    padding: 1rem;
    margin-bottom: 1.25rem;
  }

  .light .player-stats {
    background: #f8fafc;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .stat-label {
    font-size: 0.7rem;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.3);
    letter-spacing: 0.05em;
    margin-bottom: 0.25rem;
  }

  .light .stat-label {
    color: rgba(0, 0, 0, 0.4);
  }

  .stat-value {
    font-size: 0.95rem;
    font-weight: 700;
  }

  .heroes-list {
    display: flex;
    gap: 0.5rem;
  }

  .hero-item {
    position: relative;
    width: 32px;
    height: 32px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 700;
    transition: all 0.2s ease;
  }

  .hero-item:hover {
    background: rgba(59, 130, 246, 0.2);
    color: #60a5fa;
  }

  .stat-value.danger {
    color: #ef4444;
  }

  .role-badge-small {
    font-size: 0.7rem;
    padding: 0.2rem 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: rgba(255, 255, 255, 0.6);
    text-transform: uppercase;
    font-weight: 700;
  }

  .light .role-badge-small {
    background: rgba(0, 0, 0, 0.03);
    border-color: rgba(0, 0, 0, 0.08);
    color: rgba(0, 0, 0, 0.5);
  }

  .kickpoints-details {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
  }

  .light .kickpoints-details {
    border-top-color: rgba(0, 0, 0, 0.05);
  }

  .details-title {
    display: block;
    font-size: 0.75rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.4);
    margin-bottom: 0.5rem;
  }

  .light .details-title {
    color: rgba(0, 0, 0, 0.4);
  }

  .kp-reason {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.8rem;
    margin-bottom: 0.4rem;
  }

  .kp-amount {
    color: #ef4444;
    font-weight: 700;
    flex-shrink: 0;
  }

  .kp-desc {
    flex-grow: 1;
    color: rgba(255, 255, 255, 0.8);
  }

  .light .kp-desc {
    color: rgba(0, 0, 0, 0.8);
  }

  .kp-date {
    font-size: 0.7rem;
    color: rgba(255, 255, 255, 0.3);
    font-family: 'JetBrains Mono', monospace;
  }

  .light .kp-date {
    color: rgba(0, 0, 0, 0.3);
  }

  .league-icon-placeholder {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 12px;
    font-size: 1.5rem;
  }

  .light .league-icon-placeholder {
    background: rgba(0, 0, 0, 0.03);
  }

  .loading-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1.5rem;
  }

  .skeleton-card {
    height: 200px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 20px;
    position: relative;
    overflow: hidden;
  }

  .skeleton-card::after {
    content: "";
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    transform: translateX(-100%);
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.05), transparent);
    animation: shimmer 1.5s infinite;
  }

  @keyframes shimmer {
    100% { transform: translateX(100%); }
  }

  .loading-state, .error-state, .empty-state {
    text-align: center;
    padding: 5rem 2rem;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 1.5rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 768px) {
    .user-info {
      flex-direction: column;
      text-align: center;
      gap: 1.5rem;
    }
    
    .profile-header {
      padding: 2rem 1.5rem;
    }
    
    .accounts-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
