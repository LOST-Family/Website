<script lang="ts">
  import ThemeToggle from './ThemeToggle.svelte';
  import { createEventDispatcher } from 'svelte';
  
  export let theme: 'dark' | 'light' = 'dark';
  export let logo: string;
  
  let mobileMenuOpen = false;
  
  const dispatch = createEventDispatcher<{ themeToggle: { theme: 'dark' | 'light' } }>();
  
  function handleThemeToggle(event: CustomEvent<{ theme: 'dark' | 'light' }>) {
    dispatch('themeToggle', event.detail);
  }
  
  function toggleMobileMenu() {
    mobileMenuOpen = !mobileMenuOpen;
  }
</script>

<header class="header" class:light={theme === 'light'}>
  <div class="header-content">
    <div class="logo">
      <img src={logo} alt="Lost Family" class="logo-icon" />
      <span class="logo-text">Lost Family</span>
    </div>
    
    <nav class="nav" class:open={mobileMenuOpen} class:light={theme === 'light'}>
      <div class="nav-item dropdown">
        <button class="nav-link">
          Community
          <svg class="dropdown-arrow" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
          </svg>
        </button>
        <div class="dropdown-menu">
          <a href="#discord" class="dropdown-item">Discord Servers</a>
          <a href="#features" class="dropdown-item">Events & Activities</a>
          <a href="#coc-clans" class="dropdown-item">Clash of Clans</a>
          <a href="#cr-clans" class="dropdown-item">Clash Royale</a>
        </div>
      </div>
      
      <div class="nav-item dropdown">
        <button class="nav-link">
          Resources
          <svg class="dropdown-arrow" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
          </svg>
        </button>
        <div class="dropdown-menu">
          <a href="#guides" class="dropdown-item">Guides</a>
          <a href="#tools" class="dropdown-item">Tools</a>
          <a href="#faq" class="dropdown-item">FAQ</a>
        </div>
      </div>
      
      <a href="#about" class="nav-link">About</a>
      <a href="#contact" class="nav-link">Contact</a>
    </nav>
    
    <div class="header-actions">
      <ThemeToggle {theme} on:toggle={handleThemeToggle} />
      
      <button class="mobile-menu-btn" on:click={toggleMobileMenu} aria-label="Toggle menu">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          {#if mobileMenuOpen}
            <path d="M6 18L18 6M6 6l12 12" />
          {:else}
            <path d="M4 6h16M4 12h16M4 18h16" />
          {/if}
        </svg>
      </button>
    </div>
  </div>
</header>

<style>
  .header {
    position: sticky;
    top: 0;
    z-index: 100;
    background: rgba(10, 10, 15, 0.8);
    backdrop-filter: blur(20px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    transition: all 0.4s ease;
  }
  
  .header.light {
    background: rgba(255, 255, 255, 0.8);
    border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  }
  
  .header-content {
    max-width: 1400px;
    margin: 0 auto;
    padding: 0 2rem;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 2rem;
  }
  
  .logo {
    display: flex;
    align-items: center;
    gap: 12px;
    text-decoration: none;
    flex-shrink: 0;
  }
  
  .logo-icon {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    transition: transform 0.3s ease;
  }
  
  .logo:hover .logo-icon {
    transform: scale(1.05);
  }
  
  .logo-text {
    font-size: 1.25rem;
    font-weight: 700;
    color: #fff;
    transition: color 0.4s ease;
  }
  
  .header.light .logo-text {
    color: #1a1a2e;
  }
  
  /* Navigation */
  .nav {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  
  .nav-item {
    position: relative;
  }
  
  .nav-link {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 10px 16px;
    color: rgba(255, 255, 255, 0.7);
    text-decoration: none;
    font-size: 0.9rem;
    font-weight: 500;
    border-radius: 8px;
    border: none;
    background: transparent;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .nav-link:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }
  
  .nav.light .nav-link {
    color: rgba(0, 0, 0, 0.6);
  }
  
  .nav.light .nav-link:hover {
    color: #1a1a2e;
    background: rgba(0, 0, 0, 0.05);
  }
  
  .dropdown-arrow {
    width: 14px;
    height: 14px;
    transition: transform 0.2s ease;
    opacity: 0.6;
  }
  
  .dropdown:hover .dropdown-arrow {
    transform: rotate(180deg);
  }
  
  .dropdown-menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%) translateY(8px);
    min-width: 180px;
    padding: 8px;
    background: rgba(30, 32, 35, 0.98);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    opacity: 0;
    visibility: hidden;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  .dropdown:hover .dropdown-menu {
    opacity: 1;
    visibility: visible;
    transform: translateX(-50%) translateY(0);
  }
  
  .nav.light .dropdown-menu {
    background: rgba(255, 255, 255, 0.98);
    border: 1px solid rgba(0, 0, 0, 0.08);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.12);
  }
  
  .dropdown-item {
    display: block;
    padding: 10px 14px;
    color: rgba(255, 255, 255, 0.8);
    text-decoration: none;
    font-size: 0.875rem;
    border-radius: 8px;
    transition: all 0.2s ease;
  }
  
  .dropdown-item:hover {
    color: #fff;
    background: rgba(88, 101, 242, 0.2);
  }
  
  .nav.light .dropdown-item {
    color: rgba(0, 0, 0, 0.7);
  }
  
  .nav.light .dropdown-item:hover {
    color: #1a1a2e;
    background: rgba(88, 101, 242, 0.1);
  }
  
  /* Header Actions */
  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .header-actions :global(.theme-toggle) {
    position: static;
  }
  
  .mobile-menu-btn {
    display: none;
    width: 40px;
    height: 40px;
    padding: 8px;
    background: transparent;
    border: none;
    border-radius: 8px;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .mobile-menu-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }
  
  .header.light .mobile-menu-btn {
    color: rgba(0, 0, 0, 0.6);
  }
  
  .header.light .mobile-menu-btn:hover {
    background: rgba(0, 0, 0, 0.05);
    color: #1a1a2e;
  }
  
  .mobile-menu-btn svg {
    width: 100%;
    height: 100%;
  }
  
  /* Mobile Styles */
  @media (max-width: 900px) {
    .nav {
      position: fixed;
      top: 64px;
      left: 0;
      right: 0;
      bottom: 0;
      flex-direction: column;
      align-items: stretch;
      padding: 1rem;
      background: rgba(10, 10, 15, 0.98);
      backdrop-filter: blur(20px);
      transform: translateX(-100%);
      transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
      overflow-y: auto;
    }
    
    .nav.open {
      transform: translateX(0);
    }
    
    .nav.light {
      background: rgba(255, 255, 255, 0.98);
    }
    
    .dropdown-menu {
      position: static;
      opacity: 1;
      visibility: visible;
      transform: none;
      box-shadow: none;
      border: none;
      background: transparent;
      padding-left: 1rem;
      min-width: auto;
    }
    
    .mobile-menu-btn {
      display: flex;
      align-items: center;
      justify-content: center;
    }
  }

  @media (max-width: 800px) {
    .header-content {
      padding: 0 1rem;
    }
    
    .logo-text {
      display: none;
    }
  }
</style>
