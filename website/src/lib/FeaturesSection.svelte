<script lang="ts">
  export let theme: 'dark' | 'light' = 'dark';
  export let kothImage: string;
  export let pushEventImage: string;
  export let bigFarmImage: string;
  export let twoVtwoImage: string;
</script>

<section class="content-section" id="features" class:light={theme === 'light'}>
  <div class="section-container">
    <div class="section-header">
      <div class="section-icon feature-icon">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
        </svg>
      </div>
      <div class="section-title-group">
        <h2 class="section-title">Events & Aktivitäten</h2>
        <p class="section-description">Spannende Events für unsere Community</p>
      </div>
    </div>
    
    <div class="features-grid">
      <div class="feature-card">
        <div class="feature-image">
          <img src={kothImage} alt="King of the Hill" />
        </div>
        <div class="feature-content">
          <h3 class="feature-title">King of the Hill</h3>
          <p class="feature-description">Kämpfe um die Spitze und verteidige deinen Thron!</p>
        </div>
      </div>
      
      <div class="feature-card">
        <div class="feature-image">
          <img src={pushEventImage} alt="Push Event" />
        </div>
        <div class="feature-content">
          <h3 class="feature-title">Push Events</h3>
          <p class="feature-description">Steige in den Rängen auf und gewinne Belohnungen!</p>
        </div>
      </div>
      
      <div class="feature-card">
        <div class="feature-image">
          <img src={bigFarmImage} alt="Big Farm" />
        </div>
        <div class="feature-content">
          <h3 class="feature-title">Big Farm</h3>
          <p class="feature-description">Sammle gemeinsam Ressourcen mit der Community!</p>
        </div>
      </div>
      
      <div class="feature-card">
        <div class="feature-image">
          <img src={twoVtwoImage} alt="2v2 Battles" />
        </div>
        <div class="feature-content">
          <h3 class="feature-title">2v2 Battles</h3>
          <p class="feature-description">Team up und zeigt eure Skills im Duo!</p>
        </div>
      </div>
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
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    flex-shrink: 0;
  }
  
  .section-icon.feature-icon {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    box-shadow: 0 8px 24px rgba(245, 158, 11, 0.3);
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
  
  .features-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
  }
  
  .feature-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 16px;
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  .feature-card:hover {
    transform: translateY(-6px);
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.1);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
  }
  
  .content-section.light .feature-card {
    background: rgba(255, 255, 255, 0.7);
    border: 1px solid rgba(0, 0, 0, 0.06);
  }
  
  .content-section.light .feature-card:hover {
    background: rgba(255, 255, 255, 0.9);
    border-color: rgba(0, 0, 0, 0.1);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  }
  
  .feature-image {
    width: 100%;
    overflow: hidden;
    background: rgba(0, 0, 0, 0.2);
  }
  
  .feature-image img {
    width: 100%;
    height: auto;
    display: block;
    transition: transform 0.4s ease;
  }
  
  .feature-card:hover .feature-image img {
    transform: scale(1.05);
  }
  
  .feature-content {
    padding: 1.25rem;
  }
  
  .feature-title {
    margin: 0 0 0.5rem;
    font-size: 1.1rem;
    font-weight: 700;
    color: #fff;
    transition: color 0.4s ease;
  }
  
  .content-section.light .feature-title {
    color: #0f172a;
  }
  
  .feature-description {
    margin: 0;
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.6);
    line-height: 1.5;
    transition: color 0.4s ease;
  }
  
  .content-section.light .feature-description {
    color: #64748b;
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
  }
</style>
