<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  
  export let theme: 'dark' | 'light' = 'dark';
  
  const dispatch = createEventDispatcher();
  
  function toggle() {
    theme = theme === 'dark' ? 'light' : 'dark';
    dispatch('toggle', { theme });
    localStorage.setItem('theme', theme);
  }
  
  onMount(() => {
    const saved = localStorage.getItem('theme') as 'dark' | 'light' | null;
    if (saved) {
      theme = saved;
      dispatch('toggle', { theme });
    }
  });
</script>

<button 
  class="theme-toggle" 
  on:click={toggle} 
  aria-label="Toggle theme"
  title={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
>
  <div class="toggle-track" class:light={theme === 'light'}>
    <div class="toggle-thumb">
      {#if theme === 'dark'}
        <svg class="icon moon" viewBox="0 0 24 24" fill="currentColor">
          <path d="M21.752 15.002A9.718 9.718 0 0118 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 003 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 009.002-5.998z" />
        </svg>
      {:else}
        <svg class="icon sun" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2.25a.75.75 0 01.75.75v2.25a.75.75 0 01-1.5 0V3a.75.75 0 01.75-.75zM7.5 12a4.5 4.5 0 119 0 4.5 4.5 0 01-9 0zM18.894 6.166a.75.75 0 00-1.06-1.06l-1.591 1.59a.75.75 0 101.06 1.061l1.591-1.59zM21.75 12a.75.75 0 01-.75.75h-2.25a.75.75 0 010-1.5H21a.75.75 0 01.75.75zM17.834 18.894a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 10-1.061 1.06l1.59 1.591zM12 18a.75.75 0 01.75.75V21a.75.75 0 01-1.5 0v-2.25A.75.75 0 0112 18zM7.758 17.303a.75.75 0 00-1.061-1.06l-1.591 1.59a.75.75 0 001.06 1.061l1.591-1.59zM6 12a.75.75 0 01-.75.75H3a.75.75 0 010-1.5h2.25A.75.75 0 016 12zM6.697 7.757a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 00-1.061 1.06l1.59 1.591z" />
        </svg>
      {/if}
    </div>
  </div>
</button>

<style>
  .theme-toggle {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    outline: none;
  }
  
  .toggle-track {
    width: 56px;
    height: 30px;
    background: linear-gradient(135deg, #1e3a5f 0%, #0f172a 100%);
    border-radius: 15px;
    position: relative;
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 
      inset 0 2px 4px rgba(0, 0, 0, 0.3),
      0 2px 8px rgba(0, 0, 0, 0.2);
    overflow: hidden;
  }
  
  .toggle-track::before {
    content: '';
    position: absolute;
    top: 4px;
    left: 8px;
    width: 4px;
    height: 4px;
    background: rgba(255, 255, 255, 0.6);
    border-radius: 50%;
    box-shadow: 
      12px 8px 0 1px rgba(255, 255, 255, 0.4),
      24px 2px 0 0 rgba(255, 255, 255, 0.3),
      32px 10px 0 1px rgba(255, 255, 255, 0.2);
    transition: opacity 0.3s ease;
  }
  
  .toggle-track.light {
    background: linear-gradient(135deg, #87CEEB 0%, #4FC3F7 100%);
  }
  
  .toggle-track.light::before {
    opacity: 0;
  }
  
  .toggle-track.light::after {
    content: '';
    position: absolute;
    top: 6px;
    right: 10px;
    width: 20px;
    height: 8px;
    background: rgba(255, 255, 255, 0.8);
    border-radius: 4px;
    opacity: 1;
    animation: float 3s ease-in-out infinite;
  }
  
  @keyframes float {
    0%, 100% { transform: translateX(0); }
    50% { transform: translateX(-3px); }
  }
  
  .toggle-thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 24px;
    height: 24px;
    background: linear-gradient(145deg, #ffd93d, #ffb347);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 
      0 2px 8px rgba(255, 183, 77, 0.4),
      inset 0 -2px 4px rgba(0, 0, 0, 0.1);
  }
  
  .toggle-track.light .toggle-thumb {
    transform: translateX(26px);
    background: linear-gradient(145deg, #fff9c4, #ffeb3b);
    box-shadow: 
      0 2px 12px rgba(255, 235, 59, 0.6),
      0 0 20px rgba(255, 235, 59, 0.3);
  }
  
  .icon {
    width: 14px;
    height: 14px;
    transition: all 0.3s ease;
  }
  
  .moon {
    color: #1e3a5f;
  }
  
  .sun {
    color: #f57c00;
  }
  
  .theme-toggle:hover .toggle-track {
    box-shadow: 
      inset 0 2px 4px rgba(0, 0, 0, 0.3),
      0 4px 16px rgba(0, 0, 0, 0.3);
  }
  
  .theme-toggle:hover .toggle-thumb {
    transform: scale(1.05);
  }
  
  .theme-toggle:hover .toggle-track.light .toggle-thumb {
    transform: translateX(26px) scale(1.05);
  }
  
  .theme-toggle:active .toggle-thumb {
    transform: scale(0.95);
  }
  
  .theme-toggle:active .toggle-track.light .toggle-thumb {
    transform: translateX(26px) scale(0.95);
  }
</style>
