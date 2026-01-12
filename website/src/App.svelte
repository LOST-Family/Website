<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchUser } from './lib/auth';

    // Components
    import Header from './lib/Header.svelte';
    import Hero from './lib/Hero.svelte';
    import DiscordSection from './lib/DiscordSection.svelte';
    import FeaturesSection from './lib/FeaturesSection.svelte';
    import ClansSection from './lib/ClansSection.svelte';
    import ClansPage from './lib/ClansPage.svelte';
    import ProfilePage from './lib/ProfilePage.svelte';
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
    let currentPath = window.location.pathname;

    const apiBaseUrl =
        import.meta.env.VITE_API_BASE_URL !== undefined
            ? import.meta.env.VITE_API_BASE_URL
            : '';

    onMount(() => {
        mounted = true;
        fetchUser();

        // Simple SPA routing
        const handlePopState = () => {
            currentPath = window.location.pathname;
        };

        window.addEventListener('popstate', handlePopState);
        return () => window.removeEventListener('popstate', handlePopState);
    });

    function handleThemeToggle(
        event: CustomEvent<{ theme: 'dark' | 'light' }>
    ) {
        theme = event.detail.theme;
    }

    function handleNavigate(event: CustomEvent<string>) {
        const newPath = event.detail === 'home' ? '/' : `/${event.detail}`;
        if (currentPath !== newPath) {
            window.history.pushState({}, '', newPath);
            currentPath = newPath;
            window.scrollTo({ top: 0, behavior: 'smooth' });
        }
    }
</script>

<div class="app" class:light={theme === 'light'} class:mounted>
    <Header
        {theme}
        logo={lostLogo}
        on:themeToggle={handleThemeToggle}
        on:navigate={handleNavigate}
    />

    <main>
        {#if currentPath === '/' || currentPath === ''}
            <Hero banner={lostBanner} {theme} {mounted} />

            <DiscordSection {theme} fallbackIcon={lostLogo} />

            <ClansSection
                {theme}
                {apiBaseUrl}
                gameType="coc"
                title="Clash of Clans"
                description="Unsere Clans in Clash of Clans"
            />

            <FeaturesSection
                {theme}
                {kothImage}
                {pushEventImage}
                {bigFarmImage}
                {twoVtwoImage}
            />
        {:else if currentPath === '/coc/clans'}
            <ClansPage {theme} {apiBaseUrl} />
        {:else if currentPath === '/account'}
            <ProfilePage {theme} {apiBaseUrl} />
        {/if}

        <Footer {theme} logo={lostLogo} />
    </main>
</div>

<style>
    :global(body) {
        margin: 0;
        font-family:
            'Inter',
            system-ui,
            -apple-system,
            sans-serif;
        overflow-x: hidden;
    }

    .app {
        min-height: 100vh;
        background: linear-gradient(
            180deg,
            #0a0a0f 0%,
            #12121a 50%,
            #0f1419 100%
        );
        background-attachment: fixed;
        transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .app.light {
        background: linear-gradient(
            180deg,
            #f8fafc 0%,
            #f1f5f9 50%,
            #e2e8f0 100%
        );
    }

    .app::before {
        content: '';
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: radial-gradient(
                ellipse 80% 50% at 20% 20%,
                rgba(88, 101, 242, 0.12) 0%,
                transparent 50%
            ),
            radial-gradient(
                ellipse 60% 40% at 80% 80%,
                rgba(59, 165, 92, 0.08) 0%,
                transparent 50%
            ),
            radial-gradient(
                ellipse 50% 30% at 50% 50%,
                rgba(255, 183, 77, 0.05) 0%,
                transparent 50%
            );
        pointer-events: none;
        z-index: 0;
        transition: opacity 0.5s ease;
    }

    .app.light::before {
        background: radial-gradient(
                ellipse 80% 50% at 20% 20%,
                rgba(88, 101, 242, 0.08) 0%,
                transparent 50%
            ),
            radial-gradient(
                ellipse 60% 40% at 80% 80%,
                rgba(59, 165, 92, 0.06) 0%,
                transparent 50%
            );
    }

    main {
        position: relative;
        z-index: 1;
    }
</style>
