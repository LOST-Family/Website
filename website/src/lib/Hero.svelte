<script lang="ts">
  export let banner: string;
  export let theme: 'dark' | 'light' = 'dark';
  export let mounted: boolean = false;
</script>

<section class="hero" class:light={theme === 'light'} class:mounted>
  <div class="hero-banner">
    <img src={banner} alt="Lost Family Banner" class="banner-image" />
    <div class="hero-overlay"></div>
  </div>
  <div class="hero-content">
    <h1 class="hero-title">Lost Family</h1>
    <p class="hero-subtitle">Clash of Clans Alliance</p>
    <div class="hero-scroll-indicator">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M7 13l5 5 5-5M7 6l5 5 5-5"/>
      </svg>
    </div>
  </div>
</section>

<style>
  @keyframes fadeInUp {
    from {
      opacity: 0;
      transform: translateY(30px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  
  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-8px); }
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 1; }
  }

  .hero {
    position: relative;
    width: 100%;
    height: 450px;
    overflow: hidden;
  }
  
  .hero-banner {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }
  
  .banner-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: center 30%;
    animation: fadeIn 1s ease;
  }
  
  .hero-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      to bottom,
      rgba(10, 10, 15, 0.2) 0%,
      rgba(10, 10, 15, 0.4) 40%,
      rgba(10, 10, 15, 0.95) 100%
    );
    transition: background 0.5s ease;
  }
  
  .hero.light .hero-overlay {
    background: linear-gradient(
      to bottom,
      rgba(248, 250, 252, 0.1) 0%,
      rgba(248, 250, 252, 0.3) 40%,
      rgba(248, 250, 252, 0.98) 100%
    );
  }
  
  .hero-content {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 3rem 2rem;
    text-align: center;
  }
  
  .hero.mounted .hero-title {
    animation: fadeInUp 0.8s ease forwards;
  }
  
  .hero.mounted .hero-subtitle {
    animation: fadeInUp 0.8s ease 0.15s forwards;
    opacity: 0;
  }
  
  .hero.mounted .hero-scroll-indicator {
    animation: fadeIn 1s ease 0.5s forwards, float 2s ease-in-out 1.5s infinite;
    opacity: 0;
  }
  
  .hero-title {
    margin: 0;
    font-size: 3.5rem;
    font-weight: 800;
    color: #fff;
    text-shadow: 0 4px 30px rgba(0, 0, 0, 0.5);
    letter-spacing: -0.03em;
    transition: color 0.4s ease;
  }
  
  .hero.light .hero-title {
    color: #0f172a;
    text-shadow: none;
  }
  
  .hero-subtitle {
    margin: 0.75rem 0 0;
    font-size: 1.25rem;
    color: rgba(255, 255, 255, 0.8);
    font-weight: 500;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    transition: color 0.4s ease;
  }
  
  .hero.light .hero-subtitle {
    color: #64748b;
  }
  
  .hero-scroll-indicator {
    margin-top: 2rem;
    color: rgba(255, 255, 255, 0.5);
    transition: color 0.4s ease;
  }
  
  .hero-scroll-indicator svg {
    width: 28px;
    height: 28px;
    animation: pulse 2s ease-in-out infinite;
  }
  
  .hero.light .hero-scroll-indicator {
    color: rgba(0, 0, 0, 0.3);
  }

  @media (max-width: 800px) {
    .hero {
      height: 320px;
    }
    
    .hero-title {
      font-size: 2.25rem;
    }
    
    .hero-subtitle {
      font-size: 1rem;
    }
    
    .hero-content {
      padding: 2rem 1rem;
    }
  }
</style>
