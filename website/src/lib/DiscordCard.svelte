<script lang="ts">
  import { onMount } from 'svelte';

  export let guildId: string;
  export let inviteCode: string | null = null;
  export let fallbackIcon: string | null = null;
  export let theme: 'dark' | 'light' = 'dark';
  export let totalMembers: number | null = null;
  
  interface Member {
    id: string;
    username: string;
    status: string;
    avatar_url: string;
    game?: { name: string };
  }

  interface DiscordData {
    name: string;
    instant_invite: string | null;
    presence_count: number;
    members: Member[];
  }

  let data: DiscordData | null = null;
  let serverIcon: string | null = null;
  let serverDescription: string | null = null;
  let inviteUrl: string | null = null;
  let loading = true;
  let error = false;

  onMount(async () => {
    try {
      const response = await fetch(`https://discord.com/api/guilds/${guildId}/widget.json`);
      if (!response.ok) throw new Error('Failed to fetch widget');
      data = await response.json();
      
      inviteUrl = data?.instant_invite ?? null;
      let code = inviteCode || data?.instant_invite?.split('/').pop();

      if (code && !totalMembers) {
        const inviteRes = await fetch(`https://discord.com/api/v10/invites/${code}?with_counts=true`);
        if (inviteRes.ok) {
          const inviteData = await inviteRes.json();
          totalMembers = inviteData.approximate_member_count;
          serverDescription = inviteData.guild?.description;
          if (inviteData.guild?.icon) {
            serverIcon = `https://cdn.discordapp.com/icons/${guildId}/${inviteData.guild.icon}.png?size=128`;
          }
          if (!inviteUrl) inviteUrl = `https://discord.gg/${code}`;
        }
      }
    } catch (e) {
      error = true;
    } finally {
      loading = false;
    }
  });

  function getStatusColor(status: string) {
    switch (status) {
      case 'online': return '#3ba55c';
      case 'dnd': return '#ed4245';
      case 'idle': return '#faa61a';
      default: return '#747f8d';
    }
  }
</script>

<div class="discord-card" class:loading class:light={theme === 'light'}>
  {#if loading}
    <div class="skeleton">
      <div class="skeleton-header"></div>
      <div class="skeleton-body"></div>
    </div>
  {:else if error || !data}
    <div class="error">
      Failed to load Discord Widget. 
      <br>Make sure "Enable Widget" is on in Server Settings.
    </div>
  {:else}
    <div class="header">
      <div class="server-info">
        {#if serverIcon}
          <img src={serverIcon} alt={data.name} class="server-icon" />
        {:else if fallbackIcon}
          <img src={fallbackIcon} alt={data.name} class="server-icon" />
        {:else}
          <svg class="discord-logo" viewBox="0 0 127.14 96.36">
            <path fill="#5865F2" d="M107.7,8.07A105.15,105.15,0,0,0,81.47,0a72.06,72.06,0,0,0-3.36,6.83A97.68,97.68,0,0,0,49,6.83,72.37,72.37,0,0,0,45.64,0,105.89,105.89,0,0,0,19.39,8.09C2.71,32.65-1.82,56.6.48,80.21a105.73,105.73,0,0,0,32.22,16.15,77.7,77.7,0,0,0,7.39-12,67.69,67.69,0,0,0-11.86-5.62c2.85-2.1,5.63-4.38,8.23-6.81a72.13,72.13,0,0,0,54.5,0c2.62,2.45,5.4,4.74,8.26,6.84a67.36,67.36,0,0,0-11.88,5.63,77.74,77.74,0,0,0,7.36,12,105.9,105.9,0,0,0,32.27-16.15C130.39,52.2,124.07,28.53,107.7,8.07ZM42.45,65.69C36.18,65.69,31,60,31,53s5-12.74,11.43-12.74S54,46,53.89,53,48.84,65.69,42.45,65.69Zm42.24,0C78.41,65.69,73.25,60,73.25,53s5-12.74,11.44-12.74S96.23,46,96.12,53,91.07,65.69,84.69,65.69Z"/>
          </svg>
        {/if}
        <div class="text">
          <h3>{data.name}</h3>
          <div class="stats">
            {#if totalMembers}
              <span class="total-count">{totalMembers.toLocaleString()} Members</span>
              <span class="dot">•</span>
            {/if}
            <span class="online-count">● {data.presence_count} Online</span>
          </div>
        </div>
      </div>
      {#if serverDescription}
        <p class="description">{serverDescription}</p>
      {/if}
    </div>

    <div class="footer">
      {#if inviteUrl}
        <a href={inviteUrl} target="_blank" rel="noopener noreferrer" class="join-btn">
          Server beitreten
        </a>
      {:else}
        <button class="join-btn disabled" disabled>
          Beitritt nach Bewerbung
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .discord-card {
    width: 350px;
    background: #2f3136;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-family: 'gg sans', 'Noto Sans', 'Helvetica Neue', Helvetica, Arial, sans-serif;
    color: #dcddde;
    box-shadow: 0 8px 24px rgba(0,0,0,0.2);
    border: 1px solid #202225;
    transition: all 0.4s ease;
  }

  .discord-card.light {
    background: #ffffff;
    color: #2e3338;
    border: 1px solid #e3e5e8;
    box-shadow: 0 8px 24px rgba(0,0,0,0.08);
  }

  .discord-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 12px 32px rgba(0,0,0,0.3);
  }

  .discord-card.light:hover {
    box-shadow: 0 12px 32px rgba(0,0,0,0.12);
  }

  .header {
    padding: 24px 16px;
    background: #202225;
    box-shadow: 0 1px 0 rgba(4,4,5,0.2),0 1.5px 0 rgba(6,6,7,0.05),0 2px 0 rgba(4,4,5,0.05);
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    transition: background 0.4s ease;
  }

  .discord-card.light .header {
    background: linear-gradient(180deg, #f2f3f5 0%, #e3e5e8 100%);
    box-shadow: 0 1px 0 rgba(0,0,0,0.05);
  }

  .server-info {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .discord-logo {
    width: 32px;
    height: 32px;
  }

  .server-icon {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.4);
    transition: box-shadow 0.4s ease;
  }

  .discord-card.light .server-icon {
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
  }

  .description {
    margin: 16px 0 0 0;
    font-size: 14px;
    color: #b9bbbe;
    line-height: 1.5;
    transition: color 0.4s ease;
  }

  .discord-card.light .description {
    color: #4f5660;
  }

  .text h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 700;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 250px;
    transition: color 0.4s ease;
  }

  .discord-card.light .text h3 {
    color: #060607;
  }

  .stats {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }

  .total-count {
    font-size: 13px;
    color: #b9bbbe;
    transition: color 0.4s ease;
  }

  .discord-card.light .total-count {
    color: #4f5660;
  }

  .dot {
    font-size: 10px;
    color: #4f545c;
  }

  .online-count {
    font-size: 13px;
    color: #3ba55c;
    font-weight: 600;
  }

  .footer {
    padding: 20px 16px;
    background: #2f3136;
    display: flex;
    justify-content: center;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    transition: all 0.4s ease;
  }

  .discord-card.light .footer {
    background: #ffffff;
    border-top: 1px solid #e3e5e8;
  }

  .join-btn {
    display: inline-block;
    padding: 12px 28px;
    background: linear-gradient(135deg, #3ba55c 0%, #2d8049 100%);
    color: white;
    text-align: center;
    border-radius: 8px;
    text-decoration: none;
    font-size: 14px;
    font-weight: 600;
    transition: all 0.3s ease;
    border: none;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(59, 165, 92, 0.3);
  }

  .join-btn:hover {
    background: linear-gradient(135deg, #2d8049 0%, #236b3a 100%);
    transform: translateY(-1px);
    box-shadow: 0 6px 16px rgba(59, 165, 92, 0.4);
  }

  .join-btn.disabled {
    background: linear-gradient(135deg, #4f545c 0%, #36393f 100%);
    cursor: not-allowed;
    opacity: 0.7;
    box-shadow: none;
  }

  .discord-card.light .join-btn.disabled {
    background: linear-gradient(135deg, #747f8d 0%, #5c6470 100%);
  }

  .error {
    padding: 20px;
    text-align: center;
    color: #ed4245;
    font-size: 14px;
  }

  /* Skeleton animation */
  @keyframes shine {
    to { background-position: 200% center; }
  }

  .skeleton {
    padding: 16px;
    height: 100%;
  }

  .skeleton-header {
    height: 48px;
    background: linear-gradient(90deg, #202225 25%, #2a2d31 50%, #202225 75%);
    background-size: 200% 100%;
    animation: shine 1.5s infinite linear;
    border-radius: 4px;
    margin-bottom: 20px;
  }

  .discord-card.light .skeleton-header {
    background: linear-gradient(90deg, #e3e5e8 25%, #f2f3f5 50%, #e3e5e8 75%);
    background-size: 200% 100%;
  }

  .skeleton-body {
    height: 300px;
    background: linear-gradient(90deg, #202225 25%, #2a2d31 50%, #202225 75%);
    background-size: 200% 100%;
    animation: shine 1.5s infinite linear;
    border-radius: 4px;
  }

  .discord-card.light .skeleton-body {
    background: linear-gradient(90deg, #e3e5e8 25%, #f2f3f5 50%, #e3e5e8 75%);
    background-size: 200% 100%;
  }
</style>