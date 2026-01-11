<script lang="ts">
  import { onMount } from 'svelte';
  import DiscordCard from './DiscordCard.svelte';
  
  export let theme: 'dark' | 'light' = 'dark';
  export let fallbackIcon: string;

  let memberCount: number | null = null;
  const apiBaseUrl = import.meta.env.VITE_API_BASE_URL !== undefined ? import.meta.env.VITE_API_BASE_URL : '';

  onMount(async () => {
    try {
      const response = await fetch(`${apiBaseUrl}/api/guild`);
      if (response.ok) {
        const data = await response.json();
        // The API returns the guild object, we want the member count
        memberCount = data.membercount;
      }
    } catch (e) {
      console.error('Failed to fetch guild info:', e);
    }
  });
</script>

<section class="content-section" id="discord" class:light={theme === 'light'}>
  <div class="section-container">
    <div class="section-header">
      <div class="section-icon">
        <svg viewBox="0 0 127.14 96.36" fill="currentColor">
          <path d="M107.7,8.07A105.15,105.15,0,0,0,81.47,0a72.06,72.06,0,0,0-3.36,6.83A97.68,97.68,0,0,0,49,6.83,72.37,72.37,0,0,0,45.64,0,105.89,105.89,0,0,0,19.39,8.09C2.71,32.65-1.82,56.6.48,80.21a105.73,105.73,0,0,0,32.22,16.15,77.7,77.7,0,0,0,7.39-12,67.69,67.69,0,0,0-11.86-5.62c2.85-2.1,5.63-4.38,8.23-6.81a72.13,72.13,0,0,0,54.5,0c2.62,2.45,5.4,4.74,8.26,6.84a67.36,67.36,0,0,0-11.88,5.63,77.74,77.74,0,0,0,7.36,12,105.9,105.9,0,0,0,32.27-16.15C130.39,52.2,124.07,28.53,107.7,8.07ZM42.45,65.69C36.18,65.69,31,60,31,53s5-12.74,11.43-12.74S54,46,53.89,53,48.84,65.69,42.45,65.69Zm42.24,0C78.41,65.69,73.25,60,73.25,53s5-12.74,11.44-12.74S96.23,46,96.12,53,91.07,65.69,84.69,65.69Z"/>
        </svg>
      </div>
      <div class="section-title-group">
        <h2 class="section-title">Komm in die Community</h2>
        <p class="section-description">Die Angriffsplannung erfolgt über Discord</p>
      </div>
    </div>
    
    <div class="discord-grid">
      <DiscordCard guildId="1108449987827876022" {theme} />
      <DiscordCard guildId="733857906117574717" {fallbackIcon} {theme} totalMembers={memberCount} />
    </div>
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
    background: linear-gradient(90deg, transparent, rgba(88, 101, 242, 0.5), transparent);
  }
  
  .section-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 0 2rem;
  }
  
  .section-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 2.5rem;
  }
  
  .section-icon {
    width: 48px;
    height: 48px;
    background: linear-gradient(135deg, #5865f2 0%, #4752c4 100%);
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    box-shadow: 0 8px 24px rgba(88, 101, 242, 0.3);
    flex-shrink: 0;
  }
  
  .section-icon svg {
    width: 24px;
    height: 24px;
  }
  
  .section-title-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .section-title {
    margin: 0;
    font-size: 1.75rem;
    font-weight: 700;
    color: #fff;
    transition: color 0.4s ease;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .member-badge {
    font-size: 0.85rem;
    font-weight: 600;
    padding: 0.25rem 0.75rem;
    background: rgba(88, 101, 242, 0.15);
    color: #5865f2;
    border: 1px solid rgba(88, 101, 242, 0.3);
    border-radius: 20px;
    letter-spacing: 0.02em;
  }

  .content-section.light .member-badge {
    background: rgba(88, 101, 242, 0.1);
    border-color: rgba(88, 101, 242, 0.2);
  }
  
  .content-section.light .section-title {
    color: #0f172a;
  }
  
  .section-description {
    margin: 0;
    font-size: 0.95rem;
    color: rgba(255, 255, 255, 0.5);
    transition: color 0.4s ease;
  }
  
  .content-section.light .section-description {
    color: #64748b;
  }
  
  .discord-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
    justify-content: center;
  }
  
  .discord-grid :global(.discord-card) {
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1), box-shadow 0.3s ease;
  }
  
  .discord-grid :global(.discord-card:hover) {
    transform: translateY(-8px);
  }

  @media (max-width: 800px) {
    .content-section {
      padding: 3rem 0;
    }
    
    .section-container {
      padding: 0 1rem;
    }
    
    .section-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.75rem;
    }
    
    .section-title {
      font-size: 1.5rem;
    }
    
    .discord-grid {
      gap: 1rem;
    }
  }
</style>
