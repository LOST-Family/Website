<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchUser } from './lib/auth';
  
  // Components
  import Header from './lib/Header.svelte';
  import Hero from './lib/Hero.svelte';
  import DiscordSection from './lib/DiscordSection.svelte';
  import FeaturesSection from './lib/FeaturesSection.svelte';
  import ClansSection from './lib/ClansSection.svelte';
  import Footer from './lib/Footer.svelte';
  
  // Logos
  import lostLogo from './assets/Logos/Lost.png';
  
  // Banners
  import lostBanner from './assets/Assets/banner-lost.png';
  
  // Assets for features
  import kothImage from './assets/Assets/koth.png';
  import pushEventImage from './assets/Assets/push-event.png';
  import bigFarmImage from './assets/Assets/BIG-FARM.png';
  import twoVtwoImage from './assets/Assets/2v2.png';
  
  let theme: 'dark' | 'light' = 'dark';
  let mounted = false;
  
  const apiBaseUrl = import.meta.env.VITE_API_BASE_URL;
  
  onMount(() => {
    mounted = true;
    fetchUser();
  });
  
  function handleThemeToggle(event: CustomEvent<{ theme: 'dark' | 'light' }>) {
    theme = event.detail.theme;
  }
</script>

<div class="app" class:light={theme === 'light'} class:mounted>
  <Header {theme} logo={lostLogo} on:themeToggle={handleThemeToggle} />
  
  <main>
    <Hero banner={lostBanner} {theme} {mounted} />
    
    <DiscordSection {theme} fallbackIcon={lostLogo} />
    
    <FeaturesSection 
      {theme} 
      {kothImage} 
      {pushEventImage} 
      {bigFarmImage} 
      {twoVtwoImage} 
    />
    
    <ClansSection 
      {theme} 
      {apiBaseUrl}
      gameType="coc"
      title="Clash of Clans"
      description="Unsere Clans in Clash of Clans"
    />
    
    <ClansSection 
      {theme} 
      {apiBaseUrl}
      gameType="cr"
      title="Clash Royale"
      description="Unsere Clans in Clash Royale"
    />
    
    <Footer {theme} logo={lostLogo} />
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    overflow-x: hidden;
  }
  
  .app {
    min-height: 100vh;
    background: linear-gradient(180deg, #0a0a0f 0%, #12121a 50%, #0f1419 100%);
    background-attachment: fixed;
    transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  .app.light {
    background: linear-gradient(180deg, #f8fafc 0%, #f1f5f9 50%, #e2e8f0 100%);
  }
  
  .app::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: 
      radial-gradient(ellipse 80% 50% at 20% 20%, rgba(88, 101, 242, 0.12) 0%, transparent 50%),
      radial-gradient(ellipse 60% 40% at 80% 80%, rgba(59, 165, 92, 0.08) 0%, transparent 50%),
      radial-gradient(ellipse 50% 30% at 50% 50%, rgba(255, 183, 77, 0.05) 0%, transparent 50%);
    pointer-events: none;
    z-index: 0;
    transition: opacity 0.5s ease;
  }
  
  .app.light::before {
    background: 
      radial-gradient(ellipse 80% 50% at 20% 20%, rgba(88, 101, 242, 0.08) 0%, transparent 50%),
      radial-gradient(ellipse 60% 40% at 80% 80%, rgba(59, 165, 92, 0.06) 0%, transparent 50%);
  }

  main {
    position: relative;
    z-index: 1;
  }
</style>
