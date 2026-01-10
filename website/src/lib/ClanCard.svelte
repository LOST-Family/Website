<script lang="ts">
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';

  export let apiBaseUrl: string = '';
  export let theme: 'dark' | 'light' = 'dark';
  export let gameType: 'coc' | 'cr' = 'coc'; // Clash of Clans or Clash Royale
  
  // Exported stats for parent component
  export let clanCount: number = 0;
  export let playerCount: number = 0;

  interface Clan {
    tag: string;
    nameDB: string;
    nameAPI?: string;
  }

  interface Player {
    tag: string;
    userId: string | null;
    roleInClan: string;
    clanDB: string | null;
    // These may or may not exist depending on API
    nameAPI?: string;
    nameDB?: string;
  }

  interface ClanWithMembers extends Clan {
    members: Player[];
    loading: boolean;
    error: string | null;
  }

  let clans: ClanWithMembers[] = [];
  let mainLoading = true;
  let mainError: string | null = null;
  
  // Update exported counts reactively
  $: clanCount = clans.length;
  $: playerCount = clans.reduce((sum, c) => sum + c.members.length, 0);
  
  const roleOrder: Record<string, number> = {
    LEADER: 0,
    COLEADER: 1,
    ELDER: 2,
    MEMBER: 3,
    NOTINCLAN: 4,
  };

  const roleColors: Record<string, string> = {
    LEADER: '#FFD700', // Gold
    COLEADER: '#e91e63', // Pink/Purple
    ELDER: '#00bcd4', // Cyan
    MEMBER: '#8bc34a', // Light Green
    NOTINCLAN: '#9e9e9e', // Grey
  };

  const roleLabels: Record<string, string> = {
    LEADER: 'Leader',
    COLEADER: 'Co-Leaders',
    ELDER: 'Elders',
    MEMBER: 'Members',
    NOTINCLAN: 'Guests',
  };

  // Helper to get display name - use nameAPI, nameDB, or tag
  function getPlayerName(player: Player): string {
    return player.nameAPI || player.nameDB || player.tag || 'Unknown';
  }

  onMount(async () => {
    try {
      // 1. Fetch Clans (filtered by game type)
      const response = await fetch(`${apiBaseUrl}/api/clans?game=${gameType}`);
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const clanData: Clan[] = await response.json();
      
      // Initialize with loading state
      clans = clanData.map((clan) => ({
        ...clan,
        members: [],
        loading: true,
        error: null
      }));
      mainLoading = false;

      // 2. Fetch Members for ALL clans in parallel
      await Promise.all(clans.map(async (clan, index) => {
        try {
          const encodedTag = encodeURIComponent(clan.tag);
          const res = await fetch(`${apiBaseUrl}/api/clans/${encodedTag}/members`);
          
          if (!res.ok) throw new Error(`Failed to load`);
          
          let members: Player[] = await res.json();
          
          if (Array.isArray(members)) {
            // Normalize and sort - keep all players that have a tag
            members = members
              .filter(m => m && m.tag) // Only need a tag to be valid
              .map(m => ({
                ...m,
                roleInClan: m.roleInClan ? m.roleInClan.toUpperCase() : 'NOTINCLAN'
              }))
              .sort((a, b) => {
                const roleA = roleOrder[a.roleInClan] ?? 99;
                const roleB = roleOrder[b.roleInClan] ?? 99;
                return roleA - roleB;
              });
            
            clans[index].members = members;
          }
        } catch (e) {
          clans[index].error = "Could not load members";
          console.error(`Error loading members for ${clan.tag}:`, e);
        } finally {
          clans[index].loading = false;
          // Trigger reactivity
          clans = [...clans];
        }
      }));

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
    return [...grouped.entries()].sort((a, b) => (roleOrder[a[0]] ?? 99) - (roleOrder[b[0]] ?? 99));
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
      {#each clans as clan}
        <div class="clan-card-item" transition:fade>
          <div class="clan-card-header">
            <h3 class="clan-name">{clan.nameAPI || clan.nameDB || 'Unknown Clan'}</h3>
            <span class="clan-tag">{clan.tag}</span>
            {#if !clan.loading}
              <div class="member-count-badge">
                {clan.members.length} members
              </div>
            {/if}
          </div>

          <div class="clan-card-body">
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
                <span>No members</span>
              </div>
            {:else}
              <div class="roles-flow">
                {#each getPlayersByRole(clan.members) as [role, players]}
                  <div class="role-section">
                    <div class="role-header">
                      <span class="role-dot" style="background-color: {roleColors[role] || '#fff'}"></span>
                      <span class="role-title">{roleLabels[role] || role}</span>
                      <span class="role-count">{players.length}</span>
                    </div>
                    
                    <div class="members-list">
                      {#each players as player}
                        <div class="member-item">
                          <div class="member-avatar" style="border-color: {roleColors[role] || '#fff'}">
                            {getPlayerName(player).charAt(0).toUpperCase()}
                          </div>
                          <span class="member-name" title={getPlayerName(player)}>{getPlayerName(player)}</span>
                        </div>
                      {/each}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
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
  }

  .clan-card-item {
    background: #36393f;
    border-radius: 10px;
    overflow: hidden;
    border: 1px solid #202225;
    display: flex;
    flex-direction: column;
    transition: all 0.3s ease;
  }

  .card-container.light .clan-card-item {
    background: #ffffff;
    border: 1px solid #e3e5e8;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  }

  .clan-card-item:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
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
  }

  .card-container.light .clan-card-header {
    background: linear-gradient(180deg, #f2f3f5 0%, #e9ecef 100%);
    border-bottom: 1px solid #e3e5e8;
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
    background: rgba(0,0,0,0.3);
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
    max-height: 400px;
    overflow-y: auto;
    transition: background 0.4s ease;
  }

  .card-container.light .clan-card-body {
    background: #ffffff;
  }

  .loading-members, .empty-members, .member-error {
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
    background: rgba(0, 0, 0, 0.15);
    border-radius: 8px;
    padding: 12px;
    transition: background 0.4s ease;
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
    font-size: 0.7rem;
    color: #72767d;
    background: rgba(0, 0, 0, 0.2);
    padding: 2px 8px;
    border-radius: 10px;
    transition: all 0.4s ease;
  }

  .card-container.light .role-count {
    background: rgba(0, 0, 0, 0.06);
    color: #5c6470;
  }

  .members-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .member-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px 4px 4px;
    background: #2f3136;
    border-radius: 20px;
    transition: all 0.2s ease;
  }

  .card-container.light .member-item {
    background: #f2f3f5;
  }

  .member-item:hover {
    background: #40444b;
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
    font-size: 0.8rem;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 120px;
    transition: color 0.4s ease;
  }

  .card-container.light .member-name {
    color: #2e3338;
  }

  .spinner, .mini-spinner {
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
    to { transform: rotate(360deg); }
  }

  /* Custom Scrollbar */
  .card-body::-webkit-scrollbar,
  .clan-card-body::-webkit-scrollbar {
    width: 6px;
  }
  .card-body::-webkit-scrollbar-track,
  .clan-card-body::-webkit-scrollbar-track {
    background: transparent;
  }
  .card-body::-webkit-scrollbar-thumb,
  .clan-card-body::-webkit-scrollbar-thumb {
    background: #18191c;
    border-radius: 3px;
  }
  
  .card-container.light .card-body::-webkit-scrollbar-thumb,
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
    
    .card-header {
      padding: 16px;
    }
    
    .card-body {
      padding: 16px;
    }
    
    .total-stats {
      width: 100%;
      margin-left: 0;
      margin-top: 12px;
      justify-content: center;
    }
  }
</style>
